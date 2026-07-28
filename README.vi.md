<p align="center">
  <img src="assets/donglao-g2p-logo.png" width="200" alt="Logo DongLao G2P">
</p>

<h1 align="center">donglao-g2p</h1>

<p align="center">
  Chuẩn hóa văn bản và chuyển chữ viết thành âm vị Việt–Anh tốc độ cao cho TTS.
</p>

<p align="center">
  <a href="README.md">English</a> · <strong>Tiếng Việt</strong>
</p>

<p align="center">
  <img alt="Python 3.9–3.13" src="https://img.shields.io/badge/Python-3.9%E2%80%933.13-3776AB?logo=python&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-000000?logo=rust&logoColor=white">
  <img alt="Apache 2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue">
  <img alt="Trạng thái dự án: alpha" src="https://img.shields.io/badge/status-alpha-f59e0b">
</p>

`donglao-g2p` là package Python có Rust core, dùng để chuẩn bị văn bản tiếng
Việt, tiếng Anh và câu code-switch cho tổng hợp tiếng nói. Hệ thống tự xác định
ngôn ngữ; input không cần tag.

```text
Hôm nay tôi có meeting John.
→ hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn .
```

Dự án hướng đến tiếng Việt giọng Hà Nội và tiếng Anh General American dạng
broad. Phiên bản hiện tại là alpha: cần đánh giá trên giọng đọc và miền dữ liệu
của bạn trước khi dùng phoneme làm nhãn train.

## Vì sao dùng donglao-g2p?

- Chuẩn hóa văn bản và G2P âm tiết tiếng Việt bằng luật.
- Tự động định tuyến Việt–Anh dựa trên ngữ cảnh câu.
- Tiếng Anh dùng CMUdict và graphone fallback cho OOV.
- Output phonemic gọn với hậu tố thanh tiếng Việt `1–6`.
- Custom lexicon cho dạng đọc và phoneme.
- Pipeline deterministic, immutable và thread-safe.
- Batch song song bằng Rayon, giải phóng GIL.
- Typed Python API, CLI, ABI3 wheel và công cụ đánh giá.
- Giấy phép Apache-2.0 cho phép sử dụng open-source và thương mại.

## Cài đặt

Yêu cầu Python 3.9 trở lên. Release wheel hiện hướng đến Linux x86-64 và ARM64.

Cài gói đã phát hành bằng pip:

```bash
python -m pip install donglao-g2p
```

Thêm vào project đang được uv quản lý:

```bash
uv add donglao-g2p
```

Hoặc cài vào virtual environment do uv quản lý:

```bash
uv venv
uv pip install donglao-g2p
```

Khi chưa phát hành release, có thể cài wheel được build cục bộ bằng một trong
hai công cụ:

```bash
python -m pip install target/wheels/donglao_g2p-*.whl
uv pip install target/wheels/donglao_g2p-*.whl
```

Phát triển trực tiếp từ mã nguồn bằng uv:

```bash
git clone <repository-url>
cd donglao_g2p
uv sync --dev
uv run pytest
```

Quy trình tương đương bằng pip:

```bash
git clone <repository-url>
cd donglao_g2p
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin pytest
maturin develop --release --locked
pytest
```

## Bắt đầu nhanh

```python
from donglao_g2p import Pipeline

g2p = Pipeline()

print(g2p.normalize("25 kg lúc 12:30"))
# hai mươi lăm ki-lô-gam lúc mười hai giờ ba mươi phút.

print(g2p.phonemize("Hôm nay tôi có meeting John."))
# hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn .
```

Tạo một pipeline cho mỗi process và tái sử dụng:

```python
g2p = Pipeline(
    ensure_terminal=True,
    decimal_style="cardinal",
    num_threads=None,
)
```

`Pipeline` immutable và có thể chia sẻ an toàn giữa các thread.

## API

### Chuẩn hóa văn bản

```python
g2p.normalize("Giá trị là 3,14 kg")
# giá trị là ba phẩy mười bốn ki-lô-gam.

g2p.normalize_batch(["25 kg", "12:30"])
```

