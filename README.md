<p align="center">
  <img src="assets/donglao-g2p-logo.png" width="200" alt="DongLao G2P logo">
</p>

<h1 align="center">donglao-g2p</h1>

<p align="center">
  Fast Vietnamese–English text normalization and grapheme-to-phoneme conversion for TTS.
</p>

<p align="center">
  <strong>English</strong> · <a href="README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <img alt="Python 3.9–3.13" src="https://img.shields.io/badge/Python-3.9%E2%80%933.13-3776AB?logo=python&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-000000?logo=rust&logoColor=white">
  <img alt="Apache 2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue">
  <img alt="Project status: alpha" src="https://img.shields.io/badge/status-alpha-f59e0b">
</p>

`donglao-g2p` is a Rust-backed Python package for preparing Vietnamese,
English, and code-switched text for speech synthesis. Language selection is
automatic; input text does not require language tags.

```text
Hôm nay tôi có meeting John.
→ hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn .
```

The project targets Hanoi Vietnamese and broad General American English. It is
currently alpha: evaluate pronunciation on your own speakers and domains
before using generated phonemes as training labels.

## Why donglao-g2p?

- Vietnamese text normalization and rule-based syllable G2P.
- Automatic sentence-context Vietnamese–English routing.
- CMUdict-backed English pronunciation with a graphone OOV fallback.
- Compact phonemic output with Vietnamese tone suffixes `1–6`.
- Custom spoken-form and phoneme lexicons.
- Deterministic, thread-safe pipelines.
- GIL-free parallel batch processing through Rayon.
- Typed Python API, CLI, ABI3 wheels, and evaluation tools.
- Apache-2.0 licensed for open-source and commercial use.

## Installation

Python 3.9 or newer is required. Release wheels currently target Linux x86-64
and ARM64.

Install the published package with pip:

```bash
python -m pip install donglao-g2p
```

Add it to a uv-managed project:

```bash
uv add donglao-g2p
```

Or install it into a uv-managed virtual environment:

```bash
uv venv
uv pip install donglao-g2p
```

Until a release is published, install a locally built wheel with either tool:

```bash
python -m pip install target/wheels/donglao_g2p-*.whl
uv pip install target/wheels/donglao_g2p-*.whl
```

For development from source with uv:

```bash
git clone https://github.com/DongLaoAI/donglao-g2p.git
cd donglao_g2p
uv sync --dev
uv run pytest
```

The equivalent pip workflow is:

```bash
git clone https://github.com/DongLaoAI/donglao-g2p.git
cd donglao_g2p
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin pytest
maturin develop --release --locked
pytest
```

## Quick start

```python
from donglao_g2p import Pipeline

g2p = Pipeline()

print(g2p.normalize("25 kg lúc 12:30"))
# hai mươi lăm ki-lô-gam lúc mười hai giờ ba mươi phút.

print(g2p.phonemize("Hôm nay tôi có meeting John."))
# hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn .
```

Create one pipeline per process and reuse it:

```python
g2p = Pipeline(
    ensure_terminal=True,
    decimal_style="cardinal",
    num_threads=None,
)
```

`Pipeline` is immutable and safe to share between threads.

## API

### Normalize text

```python
g2p.normalize("Giá trị là 3,14 kg")
# giá trị là ba phẩy mười bốn ki-lô-gam.

g2p.normalize_batch(["25 kg", "12:30"])
```

Normalization covers numbers, grouped and decimal values, dates, time,
currency, measurement units, percentages, ranges, phone numbers, URLs, email,
versions, acronyms, Unicode punctuation, and custom spoken forms.

Decimal notation is locale-aware:

```text
3.14       → ba chấm mười bốn
3,14       → ba phẩy mười bốn
0.05       → không chấm không năm
1.234      → một nghìn hai trăm ba mươi tư
12.345,67  → ... phẩy sáu mươi bảy
12,345.67  → ... chấm sáu mươi bảy
```

Use digit-by-digit fractional reading for technical data:

```python
digits = Pipeline(decimal_style="digits")
digits.normalize("3.14 và 3,14")
# ba chấm một bốn và ba phẩy một bốn.
```

### Phonemize

```python
g2p.phonemize("Hôm nay OpenAI có meeting.")
# hom1 naj1 oʊpən eɪ aɪ kɔ5 miːtɪŋ .
```

Normalization is enabled by default. Disable it only for canonical,
pre-normalized input:

```python
g2p.phonemize("hôm nay, tôi có meeting.", normalize=False)
g2p.phonemize_batch(normalized_texts, normalize=False)
```

When `normalize=False`, the caller must expand numbers and symbols and use
canonical punctuation.

### Process batches

```python
texts = [
    "Xin chào.",
    "Nice to meet you.",
    "Hôm nay có planning.",
]

phones = g2p.phonemize_batch(texts)
```

Batch methods preserve order and release the Python GIL. For multi-process
services, start with approximately:

```text
num_threads = available CPUs / worker processes
```

Then benchmark inside the actual production CPU quota.

### Inspect language and OOV decisions

