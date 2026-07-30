"""Stream the language|text debug corpus without materializing it in memory."""

from __future__ import annotations

import argparse
import csv
import json
import time
from collections import Counter
from pathlib import Path

from donglao_g2p import Pipeline


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("corpus", type=Path)
    parser.add_argument("--limit", type=int)
    parser.add_argument("--top", type=int, default=30)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.limit is not None and args.limit <= 0:
        parser.error("--limit must be greater than zero")
    if args.top <= 0:
        parser.error("--top must be greater than zero")

    pipeline = Pipeline()
    rows = tokens = empty = 0
    labels: Counter[str] = Counter()
    detected: Counter[tuple[str, str]] = Counter()
    sources: Counter[tuple[str, str]] = Counter()
    mismatches: Counter[tuple[str, str]] = Counter()
    oov: Counter[str] = Counter()
    started = time.perf_counter()

    with args.corpus.open(newline="", encoding="utf-8") as source:
        reader = csv.DictReader(source, delimiter="|")
        if reader.fieldnames is None or not {"language", "text"}.issubset(
            reader.fieldnames
        ):
            raise ValueError("expected pipe-delimited language|text columns")
        for row in reader:
            label = row["language"]
            labels[label] += 1
            rows += 1
            for token in pipeline.analyze(row["text"]).tokens:
                if token.language == "punc":
                    continue
                tokens += 1
                detected[(label, token.language)] += 1
                sources[(label, token.source)] += 1
                if not token.phonemes:
                    empty += 1
                if token.language != label:
                    mismatches[(label, token.token.lower())] += 1
                if token.source == "oov":
                    oov[token.token.lower()] += 1
            if args.limit is not None and rows >= args.limit:
                break

    payload = {
        "corpus": str(args.corpus.resolve()),
        "rows": rows,
        "tokens": tokens,
        "seconds": time.perf_counter() - started,
        "labels": dict(labels),
        "detected": {
            f"{label}->{language}": count
            for (label, language), count in detected.items()
        },
        "sources": {
            f"{label}:{source}": count
            for (label, source), count in sources.items()
        },
        "empty_phone_tokens": empty,
        "top_oov": oov.most_common(args.top),
        "top_language_mismatches": {
            label: Counter(
                {
                    token: count
                    for (item_label, token), count in mismatches.items()
                    if item_label == label
                }
            ).most_common(args.top)
            for label in labels
        },
        "limitations": [
            "Corpus labels are sentence-level proxies, not token-level gold labels.",
            "The corpus contains no human-reviewed target phonemes.",
        ],
    }
    rendered = json.dumps(payload, ensure_ascii=False, indent=2)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered + "\n", encoding="utf-8")
    print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
