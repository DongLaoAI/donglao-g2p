from __future__ import annotations

import statistics
import sys
import time

from donglao_g2p import Pipeline


def main() -> None:
    pipeline = Pipeline()
    sentence = "Hôm nay tôi có meeting với John lúc 12:30, hành lý nặng 25 kg."
    batch = [sentence] * 10_000
    for _ in range(100):
        pipeline.phonemize(sentence)

    samples = []
    for _ in range(5_000):
        start = time.perf_counter_ns()
        pipeline.phonemize(sentence)
        samples.append((time.perf_counter_ns() - start) / 1_000_000)
    samples.sort()
    p95 = samples[int(len(samples) * 0.95)]

    start = time.perf_counter()
    pipeline.phonemize_batch(batch)
    elapsed = time.perf_counter() - start
    chars_per_second = sum(map(len, batch)) / elapsed
    print(f"median_ms={statistics.median(samples):.4f}")
    print(f"p95_ms={p95:.4f}")
    print(f"batch_chars_per_second={chars_per_second:.0f}")
    if p95 >= 1.0:
        print("FAIL: warm p95 must be below 1 ms", file=sys.stderr)
        raise SystemExit(1)
    if chars_per_second < 50_000:
        print("FAIL: batch throughput must exceed 50k chars/s", file=sys.stderr)
        raise SystemExit(1)


if __name__ == "__main__":
    main()
