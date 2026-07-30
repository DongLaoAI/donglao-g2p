use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use unicode_normalization::UnicodeNormalization;

use crate::g2p::{is_english_dictionary_word, language_cost, Override};
use crate::numbers::{
    digits_to_words, english_digits_to_words, english_integer_to_words, english_ordinal_to_words,
    integer_to_words,
};

#[derive(Clone, Copy)]
pub enum DecimalStyle {
    Cardinal,
    Digits,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SpokenLanguage {
    Vi,
    En,
}

static TIME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([01]?\d|2[0-3]):([0-5]\d)\b").unwrap());
static DATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:\bngày\s+)?\b(0?[1-9]|[12]\d|3[01])[/.-](0?[1-9]|1[0-2])[/.-](\d{4})\b")
        .unwrap()
});
static VERSION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bv(\d+(?:\.\d+){1,3})\b").unwrap());
static FORMATTED_NUMBER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+(?:[.,]\d+)+\b").unwrap());
static FRACTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)/(\d+)\b").unwrap());
static RANGE_PERCENT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(\d+)\s*[-–]\s*(\d+)\s*%").unwrap());
static RANGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(\d+)\s*[-–]\s*(\d+)(?:\s*(kg|km|cm|mm|mg|g|ml|l|hz|khz|mhz|gb|mb|kb|°c))?\b",
    )
    .unwrap()
});
static PERCENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+(?:[.,]\d+)*)\s*%").unwrap());
static CURRENCY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(₫|đ\b|vnd\b|\$|usd\b|€|eur\b)\s*(\d+(?:[.,]\d+)*)|(\d+(?:[.,]\d+)*)\s*(₫|đ\b|vnd\b|usd\b|eur\b|€|\$)",
    )
    .unwrap()
});
static UNIT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)*)\s*(kg|km|cm|mm|mg|g|ml|l|hz|khz|mhz|gb|mb|kb|°c)\b")
        .unwrap()
});
static PHONE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?x)(?:\+84|0)(?:[\s.-]?\d){8,10}\b").unwrap());
static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b([a-z0-9._%+\-]+)@([a-z0-9\-]+(?:\.[a-z0-9\-]+)+)\b").unwrap());
static URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:https?://|www\.)[a-z0-9][a-z0-9./?&=_:%+\-]*").unwrap());
static DOTTED_WORD_ACRONYM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b[A-Z]{2,4}(?:\.[A-Z]{2,4})+\b").unwrap());
static DOTTED_INITIALS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(?:[A-Z]\.){2,}").unwrap());
static ALNUM_ACRONYM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Z]{2,8})(\d{1,6})\b").unwrap());
static ENGLISH_ORDINAL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(\d+)(?:st|nd|rd|th)\b").unwrap());
static GENERAL_ALNUM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:([a-z]+)(\d+)|(\d+)([a-z]+))\b").unwrap());
static NEGATIVE_NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(^|[^\p{L}\p{N}])-(\d+(?:[.,]\d+)*)\b").unwrap());
static INTEGER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+\b").unwrap());
static ACRONYM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-Z]{2,4}\b").unwrap());
static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
static WORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\p{L}+(?:['’]\p{L}+)?").unwrap());

const VI_WORDS: &[&str] = &[
    "Ai", "Ba", "Ban", "Co", "Con", "Da", "Day", "Di", "Duoc", "Hai", "Hay", "Khong", "La", "Mot",
    "Nam", "Ngay", "Nay", "Nguoi", "Nhung", "Phut", "Sau", "Thang", "Toi", "Trong", "Va", "Voi",
    "Xin",
];

fn replace_all_owned<R: regex::Replacer>(text: String, regex: &Regex, replacer: R) -> String {
    if !regex.is_match(&text) {
        return text;
    }
    regex.replace_all(&text, replacer).into_owned()
}

fn is_vietnamese_mark(c: char) -> bool {
    "ăâđêôơưáàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ"
        .contains(c.to_lowercase().next().unwrap_or(c))
}

fn likely_unmarked_vietnamese(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let core = word.trim_matches(|c: char| !c.is_alphabetic() && c != '-');
        core.chars().next().is_some_and(char::is_uppercase)
            && VI_WORDS.iter().any(|item| item.eq_ignore_ascii_case(core))
    })
}

#[derive(Clone, Copy, Default)]
struct NormalizationFeatures {
    required: bool,
    has_digit: bool,
    has_at: bool,
    has_url: bool,
    has_version: bool,
    has_dotted_uppercase: bool,
    has_date: bool,
    has_time: bool,
    has_phone: bool,
    has_currency: bool,
    has_percent: bool,
    has_fraction: bool,
    has_numeric_dash: bool,
    has_negative: bool,
    has_formatted_number: bool,
    has_alnum: bool,
    has_acronym: bool,
}

