use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use unicode_normalization::UnicodeNormalization;

use crate::g2p::{is_english_dictionary_word, Override};
use crate::numbers::{digits_to_words, integer_to_words};

#[derive(Clone, Copy)]
pub enum DecimalStyle {
    Cardinal,
    Digits,
}

static TIME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([01]?\d|2[0-3]):([0-5]\d)\b").unwrap());
static DATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:\bngày\s+)?\b(0?[1-9]|[12]\d|3[01])[/.-](0?[1-9]|1[0-2])[/.-](\d{4})\b")
        .unwrap()
});
static VERSION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bv(\d+(?:\.\d+){1,3})\b").unwrap());
static FORMATTED_NUMBER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+(?:[.,]\d+)+\b").unwrap());
static FRACTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)/(\d+)\b").unwrap());
static RANGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(\d+)\s*[-–]\s*(\d+)(?:\s*(kg|km|cm|mm|mg|g|ml|l|hz|khz|mhz|gb|mb|kb|°c))?\b",
    )
    .unwrap()
});
static PERCENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+(?:[.,]\d+)*)\s*%").unwrap());
static CURRENCY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(₫|đ|vnd|\$|usd|€|eur)\s*(\d+(?:[.,]\d+)*)|(\d+(?:[.,]\d+)*)\s*(₫|vnd|usd|eur|€|\$)",
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
static INTEGER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d+\b").unwrap());
static ACRONYM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[A-Z]{2,4}\b").unwrap());
static SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

fn read_fraction(fraction: &str, style: DecimalStyle) -> String {
    if matches!(style, DecimalStyle::Digits) {
        return digits_to_words(fraction);
    }
    let leading_zeros = fraction.chars().take_while(|&c| c == '0').count();
    if leading_zeros == fraction.len() {
        return std::iter::repeat("không")
            .take(leading_zeros)
            .collect::<Vec<_>>()
            .join(" ");
    }
    let mut words = vec!["không".to_owned(); leading_zeros];
    words.push(integer_to_words(&fraction[leading_zeros..]));
    words.join(" ")
}

fn grouped_integer(parts: &[&str]) -> bool {
    parts.len() > 1
        && !parts[0].is_empty()
        && parts[0].len() <= 3
        && parts[1..].iter().all(|part| part.len() == 3)
}

fn read_formatted_number(raw: &str, style: DecimalStyle) -> String {
    let dots = raw.matches('.').count();
    let commas = raw.matches(',').count();
    if dots == 0 && commas == 0 {
        return integer_to_words(raw);
    }

    if dots > 0 && commas > 0 {
        let decimal_index = raw.rfind(['.', ',']).unwrap();
        let decimal_mark = raw[decimal_index..].chars().next().unwrap();
        let whole = raw[..decimal_index].replace(['.', ','], "");
        let fraction = &raw[decimal_index + decimal_mark.len_utf8()..];
        let separator = if decimal_mark == ',' {
            "phẩy"
        } else {
            "chấm"
        };
        return format!(
            "{} {} {}",
            integer_to_words(&whole),
            separator,
            read_fraction(fraction, style)
        );
    }

    let mark = if dots > 0 { '.' } else { ',' };
    let parts = raw.split(mark).collect::<Vec<_>>();
    if grouped_integer(&parts) && (parts.len() > 2 || (mark == '.' && parts[0] != "0")) {
        integer_to_words(&parts.concat())
    } else {
        let fraction = parts.last().copied().unwrap_or_default();
        let whole = parts[..parts.len() - 1].concat();
        let separator = if mark == ',' { "phẩy" } else { "chấm" };
        format!(
            "{} {} {}",
            integer_to_words(&whole),
            separator,
            read_fraction(fraction, style)
        )
    }
}

