mod g2p;
mod normalizer;
mod numbers;

use std::collections::HashMap;
use std::sync::Arc;

use g2p::{phonemize_text, Analysis, Override};
use normalizer::{normalize_text, DecimalStyle};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;

#[pyclass(module = "donglao_g2p._native")]
struct NativePipeline {
    ensure_terminal: bool,
    decimal_style: DecimalStyle,
    overrides: Arc<HashMap<String, Override>>,
    pool: Option<Arc<rayon::ThreadPool>>,
}

#[pymethods]
impl NativePipeline {
    #[new]
    #[pyo3(signature = (overrides=None, ensure_terminal=true, num_threads=None, decimal_style="cardinal"))]
    fn new(
        overrides: Option<HashMap<String, (Option<String>, Option<String>, String, bool)>>,
        ensure_terminal: bool,
        num_threads: Option<usize>,
        decimal_style: &str,
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
        let overrides = overrides
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
            pool,
        })
    }

    fn normalize(&self, text: &str) -> String {
        normalize_text(
            text,
            self.ensure_terminal,
            self.decimal_style,
            &self.overrides,
        )
    }

    fn normalize_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        let ensure_terminal = self.ensure_terminal;
        let decimal_style = self.decimal_style;
        let overrides = Arc::clone(&self.overrides);
        let work = || {
            texts
                .par_iter()
                .map(|text| normalize_text(text, ensure_terminal, decimal_style, &overrides))
                .collect()
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
            )
        } else {
            text.to_owned()
        };
        phonemize_text(&prepared, &self.overrides).phonemes
    }

    #[pyo3(signature = (texts, normalize=true))]
    fn phonemize_batch(&self, py: Python<'_>, texts: Vec<String>, normalize: bool) -> Vec<String> {
        let ensure_terminal = self.ensure_terminal;
        let decimal_style = self.decimal_style;
        let overrides = Arc::clone(&self.overrides);
        let work = || {
            texts
                .par_iter()
                .map(|text| {
                    if normalize {
                        let normalized =
                            normalize_text(text, ensure_terminal, decimal_style, &overrides);
                        phonemize_text(&normalized, &overrides).phonemes
                    } else {
                        phonemize_text(text, &overrides).phonemes
                    }
                })
                .collect()
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
        );
        let mut result = phonemize_text(&normalized, &self.overrides);
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
    Ok(())
}
