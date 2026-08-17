# Contributing to donglao-g2p

Thank you for helping improve Vietnamese–English text normalization and G2P.
Bug reports, linguistic reviews, documentation, tests, and code changes are
welcome.

## Before opening an issue

- Search existing issues for the same normalization or pronunciation case.
- Include the exact input, actual output, expected output, dialect, and package
  version.
- For pronunciation changes, provide a reputable linguistic source or a
  human-reviewed example.
- Do not attach private speech data, credentials, or material that you cannot
  legally redistribute.

## Development setup

Requirements:

- Python 3.9 or newer
- Rust 1.83 or newer, as recorded in `rust-version` (required by pyo3 0.29).
  An older toolchain also cannot read the v4 `Cargo.lock`, so `--locked` builds
  fail before compilation starts.
- uv, or pip with `maturin` and `pytest`

Using uv:

```bash
uv sync --dev
uv run pytest
```

Using pip:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin pytest
maturin develop
pytest
```

## Required checks

```bash
cargo fmt --check
cargo test --locked
uv run pytest
python benchmarks/benchmark.py
```

When using the pip environment, replace `uv run pytest` with `pytest`.

Changes that affect batch execution should also run:

```bash
python tests/benchmark_batch.py --materialize-inputs
```

## Pull requests

Keep each pull request focused. It should:

1. Explain the problem and the intended linguistic or engineering behavior.
2. Add a minimal golden test for every changed output contract.
3. Add Unicode or property regression coverage when applicable.
4. Report before/after performance for changes on a hot path.
5. Update both `README.md` and `README.vi.md` when public behavior changes.

Avoid unrelated formatting or generated-file changes.

## Linguistic contributions

Vietnamese rules target the Hanoi dialect and use a compact phonemic
representation. English rules target broad General American pronunciation.
Please identify the target dialect when proposing a different realization.

`sea-g2p` may be used only as a black-box comparison. Do not copy source code,
models, dictionaries, test fixtures, or generated datasets from incompatible
projects.

Any contributed lexicon or corpus must include its origin, license, revision,
and checksum. Data without clear redistribution rights cannot be accepted.

## License

By intentionally submitting a contribution for inclusion in this project, you
agree that it is licensed under the Apache License 2.0, as described in
Section 5 of [LICENSE](LICENSE), unless you explicitly state otherwise.
