import random
import string

from donglao_g2p import Pipeline


def test_normalization_is_idempotent() -> None:
    pipeline = Pipeline()
    samples = [
        "Giá 25 kg... lúc 12:30!!!",
        "email support@example.com",
        "https://example.com/a?x=1",
        "Ngày 28/07/2026",
        "Từ 3-5 kg",
    ]
    for sample in samples:
        once = pipeline.normalize(sample)
        assert pipeline.normalize(once) == once


def test_random_unicode_never_crashes() -> None:
    pipeline = Pipeline()
    alphabet = string.printable + "ăâđêôơưáàảãạ…，。？！🙂"
    random.seed(7)
    values = [
        "".join(random.choice(alphabet) for _ in range(random.randrange(0, 200)))
        for _ in range(250)
    ]
    outputs = pipeline.phonemize_batch(values)
    assert len(outputs) == len(values)
    assert all(isinstance(output.encode("utf-8"), bytes) for output in outputs)
    for value in values:
        analysis = pipeline.analyze(value)
        assert all(
            token.phonemes
            for token in analysis.tokens
            if token.language != "punc"
        )