```python
analysis = g2p.analyze("Hôm nay OpenAI có planning.")

print(analysis.normalized)
print(analysis.phonemes)
print(analysis.warnings)

for token in analysis.tokens:
    print(token.token, token.language, token.source, token.phonemes)
```

Token languages are `vi`, `en`, or `punc`. Unknown English words produce an
`english_oov:<word>` warning.

### Add pronunciation overrides

```python
from donglao_g2p import LexiconEntry, Pipeline

g2p = Pipeline(
    overrides={
        "DongLao": LexiconEntry(
            phonemes="dɔŋ1 laːw1",
            language="vi",
            case_sensitive=True,
        ),
        "canxi": LexiconEntry(
            spoken="can-xi",
            language="vi",
        ),
    }
)
```

Explicit phonemes are recommended for people, products, abbreviations, and
specialist vocabulary.

## Output convention

Vietnamese output is a compact phonemic representation rather than narrow
phonetic IPA. Predictable duration and coarticulation are left to the acoustic
model.

Examples:

```text
hôm → hom1
nay → naj1
tôi → toj1
tai → taːj1
tay → taj1
```

Tone suffixes:

| Suffix | Vietnamese tone |
|---:|---|
| `1` | ngang |
| `2` | huyền |
| `3` | hỏi |
| `4` | ngã |
| `5` | sắc |
| `6` | nặng |

English output uses broad General American IPA without lexical stress marks.
`OpenAI` remains an English token and is pronounced `oʊpən eɪ aɪ`; use an
override only when a Vietnamese-localized reading is intentional.

### Punctuation

Public output uses only two prosodic tokens:

| Token | Function |
|---|---|
| `,` | intermediate pause |
| `.` | sentence boundary |

Semicolons, colons, standalone dashes, and medial ellipses become commas.
Question marks, exclamation marks, and terminal ellipses become periods.
Set `ensure_terminal=False` to disable automatic terminal punctuation.

## CLI

```bash
donglao-g2p "Hôm nay tôi có meeting John."
donglao-g2p --normalize-only "25 kg lúc 12:30"
donglao-g2p --analyze "Hôm nay có planning."
donglao-g2p --decimal-style digits "3.14"
donglao-g2p --no-normalize "hôm nay, tôi có meeting."
donglao-g2p --no-terminal "xin chào"
```

The CLI reads UTF-8 from standard input when text is omitted:

```bash
printf 'Xin chào.' | donglao-g2p
```

## Method

```text
Unicode NFC
  → protect structured expressions
  → text normalization
  → punctuation canonicalization
  → sentence-context language routing
  → Vietnamese rules or English dictionary/OOV G2P
  → compact phoneme rendering
```

Vietnamese rules operate on onset, nucleus, coda, and tone. English dictionary
pronunciations are converted from ARPAbet to IPA. A Viterbi decoder selects
Vietnamese or English for each token using orthography, syllable validity,
dictionary membership, capitalization, neighboring tokens, and a language
switch cost.

## Validation

Run the correctness suite:

```bash
cargo test --locked
pytest
```

Run the explicit 50,000-sentence resource benchmark:

```bash
python tests/benchmark_batch.py
python tests/benchmark_batch.py --materialize-inputs
python tests/benchmark_batch.py --threads 8 --json > benchmark.json
```

On an AMD Ryzen Threadripper 9960X with 48 logical CPUs, a repeated 62-character
sentence reached approximately 485,000 sentences/s or 30 million characters/s,
with about 100 MiB peak RSS. This is a reference measurement, not a portable
performance guarantee.

Linguistic release gates require a human-reviewed JSONL corpus:

```bash
python evaluation/evaluate.py /path/to/reviewed-evaluation.jsonl
```

The metadata evaluator measures routing proxies, OOV coverage, invariants,
latency, and throughput:

```bash
python evaluation/evaluate_metadata.py
```

Text-only metadata does not contain gold phonemes and therefore cannot measure
true pronunciation accuracy. Cross-system agreement is also not a gold
standard.

## Known limitations

- Vietnamese pronunciation targets the Hanoi dialect.
- English OOV names and loanwords may require overrides.
- Ambiguous numbers and abbreviations cannot always be resolved from text.
- English lexical stress is not represented in the public output.
- The two-token punctuation policy does not preserve question or exclamation
  prosody.
- The package prepares text and phonemes; it does not train or serve a TTS
  acoustic model.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before
opening a pull request. Linguistic changes must include a minimal golden test
and identify the intended dialect or pronunciation convention.

Please do not contribute dictionaries or datasets without clear redistribution
rights.

## Data and attribution

The English dictionary is based on CMUdict 0.7b. CMUdict permits research and
commercial use and requests acknowledgement when redistributed. Attribution is
retained in [NOTICE](NOTICE). Exact Rust dependency versions are pinned in `Cargo.lock`.

## License

Copyright 2026 DongLao.

Licensed under the [Apache License 2.0](LICENSE). You may use, modify, and
distribute this project, including commercially, subject to the license terms
and retained notices.
# donglao-g2p