Normalization bao phủ số, số có phân nhóm và phần thập phân, ngày giờ, tiền tệ,
đơn vị, phần trăm, khoảng, số điện thoại, URL, email, phiên bản, acronym, dấu
câu Unicode và custom spoken form.

Quy tắc số nhận biết locale:

```text
3.14       → ba chấm mười bốn
3,14       → ba phẩy mười bốn
0.05       → không chấm không năm
1.234      → một nghìn hai trăm ba mươi tư
12.345,67  → ... phẩy sáu mươi bảy
12,345.67  → ... chấm sáu mươi bảy
```

Với dữ liệu kỹ thuật cần đọc riêng từng chữ số:

```python
digits = Pipeline(decimal_style="digits")
digits.normalize("3.14 và 3,14")
# ba chấm một bốn và ba phẩy một bốn.
```

### Chuyển thành phoneme

```python
g2p.phonemize("Hôm nay OpenAI có meeting.")
# hom1 naj1 oʊpən eɪ aɪ kɔ5 miːtɪŋ .
```

Normalization được bật mặc định. Chỉ tắt khi input đã ở dạng canonical:

```python
g2p.phonemize("hôm nay, tôi có meeting.", normalize=False)
g2p.phonemize_batch(normalized_texts, normalize=False)
```

Khi `normalize=False`, caller phải tự mở rộng số, ký hiệu và dùng dấu câu
canonical.

### Xử lý batch

```python
texts = [
    "Xin chào.",
    "Nice to meet you.",
    "Hôm nay có planning.",
]

phones = g2p.phonemize_batch(texts)
```

Batch giữ nguyên thứ tự và giải phóng Python GIL. Với service chạy nhiều
process, điểm khởi đầu hợp lý là:

```text
num_threads = số CPU khả dụng / số worker process
```

Sau đó benchmark trong đúng CPU quota của production.

### Xem quyết định ngôn ngữ và OOV

```python
analysis = g2p.analyze("Hôm nay OpenAI có planning.")

print(analysis.normalized)
print(analysis.phonemes)
print(analysis.warnings)

for token in analysis.tokens:
    print(token.token, token.language, token.source, token.phonemes)
```

Nhãn ngôn ngữ là `vi`, `en` hoặc `punc`. Từ tiếng Anh ngoài từ điển sinh cảnh
báo `english_oov:<word>`.

### Thêm pronunciation override

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

Nên dùng explicit phoneme cho tên người, sản phẩm, abbreviation và thuật ngữ
chuyên ngành.

## Quy ước output

Output tiếng Việt là biểu diễn phonemic gọn, không phải narrow phonetic IPA.
Duration và coarticulation có thể dự đoán được sẽ do acoustic model học.

Ví dụ:

```text
hôm → hom1
nay → naj1
tôi → toj1
tai → taːj1
tay → taj1
```

Hậu tố thanh:

| Suffix | Thanh |
|---:|---|
| `1` | ngang |
| `2` | huyền |
| `3` | hỏi |
| `4` | ngã |
| `5` | sắc |
| `6` | nặng |

Tiếng Anh dùng broad General American IPA và bỏ lexical stress mark. `OpenAI`
được giữ là token tiếng Anh và đọc `oʊpən eɪ aɪ`; chỉ override khi chủ đích cần
cách đọc Việt hóa.

### Dấu câu

Public output chỉ có hai prosody token:

| Token | Chức năng |
|---|---|
| `,` | ngắt trung gian |
| `.` | kết thúc câu |

Dấu chấm phẩy, hai chấm, dash đứng độc lập và ellipsis giữa câu được đổi thành
dấu phẩy. Dấu hỏi, cảm thán và ellipsis cuối câu được đổi thành dấu chấm. Dùng
`ensure_terminal=False` để tắt tự động thêm dấu kết câu.

## CLI

