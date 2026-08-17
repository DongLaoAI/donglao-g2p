use std::collections::HashMap;

use pyo3::prelude::*;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[derive(Clone, Debug)]
pub struct Override {
    pub spoken: Option<String>,
    pub phonemes: Option<String>,
    pub language: String,
    pub case_sensitive: bool,
}

// skip_from_py_object: both classes are only ever returned to Python, never
// accepted as an argument, and pyo3 is making that derive opt-in.
#[pyclass(module = "donglao_g2p._native", skip_from_py_object)]
#[derive(Clone, Debug)]
pub struct TokenAnalysis {
    #[pyo3(get)]
    pub token: String,
    #[pyo3(get)]
    pub language: String,
    #[pyo3(get)]
    pub phonemes: String,
    #[pyo3(get)]
    pub source: String,
}

// skip_from_py_object: both classes are only ever returned to Python, never
// accepted as an argument, and pyo3 is making that derive opt-in.
#[pyclass(module = "donglao_g2p._native", skip_from_py_object)]
#[derive(Clone, Debug, Default)]
pub struct Analysis {
    #[pyo3(get)]
    pub input: String,
    #[pyo3(get)]
    pub normalized: String,
    #[pyo3(get)]
    pub phonemes: String,
    #[pyo3(get)]
    pub tokens: Vec<TokenAnalysis>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
}

pub fn find_override<'a>(
    token: &str,
    overrides: &'a HashMap<String, Override>,
) -> Option<&'a Override> {
    if let Some(entry) = overrides.get(token) {
        return Some(entry);
    }
    let lower = token.to_lowercase();
    overrides.get(&lower).filter(|entry| !entry.case_sensitive)
}