fn read_letters(raw: &str) -> String {
    raw.chars()
        .filter_map(|c| {
            Some(match c.to_ascii_uppercase() {
                'A' => "ây",
                'B' => "bi",
                'C' => "xi",
                'D' => "đi",
                'E' => "i",
                'F' => "ép",
                'G' => "gi",
                'H' => "âych",
                'I' => "ai",
                'J' => "giây",
                'K' => "cây",
                'L' => "eo",
                'M' => "em",
                'N' => "en",
                'O' => "âu",
                'P' => "pi",
                'Q' => "kiu",
                'R' => "a",
                'S' => "ét",
                'T' => "ti",
                'U' => "diu",
                'V' => "vi",
                'W' => "đắp-bồ-diu",
                'X' => "ích",
                'Y' => "quai",
                'Z' => "di",
                _ => return None,
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn unit_name(unit: &str) -> &'static str {
    match unit.to_ascii_lowercase().as_str() {
        "kg" => "ki-lô-gam",
        "km" => "ki-lô-mét",
        "cm" => "xen-ti-mét",
        "mm" => "mi-li-mét",
        "mg" => "mi-li-gam",
        "g" => "gam",
        "ml" => "mi-li-lít",
        "l" => "lít",
        "hz" => "héc",
        "khz" => "ki-lô-héc",
        "mhz" => "mê-ga-héc",
        "gb" => "gi-ga-bai",
        "mb" => "mê-ga-bai",
        "kb" => "ki-lô-bai",
        "°c" => "độ xê",
        _ => "",
    }
}

fn currency_name(unit: &str) -> &'static str {
    match unit.to_ascii_lowercase().as_str() {
        "₫" | "đ" | "vnd" => "đồng",
        "$" | "usd" => "đô-la Mỹ",
        "€" | "eur" => "ơ-rô",
        _ => "",
    }
}

fn expand_url(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let without_scheme = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .unwrap_or(&lower);
    without_scheme
        .replace("www.", "vê kép vê kép vê chấm ")
        .replace('.', " chấm ")
        .replace('/', " gạch chéo ")
        .replace('?', " hỏi ")
        .replace('&', " và ")
        .replace('=', " bằng ")
        .replace('_', " gạch dưới ")
        .replace('-', " gạch ngang ")
}

fn apply_spoken_overrides(text: String, overrides: &HashMap<String, Override>) -> String {
    let mut keys = overrides
        .iter()
        .filter(|(_, entry)| entry.spoken.is_some())
        .map(|(key, entry)| (key, entry))
        .collect::<Vec<_>>();
    keys.sort_by_key(|(key, _)| std::cmp::Reverse(key.len()));
    let mut out = text;
    for (key, entry) in keys {
        let pattern = format!(r"(?i)(^|[^\p{{L}}\p{{N}}])({})", regex::escape(key));
        if let Ok(re) = Regex::new(&pattern) {
            let spoken = entry.spoken.as_deref().unwrap_or_default();
            out = re
                .replace_all(&out, |caps: &Captures<'_>| {
                    let end = caps.get(0).map_or(0, |m| m.end());
                    let continues_word =
                        out[end..].chars().next().is_some_and(char::is_alphanumeric);
                    if continues_word || (entry.case_sensitive && &caps[2] != key) {
                        caps[0].to_owned()
                    } else {
                        format!("{}{}", &caps[1], spoken)
                    }
                })
                .into_owned();
        }
    }
    out
}

fn lowercase_vietnamese(text: &str) -> String {
    const VI_WORDS: &[&str] = &[
        "Ai", "Ba", "Ban", "Co", "Con", "Da", "Day", "Di", "Duoc", "Hai", "Hay", "Khong", "La",
        "Mot", "Nam", "Ngay", "Nay", "Nguoi", "Nhung", "Phut", "Sau", "Thang", "Toi", "Trong",
        "Va", "Voi", "Xin",
    ];
    text.split_whitespace()
        .map(|word| {
            let core = word.trim_matches(|c: char| !c.is_alphabetic() && c != '-');
            let has_vi_mark = core.chars().any(|c| {
                "ăâđêôơưáàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ"
                    .contains(c.to_lowercase().next().unwrap_or(c))
            });
            if has_vi_mark || VI_WORDS.iter().any(|v| v.eq_ignore_ascii_case(core)) {
                word.to_lowercase()
            } else {
                word.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_punc_char(c: char) -> bool {
    matches!(
        c,
        '.' | ',' | '!' | '?' | ';' | ':' | '…' | '。' | '，' | '、' | '！' | '？' | '；' | '：'
    )
}

fn normalize_punctuation(text: &str, ensure_terminal: bool) -> String {
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
            rough.push_str("và ");
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
) -> String {
    if input.trim().is_empty() {
        return String::new();
    }
    let mut text = input.nfc().collect::<String>();
    text = apply_spoken_overrides(text, overrides);
    text = EMAIL_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!(
                "{} a còng {}",
                c[1].replace('.', " chấm ").replace('_', " gạch dưới "),
                c[2].replace('.', " chấm ").replace('-', " gạch ngang ")
            )
        })
        .into_owned();
    text = URL_RE
        .replace_all(&text, |c: &Captures<'_>| expand_url(&c[0]))
        .into_owned();
    text = VERSION_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!("vê {}", c[1].replace('.', " chấm "))
        })
        .into_owned();
    text = DOTTED_WORD_ACRONYM_RE
        .replace_all(&text, |c: &Captures<'_>| {
            read_letters(
                &c[0]
                    .chars()
                    .filter(char::is_ascii_alphabetic)
                    .collect::<String>(),
            )
        })
        .into_owned();
    text = DOTTED_INITIALS_RE
        .replace_all(&text, |c: &Captures<'_>| {
            read_letters(
                &c[0]
                    .chars()
                    .filter(char::is_ascii_alphabetic)
                    .collect::<String>(),
            )
        })
        .into_owned();
    text = DATE_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!(
                "ngày {} tháng {} năm {}",
                integer_to_words(&c[1]),
                integer_to_words(&c[2]),
                integer_to_words(&c[3])
            )
        })
        .into_owned();
    text = TIME_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!(
                "{} giờ {} phút",
                integer_to_words(&c[1]),
                integer_to_words(&c[2])
            )
        })
        .into_owned();
    text = PHONE_RE
        .replace_all(&text, |c: &Captures<'_>| {
            let raw = c[0].replace("+84", "0");
            digits_to_words(&raw)
        })
        .into_owned();
    text = CURRENCY_RE
        .replace_all(&text, |c: &Captures<'_>| {
            let (unit, number) = if c.get(1).is_some() {
                (&c[1], &c[2])
            } else {
                (&c[4], &c[3])
            };
            format!(
                "{} {}",
                read_formatted_number(number, decimal_style),
                currency_name(unit)
            )
        })
        .into_owned();
    text = PERCENT_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!("{} phần trăm", read_formatted_number(&c[1], decimal_style))
        })
        .into_owned();
    text = FRACTION_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!(
                "{} phần {}",
                integer_to_words(&c[1]),
                integer_to_words(&c[2])
            )
        })
        .into_owned();
    text = RANGE_RE
        .replace_all(&text, |c: &Captures<'_>| {
            let range = format!(
                "{} đến {}",
                integer_to_words(&c[1]),
                integer_to_words(&c[2])
            );
            match c.get(3) {
                Some(unit) => format!("{range} {}", unit_name(unit.as_str())),
                None => range,
            }
        })
        .into_owned();
    text = UNIT_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!(
                "{} {}",
                read_formatted_number(&c[1], decimal_style),
                unit_name(&c[2])
            )
        })
        .into_owned();
    text = FORMATTED_NUMBER_RE
        .replace_all(&text, |c: &Captures<'_>| {
            read_formatted_number(&c[0], decimal_style)
        })
        .into_owned();
    text = ALNUM_ACRONYM_RE
        .replace_all(&text, |c: &Captures<'_>| {
            format!("{} {}", read_letters(&c[1]), digits_to_words(&c[2]))
        })
        .into_owned();
    text = INTEGER_RE
        .replace_all(&text, |c: &Captures<'_>| integer_to_words(&c[0]))
        .into_owned();
    text = ACRONYM_RE
        .replace_all(&text, |c: &Captures<'_>| match &c[0] {
            "AI" => "ây ai".to_owned(),
            "TTS" => "ti ti ét".to_owned(),
            raw if is_english_dictionary_word(raw) => raw.to_owned(),
            raw => read_letters(raw),
        })
        .into_owned();
    text = lowercase_vietnamese(&text);
    normalize_punctuation(&text, ensure_terminal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(text: &str) -> String {
        normalize_text(text, true, DecimalStyle::Cardinal, &HashMap::new())
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
            normalize_text("3.14 và 3,14", true, DecimalStyle::Digits, &HashMap::new()),
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