fn normalization_features(text: &str) -> NormalizationFeatures {
    let bytes = text.as_bytes();
    let mut features = NormalizationFeatures::default();
    let mut uppercase_run = 0usize;
    for c in text.chars() {
        if c.is_ascii_digit() {
            features.required = true;
            features.has_digit = true;
        }
        match c {
            '&' | '°' | '_' => features.required = true,
            '–' => {
                features.required = true;
                features.has_numeric_dash = true;
            }
            '@' => {
                features.required = true;
                features.has_at = true;
            }
            '$' | '€' | '₫' => {
                features.required = true;
                features.has_currency = true;
            }
            '%' => {
                features.required = true;
                features.has_percent = true;
            }
            ':' => {
                features.required = true;
                features.has_time = true;
            }
            _ => {}
        }
        if c.is_ascii_uppercase() {
            uppercase_run += 1;
            if uppercase_run >= 2 {
                features.required = true;
                features.has_acronym = true;
            }
        } else {
            uppercase_run = 0;
        }
    }
    let digit_count = bytes.iter().filter(|value| value.is_ascii_digit()).count();
    features.has_phone = digit_count >= 9;
    for part in bytes.windows(2) {
        let alpha_digit = part[0].is_ascii_alphabetic() && part[1].is_ascii_digit();
        let digit_alpha = part[0].is_ascii_digit() && part[1].is_ascii_alphabetic();
        features.has_alnum |= alpha_digit || digit_alpha;
        features.has_version |= part[0].eq_ignore_ascii_case(&b'v') && part[1].is_ascii_digit();
        features.has_negative |= part[0] == b'-' && part[1].is_ascii_digit();
    }
    for part in bytes.windows(3) {
        let numeric = part[0].is_ascii_digit() && part[2].is_ascii_digit();
        features.has_date |= numeric && matches!(part[1], b'/' | b'.' | b'-');
        features.has_fraction |= numeric && part[1] == b'/';
        features.has_numeric_dash |= numeric && part[1] == b'-';
        features.has_formatted_number |= numeric && matches!(part[1], b'.' | b',');
        features.has_dotted_uppercase |=
            part[0].is_ascii_uppercase() && part[1] == b'.' && part[2].is_ascii_uppercase();
    }
    features.has_url = text.contains("://") || text.contains("www.");
    features.has_currency |= features.has_digit
        && (text.chars().any(|c| matches!(c, 'đ' | 'Đ'))
            || text.split(|c: char| !c.is_ascii_alphabetic()).any(|word| {
                ["vnd", "usd", "eur"]
                    .iter()
                    .any(|unit| word.eq_ignore_ascii_case(unit))
            }));
    features.required |= features.has_url
        || features.has_dotted_uppercase
        || features.has_alnum
        || features.has_currency;
    features
}

fn integer_words(raw: &str, language: SpokenLanguage) -> String {
    match language {
        SpokenLanguage::Vi => integer_to_words(raw),
        SpokenLanguage::En => english_integer_to_words(raw),
    }
}

fn digit_words(raw: &str, language: SpokenLanguage) -> String {
    match language {
        SpokenLanguage::Vi => digits_to_words(raw),
        SpokenLanguage::En => english_digits_to_words(raw),
    }
}

fn read_fraction(fraction: &str, style: DecimalStyle, language: SpokenLanguage) -> String {
    if matches!(style, DecimalStyle::Digits) {
        return digit_words(fraction, language);
    }
    let leading_zeros = fraction.chars().take_while(|&c| c == '0').count();
    if leading_zeros == fraction.len() {
        let zero = match language {
            SpokenLanguage::Vi => "không",
            SpokenLanguage::En => "zero",
        };
        return std::iter::repeat(zero)
            .take(leading_zeros)
            .collect::<Vec<_>>()
            .join(" ");
    }
    let zero = match language {
        SpokenLanguage::Vi => "không",
        SpokenLanguage::En => "zero",
    };
    let mut words = vec![zero.to_owned(); leading_zeros];
    words.push(integer_words(&fraction[leading_zeros..], language));
    words.join(" ")
}

fn grouped_integer(parts: &[&str]) -> bool {
    parts.len() > 1
        && !parts[0].is_empty()
        && parts[0].len() <= 3
        && parts[1..].iter().all(|part| part.len() == 3)
}