```bash
donglao-g2p "Hôm nay tôi có meeting John."
donglao-g2p --normalize-only "25 kg lúc 12:30"
donglao-g2p --analyze "Hôm nay có planning."
donglao-g2p --decimal-style digits "3.14"
donglao-g2p --no-normalize "hôm nay, tôi có meeting."
donglao-g2p --no-terminal "xin chào"
```

Nếu không truyền text, CLI đọc UTF-8 từ standard input:

```bash
printf 'Xin chào.' | donglao-g2p
```

## Phương pháp

```text
Unicode NFC
  → bảo vệ biểu thức có cấu trúc
  → chuẩn hóa văn bản
  → canonicalize dấu câu
  → định tuyến ngôn ngữ theo ngữ cảnh câu
  → luật tiếng Việt hoặc English dictionary/OOV G2P
  → render compact phoneme
```

Luật tiếng Việt phân tích âm đầu, âm chính, âm cuối và thanh. Pronunciation
tiếng Anh được chuyển từ ARPAbet sang IPA. Bộ giải mã Viterbi chọn ngôn ngữ cho
từng token dựa trên chữ viết, tính hợp lệ của âm tiết, dictionary,
capitalization, token lân cận và language-switch cost.

## Kiểm định

Chạy correctness suite:

```bash
cargo test --locked
pytest
```

Chạy benchmark tài nguyên với 50.000 câu:

```bash
python tests/benchmark_batch.py
python tests/benchmark_batch.py --materialize-inputs
python tests/benchmark_batch.py --threads 8 --json > benchmark.json
```

Trên AMD Ryzen Threadripper 9960X với 48 logical CPU, một câu lặp lại dài 62 ký
tự đạt khoảng 485.000 câu/giây hoặc 30 triệu ký tự/giây, peak RSS khoảng 100
MiB. Đây là số tham khảo, không phải cam kết hiệu năng trên mọi máy.

Release gate ngôn ngữ cần JSONL đã được con người kiểm duyệt:

```bash
python evaluation/evaluate.py /path/to/reviewed-evaluation.jsonl
```

Metadata evaluator đo routing proxy, OOV coverage, invariant, latency và
throughput:

```bash
python evaluation/evaluate_metadata.py
```

Metadata chỉ có text không chứa gold phoneme, vì vậy không thể đo độ chính xác
phát âm thực. Việc hai hệ G2P cho cùng output cũng không phải gold standard.

## Giới hạn

- Phát âm tiếng Việt hướng đến giọng Hà Nội.
- English OOV, tên riêng và từ mượn có thể cần override.
- Không thể suy ra đúng mọi số hoặc abbreviation nhập nhằng chỉ từ text.
- Public output không biểu diễn lexical stress tiếng Anh.
- Chính sách hai dấu câu không giữ prosody riêng của câu hỏi và cảm thán.
- Package chuẩn bị text và phoneme; không train hoặc serve TTS acoustic model.

## Đóng góp

Mọi đóng góp đều được chào đón. Đọc [CONTRIBUTING.md](CONTRIBUTING.md) trước
khi mở pull request. Thay đổi ngôn ngữ phải có golden test tối thiểu và ghi rõ
phương ngữ hoặc quy ước phát âm.

Không đóng góp dictionary hoặc dataset khi chưa có quyền phân phối rõ ràng.

## Dữ liệu và attribution

English dictionary dựa trên CMUdict 0.7b. CMUdict cho phép dùng trong nghiên cứu
và thương mại, đồng thời yêu cầu ghi nhận nguồn khi phân phối lại. Attribution
được giữ trong [NOTICE](NOTICE).

Phiên bản chính xác của Rust dependency được pin trong `Cargo.lock`. Dự án
không chứa mã nguồn, model, dictionary hoặc binary từ `sea-g2p`.

## Giấy phép

Copyright 2026 DongLao.

Dự án dùng [Apache License 2.0](LICENSE). Bạn có thể sử dụng, sửa đổi và phân
phối, kể cả cho mục đích thương mại, với điều kiện tuân thủ giấy phép và giữ
các attribution notice.