fn tokenize(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = None;
    for (index, c) in text.char_indices() {
        if matches!(c, ',' | '.') {
            if let Some(token_start) = start.take() {
                out.push(&text[token_start..index]);
            }
            out.push(&text[index..index + c.len_utf8()]);
        } else if c.is_whitespace() || c == '-' {
            if let Some(token_start) = start.take() {
                out.push(&text[token_start..index]);
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(token_start) = start {
        out.push(&text[token_start..]);
    }
    out
}

fn is_punctuation(token: &str) -> bool {
    matches!(token, "," | ".")
}

/// Only a full stop ends a language segment. Commas stay transparent so a
/// single Vietnamese word between them keeps its sentence context.
fn is_sentence_break(token: &str) -> bool {
    token == "."
}

fn has_vietnamese_mark(word: &str) -> bool {
    word.chars().any(|c| {
        "ăâđêôơưáàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ"
            .contains(c.to_lowercase().next().unwrap_or(c))
    })
}

fn common_vietnamese_ascii(word: &str) -> bool {
    matches!(
        word,
        "ba" | "bay"
            | "bi"
            | "bon"
            | "chi"
            | "cho"
            | "con"
            | "da"
            | "den"
            | "di"
            | "do"
            | "hai"
            | "hay"
            | "khi"
            | "khong"
            | "la"
            | "lam"
            | "le"
            | "met"
            | "mot"
            | "muoi"
            | "nam"
            | "nay"
            | "ngay"
            | "nghin"
            | "nguoi"
            | "nhung"
            | "phut"
            | "sau"
            | "tam"
            | "thang"
            | "thi"
            | "trong"
            | "tu"
            | "va"
            | "voi"
    )
}

fn english_lexicon(word: &str) -> Option<&'static str> {
    Some(match word {
        "a" => "ə",
        "about" => "əbaʊt",
        "after" => "æftɚ",
        "again" => "əɡɛn",
        "all" => "ɔl",
        "am" => "æm",
        "an" => "æn",
        "and" => "ænd",
        "apple" => "æpəl",
        "are" => "ɑɹ",
        "at" => "æt",
        "be" => "biː",
        "because" => "bɪkɔz",
        "before" => "bɪfɔɹ",
        "but" => "bʌt",
        "by" => "baɪ",
        "can" => "kæn",
        "chat" => "tʃæt",
        "code" => "koʊd",
        "data" => "deɪtə",
        "day" => "deɪ",
        "do" => "duː",
        "email" => "iːmeɪl",
        "english" => "ɪŋɡlɪʃ",
        "for" => "fɔɹ",
        "from" => "fɹʌm",
        "good" => "ɡʊd",
        "google" => "ɡuːɡəl",
        "great" => "ɡɹeɪt",
        "hello" => "hɛloʊ",
        "help" => "hɛlp",
        "how" => "haʊ",
        "i" => "aɪ",
        "in" => "ɪn",
        "is" => "ɪz",
        "it" => "ɪt",
        "john" => "dʒɔn",
        "like" => "laɪk",
        "make" => "meɪk",
        "meet" => "miːt",
        "meeting" => "miːtɪŋ",
        "model" => "mɑdəl",
        "my" => "maɪ",
        "name" => "neɪm",
        "nice" => "naɪs",
        "no" => "noʊ",
        "not" => "nɑt",
        "of" => "ʌv",
        "on" => "ɑn",
        "open" => "oʊpən",
        "openai" => "oʊpən eɪ aɪ",
        "or" => "ɔɹ",
        "please" => "pliːz",
        "production" => "pɹədʌkʃən",
        "python" => "paɪθɑn",
        "rust" => "ɹʌst",
        "server" => "sɜɹvɚ",
        "speech" => "spiːtʃ",
        "support" => "səpɔɹt",
        "test" => "tɛst",
        "text" => "tɛkst",
        "thank" => "θæŋk",
        "that" => "ðæt",
        "the" => "ðə",
        "this" => "ðɪs",
        "time" => "taɪm",
        "to" => "tuː",
        "today" => "tədeɪ",
        "voice" => "vɔɪs",
        "we" => "wiː",
        "with" => "wɪð",
        "world" => "wɜɹld",
        "yes" => "jɛs",
        "you" => "juː",
        "your" => "jɔɹ",
        _ => return None,
    })
}

pub fn is_english_dictionary_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    english_lexicon(&lower).is_some()
        || arpabet_cmudict::load_cmudict()
            .get_polyphone_ref(&lower)
            .is_some()
}

fn arpabet_to_ipa(symbols: &[&str]) -> String {
    let mut out = String::new();
    for raw in symbols {
        let symbol = raw.trim_end_matches(|c: char| c.is_ascii_digit());
        out.push_str(match symbol {
            "AA" => "ɑ",
            "AE" => "æ",
            "AH" if raw.ends_with('0') => "ə",
            "AH" => "ʌ",
            "AO" => "ɔ",
            "AW" => "aʊ",
            "AY" => "aɪ",
            "EH" => "ɛ",
            "ER" if raw.ends_with('0') => "ɚ",
            "ER" => "ɜɹ",
            "EY" => "eɪ",
            "IH" => "ɪ",
            "IY" => "iː",
            "OW" => "oʊ",
            "OY" => "ɔɪ",
            "UH" => "ʊ",
            "UW" => "uː",
            "B" => "b",
            "CH" => "tʃ",
            "D" => "d",
            "DH" => "ð",
            "F" => "f",
            "G" => "ɡ",
            "HH" => "h",
            "JH" => "dʒ",
            "K" => "k",
            "L" => "l",
            "M" => "m",
            "N" => "n",
            "NG" => "ŋ",
            "P" => "p",
            "R" => "ɹ",
            "S" => "s",
            "SH" => "ʃ",
            "T" => "t",
            "TH" => "θ",
            "V" => "v",
            "W" => "w",
            "Y" => "j",
            "Z" => "z",
            "ZH" => "ʒ",
            _ => "",
        });
    }
    out
}

fn cmu_pronunciation(word: &str) -> Option<String> {
    let symbols = arpabet_cmudict::load_cmudict().get_polyphone_str(word)?;
    Some(arpabet_to_ipa(&symbols))
}

pub(crate) fn language_cost(token: &str, overrides: &HashMap<String, Override>) -> (f32, f32) {
    if let Some(entry) = find_override(token, overrides) {
        return if entry.language.eq_ignore_ascii_case("en") {
            (100.0, 0.0)
        } else {
            (0.0, 100.0)
        };
    }
    let lower = token.to_lowercase();
    if has_vietnamese_mark(&lower) {
        return (0.0, 12.0);
    }
    let valid_vi = valid_vietnamese_syllable(&lower);
    let valid_en = is_english_dictionary_word(&lower);
    if valid_vi && valid_en {
        // ASCII collisions such as "ta", "ra", "do", "to", and "can" need
        // sentence context. Corpus frequency settles most of them outright;
        // the rest keep a small English lexical prior and let the Viterbi
        // transition cost carry surrounding marked Vietnamese over.
        if let Ok(slot) =
            crate::lang_prior::LANGUAGE_PRIOR.binary_search_by(|probe| probe.0.cmp(lower.as_str()))
        {
            let (_, vi, en) = crate::lang_prior::LANGUAGE_PRIOR[slot];
            return (vi, en);
        }
        (0.15, 0.0)
    } else if valid_vi || common_vietnamese_ascii(&lower) {
        (0.0, 6.0)
    } else if valid_en {
        (8.0, 0.0)
    } else if token.chars().next().is_some_and(char::is_uppercase)
        || lower.ends_with("ing")
        || lower.ends_with("tion")
        || lower.ends_with("ness")
        || lower.ends_with("able")
    {
        (5.0, 1.0)
    } else {
        (6.0, 1.0)
    }
}

fn valid_vietnamese_syllable(word: &str) -> bool {
    if word.is_empty()
        || word.len() > 9
        || !word
            .chars()
            .all(|c| c.is_ascii_alphabetic() || "ăâđêôơư".contains(c))
    {
        return false;
    }
    let (clean, _) = strip_tone(word);
    let (onset, mut rime) = split_onset(&clean);
    if onset == "qu" {
        rime = rime.strip_prefix('u').unwrap_or(rime);
    }
    if onset == "gi" && rime.starts_with('i') {
        rime = rime.strip_prefix('i').unwrap_or(rime);
    }
    if rime.is_empty() {
        return onset == "gi";
    }
    const NUCLEI: &[&str] = &[
        "a", "ă", "â", "e", "ê", "i", "y", "o", "ô", "ơ", "u", "ư", "ai", "ao", "au", "ay", "âu",
        "ây", "eo", "êu", "ia", "iê", "yê", "iêu", "yêu", "iu", "oa", "oă", "oe", "oi", "ôi", "ơi",
        "oo", "oai", "oay", "ua", "uâ", "uê", "ui", "uô", "uơ", "uy", "ưa", "ưi", "ưu", "ươ",
        "ươu", "uây", "uôi", "ươi", "uya", "uyê",
    ];
    if NUCLEI.contains(&rime) {
        return true;
    }
    ["ch", "nh", "ng", "c", "m", "n", "p", "t"]
        .iter()
        .any(|coda| {
            rime.strip_suffix(coda)
                .is_some_and(|nucleus| NUCLEI.contains(&nucleus))
        })
}

fn detect_languages(
    tokens: &[&str],
    overrides: &HashMap<String, Override>,
    forced_language: Option<&'static str>,
) -> Vec<&'static str> {
    if let Some(language) = forced_language {
        return tokens
            .iter()
            .map(|token| {
                if is_punctuation(token) {
                    "punc"
                } else {
                    language
                }
            })
            .collect();
    }
    let mut result = vec!["punc"; tokens.len()];
    let mut start = 0;
    while start < tokens.len() {
        while start < tokens.len() && is_sentence_break(&tokens[start]) {
            start += 1;
        }
        if start >= tokens.len() {
            break;
        }
        let mut indices = Vec::new();
        let mut cursor = start;
        while cursor < tokens.len() && !is_sentence_break(&tokens[cursor]) {
            if !is_punctuation(&tokens[cursor]) {
                indices.push(cursor);
            }
            cursor += 1;
        }
        if indices.is_empty() {
            start = cursor.saturating_add(1);
            continue;
        }

        let mut emissions = indices
            .iter()
            .map(|&idx| language_cost(&tokens[idx], overrides))
            .collect::<Vec<_>>();

        // Sentence-level evidence. A segment already carrying Vietnamese
        // diacritics makes its undecided ASCII tokens more likely Vietnamese
        // too, which is what pulls "cho" out of "hoàng tử Joachim cho hay".
        let marked = indices
            .iter()
            .filter(|&&idx| has_vietnamese_mark(&tokens[idx].to_lowercase()))
            .count();
        // Gated on that evidence, so an all-ASCII English sentence keeps its
        // original scores and cannot regress. High-frequency Vietnamese
        // spellings get the stronger pull; other undecided tokens a mild one.
        if marked * 4 >= indices.len() {
            for (position, (slot, &idx)) in emissions.iter_mut().zip(indices.iter()).enumerate() {
                // A capitalised token away from the segment start is a proper
                // noun far more often than a Vietnamese word, so leave titles
                // like "The Velvet Rope" and "South Australia Loop" alone.
                if position > 0 && tokens[idx].chars().next().is_some_and(char::is_uppercase) {
                    continue;
                }
                if (slot.0 - slot.1).abs() < 1.0 {
                    slot.1 += if common_vietnamese_ascii(&tokens[idx].to_lowercase()) {
                        6.0
                    } else {
                        1.0
                    };
                }
            }
        }
        let mut vi_cost = 0.0f32;
        let mut en_cost = 0.0f32;
        let mut back = Vec::with_capacity(indices.len());
        for position in 0..indices.len() {
            let (mut emit_vi, mut emit_en) = emissions[position];
            if (emit_vi - emit_en).abs() < 1.0 {
                if let Some(&(next_vi, next_en)) = emissions.get(position + 1) {
                    if next_en - next_vi >= 4.0 {
                        emit_vi = 0.0;
                        emit_en = 3.0;
                    }
                }
            }
            if position == 0 {
                vi_cost = emit_vi;
                en_cost = emit_en;
                back.push((0u8, 1u8));
                continue;
            }
            let switch_cost = if (emit_vi - emit_en).abs() >= 4.0 {
                0.25
            } else {
                1.0
            };
            let (new_vi, vi_prev) = if vi_cost <= en_cost + switch_cost {
                (vi_cost + emit_vi, 0)
            } else {
                (en_cost + switch_cost + emit_vi, 1)
            };
            let (new_en, en_prev) = if en_cost <= vi_cost + switch_cost {
                (en_cost + emit_en, 1)
            } else {
                (vi_cost + switch_cost + emit_en, 0)
            };
            back.push((vi_prev, en_prev));
            vi_cost = new_vi;
            en_cost = new_en;
        }
        let mut state = if vi_cost <= en_cost { 0u8 } else { 1u8 };
        for position in (0..indices.len()).rev() {
            result[indices[position]] = if state == 0 { "vi" } else { "en" };
            if position > 0 {
                state = if state == 0 {
                    back[position].0
                } else {
                    back[position].1
                };
            }
        }
        start = cursor.saturating_add(1);
    }
    result
}