fn read_formatted_number(raw: &str, style: DecimalStyle, language: SpokenLanguage) -> String {
    let dots = raw.matches('.').count();
    let commas = raw.matches(',').count();
    if dots == 0 && commas == 0 {
        return integer_words(raw, language);
    }

    if dots > 0 && commas > 0 {
        let decimal_index = raw.rfind(['.', ',']).unwrap();
        let decimal_mark = raw[decimal_index..].chars().next().unwrap();
        let whole = raw[..decimal_index].replace(['.', ','], "");
        let fraction = &raw[decimal_index + decimal_mark.len_utf8()..];
        let separator = match (language, decimal_mark) {
            (SpokenLanguage::Vi, ',') => "phẩy",
            (SpokenLanguage::Vi, _) => "chấm",
            (SpokenLanguage::En, _) => "point",
        };
        return format!(
            "{} {} {}",
            integer_words(&whole, language),
            separator,
            read_fraction(fraction, style, language)
        );
    }

    let mark = if dots > 0 { '.' } else { ',' };
    let parts = raw.split(mark).collect::<Vec<_>>();
    if grouped_integer(&parts) && (parts.len() > 2 || (mark == '.' && parts[0] != "0")) {
        integer_words(&parts.concat(), language)
    } else {
        let fraction = parts.last().copied().unwrap_or_default();
        let whole = parts[..parts.len() - 1].concat();
        let separator = match (language, mark) {
            (SpokenLanguage::Vi, ',') => "phẩy",
            (SpokenLanguage::Vi, _) => "chấm",
            (SpokenLanguage::En, _) => "point",
        };
        format!(
            "{} {} {}",
            integer_words(&whole, language),
            separator,
            read_fraction(fraction, style, language)
        )
    }
}

