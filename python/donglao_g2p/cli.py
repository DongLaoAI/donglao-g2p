from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict

from . import Pipeline


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="donglao-g2p")
    parser.add_argument("text", nargs="*", help="text; stdin is used when omitted")
    parser.add_argument(
        "--normalize-only", action="store_true", help="print normalized text"
    )
    parser.add_argument("--analyze", action="store_true", help="print a JSON trace")
    parser.add_argument(
        "--no-terminal", action="store_true", help="do not append terminal punctuation"
    )
    parser.add_argument(
        "--no-normalize",
        action="store_true",
        help="phonemize input as-is without text normalization",
    )
    parser.add_argument(
        "--decimal-style",
        choices=("cardinal", "digits"),
        default="cardinal",
        help="read fractional digits as a cardinal number or digit by digit",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    text = " ".join(args.text) if args.text else sys.stdin.read().strip()
    pipeline = Pipeline(
        ensure_terminal=not args.no_terminal,
        decimal_style=args.decimal_style,
    )
    if args.analyze:
        print(json.dumps(asdict(pipeline.analyze(text)), ensure_ascii=False, indent=2))
    elif args.normalize_only:
        print(pipeline.normalize(text))
    else:
        print(pipeline.phonemize(text, normalize=not args.no_normalize))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
