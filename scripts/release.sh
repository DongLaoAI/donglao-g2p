#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"

cargo test --locked
if command -v cargo-audit >/dev/null 2>&1; then
  cargo audit
else
  echo "cargo-audit is required for a production release" >&2
  exit 2
fi
maturin build --release --locked
python3 -m pip install --force-reinstall target/wheels/donglao_g2p-*.whl
python3 -m pytest
python3 benchmarks/benchmark.py
if [[ -z "${DONGLAO_G2P_EVAL_CORPUS:-}" ]]; then
  echo "DONGLAO_G2P_EVAL_CORPUS is required for linguistic release gates" >&2
  exit 2
fi
python3 evaluation/evaluate.py "$DONGLAO_G2P_EVAL_CORPUS"

sha256sum target/wheels/*.whl > target/wheels/SHA256SUMS

if command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo cyclonedx --format json
  cp donglao-g2p.cdx.json target/wheels/
else
  echo "cargo-cyclonedx is required for a signed production release" >&2
  exit 2
fi

if command -v cosign >/dev/null 2>&1; then
  for wheel in target/wheels/*.whl; do
    cosign sign-blob --yes --bundle "${wheel}.sigstore.json" "$wheel"
  done
else
  echo "cosign is required for a signed production release" >&2
  exit 2
fi
