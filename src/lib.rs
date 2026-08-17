mod g2p;
mod lang_prior;
mod normalizer;
mod numbers;

use std::collections::HashMap;
use std::sync::Arc;

use g2p::{phonemize_only, phonemize_text, Analysis, Override};
use normalizer::{normalize_text, prepare_spoken_overrides, DecimalStyle, PreparedSpokenOverride};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;

const PARALLEL_BATCH_MIN: usize = 64;

#[derive(Clone, Copy)]
enum LanguageMode {
    Auto,
    Vi,
    En,
}

impl LanguageMode {
    fn forced(self) -> Option<&'static str> {
        match self {
            Self::Auto => None,
            Self::Vi => Some("vi"),
            Self::En => Some("en"),
        }
    }
}

#[pyclass(module = "donglao_g2p._native")]
struct NativePipeline {
    ensure_terminal: bool,
    decimal_style: DecimalStyle,
    overrides: Arc<HashMap<String, Override>>,
    spoken_overrides: Arc<Vec<PreparedSpokenOverride>>,
    language: LanguageMode,
    pool: Option<Arc<rayon::ThreadPool>>,
}

#[pymethods]
impl NativePipeline {
    #[new]
    #[pyo3(signature = (overrides=None, ensure_terminal=false, num_threads=None, decimal_style="cardinal", language="auto"))]
    fn new(
        overrides: Option<HashMap<String, (Option<String>, Option<String>, String, bool)>>,
        ensure_terminal: bool,
        num_threads: Option<usize>,
        decimal_style: &str,
        language: &str,
    ) -> PyResult<Self> {
        if matches!(num_threads, Some(0)) {
            return Err(PyValueError::new_err(
                "num_threads must be greater than zero",
            ));
        }
        let decimal_style = match decimal_style {
            "cardinal" => DecimalStyle::Cardinal,
            "digits" => DecimalStyle::Digits,
            _ => {
                return Err(PyValueError::new_err(
                    "decimal_style must be 'cardinal' or 'digits'",
                ))
            }
        };
        let language = match language {
            "auto" => LanguageMode::Auto,
            "vi" => LanguageMode::Vi,
            "en" => LanguageMode::En,
            _ => {
                return Err(PyValueError::new_err(
                    "language must be 'auto', 'vi', or 'en'",
                ))
            }
        };
        let overrides: HashMap<String, Override> = overrides
            .unwrap_or_default()
            .into_iter()
            .map(|(surface, (spoken, phonemes, language, case_sensitive))| {
                (
                    if case_sensitive {
                        surface
                    } else {
                        surface.to_lowercase()
                    },
                    Override {
                        spoken,
                        phonemes,
                        language,
                        case_sensitive,
                    },
                )
            })
            .collect();
        let spoken_overrides = Arc::new(prepare_spoken_overrides(&overrides));
        let pool = match num_threads {
            Some(n) => Some(Arc::new(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(n)
                    .thread_name(|i| format!("donglao-g2p-{i}"))
                    .build()
                    .map_err(|e| PyValueError::new_err(e.to_string()))?,
            )),
            None => None,
        };
        Ok(Self {
            ensure_terminal,
            decimal_style,
            overrides: Arc::new(overrides),
            spoken_overrides,
            language,
            pool,
        })
    }

    fn normalize(&self, text: &str) -> String {
        normalize_text(
            text,
            self.ensure_terminal,
            self.decimal_style,
            &self.overrides,
            &self.spoken_overrides,
            self.language.forced(),
        )
    }

    fn normalize_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        let ensure_terminal = self.ensure_terminal;
        let decimal_style = self.decimal_style;
        let overrides = Arc::clone(&self.overrides);
        let spoken_overrides = Arc::clone(&self.spoken_overrides);
        let language = self.language.forced();
        let work = || {
            let normalize_one = |text: &String| {
                normalize_text(
                    text,
                    ensure_terminal,
                    decimal_style,
                    &overrides,
                    &spoken_overrides,
                    language,
                )
            };
            if texts.len() < PARALLEL_BATCH_MIN {
                texts.iter().map(normalize_one).collect()
            } else {
                texts.par_iter().map(normalize_one).collect()
            }
        };
        py.allow_threads(|| match &self.pool {
            Some(pool) => pool.install(work),
            None => work(),
        })
    }

    #[pyo3(signature = (text, normalize=true))]
    fn phonemize(&self, text: &str, normalize: bool) -> String {
        let prepared = if normalize {
            normalize_text(
                text,
                self.ensure_terminal,
                self.decimal_style,
                &self.overrides,
                &self.spoken_overrides,
                self.language.forced(),
            )
        } else {
            text.to_owned()
        };
        phonemize_only(&prepared, &self.overrides, self.language.forced())
    }

    #[pyo3(signature = (texts, normalize=true))]
    fn phonemize_batch(&self, py: Python<'_>, texts: Vec<String>, normalize: bool) -> Vec<String> {
        let ensure_terminal = self.ensure_terminal;
        let decimal_style = self.decimal_style;
        let overrides = Arc::clone(&self.overrides);
        let spoken_overrides = Arc::clone(&self.spoken_overrides);
        let language = self.language.forced();
        let work = || {
            let phonemize_one = |text: &String| {
                if normalize {
                    let normalized = normalize_text(
                        text,
                        ensure_terminal,
                        decimal_style,
                        &overrides,
                        &spoken_overrides,
                        language,
                    );
                    phonemize_only(&normalized, &overrides, language)
                } else {
                    phonemize_only(text, &overrides, language)
                }
            };
            if texts.len() < PARALLEL_BATCH_MIN {
                texts.iter().map(phonemize_one).collect()
            } else {
                texts.par_iter().map(phonemize_one).collect()
            }
        };
        py.allow_threads(|| match &self.pool {
            Some(pool) => pool.install(work),
            None => work(),
        })
    }

    fn analyze(&self, text: &str) -> Analysis {
        let normalized = normalize_text(
            text,
            self.ensure_terminal,
            self.decimal_style,
            &self.overrides,
            &self.spoken_overrides,
            self.language.forced(),
        );
        let mut result = phonemize_text(&normalized, &self.overrides, self.language.forced());
        result.input = text.to_owned();
        result.normalized = normalized;
        result
    }
}

#[pymodule]
fn _native(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<NativePipeline>()?;
    module.add_class::<Analysis>()?;
    module.add_class::<g2p::TokenAnalysis>()?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add("__phoneme_profile__", "compact-v2")?;
    Ok(())
}
