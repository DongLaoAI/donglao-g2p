<p align="center">
  <img src="https://raw.githubusercontent.com/DongLaoAI/donglao-g2p/main/assets/donglao-g2p-logo.png" width="200" alt="Logo DongLao G2P">
</p>

<h1 align="center">donglao-g2p</h1>

<p align="center">
  Chuẩn hóa văn bản và chuyển chữ viết thành âm vị Việt–Anh tốc độ cao cho TTS.
</p>

<p align="center">
  <a href="https://github.com/DongLaoAI/donglao-g2p/blob/main/README.md">English</a> · <strong>Tiếng Việt</strong>
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
→ hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn.
```

Dự án hướng đến tiếng Việt giọng Hà Nội và tiếng Anh General American dạng
broad. Phiên bản hiện tại là alpha: cần đánh giá trên giọng đọc và miền dữ liệu
của bạn trước khi dùng phoneme làm nhãn train.

## Vì sao dùng donglao-g2p?

- Chuẩn hóa văn bản và G2P âm tiết tiếng Việt bằng luật.
- Tự động định tuyến Việt–Anh dựa trên ngữ cảnh câu, kèm prior tần suất corpus
  cho những chuỗi ASCII mà cả hai ngôn ngữ đều nhận là của mình.
- Tiếng Anh dùng CMUdict và graphone fallback cho OOV.
- Output phonemic gọn với hậu tố thanh tiếng Việt `1–6`.
- Custom lexicon cho dạng đọc và phoneme.
- Pipeline deterministic, immutable và thread-safe.
- Batch song song bằng Rayon, giải phóng GIL.
- Typed Python API, CLI, ABI3 wheel và công cụ đánh giá.
- Giấy phép Apache-2.0 cho phép sử dụng open-source và thương mại.

## Cài đặt

Yêu cầu Python 3.9 trở lên. Release wheel được build cho Linux x86-64 và
aarch64 (manylinux2014). Đây là wheel ABI3 nên một wheel cho mỗi kiến trúc là đủ
cho mọi phiên bản Python được hỗ trợ.

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
git clone https://github.com/DongLaoAI/donglao-g2p.git
cd donglao-g2p
uv sync --dev
uv run pytest
```

Quy trình tương đương bằng pip:

```bash
git clone https://github.com/DongLaoAI/donglao-g2p.git
cd donglao-g2p
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
# hai mươi lăm ki-lô-gam lúc mười hai giờ ba mươi phút

print(g2p.phonemize("Hôm nay tôi có meeting John."))
# hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn.
```

Tạo một pipeline cho mỗi process và tái sử dụng:

```python
g2p = Pipeline(
    ensure_terminal=False,
    decimal_style="cardinal",
    language="auto",
    num_threads=None,
)
```

`Pipeline` immutable và có thể chia sẻ an toàn giữa các thread.

## API

### Chuẩn hóa văn bản

```python
g2p.normalize("Giá trị là 3,14 kg")
# giá trị là ba phẩy mười bốn ki-lô-gam

g2p.normalize_batch(["25 kg", "12:30"])
```

Normalization bao phủ số, số có phân nhóm và phần thập phân, ngày giờ, tiền tệ,
đơn vị, phần trăm, khoảng, số điện thoại, URL, email, phiên bản, acronym, dấu
câu Unicode và custom spoken form.

### Chọn ngôn ngữ

Mặc định pipeline tiếp tục tự động routing theo ngữ cảnh câu:

```python
Pipeline(language="auto")
```

Ép một ngôn ngữ khi caller đã biết chắc:

```python
vi = Pipeline(language="vi")
en = Pipeline(language="en")

vi.normalize("20 kg")  # hai mươi ki-lô-gam
en.normalize("20 kg")  # twenty kilograms
```

Chế độ ép ngôn ngữ áp dụng cho toàn bộ input, gồm cả normalization và G2P.
Không nên ép ngôn ngữ cho câu code-switch trừ khi đó là chủ ý. Chế độ này cũng
bỏ qua hoàn toàn phần routing, nên không dùng tới bất kỳ bằng chứng nào dưới đây.

Ở chế độ `auto`, bộ định tuyến làm việc trên **từng token trong cả câu**, không
phải đoán ngôn ngữ cho cả câu. Bằng chứng, mạnh trước:

1. Token có dấu tiếng Việt ở bất kỳ đâu thì quyết định luôn.
2. Với chuỗi ASCII trần vừa là âm tiết Việt hợp lệ vừa có trong CMUdict, một bảng
   tần suất dựng sẵn quyết định. Trước đây chỉ xét việc có mặt trong từ điển nên
   `theo` bị đọc thành `θiːoʊ` và `ba` thành tên viết tắt `biːeɪ`.
3. Câu đã có dấu tiếng Việt sẽ kéo các token ASCII còn lưỡng lự về phía tiếng
   Việt. Token viết hoa không ở đầu đoạn được miễn, nên `South Australia Loop` và
   `The Velvet Rope` giữ nguyên cách đọc tiếng Anh.
4. Còn lại thì switch cost giữ token đi cùng các token lân cận.

Chỉ dấu chấm mới kết thúc một đoạn routing. Dấu phẩy trong suốt, nên từ nằm giữa
hai dấu phẩy vẫn giữ được ngữ cảnh xung quanh:

```python
g2p.phonemize("phía đông, nam, dãy đồi.", normalize=False)
# fiə5 ɗoŋ1, naːm1, zaj4 ɗoj2.   ("nam" vẫn là tiếng Việt)
```