fn read_letters(raw: &str, language: SpokenLanguage) -> String {
    raw.chars()
        .filter_map(|c| {
            let upper = c.to_ascii_uppercase();
            Some(match (language, upper) {
                (SpokenLanguage::Vi, 'A') => "ây",
                (SpokenLanguage::Vi, 'B') => "bi",
                (SpokenLanguage::Vi, 'C') => "xi",
                (SpokenLanguage::Vi, 'D') => "đi",
                (SpokenLanguage::Vi, 'E') => "i",
                (SpokenLanguage::Vi, 'F') => "ép",
                (SpokenLanguage::Vi, 'G') => "gi",
                (SpokenLanguage::Vi, 'H') => "âych",
                (SpokenLanguage::Vi, 'I') => "ai",
                (SpokenLanguage::Vi, 'J') => "giây",
                (SpokenLanguage::Vi, 'K') => "cây",
                (SpokenLanguage::Vi, 'L') => "eo",
                (SpokenLanguage::Vi, 'M') => "em",
                (SpokenLanguage::Vi, 'N') => "en",
                (SpokenLanguage::Vi, 'O') => "âu",
                (SpokenLanguage::Vi, 'P') => "pi",
                (SpokenLanguage::Vi, 'Q') => "kiu",
                (SpokenLanguage::Vi, 'R') => "a",
                (SpokenLanguage::Vi, 'S') => "ét",
                (SpokenLanguage::Vi, 'T') => "ti",
                (SpokenLanguage::Vi, 'U') => "diu",
                (SpokenLanguage::Vi, 'V') => "vi",
                (SpokenLanguage::Vi, 'W') => "đắp-bồ-diu",
                (SpokenLanguage::Vi, 'X') => "ích",
                (SpokenLanguage::Vi, 'Y') => "quai",
                (SpokenLanguage::Vi, 'Z') => "di",
                (SpokenLanguage::En, 'A') => "A",
                (SpokenLanguage::En, 'B') => "B",
                (SpokenLanguage::En, 'C') => "C",
                (SpokenLanguage::En, 'D') => "D",
                (SpokenLanguage::En, 'E') => "E",
                (SpokenLanguage::En, 'F') => "F",
                (SpokenLanguage::En, 'G') => "G",
                (SpokenLanguage::En, 'H') => "H",
                (SpokenLanguage::En, 'I') => "eye",
                (SpokenLanguage::En, 'J') => "J",
                (SpokenLanguage::En, 'K') => "K",
                (SpokenLanguage::En, 'L') => "L",
                (SpokenLanguage::En, 'M') => "M",
                (SpokenLanguage::En, 'N') => "N",
                (SpokenLanguage::En, 'O') => "O",
                (SpokenLanguage::En, 'P') => "P",
                (SpokenLanguage::En, 'Q') => "Q",
                (SpokenLanguage::En, 'R') => "R",
                (SpokenLanguage::En, 'S') => "S",
                (SpokenLanguage::En, 'T') => "T",
                (SpokenLanguage::En, 'U') => "U",
                (SpokenLanguage::En, 'V') => "V",
                (SpokenLanguage::En, 'W') => "W",
                (SpokenLanguage::En, 'X') => "X",
                (SpokenLanguage::En, 'Y') => "Y",
                (SpokenLanguage::En, 'Z') => "Z",
                _ => return None,
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn unit_name(unit: &str, language: SpokenLanguage) -> &'static str {
    match (language, unit.to_ascii_lowercase().as_str()) {
        (SpokenLanguage::Vi, "kg") => "ki-lô-gam",
        (SpokenLanguage::Vi, "km") => "ki-lô-mét",
        (SpokenLanguage::Vi, "cm") => "xen-ti-mét",
        (SpokenLanguage::Vi, "mm") => "mi-li-mét",
        (SpokenLanguage::Vi, "mg") => "mi-li-gam",
        (SpokenLanguage::Vi, "g") => "gam",
        (SpokenLanguage::Vi, "ml") => "mi-li-lít",
        (SpokenLanguage::Vi, "l") => "lít",
        (SpokenLanguage::Vi, "hz") => "héc",
        (SpokenLanguage::Vi, "khz") => "ki-lô-héc",
        (SpokenLanguage::Vi, "mhz") => "mê-ga-héc",
        (SpokenLanguage::Vi, "gb") => "gi-ga-bai",
        (SpokenLanguage::Vi, "mb") => "mê-ga-bai",
        (SpokenLanguage::Vi, "kb") => "ki-lô-bai",
        (SpokenLanguage::Vi, "°c") => "độ xê",
        (SpokenLanguage::En, "kg") => "kilograms",
        (SpokenLanguage::En, "km") => "kilometers",
        (SpokenLanguage::En, "cm") => "centimeters",
        (SpokenLanguage::En, "mm") => "millimeters",
        (SpokenLanguage::En, "mg") => "milligrams",
        (SpokenLanguage::En, "g") => "grams",
        (SpokenLanguage::En, "ml") => "milliliters",
        (SpokenLanguage::En, "l") => "liters",
        (SpokenLanguage::En, "hz") => "hertz",
        (SpokenLanguage::En, "khz") => "kilohertz",
        (SpokenLanguage::En, "mhz") => "megahertz",
        (SpokenLanguage::En, "gb") => "gigabytes",
        (SpokenLanguage::En, "mb") => "megabytes",
        (SpokenLanguage::En, "kb") => "kilobytes",
        (SpokenLanguage::En, "°c") => "degrees Celsius",
        _ => "",
    }
}

fn currency_name(unit: &str, language: SpokenLanguage) -> &'static str {
    match (language, unit.to_ascii_lowercase().as_str()) {
        (SpokenLanguage::Vi, "₫" | "đ" | "vnd") => "đồng",
        (SpokenLanguage::Vi, "$" | "usd") => "đô-la Mỹ",
        (SpokenLanguage::Vi, "€" | "eur") => "ơ-rô",
        (SpokenLanguage::En, "₫" | "đ" | "vnd") => "Vietnamese dong",
        (SpokenLanguage::En, "$" | "usd") => "U S dollars",
        (SpokenLanguage::En, "€" | "eur") => "euros",
        _ => "",
    }
}

fn expand_url(raw: &str, language: SpokenLanguage) -> String {
    let lower = raw.to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    match language {
        SpokenLanguage::Vi => without_scheme
            .replace("www.", "vê kép vê kép vê chấm ")
            .replace('.', " chấm ")
            .replace('/', " gạch chéo ")
            .replace('?', " hỏi ")
            .replace('&', " và ")
            .replace('=', " bằng ")
            .replace('_', " gạch dưới ")
            .replace('-', " gạch ngang "),
        SpokenLanguage::En => without_scheme
            .replace("www.", "W W W dot ")
            .replace('.', " dot ")
            .replace('/', " slash ")
            .replace('?', " question mark ")
            .replace('&', " and ")
            .replace('=', " equals ")
            .replace('_', " underscore ")
            .replace('-', " hyphen "),
    }
}

pub struct PreparedSpokenOverride {
    key: String,
    spoken: String,
    regex: Regex,
}

pub fn prepare_spoken_overrides(
    overrides: &HashMap<String, Override>,
) -> Vec<PreparedSpokenOverride> {
    let mut prepared = overrides
        .iter()
        .filter_map(|(key, entry)| {
            let spoken = entry.spoken.as_ref()?;
            let flag = if entry.case_sensitive { "" } else { "(?i)" };
            let pattern = format!(r"{flag}(^|[^\p{{L}}\p{{N}}])({})", regex::escape(key));
            Some(PreparedSpokenOverride {
                key: key.clone(),
                spoken: spoken.clone(),
                regex: Regex::new(&pattern).expect("escaped override regex must compile"),
            })
        })
        .collect::<Vec<_>>();
    prepared.sort_by_key(|entry| std::cmp::Reverse(entry.key.len()));
    prepared
}

fn in_any_span(index: usize, spans: &[(usize, usize)]) -> bool {
    spans
        .iter()
        .any(|&(start, end)| start <= index && index < end)
}

fn detect_spoken_language(text: &str, overrides: &HashMap<String, Override>) -> SpokenLanguage {
    let ignored = EMAIL_RE
        .find_iter(text)
        .chain(URL_RE.find_iter(text))
        .chain(DOTTED_WORD_ACRONYM_RE.find_iter(text))
        .chain(DOTTED_INITIALS_RE.find_iter(text))
        .map(|item| (item.start(), item.end()))
        .collect::<Vec<_>>();
    let mut vi_cost = 0.0f32;
    let mut en_cost = 0.0f32;
    let mut evidence = 0usize;
    for item in WORD_RE.find_iter(text) {
        if in_any_span(item.start(), &ignored) {
            continue;
        }
        let word = item.as_str();
        let before_digit = text[..item.start()]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_digit());
        let after_digit = text[item.end()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit());
        let lower = word.to_lowercase();
        let neutral_unit = matches!(
            lower.as_str(),
            "kg" | "km"
                | "cm"
                | "mm"
                | "mg"
                | "g"
                | "ml"
                | "l"
                | "hz"
                | "khz"
                | "mhz"
                | "gb"
                | "mb"
                | "kb"
                | "vnd"
                | "usd"
                | "eur"
        );
        if before_digit
            || after_digit
            || neutral_unit
            || (word.len() >= 2 && word.chars().all(|c| c.is_ascii_uppercase()))
        {
            continue;
        }
        let (word_vi, word_en) = language_cost(word, overrides);
        vi_cost += word_vi;
        en_cost += word_en;
        evidence += 1;
    }
    if evidence > 0 && en_cost < vi_cost {
        SpokenLanguage::En
    } else {
        SpokenLanguage::Vi
    }
}

fn apply_spoken_overrides(text: String, overrides: &[PreparedSpokenOverride]) -> String {
    let mut out = text;
    for entry in overrides {
        out = entry
            .regex
            .replace_all(&out, |caps: &Captures<'_>| {
                let end = caps.get(0).map_or(0, |m| m.end());
                let continues_word = out[end..].chars().next().is_some_and(char::is_alphanumeric);
                if continues_word {
                    caps[0].to_owned()
                } else {
                    format!("{}{}", &caps[1], entry.spoken)
                }
            })
            .into_owned();
    }
    out
}

fn lowercase_vietnamese(text: String) -> String {
    if !text.chars().any(char::is_uppercase) {
        return text;
    }
    let mut out = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        let core = word.trim_matches(|c: char| !c.is_alphabetic() && c != '-');
        let should_lowercase = core.chars().any(char::is_uppercase)
            && (core.chars().any(is_vietnamese_mark)
                || VI_WORDS.iter().any(|item| item.eq_ignore_ascii_case(core)));
        if should_lowercase {
            out.extend(word.chars().flat_map(char::to_lowercase));
        } else {
            out.push_str(word);
        }
    }
    out
}