fn strip_tone(word: &str) -> (String, u8) {
    let mut tone = 1;
    let mut clean = String::new();
    for c in word.to_lowercase().nfc() {
        let (base, found) = match c {
            'à' => ('a', 2),
            'ả' => ('a', 3),
            'ã' => ('a', 4),
            'á' => ('a', 5),
            'ạ' => ('a', 6),
            'ằ' => ('ă', 2),
            'ẳ' => ('ă', 3),
            'ẵ' => ('ă', 4),
            'ắ' => ('ă', 5),
            'ặ' => ('ă', 6),
            'ầ' => ('â', 2),
            'ẩ' => ('â', 3),
            'ẫ' => ('â', 4),
            'ấ' => ('â', 5),
            'ậ' => ('â', 6),
            'è' => ('e', 2),
            'ẻ' => ('e', 3),
            'ẽ' => ('e', 4),
            'é' => ('e', 5),
            'ẹ' => ('e', 6),
            'ề' => ('ê', 2),
            'ể' => ('ê', 3),
            'ễ' => ('ê', 4),
            'ế' => ('ê', 5),
            'ệ' => ('ê', 6),
            'ì' => ('i', 2),
            'ỉ' => ('i', 3),
            'ĩ' => ('i', 4),
            'í' => ('i', 5),
            'ị' => ('i', 6),
            'ò' => ('o', 2),
            'ỏ' => ('o', 3),
            'õ' => ('o', 4),
            'ó' => ('o', 5),
            'ọ' => ('o', 6),
            'ồ' => ('ô', 2),
            'ổ' => ('ô', 3),
            'ỗ' => ('ô', 4),
            'ố' => ('ô', 5),
            'ộ' => ('ô', 6),
            'ờ' => ('ơ', 2),
            'ở' => ('ơ', 3),
            'ỡ' => ('ơ', 4),
            'ớ' => ('ơ', 5),
            'ợ' => ('ơ', 6),
            'ù' => ('u', 2),
            'ủ' => ('u', 3),
            'ũ' => ('u', 4),
            'ú' => ('u', 5),
            'ụ' => ('u', 6),
            'ừ' => ('ư', 2),
            'ử' => ('ư', 3),
            'ữ' => ('ư', 4),
            'ứ' => ('ư', 5),
            'ự' => ('ư', 6),
            'ỳ' => ('y', 2),
            'ỷ' => ('y', 3),
            'ỹ' => ('y', 4),
            'ý' => ('y', 5),
            'ỵ' => ('y', 6),
            other => (other, 0),
        };
        clean.push(base);
        if found > 0 {
            tone = found;
        }
    }
    (clean, tone)
}

