<p align="center">
  <img src="https://raw.githubusercontent.com/DongLaoAI/donglao-g2p/main/assets/donglao-g2p-logo.png" width="200" alt="DongLao G2P logo">
</p>

<h1 align="center">donglao-g2p</h1>

<p align="center">
  Fast Vietnamese–English text normalization and grapheme-to-phoneme conversion for TTS.
</p>

<p align="center">
  <strong>English</strong> · <a href="https://github.com/DongLaoAI/donglao-g2p/blob/main/README.vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <img alt="Python 3.9–3.13" src="https://img.shields.io/badge/Python-3.9%E2%80%933.13-3776AB?logo=python&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-000000?logo=rust&logoColor=white">
  <img alt="Apache 2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue">
  <img alt="Project status: stable" src="https://img.shields.io/badge/status-stable-3fb950">
</p>

`donglao-g2p` is a Rust-backed Python package for preparing Vietnamese,
English, and code-switched text for speech synthesis. Language selection is
automatic; input text does not require language tags.

```text
Hôm nay tôi có meeting John.
→ hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn.
```

The project targets Hanoi Vietnamese and broad General American English. The
public API and the phoneme output convention are stable from 1.0.0 on; any
change to either is a breaking change and gets a major version. Pronunciation
itself is a judgement call, so evaluate it on your own speakers and domains
before using generated phonemes as training labels.

## Why donglao-g2p?

- Vietnamese text normalization and rule-based syllable G2P.
- Automatic sentence-context Vietnamese–English routing, with corpus-frequency
  priors for ASCII spellings that both languages claim.
- CMUdict-backed English pronunciation with a graphone OOV fallback.
- Compact phonemic output with Vietnamese tone suffixes `1–6`.
- Custom spoken-form and phoneme lexicons.
- Deterministic, thread-safe pipelines.
- GIL-free parallel batch processing through Rayon.
- Typed Python API, CLI, ABI3 wheels, and evaluation tools.
- Apache-2.0 licensed for open-source and commercial use.

## Installation

Python 3.9 or newer is required. Release wheels are built for Linux x86-64 and
aarch64 (manylinux2014). They are ABI3 wheels, so one wheel per architecture
covers every supported interpreter.

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
cd donglao-g2p
uv sync --dev
uv run pytest
```

The equivalent pip workflow is:

```bash
git clone https://github.com/DongLaoAI/donglao-g2p.git
cd donglao-g2p
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
# hai mươi lăm ki-lô-gam lúc mười hai giờ ba mươi phút

print(g2p.phonemize("Hôm nay tôi có meeting John."))
# hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn.
```

Create one pipeline per process and reuse it:

```python
g2p = Pipeline(
    ensure_terminal=False,
    decimal_style="cardinal",
    language="auto",
    num_threads=None,
)
```

`Pipeline` is immutable and safe to share between threads.

## API

### Normalize text

```python
g2p.normalize("Giá trị là 3,14 kg")
# giá trị là ba phẩy mười bốn ki-lô-gam

g2p.normalize_batch(["25 kg", "12:30"])
```

Normalization covers numbers, grouped and decimal values, dates, time,
currency, measurement units, percentages, ranges, phone numbers, URLs, email,
versions, acronyms, Unicode punctuation, and custom spoken forms.

### Select a language

Automatic sentence-context routing remains the default:

```python
Pipeline(language="auto")
```

Force one language when the caller already knows it:

```python
vi = Pipeline(language="vi")
en = Pipeline(language="en")

vi.normalize("20 kg")  # hai mươi ki-lô-gam
en.normalize("20 kg")  # twenty kilograms
```

Forced mode applies to the entire input, including normalization and G2P.
Do not force a language for code-switched text unless that is intentional.
It also bypasses routing entirely, so none of the evidence described below is
consulted.

In `auto` mode the router works per token over the whole sentence, not per
sentence. Evidence, strongest first:

1. A Vietnamese diacritic anywhere in the token decides it outright.
2. For a bare ASCII spelling that is both a legal Vietnamese syllable and an
   English dictionary word, a built-in frequency table decides. Dictionary
   membership alone used to hand these to English, which is why `theo` was read
   as `θiːoʊ` and `ba` as the initialism `biːeɪ`.
3. A sentence already carrying Vietnamese diacritics pulls its remaining
   undecided ASCII tokens toward Vietnamese. Capitalized tokens away from the
   start of a segment are exempt, so `South Australia Loop` and `The Velvet
   Rope` keep their English reading.
4. Otherwise the switch cost keeps a token with its neighbours.

Only a full stop ends a routing segment. Commas stay transparent, so a word
fenced by them keeps the surrounding context:

```python
g2p.phonemize("phía đông, nam, dãy đồi.", normalize=False)
# fiə5 ɗoŋ1, naːm1, zaj4 ɗoj2.   ("nam" stays Vietnamese)
```

Structured expressions use the lexical context of the input to choose an
English or Vietnamese verbalizer. Inputs with no lexical evidence retain the
Vietnamese default for compatibility:

```text
I have 20 apples. → I have twenty apples.
Tôi có 20 quả táo. → tôi có hai mươi quả táo.
20 kg → hai mươi ki-lô-gam.
```

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
# ba chấm một bốn và ba phẩy một bốn
```

