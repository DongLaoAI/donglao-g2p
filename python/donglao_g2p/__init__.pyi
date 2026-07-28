from dataclasses import dataclass
from typing import Iterable, Literal, Mapping, Optional, Union

Language = Literal["vi", "en"]
DecimalStyle = Literal["cardinal", "digits"]

@dataclass(frozen=True)
class LexiconEntry:
    spoken: Optional[str] = ...
    phonemes: Optional[str] = ...
    language: Language = ...
    case_sensitive: bool = ...

@dataclass(frozen=True)
class TokenAnalysis:
    token: str
    language: Literal["vi", "en", "punc"]
    phonemes: str
    source: Literal["rules", "dictionary", "oov", "override", "punctuation"]

@dataclass(frozen=True)
class Analysis:
    input: str
    normalized: str
    phonemes: str
    tokens: tuple[TokenAnalysis, ...]
    warnings: tuple[str, ...]

OverrideValue = Union[LexiconEntry, str]

class Pipeline:
    def __init__(
        self,
        overrides: Optional[Mapping[str, OverrideValue]] = ...,
        *,
        ensure_terminal: bool = ...,
        num_threads: Optional[int] = ...,
        decimal_style: DecimalStyle = ...,
    ) -> None: ...
    def normalize(self, text: str) -> str: ...
    def normalize_batch(self, texts: Iterable[str]) -> list[str]: ...
    def phonemize(self, text: str, *, normalize: bool = ...) -> str: ...
    def phonemize_batch(
        self, texts: Iterable[str], *, normalize: bool = ...
    ) -> list[str]: ...
    def analyze(self, text: str) -> Analysis: ...

__version__: str
def normalize(text: str) -> str: ...
def phonemize(text: str, *, normalize: bool = ...) -> str: ...
