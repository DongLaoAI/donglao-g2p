#!/usr/bin/env python3
"""Measure production batch throughput, CPU use, and process memory.

This is an explicit benchmark rather than a pytest test because its default
50,000-sentence workload should not slow down the normal correctness suite.
It has no third-party monitoring dependency and is intended for Linux.
"""

from __future__ import annotations

import argparse
import gc
import json
import os
import resource
import statistics
import sys
import time
from typing import Dict, List, Optional, Tuple

from donglao_g2p import Pipeline

DEFAULT_SENTENCE = (
    "Hôm nay tôi có meeting với John lúc 12:30, hành lý nặng 25 kg."
)


def _read_int(path: str) -> Optional[int]:
    try:
        with open(path, "r", encoding="ascii") as stream:
            return int(stream.read().strip())
    except (FileNotFoundError, PermissionError, ValueError):
        return None


def _current_rss_bytes() -> Optional[int]:
    try:
        with open("/proc/self/status", "r", encoding="ascii") as stream:
            for line in stream:
                if line.startswith("VmRSS:"):
                    return int(line.split()[1]) * 1024
    except (FileNotFoundError, PermissionError, ValueError, IndexError):
        pass
    return None


def _peak_rss_bytes() -> int:
    # Linux reports ru_maxrss in KiB; this benchmark is explicitly Linux-only.
    return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024


def _cgroup_cpu_stat() -> Dict[str, int]:
    result: Dict[str, int] = {}
    try:
        with open("/sys/fs/cgroup/cpu.stat", "r", encoding="ascii") as stream:
            for line in stream:
                key, value = line.split()
                result[key] = int(value)
    except (FileNotFoundError, PermissionError, ValueError):
        pass
    return result


def _cgroup_cpu_limit() -> Optional[float]:
    try:
        with open("/sys/fs/cgroup/cpu.max", "r", encoding="ascii") as stream:
            quota, period = stream.read().split()
        if quota == "max":
            return None
        return int(quota) / int(period)
    except (FileNotFoundError, PermissionError, ValueError, ZeroDivisionError):
        return None


def _available_cpus() -> int:
    try:
        return len(os.sched_getaffinity(0))
    except AttributeError:
        return os.cpu_count() or 1


def _resource_snapshot() -> Tuple[int, int, int, int]:
    usage = resource.getrusage(resource.RUSAGE_SELF)
    return (
        usage.ru_minflt,
        usage.ru_majflt,
        usage.ru_nvcsw,
        usage.ru_nivcsw,
    )


def _delta(after: Dict[str, int], before: Dict[str, int], key: str) -> int:
    return after.get(key, 0) - before.get(key, 0)


def _mib(value: Optional[int]) -> Optional[float]:
    return None if value is None else value / (1024 * 1024)


def _make_inputs(sentence: str, count: int, materialize: bool) -> List[str]:
    if not materialize:
        return [sentence] * count
    # Force independent Python string allocations to model a request payload.
    return [(sentence + "\0")[:-1] for _ in range(count)]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=50_000)
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--warmup", type=int, default=1_000)
    parser.add_argument("--sentence", default=DEFAULT_SENTENCE)
    parser.add_argument(
        "--operation", choices=("phonemize", "normalize"), default="phonemize"
    )
    parser.add_argument(
        "--no-normalize",
        action="store_true",
        help="skip normalization before phonemization",
    )
    parser.add_argument(
        "--threads",
        type=int,
        default=None,
        help="Rayon worker count; default lets the library choose",
    )
    parser.add_argument(
        "--materialize-inputs",
        action="store_true",
        help="allocate one distinct Python string per sentence",
    )
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    args = parser.parse_args()
    if args.count <= 0 or args.rounds <= 0 or args.warmup < 0:
        parser.error("count/rounds must be positive and warmup must be non-negative")
    if args.threads is not None and args.threads <= 0:
        parser.error("threads must be positive")
    return args


