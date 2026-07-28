const DIGITS: [&str; 10] = [
    "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín",
];

fn under_hundred(n: u16, full: bool) -> Vec<&'static str> {
    let tens = n / 10;
    let unit = n % 10;
    let mut out = Vec::new();
    if tens == 0 {
        if full && unit > 0 {
            out.push("lẻ");
        }
    } else if tens == 1 {
        out.push("mười");
    } else {
        out.push(DIGITS[tens as usize]);
        out.push("mươi");
    }
    if unit > 0 {
        out.push(match (tens, unit) {
            (t, 1) if t > 1 => "mốt",
            (t, 4) if t > 1 => "tư",
            (t, 5) if t > 0 => "lăm",
            _ => DIGITS[unit as usize],
        });
    }
    out
}

fn under_thousand(n: u16, force_hundreds: bool) -> Vec<&'static str> {
    let hundreds = n / 100;
    let rest = n % 100;
    let mut out = Vec::new();
    if hundreds > 0 || force_hundreds {
        out.push(DIGITS[hundreds as usize]);
        out.push("trăm");
    }
    out.extend(under_hundred(rest, hundreds > 0 || force_hundreds));
    out
}

pub fn integer_to_words(raw: &str) -> String {
    let clean = raw
        .trim_start_matches('+')
        .replace(['.', ',', '_', ' '], "");
    if clean.is_empty() || !clean.chars().all(|c| c.is_ascii_digit()) {
        return raw.to_owned();
    }
    let clean = clean.trim_start_matches('0');
    if clean.is_empty() {
        return "không".to_owned();
    }
    if clean.len() > 18 {
        return digits_to_words(clean);
    }
    let Ok(value) = clean.parse::<u64>() else {
        return digits_to_words(clean);
    };
    let scales = ["", "nghìn", "triệu", "tỷ", "nghìn tỷ", "triệu tỷ"];
    let mut groups = Vec::new();
    let mut n = value;
    while n > 0 {
        groups.push((n % 1000) as u16);
        n /= 1000;
    }
    let mut out = Vec::new();
    for idx in (0..groups.len()).rev() {
        let group = groups[idx];
        if group == 0 {
            continue;
        }
        let force = idx == 0 && groups.len() > 1 && group < 100;
        out.extend(under_thousand(group, force));
        if !scales[idx].is_empty() {
            out.extend(scales[idx].split_whitespace());
        }
    }
    out.join(" ")
}

pub fn digits_to_words(raw: &str) -> String {
    raw.chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| DIGITS[d as usize])
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinal_contracts() {
        assert_eq!(integer_to_words("25"), "hai mươi lăm");
        assert_eq!(
            integer_to_words("2026"),
            "hai nghìn không trăm hai mươi sáu"
        );
        assert_eq!(integer_to_words("1005"), "một nghìn không trăm lẻ năm");
    }
}