fn is_punc_char(c: char) -> bool {
    matches!(
        c,
        '.' | ',' | '!' | '?' | ';' | ':' | '…' | '。' | '，' | '、' | '！' | '？' | '；' | '：'
    )
}

fn punctuation_is_canonical(text: &str, ensure_terminal: bool) -> bool {
    if text.is_empty() || text.trim() != text || (ensure_terminal && !text.ends_with('.')) {
        return false;
    }
    let mut chars = text.chars().peekable();
    let mut previous = None;
    while let Some(c) = chars.next() {
        let next = chars.peek().copied();
        if c.is_whitespace() {
            if c != ' ' || previous.is_some_and(char::is_whitespace) {
                return false;
            }
        } else if matches!(c, ',' | '.') {
            if previous.map_or(true, char::is_whitespace)
                || next.is_some_and(|value| !value.is_whitespace())
            {
                return false;
            }
        } else if matches!(c, '\'' | '’' | '-') {
            if !previous.is_some_and(char::is_alphabetic) || !next.is_some_and(char::is_alphabetic)
            {
                return false;
            }
        } else if "\"“”‘`´«»‹›()[]{}<>《》「」『』&!?;:…。|，、！？」；：–—".contains(c)
        {
            return false;
        }
        previous = Some(c);
    }
    true
}