def main() -> None:
    args = parse_args()
    gc.collect()
    process_start_rss = _current_rss_bytes()
    init_start = time.perf_counter()
    pipeline = Pipeline(num_threads=args.threads)
    init_seconds = time.perf_counter() - init_start

    if args.operation == "phonemize":
        single = lambda text: pipeline.phonemize(
            text, normalize=not args.no_normalize
        )
        batch_fn = lambda texts: pipeline.phonemize_batch(
            texts, normalize=not args.no_normalize
        )
    else:
        single = pipeline.normalize
        batch_fn = pipeline.normalize_batch
    for _ in range(args.warmup):
        single(args.sentence)

    inputs = _make_inputs(args.sentence, args.count, args.materialize_inputs)
    input_rss = _current_rss_bytes()
    total_chars = len(args.sentence) * args.count
    expected = single(args.sentence)
    rounds = []

    for round_index in range(1, args.rounds + 1):
        gc.collect()
        rss_before = _current_rss_bytes()
        resource_before = _resource_snapshot()
        cgroup_before = _cgroup_cpu_stat()
        cpu_before = time.process_time()
        wall_before = time.perf_counter()

        outputs = batch_fn(inputs)

        wall_seconds = time.perf_counter() - wall_before
        cpu_seconds = time.process_time() - cpu_before
        cgroup_after = _cgroup_cpu_stat()
        resource_after = _resource_snapshot()
        rss_with_outputs = _current_rss_bytes()

        if len(outputs) != args.count or any(value != expected for value in outputs):
            raise AssertionError("batch output count/order/content is incorrect")

        minflt, majflt, voluntary, involuntary = (
            resource_after[index] - resource_before[index] for index in range(4)
        )
        rounds.append(
            {
                "round": round_index,
                "wall_seconds": wall_seconds,
                "cpu_seconds": cpu_seconds,
                "effective_cpu_cores": cpu_seconds / wall_seconds,
                "sentences_per_second": args.count / wall_seconds,
                "characters_per_second": total_chars / wall_seconds,
                "rss_before_mib": _mib(rss_before),
                "rss_with_outputs_mib": _mib(rss_with_outputs),
                "rss_delta_mib": (
                    None
                    if rss_before is None or rss_with_outputs is None
                    else _mib(rss_with_outputs - rss_before)
                ),
                "minor_page_faults": minflt,
                "major_page_faults": majflt,
                "voluntary_context_switches": voluntary,
                "involuntary_context_switches": involuntary,
                "cgroup_throttled_events": _delta(
                    cgroup_after, cgroup_before, "nr_throttled"
                ),
                "cgroup_throttled_seconds": _delta(
                    cgroup_after, cgroup_before, "throttled_usec"
                )
                / 1_000_000,
            }
        )
        del outputs

    affinity_cpus = _available_cpus()
    cgroup_limit = _cgroup_cpu_limit()
    effective_limit = min(
        affinity_cpus,
        cgroup_limit if cgroup_limit is not None else affinity_cpus,
    )
    target_workers = min(effective_limit, args.threads or effective_limit)
    median_wall = statistics.median(item["wall_seconds"] for item in rounds)
    median_cores = statistics.median(
        item["effective_cpu_cores"] for item in rounds
    )
    total_throttled = sum(item["cgroup_throttled_events"] for item in rounds)

    findings = []
    if total_throttled:
        findings.append(
            "CPU cgroup throttling occurred; raise the CPU quota or reduce Rayon threads."
        )
    if target_workers > 1.5 and median_cores < target_workers * 0.35:
        findings.append(
            "Low parallel CPU occupancy; benchmark with --threads values near the CPU quota."
        )
    if median_cores > effective_limit * 1.15:
        findings.append(
            "Measured CPU demand exceeds the cgroup quota, so throttling/queueing is likely."
        )
    rss_deltas = [
        item["rss_delta_mib"]
        for item in rounds
        if item["rss_delta_mib"] is not None
    ]
    if rss_deltas and max(rss_deltas) > 256:
        findings.append(
            "Large output allocation; split production requests into chunks of 2k-10k sentences."
        )
    if not findings:
        findings.append("No CPU, memory, or correctness anomaly detected.")

    report = {
        "configuration": {
            "operation": args.operation,
            "normalize_before_phonemize": not args.no_normalize,
            "sentence_count": args.count,
            "characters": total_chars,
            "sentence_characters": len(args.sentence),
            "rounds": args.rounds,
            "warmup": args.warmup,
            "rayon_threads": args.threads,
            "materialized_inputs": args.materialize_inputs,
            "logical_cpus": os.cpu_count(),
            "affinity_cpus": affinity_cpus,
            "cgroup_cpu_limit": cgroup_limit,
        },
        "pipeline_init_ms": init_seconds * 1_000,
        "process_start_rss_mib": _mib(process_start_rss),
        "input_ready_rss_mib": _mib(input_rss),
        "peak_process_rss_mib": _mib(_peak_rss_bytes()),
        "summary": {
            "median_wall_seconds": median_wall,
            "median_sentences_per_second": args.count / median_wall,
            "median_characters_per_second": total_chars / median_wall,
            "median_effective_cpu_cores": median_cores,
        },
        "rounds": rounds,
        "findings": findings,
    }

    if args.json:
        json.dump(report, sys.stdout, ensure_ascii=False, indent=2)
        print()
        return

    config = report["configuration"]
    summary = report["summary"]
    print(
        f"{config['operation']}: {args.count:,} sentences, "
        f"{total_chars:,} characters, {args.rounds} rounds"
    )
    print(
        f"CPU: affinity={affinity_cpus}, cgroup_limit={cgroup_limit}, "
        f"Rayon threads={args.threads or 'auto'}"
    )
    print(
        f"Init: {report['pipeline_init_ms']:.3f} ms; "
        f"RSS start/input/peak: {report['process_start_rss_mib']:.1f}/"
        f"{report['input_ready_rss_mib']:.1f}/"
        f"{report['peak_process_rss_mib']:.1f} MiB"
    )
    for item in rounds:
        print(
            f"round {item['round']}: {item['wall_seconds']:.4f} s, "
            f"{item['sentences_per_second']:,.0f} sent/s, "
            f"{item['characters_per_second']:,.0f} char/s, "
            f"CPU={item['effective_cpu_cores']:.2f} cores, "
            f"RSS delta={item['rss_delta_mib']:.1f} MiB, "
            f"throttled={item['cgroup_throttled_seconds']:.4f} s"
        )
    print(
        f"median: {summary['median_sentences_per_second']:,.0f} sent/s, "
        f"{summary['median_characters_per_second']:,.0f} char/s, "
        f"CPU={summary['median_effective_cpu_cores']:.2f} cores"
    )
    for finding in findings:
        print(f"finding: {finding}")


if __name__ == "__main__":
    main()