### Phonemize

```python
g2p.phonemize("Hôm nay OpenAI có meeting.")
# hom1 naj1 oʊpən eɪ aɪ kɔ5 miːtɪŋ.
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

For millions of records, use the bounded-memory iterators instead of building
one very large Python list:

```python
for phones in g2p.phonemize_iter(records, batch_size=4096):
    write_result(phones)

for normalized in g2p.normalize_iter(records, batch_size=4096):
    write_result(normalized)
```

Production tuning guidelines:

- Create and warm one `Pipeline` per process; do not construct it per request.
- Prefer batches of roughly 2,000–10,000 short sentences for offline jobs.
  The default iterator chunk of 4,096 is a practical starting point.
- Aggregate synchronous service requests into short micro-batches when latency
  permits. Batches below 64 items deliberately avoid Rayon scheduling overhead.
- With multiple process workers, divide the container CPU quota among their
  `num_threads` values to avoid oversubscription.
- Use `phonemize(..., normalize=False)` only when the upstream text is already
  canonical; this skips normalization but changes the caller contract.

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
`english_oov:<word>` warning. Unsupported scripts or symbols produce `<unk>`
and an `unsupported_token:<token>` warning instead of disappearing silently.

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
model. The current schema is identified by
`donglao_g2p.__phoneme_profile__ == "compact-v2"`.

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

Semicolons, colons, standalone dashes, medial ellipses, question marks, and
exclamation marks become commas. Terminal ellipses become periods.
Terminal punctuation is not added automatically. Set `ensure_terminal=True`
to append a period when the input has no terminal punctuation.

## CLI

```bash
donglao-g2p "Hôm nay tôi có meeting John."
donglao-g2p --normalize-only "25 kg lúc 12:30"
donglao-g2p --analyze "Hôm nay có planning."
donglao-g2p --decimal-style digits "3.14"
donglao-g2p --language en "20 kg"
donglao-g2p --no-normalize "hôm nay, tôi có meeting."
donglao-g2p --ensure-terminal "xin chào"
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
dictionary membership, capitalization, neighboring tokens, sentence-level
diacritic evidence, and a language switch cost. Routing segments are bounded by
full stops only.

Roughly 875 bare ASCII spellings are simultaneously a legal Vietnamese syllable
and a CMUdict entry, and membership alone cannot separate them. `src/lang_prior.rs`
resolves the 488 of those that a corpus can settle: each cost is a log frequency
ratio measured over 42.5 million Vietnamese and 28.7 million English tokens,
scaled so a single mid-confidence token cannot override a decisive run of the
other language. Vietnamese counts are for the exact surface string and are
deliberately not folded over diacritics — folding conflates `đo`, `đó`, `độ` and
`dò` into `do` and drags genuine English toward Vietnamese. The table is
generated and compiled in; the crate ships no runtime data files.

## Validation

Run the correctness suite:

```bash
cargo test --locked
pytest
```

`.github/workflows/ci.yml` runs the same suite on every push, builds the wheel in
a manylinux2014 container, and installs that exact artifact on Python 3.9 and
3.13 to check the ABI3 claim. Tagging `v*` runs `.github/workflows/release.yml`,
which repeats those gates and adds `cargo audit`, a CycloneDX SBOM, SHA256SUMS,
cosign signatures, and publication to PyPI. Bump the version with
`scripts/bump-version.sh <version>`; it keeps `Cargo.toml`, `pyproject.toml`,
`Cargo.lock` and `uv.lock` in agreement, which the release workflow verifies
against the tag before building anything.

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

For a streaming `language|text` debug corpus:

```bash
python evaluation/evaluate_unique.py debugs/unique.csv
```

Text-only metadata does not contain gold phonemes and therefore cannot measure
true pronunciation accuracy. Cross-system agreement is also not a gold
standard.

## Known limitations

- Vietnamese pronunciation targets the Hanoi dialect.
- English OOV names and loanwords may require overrides.
- Undiacriticized Vietnamese cannot be read correctly, and no amount of routing
  fixes it: `ban` stands for `bàn`, `bán`, `bản` and `bạn`, and the tone is not
  recoverable from the spelling. Restore diacritics before phonemizing.
- Vietnamese loanwords that are not a single legal syllable (`axit`, `oxy`,
  `campuchia`) fail the syllable check, never reach the frequency table, and
  fall through to the English OOV path. Use overrides for the ones you care
  about.
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
