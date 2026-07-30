"""Evaluate donglao-g2p on pipe-delimited TTS metadata files.

This is a corpus-level engineering evaluation, not a phoneme-accuracy oracle:
the metadata contains text but no human-reviewed target phonemes. It measures
runtime, language routing against each corpus' dominant language, OOV coverage,
normalization stability, and output invariants. It also records representative
errors for manual linguistic review.
"""

from __future__ import annotations

import argparse
import csv
import json
import resource
import statistics
import time
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Iterator

from donglao_g2p import Analysis, Pipeline

PUNCTUATION = frozenset(",.")


@dataclass(frozen=True)
class CorpusSpec:
    name: str
    path: Path
    expected_language: str


@dataclass
class CorpusReport:
    corpus: str
    path: str
    expected_language: str
    sentences: int
    characters: int
    word_tokens: int
    expected_language_tokens: int
    language_agreement: float
    dominant_sentence_agreement: float
    dictionary_tokens: int
    rule_tokens: int
    oov_tokens: int
    oov_rate: float
    empty_phone_tokens: int
    malformed_phone_tokens: int
    invalid_punctuation_outputs: int
    non_idempotent_normalizations: int
    changed_by_normalization: int
    analysis_seconds: float
    analysis_sentences_per_second: float
    latency_p50_ms: float
    latency_p95_ms: float
    latency_p99_ms: float
    batch_seconds: float
    batch_characters_per_second: float
    rss_peak_mb: float
    source_counts: dict[str, int]
    detected_language_counts: dict[str, int]
    top_oov: list[tuple[str, int]]
    top_language_mismatches: list[tuple[str, int]]
    oov_examples: list[dict[str, str]]
    language_mismatch_examples: list[dict[str, str]]
    malformed_examples: list[dict[str, str]]
    non_idempotent_examples: list[dict[str, str]]


def read_texts(path: Path, limit: int | None) -> list[str]:
    texts: list[str] = []
    with path.open(newline="", encoding="utf-8") as source:
        reader = csv.DictReader(source, delimiter="|")
        required = {"audio_path", "text", "speaker_id"}
        if reader.fieldnames is None or not required.issubset(reader.fieldnames):
            raise ValueError(
                f"{path}: expected pipe-delimited columns {sorted(required)}, "
                f"got {reader.fieldnames}"
            )
        for row in reader:
            text = row["text"].strip()
            if text:
                texts.append(text)
            if limit is not None and len(texts) >= limit:
                break
    return texts


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    return ordered[min(int(len(ordered) * fraction), len(ordered) - 1)]


def chunks(values: list[str], size: int) -> Iterator[list[str]]:
    for index in range(0, len(values), size):
        yield values[index : index + size]


def phones_are_well_formed(language: str, phonemes: str) -> bool:
    if not phonemes.strip():
        return False
    if language == "vi":
        return all(part[-1:] in "123456" for part in phonemes.split())
    if language == "en":
        return not any(character in phonemes for character in "123456ˈˌ")
    return phonemes in PUNCTUATION


def valid_punctuation(phonemes: str) -> bool:
    for token in phonemes.split():
        if any(character in token for character in ",.!?") and token not in PUNCTUATION:
            return False
    return True


def append_example(
    examples: list[dict[str, str]],
    *,
    text: str,
    normalized: str,
    token: str,
    phonemes: str,
    limit: int = 20,
) -> None:
    if len(examples) < limit:
        examples.append(
            {
                "text": text,
                "normalized": normalized,
                "token": token,
                "phonemes": phonemes,
            }
        )