Cách đọc biểu thức có cấu trúc được chọn từ ngữ cảnh lexical của input. Input chỉ
có biểu thức hoặc không đủ bằng chứng vẫn mặc định cách đọc tiếng Việt để giữ
tính tương thích:

```text
I have 20 apples. → I have twenty apples.
Tôi có 20 quả táo. → tôi có hai mươi quả táo.
20 kg → hai mươi ki-lô-gam.
```

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
# ba chấm một bốn và ba phẩy một bốn
```

### Chuyển thành phoneme

```python
g2p.phonemize("Hôm nay OpenAI có meeting.")
# hom1 naj1 oʊpən eɪ aɪ kɔ5 miːtɪŋ.
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

Với hàng triệu record, dùng iterator giới hạn bộ nhớ thay vì tạo một Python
list rất lớn:

```python
for phones in g2p.phonemize_iter(records, batch_size=4096):
    write_result(phones)

for normalized in g2p.normalize_iter(records, batch_size=4096):
    write_result(normalized)
```

Khuyến nghị khi chạy production:

- Tạo và warm-up một `Pipeline` cho mỗi process; không khởi tạo lại theo từng
  request.
- Với offline job, ưu tiên batch khoảng 2.000–10.000 câu ngắn. Chunk mặc định
  4.096 của iterator là điểm khởi đầu thực tế.
- Với service đồng bộ, gom request thành micro-batch ngắn nếu latency cho phép.
  Batch dưới 64 phần tử chủ động bỏ chi phí lập lịch Rayon.
- Khi chạy nhiều worker process, chia CPU quota của container cho `num_threads`
  của từng worker để tránh oversubscription.
- Chỉ dùng `phonemize(..., normalize=False)` khi upstream đã bảo đảm text ở
  dạng canonical; cách này bỏ qua normalization nhưng thay đổi contract của
  caller.

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
báo `english_oov:<word>`. Token thuộc script hoặc ký hiệu chưa hỗ trợ sinh
`<unk>` và cảnh báo `unsupported_token:<token>` thay vì bị âm thầm xóa.

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
Schema hiện tại có định danh `donglao_g2p.__phoneme_profile__ == "compact-v2"`.

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

Dấu chấm phẩy, hai chấm, dash đứng độc lập, ellipsis giữa câu, dấu hỏi và dấu
cảm thán được đổi thành dấu phẩy. Ellipsis cuối input được đổi thành dấu chấm.
Mặc định package không tự thêm dấu kết câu. Dùng `ensure_terminal=True` nếu
muốn thêm dấu chấm khi input chưa có dấu kết câu.

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
capitalization, token lân cận, bằng chứng dấu ở mức câu và language-switch cost.
Đoạn routing chỉ bị ngắt bởi dấu chấm.

Có khoảng 875 chuỗi ASCII trần đồng thời là âm tiết tiếng Việt hợp lệ và là mục
trong CMUdict; chỉ xét việc có mặt trong từ điển thì không tách được chúng.
`src/lang_prior.rs` giải quyết 488 chuỗi mà corpus đủ sức phân định: mỗi cost là
tỉ lệ log tần suất đo trên 42,5 triệu token tiếng Việt và 28,7 triệu token tiếng
Anh, scale sao cho một token có độ tin cậy trung bình không thể lật ngược một
chuỗi token đã rõ ràng thuộc ngôn ngữ kia. Tần suất tiếng Việt tính theo **đúng
dạng mặt chữ**, cố ý không gộp theo dấu — gộp sẽ dồn `đo`, `đó`, `độ`, `dò` vào
`do` và kéo tiếng Anh thật sang tiếng Việt. Bảng này được sinh ra rồi compile vào
binary; crate không kèm data file nào lúc chạy.

## Kiểm định

Chạy correctness suite:

```bash
cargo test --locked
pytest
```

`.github/workflows/ci.yml` chạy đúng suite này ở mỗi lần push, build wheel trong
container manylinux2014, rồi cài chính artifact đó lên Python 3.9 và 3.13 để kiểm
chứng cam kết ABI3. Tag `v*` sẽ kích hoạt `.github/workflows/release.yml`: lặp lại
các gate trên và thêm `cargo audit`, SBOM CycloneDX, SHA256SUMS, chữ ký cosign và
publish lên PyPI. Tăng version bằng `scripts/bump-version.sh <version>` — script
giữ `Cargo.toml`, `pyproject.toml`, `Cargo.lock` và `uv.lock` khớp nhau, và
release workflow sẽ đối chiếu chúng với tag trước khi build bất cứ thứ gì.

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

Với corpus debug dạng `language|text`, evaluator streaming không giữ toàn bộ
file trong RAM:

```bash
python evaluation/evaluate_unique.py debugs/unique.csv
```

Metadata chỉ có text không chứa gold phoneme, vì vậy không thể đo độ chính xác
phát âm thực. Việc hai hệ G2P cho cùng output cũng không phải gold standard.

## Giới hạn

- Phát âm tiếng Việt hướng đến giọng Hà Nội.
- English OOV, tên riêng và từ mượn có thể cần override.
- Tiếng Việt không dấu không thể đọc đúng, và routing tốt đến mấy cũng không cứu
  được: `ban` có thể là `bàn`, `bán`, `bản` hay `bạn`, thanh điệu không suy ra
  được từ mặt chữ. Hãy phục hồi dấu trước khi phonemize.
- Từ mượn tiếng Việt không phải một âm tiết hợp lệ (`axit`, `oxy`, `campuchia`)
  không qua được phép kiểm tra âm tiết, không bao giờ tới được bảng tần suất, và
  rơi xuống nhánh OOV tiếng Anh. Dùng override cho những từ bạn thực sự cần.
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
