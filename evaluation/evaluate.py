"""Quality gate for a linguist-reviewed JSONL evaluation corpus.

Rows have one of these forms:
  {"kind":"g2p","text":"...", "expected":"...", "oov":true}
  {"kind":"langid","text":"...", "languages":["vi","en",...]}
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from donglao_g2p import Pipeline

MULTI_PHONES = tuple(
    sorted(
        ("tʃ", "dʒ", "aɪ", "aʊ", "eɪ", "oʊ", "ɔɪ", "iː", "uː", "ɑɹ", "ɔɹ"),
        key=len,
        reverse=True,
    )
)


def phones(value: str) -> list[str]:
    result: list[str] = []
    index = 0
    while index < len(value):
        if value[index].isspace() or value[index] in ",.ˈˌ":
            index += 1
            continue
        matched = next(
            (phone for phone in MULTI_PHONES if value.startswith(phone, index)), None
        )
        if matched is not None:
            result.append(matched)
            index += len(matched)
            continue
        if value[index] in "ː123456" and result:
            result[-1] += value[index]
        else:
            result.append(value[index])
        index += 1
    return result


def edit_distance(left: list[str], right: list[str]) -> int:
    row = list(range(len(right) + 1))
    for i, lhs in enumerate(left, 1):
        new = [i]
        for j, rhs in enumerate(right, 1):
            new.append(min(new[-1] + 1, row[j] + 1, row[j - 1] + (lhs != rhs)))
        row = new
    return row[-1]


def evaluate(path: Path) -> tuple[float, float]:
    pipeline = Pipeline()
    phone_errors = phone_total = lang_correct = lang_total = 0
    with path.open(encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            if not line.strip():
                continue
            row = json.loads(line)
            if row["kind"] == "g2p" and row.get("oov", False):
                expected = phones(row["expected"])
                actual = phones(pipeline.phonemize(row["text"]))
                phone_errors += edit_distance(actual, expected)
                phone_total += len(expected)
            elif row["kind"] == "langid":
                actual = [
                    token.language
                    for token in pipeline.analyze(row["text"]).tokens
                    if token.language != "punc"
                ]
                expected = row["languages"]
                if len(actual) != len(expected):
                    raise ValueError(
                        f"line {line_number}: language label length does not match tokens"
                    )
                lang_correct += sum(a == b for a, b in zip(actual, expected))
                lang_total += len(expected)
    if phone_total == 0 or lang_total == 0:
        raise ValueError("corpus needs both OOV g2p and langid rows")
    return phone_errors / phone_total, lang_correct / lang_total


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("corpus", type=Path)
    args = parser.parse_args()
    per, accuracy = evaluate(args.corpus)
    print(f"oov_phone_error_rate={per:.4%}")
    print(f"code_switch_token_accuracy={accuracy:.4%}")
    if per > 0.08 or accuracy < 0.98:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