fn normalize_punctuation(text: &str, ensure_terminal: bool, language: SpokenLanguage) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut rough = String::with_capacity(text.len() + 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '\'' | '’') {
            let internal = i > 0
                && i + 1 < chars.len()
                && chars[i - 1].is_alphabetic()
                && chars[i + 1].is_alphabetic();
            if internal {
                rough.push('\'');
            }
            i += 1;
            continue;
        }
        if "\"“”‘`´«»‹›".contains(c) {
            i += 1;
            continue;
        }
        if "()[]{}<>《》「」『』".contains(c) {
            if rough.chars().last().is_some_and(char::is_alphanumeric)
                && chars[i + 1..]
                    .iter()
                    .next()
                    .is_some_and(|v| v.is_alphanumeric())
                && !rough.ends_with(' ')
            {
                rough.push(' ');
            }
            i += 1;
            continue;
        }
        if matches!(c, '–' | '—')
            || (c == '-' && {
                let prev_word = i > 0 && chars[i - 1].is_alphanumeric();
                let next_word = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();
                !(prev_word && next_word)
            })
        {
            rough.push(',');
            i += 1;
            continue;
        }
        if c == '&' {
            if !rough.ends_with(' ') {
                rough.push(' ');
            }
            rough.push_str(match language {
                SpokenLanguage::Vi => "và ",
                SpokenLanguage::En => "and ",
            });
            i += 1;
            continue;
        }
        if is_punc_char(c) {
            let start = i;
            while i < chars.len() && is_punc_char(chars[i]) {
                i += 1;
            }
            let run = &chars[start..i];
            let has_after = chars[i..].iter().any(|v| v.is_alphanumeric());
            let token = if run.iter().any(|v| matches!(v, '?' | '？' | '!' | '！')) {
                '.'
            } else {
                let dots = run.iter().filter(|v| matches!(v, '.' | '…' | '。')).count();
                if dots > 0 && (dots == 1 || !has_after) {
                    '.'
                } else {
                    ','
                }
            };
            rough.push(token);
            continue;
        }
        rough.push(c);
        i += 1;
    }

    let mut tokens = Vec::<String>::new();
    let mut current = String::new();
    for c in rough.chars() {
        if matches!(c, ',' | '.') {
            if !current.trim().is_empty() {
                tokens.push(current.trim().to_owned());
            }
            current.clear();
            if matches!(tokens.last().map(String::as_str), Some(",") | Some(".")) {
                tokens.pop();
            }
            tokens.push(c.to_string());
        } else if c.is_whitespace() {
            if !current.is_empty() && !current.ends_with(' ') {
                current.push(' ');
            }
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_owned());
    }
    if ensure_terminal
        && !tokens.is_empty()
        && !matches!(tokens.last().map(String::as_str), Some("."))
    {
        if matches!(tokens.last().map(String::as_str), Some(",")) {
            tokens.pop();
        }
        tokens.push(".".to_owned());
    }

    let mut out = String::new();
    for token in tokens {
        if matches!(token.as_str(), "," | ".") {
            while out.ends_with(' ') {
                out.pop();
            }
            out.push_str(&token);
            out.push(' ');
        } else {
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
            out.push_str(&SPACE_RE.replace_all(&token, " "));
            out.push(' ');
        }
    }
    out.trim().to_owned()
}