def evaluate_corpus(
    pipeline: Pipeline,
    spec: CorpusSpec,
    texts: list[str],
    batch_size: int,
) -> CorpusReport:
    latencies: list[float] = []
    source_counts: Counter[str] = Counter()
    language_counts: Counter[str] = Counter()
    oov_words: Counter[str] = Counter()
    mismatch_words: Counter[str] = Counter()
    oov_examples: list[dict[str, str]] = []
    mismatch_examples: list[dict[str, str]] = []
    malformed_examples: list[dict[str, str]] = []
    non_idempotent_examples: list[dict[str, str]] = []
    characters = word_tokens = expected_tokens = 0
    dictionary_tokens = rule_tokens = oov_tokens = 0
    empty_tokens = malformed_tokens = invalid_punctuation = 0
    non_idempotent = changed = dominant_sentences = 0

    analysis_start = time.perf_counter()
    for text in texts:
        started = time.perf_counter_ns()
        analysis: Analysis = pipeline.analyze(text)
        latencies.append((time.perf_counter_ns() - started) / 1_000_000)
        characters += len(text)
        changed += analysis.normalized != text
        renormalized = pipeline.normalize(analysis.normalized)
        if renormalized != analysis.normalized:
            non_idempotent += 1
            if len(non_idempotent_examples) < 20:
                non_idempotent_examples.append(
                    {
                        "text": text,
                        "normalized": analysis.normalized,
                        "renormalized": renormalized,
                    }
                )
        invalid_punctuation += not valid_punctuation(analysis.phonemes)

        sentence_words = [
            token for token in analysis.tokens if token.language != "punc"
        ]
        sentence_expected = sum(
            token.language == spec.expected_language for token in sentence_words
        )
        if sentence_words and sentence_expected / len(sentence_words) >= 0.9:
            dominant_sentences += 1

        for token in sentence_words:
            word_tokens += 1
            expected_tokens += token.language == spec.expected_language
            source_counts[token.source] += 1
            language_counts[token.language] += 1
            dictionary_tokens += token.source == "dictionary"
            rule_tokens += token.source == "rules"
            oov_tokens += token.source == "oov"
            if not token.phonemes.strip():
                empty_tokens += 1
            if not phones_are_well_formed(token.language, token.phonemes):
                malformed_tokens += 1
                append_example(
                    malformed_examples,
                    text=text,
                    normalized=analysis.normalized,
                    token=token.token,
                    phonemes=token.phonemes,
                )
            if token.source == "oov":
                oov_words[token.token.lower()] += 1
                append_example(
                    oov_examples,
                    text=text,
                    normalized=analysis.normalized,
                    token=token.token,
                    phonemes=token.phonemes,
                )
            if token.language != spec.expected_language:
                mismatch_words[token.token.lower()] += 1
                append_example(
                    mismatch_examples,
                    text=text,
                    normalized=analysis.normalized,
                    token=token.token,
                    phonemes=token.phonemes,
                )
    analysis_seconds = time.perf_counter() - analysis_start

    batch_start = time.perf_counter()
    batch_output_count = 0
    for batch in chunks(texts, batch_size):
        batch_output_count += len(pipeline.phonemize_batch(batch))
    batch_seconds = time.perf_counter() - batch_start
    if batch_output_count != len(texts):
        raise AssertionError("batch output count differs from input count")

    return CorpusReport(
        corpus=spec.name,
        path=str(spec.path),
        expected_language=spec.expected_language,
        sentences=len(texts),
        characters=characters,
        word_tokens=word_tokens,
        expected_language_tokens=expected_tokens,
        language_agreement=expected_tokens / word_tokens if word_tokens else 0.0,
        dominant_sentence_agreement=(
            dominant_sentences / len(texts) if texts else 0.0
        ),
        dictionary_tokens=dictionary_tokens,
        rule_tokens=rule_tokens,
        oov_tokens=oov_tokens,
        oov_rate=oov_tokens / word_tokens if word_tokens else 0.0,
        empty_phone_tokens=empty_tokens,
        malformed_phone_tokens=malformed_tokens,
        invalid_punctuation_outputs=invalid_punctuation,
        non_idempotent_normalizations=non_idempotent,
        changed_by_normalization=changed,
        analysis_seconds=analysis_seconds,
        analysis_sentences_per_second=(
            len(texts) / analysis_seconds if analysis_seconds else 0.0
        ),
        latency_p50_ms=statistics.median(latencies) if latencies else 0.0,
        latency_p95_ms=percentile(latencies, 0.95),
        latency_p99_ms=percentile(latencies, 0.99),
        batch_seconds=batch_seconds,
        batch_characters_per_second=(
            characters / batch_seconds if batch_seconds else 0.0
        ),
        rss_peak_mb=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
        source_counts=dict(source_counts),
        detected_language_counts=dict(language_counts),
        top_oov=oov_words.most_common(30),
        top_language_mismatches=mismatch_words.most_common(30),
        oov_examples=oov_examples,
        language_mismatch_examples=mismatch_examples,
        malformed_examples=malformed_examples,
        non_idempotent_examples=non_idempotent_examples,
    )


def print_summary(reports: Iterable[CorpusReport]) -> None:
    for report in reports:
        print(f"\n[{report.corpus}] {report.sentences:,} sentences")
        print(
            f"  language agreement: {report.language_agreement:.2%}; "
            f"dominant sentences: {report.dominant_sentence_agreement:.2%}"
        )
        print(
            f"  OOV: {report.oov_tokens:,}/{report.word_tokens:,} "
            f"({report.oov_rate:.3%}); malformed: {report.malformed_phone_tokens:,}; "
            f"non-idempotent: {report.non_idempotent_normalizations:,}"
        )
        print(
            f"  latency p50/p95/p99: {report.latency_p50_ms:.3f}/"
            f"{report.latency_p95_ms:.3f}/{report.latency_p99_ms:.3f} ms"
        )
        print(
            f"  batch throughput: {report.batch_characters_per_second:,.0f} chars/s; "
            f"peak RSS: {report.rss_peak_mb:.1f} MB"
        )
        print(f"  top OOV: {report.top_oov[:10]}")
        print(f"  top language mismatches: {report.top_language_mismatches[:10]}")


def parse_args() -> argparse.Namespace:
    root = Path(__file__).resolve().parents[2]
    default_data = root / "tts-donglao" / "DATASET" / "raw"
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--libritts",
        type=Path,
        default=default_data / "libritts100" / "metadata.csv",
    )
    parser.add_argument(
        "--vieneu",
        type=Path,
        default=default_data / "vieneu" / "metadata.csv",
    )
    parser.add_argument("--limit", type=int)
    parser.add_argument("--batch-size", type=int, default=4096)
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).with_name("metadata_report.json"),
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.limit is not None and args.limit <= 0:
        raise SystemExit("--limit must be greater than zero")
    if args.batch_size <= 0:
        raise SystemExit("--batch-size must be greater than zero")

    specs = [
        CorpusSpec("libritts100", args.libritts.resolve(), "en"),
        CorpusSpec("vieneu", args.vieneu.resolve(), "vi"),
    ]
    pipeline_start = time.perf_counter()
    pipeline = Pipeline()
    pipeline_init_ms = (time.perf_counter() - pipeline_start) * 1000

    reports = []
    for spec in specs:
        texts = read_texts(spec.path, args.limit)
        reports.append(evaluate_corpus(pipeline, spec, texts, args.batch_size))

    payload = {
        "pipeline_init_ms": pipeline_init_ms,
        "limitations": [
            "The CSV files contain no human-reviewed target phonemes.",
            "Language agreement uses each corpus language as a proxy label; legitimate code-switches count as mismatches.",
            "OOV coverage and output invariants do not prove pronunciation correctness.",
        ],
        "reports": [asdict(report) for report in reports],
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    print(f"pipeline init: {pipeline_init_ms:.3f} ms")
    print_summary(reports)
    print(f"\nfull report: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
