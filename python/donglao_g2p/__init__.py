"""Fast Vietnamese-English text normalization and G2P.

The public classes in this module are small Python value objects. All text
processing is performed by the Rust extension in :mod:`donglao_g2p._native`.
"""

from __future__ import annotations

from dataclasses import dataclass
from itertools import islice
from typing import Iterable, Iterator, Literal, Mapping, Optional, Union

from ._native import NativePipeline, __phoneme_profile__, __version__

Language = Literal["vi", "en"]
DecimalStyle = Literal["cardinal", "digits"]


@dataclass(frozen=True, slots=True)
class LexiconEntry:
    """An application-owned pronunciation or normalization override."""

    spoken: Optional[str] = None
    phonemes: Optional[str] = None
    language: Language = "vi"
    case_sensitive: bool = False

    def __post_init__(self) -> None:
        if self.spoken is None and self.phonemes is None:
            raise ValueError("LexiconEntry requires spoken or phonemes")
        if self.spoken is not None and self.phonemes is not None:
            raise ValueError("LexiconEntry accepts spoken or phonemes, not both")
        if self.language not in ("vi", "en"):
            raise ValueError("language must be 'vi' or 'en'")
        if self.spoken is not None and not self.spoken.strip():
            raise ValueError("spoken must not be empty")
        if self.phonemes is not None and not self.phonemes.strip():
            raise ValueError("phonemes must not be empty")


@dataclass(frozen=True, slots=True)
class TokenAnalysis:
    token: str
    language: Literal["vi", "en", "punc"]
    phonemes: str
    source: Literal["rules", "dictionary", "oov", "override", "punctuation"]


@dataclass(frozen=True, slots=True)
class Analysis:
    input: str
    normalized: str
    phonemes: str
    tokens: tuple[TokenAnalysis, ...]
    warnings: tuple[str, ...]


OverrideValue = Union[LexiconEntry, str]


class Pipeline:
    """Immutable, thread-safe normalization and phonemization pipeline."""

    __slots__ = ("_native",)

    def __init__(
        self,
        overrides: Optional[Mapping[str, OverrideValue]] = None,
        *,
        ensure_terminal: bool = True,
        num_threads: Optional[int] = None,
        decimal_style: DecimalStyle = "cardinal",
    ) -> None:
        native_overrides = {}
        for surface, value in (overrides or {}).items():
            if not isinstance(surface, str) or not surface.strip():
                raise ValueError("override keys must be non-empty strings")
            entry = value if isinstance(value, LexiconEntry) else LexiconEntry(spoken=value)
            native_overrides[surface] = (
                entry.spoken,
                entry.phonemes,
                entry.language,
                entry.case_sensitive,
            )
        self._native = NativePipeline(
            native_overrides, ensure_terminal, num_threads, decimal_style
        )

    def normalize(self, text: str) -> str:
        """Return deterministic, human-readable normalized text."""
        if not isinstance(text, str):
            raise TypeError("text must be str")
        return self._native.normalize(text)

    def normalize_batch(self, texts: Iterable[str]) -> list[str]:
        """Normalize texts in input order using the native Rayon pool."""
        values = _materialize_texts(texts)
        return self._native.normalize_batch(values)

    def phonemize(self, text: str, *, normalize: bool = True) -> str:
        """Convert text to compact IPA, normalizing it by default."""
        if not isinstance(text, str):
            raise TypeError("text must be str")
        return self._native.phonemize(text, normalize)

    def phonemize_batch(
        self, texts: Iterable[str], *, normalize: bool = True
    ) -> list[str]:
        """Phonemize texts concurrently, normalizing them by default."""
        values = _materialize_texts(texts)
        return self._native.phonemize_batch(values, normalize)

    def normalize_iter(
        self, texts: Iterable[str], *, batch_size: int = 4096
    ) -> Iterator[str]:
        """Normalize an arbitrarily large iterable in bounded-memory chunks."""
        for batch in _iter_batches(texts, batch_size):
            yield from self._native.normalize_batch(batch)

    def phonemize_iter(
        self,
        texts: Iterable[str],
        *,
        normalize: bool = True,
        batch_size: int = 4096,
    ) -> Iterator[str]:
        """Phonemize an arbitrarily large iterable in bounded-memory chunks."""
        for batch in _iter_batches(texts, batch_size):
            yield from self._native.phonemize_batch(batch, normalize)

    def analyze(self, text: str) -> Analysis:
        """Return a trace suitable for debugging OOV and language decisions."""
        if not isinstance(text, str):
            raise TypeError("text must be str")
        result = self._native.analyze(text)
        return Analysis(
            input=result.input,
            normalized=result.normalized,
            phonemes=result.phonemes,
            tokens=tuple(
                TokenAnalysis(
                    token=item.token,
                    language=item.language,
                    phonemes=item.phonemes,
                    source=item.source,
                )
                for item in result.tokens
            ),
            warnings=tuple(result.warnings),
        )


def _materialize_texts(texts: Iterable[str]) -> list[str]:
    if isinstance(texts, str):
        raise TypeError("batch input must be an iterable of str, not str")
    values = list(texts)
    if not all(isinstance(value, str) for value in values):
        raise TypeError("every batch item must be str")
    return values


def _iter_batches(texts: Iterable[str], batch_size: int) -> Iterator[list[str]]:
    if isinstance(texts, str):
        raise TypeError("batch input must be an iterable of str, not str")
    if not isinstance(batch_size, int) or isinstance(batch_size, bool) or batch_size <= 0:
        raise ValueError("batch_size must be a positive integer")
    iterator = iter(texts)
    while batch := list(islice(iterator, batch_size)):
        if not all(isinstance(value, str) for value in batch):
            raise TypeError("every batch item must be str")
        yield batch


_default_pipeline = Pipeline()


def normalize(text: str) -> str:
    """Normalize with the process-wide default pipeline."""
    return _default_pipeline.normalize(text)


def phonemize(text: str, *, normalize: bool = True) -> str:
    """Phonemize with the process-wide default pipeline."""
    return _default_pipeline.phonemize(text, normalize=normalize)


__all__ = [
    "Analysis",
    "DecimalStyle",
    "LexiconEntry",
    "Pipeline",
    "TokenAnalysis",
    "__phoneme_profile__",
    "__version__",
    "normalize",
    "phonemize",
]
