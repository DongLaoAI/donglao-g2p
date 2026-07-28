from dataclasses import FrozenInstanceError
from concurrent.futures import ThreadPoolExecutor

import pytest

from donglao_g2p import LexiconEntry, Pipeline


@pytest.fixture(scope="module")
def pipeline() -> Pipeline:
    return Pipeline()


def test_required_contract(pipeline: Pipeline) -> None:
    assert (
        pipeline.phonemize("Hôm nay tôi có meeting John.")
        == "hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn ."
    )


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        ("25 kg", "hai mươi lăm ki-lô-gam."),
        ("12:30", "mười hai giờ ba mươi phút."),
        ("2026", "hai nghìn không trăm hai mươi sáu."),
        ("3.14", "ba chấm mười bốn."),
        ("AI", "ây ai."),
        ("TTS", "ti ti ét."),
        ("OpenAI", "OpenAI."),
    ],
)
def test_normalization_golden(
    pipeline: Pipeline, source: str, expected: str
) -> None:
    assert pipeline.normalize(source) == expected


def test_punctuation_contract(pipeline: Pipeline) -> None:
    text = "Hôm nay... tôi có meeting với John!!!"
    assert pipeline.normalize(text) == "hôm nay, tôi có meeting với John."
    assert (
        pipeline.phonemize(text)
        == "hom1 naj1 , toj1 kɔ5 miːtɪŋ vəːj5 dʒɔn ."
    )


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        ("Bạn khỏe?!", "bạn khỏe."),
        ("A！ B？ C； D： E。", "A. B. C, D, E."),
        ("Chờ... tôi", "chờ, tôi."),
        ("Chờ...", "chờ."),
        ("A; B: C", "A, B, C."),
        ("“Xin chào” — John", "xin chào, John."),
        ("Giá là 3.14", "giá là ba chấm mười bốn."),
        ("TP.HCM", "ti pi âych xi em."),
        ("U.S.", "diu ét."),
        ("NAME.", "NAME."),
        ("DSL40", "đi ét eo bốn không."),
        ("hello(world)", "hello world."),
        ("I DON'T KNOW", "I DON'T KNOW."),
        ("SCROFULA", "SCROFULA."),
        ("Kane & Cabot", "Kane và Cabot."),
    ],
)
def test_punctuation_cases(
    pipeline: Pipeline, source: str, expected: str
) -> None:
    assert pipeline.normalize(source) == expected


def test_no_terminal() -> None:
    pipeline = Pipeline(ensure_terminal=False)
    assert pipeline.normalize("xin chào") == "xin chào"


def test_phonemize_normalize_flag(pipeline: Pipeline) -> None:
    assert pipeline.phonemize("xin chào") == "sin1 tʃaːw2 ."
    assert pipeline.phonemize("xin chào", normalize=True) == "sin1 tʃaːw2 ."
    assert pipeline.phonemize("xin chào", normalize=False) == "sin1 tʃaːw2"


def test_phonemize_batch_normalize_flag(pipeline: Pipeline) -> None:
    texts = ["xin chào", "hôm nay."]
    assert pipeline.phonemize_batch(texts, normalize=False) == [
        "sin1 tʃaːw2",
        "hom1 naj1 .",
    ]
    assert pipeline.phonemize_batch(texts) == [
        "sin1 tʃaːw2 .",
        "hom1 naj1 .",
    ]


def test_structured_expressions(pipeline: Pipeline) -> None:
    assert (
        pipeline.normalize("Ngày 28/07/2026 lúc 12:30")
        == "ngày hai mươi tám tháng bảy năm hai nghìn không trăm hai mươi sáu lúc mười hai giờ ba mươi phút."
    )
    assert pipeline.normalize("50% của 3-5 kg") == (
        "năm mươi phần trăm của ba đến năm ki-lô-gam."
    )
    assert pipeline.normalize("v1.2.3") == "vê một chấm hai chấm ba."
    assert pipeline.normalize("support@example.com") == (
        "support a còng example chấm com."
    )


@pytest.mark.parametrize(
    ("source", "expected"),
    [
        ("3.14", "ba chấm mười bốn."),
        ("3,14", "ba phẩy mười bốn."),
        ("0.05", "không chấm không năm."),
        ("0,05", "không phẩy không năm."),
        ("3,014", "ba phẩy không mười bốn."),
        ("1.234", "một nghìn hai trăm ba mươi tư."),
        (
            "12.345,67",
            "mười hai nghìn ba trăm bốn mươi lăm phẩy sáu mươi bảy.",
        ),
        (
            "12,345.67",
            "mười hai nghìn ba trăm bốn mươi lăm chấm sáu mươi bảy.",
        ),
    ],
)
def test_locale_aware_numbers(
    pipeline: Pipeline, source: str, expected: str
) -> None:
    assert pipeline.normalize(source) == expected


def test_english_brand_name(pipeline: Pipeline) -> None:
    assert pipeline.normalize("OpenAI") == "OpenAI."
    assert pipeline.phonemize("OpenAI") == "oʊpən eɪ aɪ ."


def test_decimal_style_digits() -> None:
    pipeline = Pipeline(decimal_style="digits")
    assert pipeline.normalize("3.14 và 3,14") == (
        "ba chấm một bốn và ba phẩy một bốn."
    )
    with pytest.raises(ValueError, match="decimal_style"):
        Pipeline(decimal_style="invalid")  # type: ignore[arg-type]


def test_batch_order_and_empty(pipeline: Pipeline) -> None:
    texts = ["Hôm nay.", "", "meeting"]
    assert pipeline.normalize_batch(texts) == ["hôm nay.", "", "meeting."]
    assert pipeline.phonemize_batch(texts) == [
        "hom1 naj1 .",
        "",
        "miːtɪŋ .",
    ]


def test_foreign_unicode_oov_is_not_empty(pipeline: Pipeline) -> None:
    analysis = pipeline.analyze("Ü-Tsang")
    assert all(token.phonemes for token in analysis.tokens if token.language != "punc")


def test_override_and_analysis() -> None:
    pipeline = Pipeline(
        {
            "DongLao": LexiconEntry(
                phonemes="dɔŋ1 laːw1", language="vi", case_sensitive=True
            ),
            "GPU": LexiconEntry(spoken="gi pi diu", language="vi"),
            "widget": LexiconEntry(spoken="quai-dờ-jét", language="vi"),
        }
    )
    assert pipeline.phonemize("DongLao.") == "dɔŋ1 laːw1 ."
    assert pipeline.normalize("GPU") == "gi pi diu."
    assert pipeline.normalize("a widget") == "a quai-dờ-jét."
    analysis = pipeline.analyze("unknowning")
    assert analysis.tokens[0].language == "en"
    assert analysis.warnings == ("english_oov:unknowning",)
    with pytest.raises(FrozenInstanceError):
        analysis.input = "changed"  # type: ignore[misc]


def test_input_validation(pipeline: Pipeline) -> None:
    with pytest.raises(TypeError):
        pipeline.phonemize_batch("not-a-batch")
    with pytest.raises(TypeError):
        pipeline.normalize(1)  # type: ignore[arg-type]
    with pytest.raises(ValueError):
        Pipeline(num_threads=0)


def test_pipeline_is_thread_safe(pipeline: Pipeline) -> None:
    source = "Hôm nay có planning với John."
    expected = pipeline.phonemize(source)
    with ThreadPoolExecutor(max_workers=8) as executor:
        outputs = list(executor.map(pipeline.phonemize, [source] * 128))
    assert outputs == [expected] * 128