fn split_onset(word: &str) -> (&str, &str) {
    const ONSETS: &[&str] = &[
        "ngh", "ng", "nh", "ch", "gh", "gi", "kh", "ph", "qu", "th", "tr", "đ", "b", "c", "d", "g",
        "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x",
    ];
    for onset in ONSETS {
        if let Some(rest) = word.strip_prefix(onset) {
            return (onset, rest);
        }
    }
    ("", word)
}

fn onset_ipa(onset: &str, rime: &str) -> &'static str {
    match onset {
        "b" => "ɓ",
        "c" | "k" | "q" => "k",
        "ch" => "tʃ",
        "d" | "gi" | "r" => "z",
        "đ" => "ɗ",
        "g" | "gh" => "ɣ",
        "h" => "h",
        "kh" => "x",
        "l" => "l",
        "m" => "m",
        "n" => "n",
        "ng" | "ngh" => "ŋ",
        "nh" => "ɲ",
        "p" => "p",
        "ph" => "f",
        "s" => "s",
        "th" => "tʰ",
        "tr" => "tʂ",
        "t" => "t",
        "v" => "v",
        "x" => "s",
        "qu" if rime.starts_with('u') => "kw",
        "qu" => "kw",
        _ => "",
    }
}

fn rime_ipa(rime: &str) -> String {
    let direct = match rime {
        "a" => "aː",
        "ă" => "ă",
        "â" => "ə",
        "e" => "ɛ",
        "ê" => "e",
        "i" | "y" => "i",
        "o" => "ɔ",
        "ô" => "o",
        "ơ" => "əː",
        "u" => "u",
        "ư" => "ɯ",
        "ai" => "aːj",
        "ay" => "aj",
        "ây" => "əj",
        "ao" => "aːw",
        "au" => "aw",
        "âu" => "əw",
        "eo" => "ɛw",
        "êu" => "ew",
        "ia" | "iê" | "yê" => "iə",
        "iu" => "iw",
        "oa" => "waː",
        "oă" => "wă",
        "oe" => "wɛ",
        "oi" => "ɔj",
        "ôi" => "oj",
        "ơi" => "əːj",
        "ua" | "uô" => "uə",
        "ưa" | "ươ" => "ɯə",
        "ui" => "uj",
        "ưi" => "ɯj",
        "ưu" => "ɯw",
        "uy" => "wi",
        "iêu" | "yêu" => "iəw",
        "oai" | "uai" => "waːj",
        "oay" => "waj",
        "uâ" => "wə",
        "uê" => "we",
        "uya" | "uyê" => "wiə",
        "uây" => "wəj",
        "uôi" => "uəj",
        "ươi" => "ɯəj",
        "ươu" => "ɯəw",
        _ => "",
    };
    if !direct.is_empty() {
        return direct.to_owned();
    }

    const CODAS: &[(&str, &str)] = &[
        ("ch", "k"),
        ("nh", "ɲ"),
        ("ng", "ŋ"),
        ("c", "k"),
        ("m", "m"),
        ("n", "n"),
        ("p", "p"),
        ("t", "t"),
    ];
    for (spelling, phone) in CODAS {
        if let Some(nucleus) = rime.strip_suffix(spelling) {
            if !nucleus.is_empty() {
                let mut out = rime_ipa(nucleus);
                out.push_str(phone);
                return out;
            }
        }
    }
    rime.chars()
        .map(|c| match c {
            'a' | 'ă' => "a",
            'â' | 'ơ' => "ə",
            'e' => "ɛ",
            'ê' => "e",
            'i' | 'y' => "i",
            'o' => "ɔ",
            'ô' => "o",
            'u' => "u",
            'ư' => "ɯ",
            _ => "",
        })
        .collect()
}