pub fn normalize_text(
    input: &str,
    ensure_terminal: bool,
    decimal_style: DecimalStyle,
    overrides: &HashMap<String, Override>,
    spoken_overrides: &[PreparedSpokenOverride],
) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    let mut text = input.nfc().collect::<String>();
    text = apply_spoken_overrides(text, spoken_overrides);
    let features = normalization_features(&text);
    let has_vi_mark = text.chars().any(is_vietnamese_mark);
    let language = if has_vi_mark || likely_unmarked_vietnamese(&text) {
        SpokenLanguage::Vi
    } else if features.required {
        detect_spoken_language(&text, overrides)
    } else {
        SpokenLanguage::En
    };
    if features.required {
        if features.has_at {
            text = replace_all_owned(text, &EMAIL_RE, |c: &Captures<'_>| match language {
                SpokenLanguage::Vi => format!(
                    "{} a còng {}",
                    c[1].replace('.', " chấm ").replace('_', " gạch dưới "),
                    c[2].replace('.', " chấm ").replace('-', " gạch ngang ")
                ),
                SpokenLanguage::En => format!(
                    "{} at {}",
                    c[1].replace('.', " dot ").replace('_', " underscore "),
                    c[2].replace('.', " dot ").replace('-', " hyphen ")
                ),
            });
        }
        if features.has_url {
            text = replace_all_owned(text, &URL_RE, |c: &Captures<'_>| {
                expand_url(&c[0], language)
            });
        }
        if features.has_version {
            text = replace_all_owned(text, &VERSION_RE, |c: &Captures<'_>| match language {
                SpokenLanguage::Vi => format!("vê {}", c[1].replace('.', " chấm ")),
                SpokenLanguage::En => format!("version {}", c[1].replace('.', " point ")),
            });
        }
        if features.has_dotted_uppercase {
            text = replace_all_owned(text, &DOTTED_WORD_ACRONYM_RE, |c: &Captures<'_>| {
                read_letters(
                    &c[0]
                        .chars()
                        .filter(char::is_ascii_alphabetic)
                        .collect::<String>(),
                    language,
                )
            });
            text = replace_all_owned(text, &DOTTED_INITIALS_RE, |c: &Captures<'_>| {
                read_letters(
                    &c[0]
                        .chars()
                        .filter(char::is_ascii_alphabetic)
                        .collect::<String>(),
                    language,
                )
            });
        }
        if features.has_date {
            text = replace_all_owned(text, &DATE_RE, |c: &Captures<'_>| match language {
                SpokenLanguage::Vi => format!(
                    "ngày {} tháng {} năm {}",
                    integer_to_words(&c[1]),
                    integer_to_words(&c[2]),
                    integer_to_words(&c[3])
                ),
                SpokenLanguage::En => format!(
                    "day {} month {} year {}",
                    english_ordinal_to_words(&c[1]),
                    english_integer_to_words(&c[2]),
                    english_integer_to_words(&c[3])
                ),
            });
        }
        if features.has_time {
            text = replace_all_owned(text, &TIME_RE, |c: &Captures<'_>| match language {
                SpokenLanguage::Vi => format!(
                    "{} giờ {} phút",
                    integer_to_words(&c[1]),
                    integer_to_words(&c[2])
                ),
                SpokenLanguage::En => format!(
                    "{} {}",
                    english_integer_to_words(&c[1]),
                    english_integer_to_words(&c[2])
                ),
            });
        }
        if features.has_phone {
            text = replace_all_owned(text, &PHONE_RE, |c: &Captures<'_>| {
                let raw = match language {
                    SpokenLanguage::Vi => c[0].replace("+84", "0"),
                    SpokenLanguage::En => c[0].trim_start_matches('+').to_owned(),
                };
                digit_words(&raw, language)
            });
        }
        if features.has_currency {
            text = replace_all_owned(text, &CURRENCY_RE, |c: &Captures<'_>| {
                let (unit, number) = if c.get(1).is_some() {
                    (&c[1], &c[2])
                } else {
                    (&c[4], &c[3])
                };
                format!(
                    "{} {}",
                    read_formatted_number(number, decimal_style, language),
                    currency_name(unit, language)
                )
            });
        }
        if features.has_percent && features.has_numeric_dash {
            text = replace_all_owned(text, &RANGE_PERCENT_RE, |c: &Captures<'_>| {
                let connector = match language {
                    SpokenLanguage::Vi => "đến",
                    SpokenLanguage::En => "to",
                };
                let percent = match language {
                    SpokenLanguage::Vi => "phần trăm",
                    SpokenLanguage::En => "percent",
                };
                format!(
                    "{} {} {} {}",
                    integer_words(&c[1], language),
                    connector,
                    integer_words(&c[2], language),
                    percent
                )
            });
        }
        if features.has_percent {
            text = replace_all_owned(text, &PERCENT_RE, |c: &Captures<'_>| {
                let percent = match language {
                    SpokenLanguage::Vi => "phần trăm",
                    SpokenLanguage::En => "percent",
                };
                format!(
                    "{} {percent}",
                    read_formatted_number(&c[1], decimal_style, language)
                )
            });
        }
        if features.has_fraction {
            text = replace_all_owned(text, &FRACTION_RE, |c: &Captures<'_>| {
                let connector = match language {
                    SpokenLanguage::Vi => "phần",
                    SpokenLanguage::En => "over",
                };
                format!(
                    "{} {connector} {}",
                    integer_words(&c[1], language),
                    integer_words(&c[2], language)
                )
            });
        }
        if features.has_numeric_dash {
            text = replace_all_owned(text, &RANGE_RE, |c: &Captures<'_>| {
                let connector = match language {
                    SpokenLanguage::Vi => "đến",
                    SpokenLanguage::En => "to",
                };
                let range = format!(
                    "{} {connector} {}",
                    integer_words(&c[1], language),
                    integer_words(&c[2], language)
                );
                match c.get(3) {
                    Some(unit) => format!("{range} {}", unit_name(unit.as_str(), language)),
                    None => range,
                }
            });
        }
        if features.has_negative {
            text = replace_all_owned(text, &NEGATIVE_NUMBER_RE, |c: &Captures<'_>| {
                format!("{}\u{e000}{}", &c[1], &c[2])
            });
        }
        if features.has_digit {
            text = replace_all_owned(text, &UNIT_RE, |c: &Captures<'_>| {
                format!(
                    "{} {}",
                    read_formatted_number(&c[1], decimal_style, language),
                    unit_name(&c[2], language)
                )
            });
        }
        if features.has_formatted_number {
            text = replace_all_owned(text, &FORMATTED_NUMBER_RE, |c: &Captures<'_>| {
                read_formatted_number(&c[0], decimal_style, language)
            });
        }
        if features.has_alnum {
            text = replace_all_owned(text, &ALNUM_ACRONYM_RE, |c: &Captures<'_>| {
                format!(
                    "{} {}",
                    read_letters(&c[1], language),
                    digit_words(&c[2], language)
                )
            });
            if language == SpokenLanguage::En {
                text = replace_all_owned(text, &ENGLISH_ORDINAL_RE, |c: &Captures<'_>| {
                    english_ordinal_to_words(&c[1])
                });
            }
            text = replace_all_owned(text, &GENERAL_ALNUM_RE, |c: &Captures<'_>| {
                if let (Some(letters), Some(digits)) = (c.get(1), c.get(2)) {
                    format!(
                        "{} {}",
                        read_letters(letters.as_str(), language),
                        digit_words(digits.as_str(), language)
                    )
                } else {
                    let digits = c.get(3).map_or("", |item| item.as_str());
                    let letters = c.get(4).map_or("", |item| item.as_str());
                    format!(
                        "{} {}",
                        integer_words(digits, language),
                        read_letters(letters, language)
                    )
                }
            });
        }
        if features.has_digit {
            text = replace_all_owned(text, &INTEGER_RE, |c: &Captures<'_>| {
                integer_words(&c[0], language)
            });
        }
        if features.has_acronym {
            text = replace_all_owned(text, &ACRONYM_RE, |c: &Captures<'_>| match &c[0] {
                "AI" if language == SpokenLanguage::Vi => "ây ai".to_owned(),
                "TTS" if language == SpokenLanguage::Vi => "ti ti ét".to_owned(),
                raw if is_english_dictionary_word(raw) => raw.to_owned(),
                raw => read_letters(raw, language),
            });
        }
        let negative = match language {
            SpokenLanguage::Vi => "âm ",
            SpokenLanguage::En => "negative ",
        };
        text = text.replace('\u{e000}', negative);
    }
    if language == SpokenLanguage::Vi {
        text = lowercase_vietnamese(text);
    }
    if punctuation_is_canonical(&text, ensure_terminal) {
        return text;
    }
    normalize_punctuation(&text, ensure_terminal, language)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(text: &str) -> String {
        normalize_text(text, true, DecimalStyle::Cardinal, &HashMap::new(), &[])
    }

    #[test]
    fn golden_normalization() {
        assert_eq!(norm("25 kg"), "hai mươi lăm ki-lô-gam.");
        assert_eq!(norm("12:30"), "mười hai giờ ba mươi phút.");
        assert_eq!(norm("2026"), "hai nghìn không trăm hai mươi sáu.");
        assert_eq!(norm("3.14"), "ba chấm mười bốn.");
        assert_eq!(norm("AI"), "ây ai.");
        assert_eq!(norm("TTS"), "ti ti ét.");
        assert_eq!(norm("OpenAI"), "OpenAI.");
    }

    #[test]
    fn locale_aware_formatted_numbers() {
        assert_eq!(norm("3.14"), "ba chấm mười bốn.");
        assert_eq!(norm("3,14"), "ba phẩy mười bốn.");
        assert_eq!(norm("0.05"), "không chấm không năm.");
        assert_eq!(norm("0,05"), "không phẩy không năm.");
        assert_eq!(norm("3,014"), "ba phẩy không mười bốn.");
        assert_eq!(norm("1.234"), "một nghìn hai trăm ba mươi tư.");
        assert_eq!(
            norm("12.345,67"),
            "mười hai nghìn ba trăm bốn mươi lăm phẩy sáu mươi bảy."
        );
        assert_eq!(
            norm("12,345.67"),
            "mười hai nghìn ba trăm bốn mươi lăm chấm sáu mươi bảy."
        );
    }

    #[test]
    fn digit_decimal_style() {
        assert_eq!(
            normalize_text(
                "3.14 và 3,14",
                true,
                DecimalStyle::Digits,
                &HashMap::new(),
                &[],
            ),
            "ba chấm một bốn và ba phẩy một bốn."
        );
    }

    #[test]
    fn punctuation_policy() {
        assert_eq!(
            norm("Hôm nay... tôi có meeting với John!!!"),
            "hôm nay, tôi có meeting với John."
        );
        assert_eq!(norm("Bạn khỏe?!"), "bạn khỏe.");
        assert_eq!(norm("Chờ..."), "chờ.");
        assert_eq!(norm("A; B: C"), "A, B, C.");
        assert_eq!(norm("A！ B？ C； D： E。"), "A. B. C, D, E.");
    }

    #[test]
    fn normalization_is_idempotent() {
        let once = norm("Giá 25 kg... lúc 12:30!!!");
        assert_eq!(norm(&once), once);
    }
}