fn vietnamese_word(word: &str) -> String {
    let (clean, tone) = strip_tone(word);
    let (onset, mut rime) = split_onset(&clean);
    if onset == "qu" {
        rime = rime.strip_prefix('u').unwrap_or(rime);
    }
    if onset == "gi" && rime.starts_with('i') {
        rime = rime.strip_prefix('i').unwrap_or(rime);
    }
    let gi_rime;
    if onset == "gi" && (rime.is_empty() || rime.starts_with('ê')) {
        gi_rime = format!("i{rime}");
        rime = &gi_rime;
    }
    format!("{}{}{}", onset_ipa(onset, rime), rime_ipa(rime), tone)
}

fn vietnamese_g2p(token: &str) -> String {
    token
        .split('-')
        .filter(|part| !part.is_empty())
        .map(vietnamese_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn english_oov(word: &str) -> String {
    let mut source = word
        .to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect::<String>();
    if source.ends_with('e') && source.len() > 2 && !source.ends_with("ee") {
        source.pop();
    }
    // Compact joint grapheme/phoneme n-grams. Decoding is a bounded beam so
    // adding a domain graphone does not require changing procedural control
    // flow or introducing a neural runtime.
    let graphones = [
        ("tion", "ʃən", 0.02f32),
        ("sion", "ʒən", 0.05),
        ("tch", "tʃ", 0.03),
        ("dge", "dʒ", 0.03),
        ("igh", "aɪ", 0.04),
        ("ph", "f", 0.02),
        ("sh", "ʃ", 0.02),
        ("ch", "tʃ", 0.03),
        ("th", "θ", 0.08),
        ("th", "ð", 0.12),
        ("ng", "ŋ", 0.02),
        ("qu", "kw", 0.02),
        ("ee", "iː", 0.02),
        ("ea", "iː", 0.06),
        ("oo", "uː", 0.07),
        ("oo", "ʊ", 0.10),
        ("ai", "eɪ", 0.03),
        ("ay", "eɪ", 0.03),
        ("oa", "oʊ", 0.03),
        ("ow", "aʊ", 0.08),
        ("ow", "oʊ", 0.10),
        ("ou", "aʊ", 0.08),
        ("oi", "ɔɪ", 0.03),
        ("oy", "ɔɪ", 0.03),
        ("er", "ɚ", 0.04),
        ("ar", "ɑɹ", 0.04),
        ("or", "ɔɹ", 0.04),
    ];
    let single_phone = |c| match c {
        'a' => "æ",
        'b' => "b",
        'c' | 'k' | 'q' => "k",
        'd' => "d",
        'e' => "ɛ",
        'f' => "f",
        'g' => "ɡ",
        'h' => "h",
        'i' | 'y' => "ɪ",
        'j' => "dʒ",
        'l' => "l",
        'm' => "m",
        'n' => "n",
        'o' => "ɑ",
        'p' => "p",
        'r' => "ɹ",
        's' => "s",
        't' => "t",
        'u' => "ʌ",
        'v' => "v",
        'w' => "w",
        'x' => "ks",
        'z' => "z",
        _ => "",
    };

    let mut beams: Vec<Vec<(String, f32)>> = vec![Vec::new(); source.len() + 1];
    beams[0].push((String::new(), 0.0));
    for position in 0..source.len() {
        if beams[position].is_empty() || !source.is_char_boundary(position) {
            continue;
        }
        let states = beams[position].clone();
        let rest = &source[position..];
        for (phones, cost) in states {
            for (letters, output, graphone_cost) in &graphones {
                if rest.starts_with(letters) {
                    let mut candidate = phones.clone();
                    candidate.push_str(output);
                    beams[position + letters.len()].push((candidate, cost + graphone_cost));
                }
            }
            if let Some(c) = rest.chars().next() {
                let mut candidate = phones;
                candidate.push_str(single_phone(c));
                beams[position + c.len_utf8()].push((candidate, cost + 1.0));
            }
        }
        for offset in 1..=4 {
            if let Some(bucket) = beams.get_mut(position + offset) {
                bucket.sort_by(|a, b| a.1.total_cmp(&b.1));
                bucket.truncate(8);
            }
        }
    }
    beams[source.len()]
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|state| state.0.clone())
        .unwrap_or_default()
}

fn english_g2p(token: &str) -> (String, &'static str) {
    let lower = token.to_lowercase();
    match english_lexicon(&lower) {
        Some(phones) => (phones.to_owned(), "dictionary"),
        None => match cmu_pronunciation(&lower) {
            Some(phones) => (phones, "dictionary"),
            None => (english_oov(&lower), "oov"),
        },
    }
}

fn append_phones(rendered: &mut String, phones: &str) {
    for phone in phones.split_whitespace() {
        if !rendered.is_empty() && !is_punctuation(phone) {
            rendered.push(' ');
        }
        rendered.push_str(phone);
    }
}

pub fn phonemize_only(
    normalized: &str,
    overrides: &HashMap<String, Override>,
    forced_language: Option<&'static str>,
) -> String {
    let input_tokens = tokenize(normalized);
    let languages = detect_languages(&input_tokens, overrides, forced_language);
    let mut rendered = String::with_capacity(normalized.len().saturating_mul(2));
    for (token, language) in input_tokens.into_iter().zip(languages) {
        if is_punctuation(token) {
            append_phones(&mut rendered, token);
            continue;
        }
        if let Some(phones) =
            find_override(token, overrides).and_then(|entry| entry.phonemes.as_deref())
        {
            append_phones(&mut rendered, phones);
            continue;
        }
        let mut phones = if language == "en" {
            english_g2p(token).0
        } else {
            vietnamese_g2p(token)
        };
        if phones.trim().is_empty() {
            phones.push_str("<unk>");
        }
        append_phones(&mut rendered, &phones);
    }
    rendered
}

pub fn phonemize_text(
    normalized: &str,
    overrides: &HashMap<String, Override>,
    forced_language: Option<&'static str>,
) -> Analysis {
    let input_tokens = tokenize(normalized);
    let languages = detect_languages(&input_tokens, overrides, forced_language);
    let mut tokens_out = Vec::new();
    let mut rendered = String::with_capacity(normalized.len().saturating_mul(2));
    let mut warnings = Vec::new();
    for (token, language) in input_tokens.into_iter().zip(languages) {
        if is_punctuation(token) {
            append_phones(&mut rendered, token);
            tokens_out.push(TokenAnalysis {
                phonemes: token.to_owned(),
                language: "punc".to_owned(),
                source: "punctuation".to_owned(),
                token: token.to_owned(),
            });
            continue;
        }
        if let Some(entry) = find_override(token, overrides) {
            if let Some(phones) = &entry.phonemes {
                append_phones(&mut rendered, phones);
                tokens_out.push(TokenAnalysis {
                    token: token.to_owned(),
                    language: entry.language.clone(),
                    phonemes: phones.clone(),
                    source: "override".to_owned(),
                });
                continue;
            }
        }
        let (mut phones, source) = if language == "en" {
            english_g2p(token)
        } else {
            (vietnamese_g2p(token), "rules")
        };
        if phones.trim().is_empty() {
            phones = "<unk>".to_owned();
            warnings.push(format!("unsupported_token:{token}"));
        } else if source == "oov" {
            warnings.push(format!("english_oov:{token}"));
        }
        append_phones(&mut rendered, &phones);
        tokens_out.push(TokenAnalysis {
            token: token.to_owned(),
            language: language.to_owned(),
            phonemes: phones,
            source: source.to_owned(),
        });
    }
    Analysis {
        input: String::new(),
        normalized: normalized.to_owned(),
        phonemes: rendered,
        tokens: tokens_out,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_sentence() {
        let result = phonemize_text("hôm nay tôi có meeting John.", &HashMap::new(), None);
        assert_eq!(result.phonemes, "hom1 naj1 toj1 kɔ5 miːtɪŋ dʒɔn.");
    }

    #[test]
    fn tone_mapping() {
        assert_eq!(vietnamese_word("ma"), "maː1");
        assert_eq!(vietnamese_word("mà"), "maː2");
        assert_eq!(vietnamese_word("mả"), "maː3");
        assert_eq!(vietnamese_word("mã"), "maː4");
        assert_eq!(vietnamese_word("má"), "maː5");
        assert_eq!(vietnamese_word("mạ"), "maː6");
        assert_eq!(vietnamese_word("gì"), "zi2");
        assert_eq!(vietnamese_word("nguyên"), "ŋwiən1");
        assert_eq!(vietnamese_word("nhiều"), "ɲiəw2");
        assert_eq!(vietnamese_word("thuyết"), "tʰwiət5");
        assert_eq!(vietnamese_word("ngoại"), "ŋwaːj6");
        assert_eq!(vietnamese_word("hôm"), "hom1");
        assert_eq!(vietnamese_word("tôi"), "toj1");
        assert_eq!(vietnamese_word("nay"), "naj1");
        assert_eq!(vietnamese_word("tai"), "taːj1");
        assert_eq!(vietnamese_word("tay"), "taj1");
        assert!(valid_vietnamese_syllable("xoay"));
    }

    #[test]
    fn vowel_length_policy_is_consistent() {
        assert_eq!(rime_ipa("ô"), "o");
        assert_eq!(rime_ipa("ôm"), "om");
        assert_eq!(rime_ipa("ôi"), "oj");
        assert_eq!(rime_ipa("ai"), "aːj");
        assert_eq!(rime_ipa("ay"), "aj");
        assert_eq!(rime_ipa("ao"), "aːw");
        assert_eq!(rime_ipa("au"), "aw");
        assert_eq!(rime_ipa("ơ"), "əː");
        assert_eq!(rime_ipa("â"), "ə");
    }

    #[test]
    fn graphone_oov_fallback() {
        assert_eq!(english_oov("shabe"), "ʃæb");
        assert_eq!(english_oov("foy"), "fɔɪ");
    }

    #[test]
    fn viterbi_switches_without_tags() {
        let tokens = tokenize("hôm nay planning với John.");
        assert_eq!(
            detect_languages(&tokens, &HashMap::new(), None),
            vec!["vi", "vi", "en", "vi", "en", "punc"]
        );
        let tokens = tokenize("tôi đi ra theo quan.");
        assert_eq!(
            detect_languages(&tokens, &HashMap::new(), None),
            vec!["vi", "vi", "vi", "vi", "vi", "punc"]
        );
        let tokens = tokenize("I do it to be kind.");
        assert_eq!(
            detect_languages(&tokens, &HashMap::new(), None),
            vec!["en", "en", "en", "en", "en", "en", "punc"]
        );
        let tokens = tokenize("high-life.");
        assert_eq!(
            detect_languages(&tokens, &HashMap::new(), None),
            vec!["en", "en", "punc"]
        );
    }

    fn languages_of(text: &str) -> Vec<&'static str> {
        detect_languages(&tokenize(text), &HashMap::new(), None)
    }

    #[test]
    fn commas_do_not_cut_language_context() {
        // A word fenced by commas used to start its own segment with no
        // evidence at all, so "nam" came out /næm/ and "tay" /teɪ/.
        for text in [
            "phía đông, nam, dãy đồi ven sông.",
            "chứng bệnh chân, tay, miệng có thể gây đau.",
            "những tiếng, anh, em, chúng ta.",
        ] {
            assert!(
                languages_of(text).iter().all(|l| *l != "en"),
                "comma-fenced Vietnamese fell to English in {text:?}"
            );
        }
    }

    #[test]
    fn sentence_level_vietnamese_prior_survives_a_foreign_name() {
        assert_eq!(
            languages_of("hoàng tử Joachim cho hay."),
            vec!["vi", "vi", "en", "vi", "vi", "punc"]
        );
    }

    #[test]
    fn that_prior_leaves_capitalised_proper_nouns_alone() {
        // "Loop" and "Australia" are valid Vietnamese syllables on paper, so
        // the prior above would happily claim them without this guard.
        let text = "trên đường vòng South Australia Loop, du khách có thể leo lên.";
        let tokens = tokenize(text);
        let languages = languages_of(text);
        for name in ["South", "Australia", "Loop"] {
            let index = tokens.iter().position(|t| *t == name).unwrap();
            assert_eq!(languages[index], "en", "{name} lost its English reading");
        }
    }

    #[test]
    fn corpus_frequency_settles_ascii_collisions() {
        // All three are CMUdict entries, so bare membership called them
        // English: "ba" was even read as the initialism "B.A." (biːeɪ).
        for (text, expected) in [("theo.", "tʰɛw1."), ("ba.", "ɓaː1."), ("cho.", "tʃɔ1.")] {
            assert_eq!(
                phonemize_text(text, &HashMap::new(), None).phonemes,
                expected
            );
        }
    }

    #[test]
    fn english_only_sentences_are_untouched() {
        for text in [
            "I do it to be kind.",
            "The server can handle it.",
            "We can deploy the new model to production tomorrow.",
        ] {
            assert!(
                languages_of(text).iter().all(|l| *l != "vi"),
                "English sentence drifted to Vietnamese: {text:?}"
            );
        }
    }
}
