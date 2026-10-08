//! Java's text conversions for numbers, as the upstream renderer uses them to
//! pass values between modifiers as strings: `Double.toString`,
//! `Double.parseDouble` and `SymbolUtilities.isNumber`.

#[cfg(test)]
mod tests;

use crate::engine::base::EngineError;

/// Java `Double.toString`: shortest round-trip digits, plain decimal for
/// magnitudes in [1e-3, 1e7), otherwise computerized scientific notation
/// ("1.0E7", "1.5E-4").
pub(crate) fn double_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_owned();
    }
    if v.is_infinite() {
        return if v < 0.0 { "-Infinity" } else { "Infinity" }.to_owned();
    }
    if v == 0.0 {
        return if v.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned();
    }
    let sign = if v < 0.0 { "-" } else { "" };
    let sci = format!("{:e}", v.abs());
    let (mantissa, exp_text) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let exp: i32 = exp_text.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let abs = v.abs();
    if (1e-3..1e7).contains(&abs) {
        format!("{sign}{}", plain(&digits, exp))
    } else {
        let (first, rest) = digits.split_at(1.min(digits.len()));
        let rest = if rest.is_empty() { "0" } else { rest };
        format!("{sign}{first}.{rest}E{exp}")
    }
}

/// Decimal notation for `digits` x 10^(exp - len + 1), at least one digit on
/// each side of the point.
fn plain(digits: &str, exp: i32) -> String {
    let len = i32::try_from(digits.len()).unwrap_or(i32::MAX);
    if exp >= 0 {
        let int_len = exp + 1;
        if len <= int_len {
            let zeros = "0".repeat(usize::try_from(int_len - len).unwrap_or(0));
            format!("{digits}{zeros}.0")
        } else {
            let (int_part, frac) = digits.split_at(usize::try_from(int_len).unwrap_or(0));
            format!("{int_part}.{frac}")
        }
    } else {
        let zeros = "0".repeat(usize::try_from(-exp - 1).unwrap_or(0));
        format!("0.{zeros}{digits}")
    }
}

/// Java `Double.parseDouble` for decimal input: surrounding whitespace
/// (anything up to U+0020) is ignored, a trailing `f`/`F`/`d`/`D` is allowed,
/// and "NaN"/"Infinity" parse. Hexadecimal floating-point text is rejected.
pub(crate) fn parse_double(text: &str) -> Result<f64, EngineError> {
    let fail = || EngineError::Number(text.to_owned());
    let t = text.trim_matches(|c: char| c <= ' ');
    let (neg, body) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let value = match body {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        _ => {
            let number = body.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(body);
            let ok = !number.is_empty()
                && number
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'))
                && number.chars().any(|c| c.is_ascii_digit());
            if !ok {
                return Err(fail());
            }
            number.parse::<f64>().map_err(|_| fail())?
        }
    };
    Ok(if neg { -value } else { value })
}

/// Upstream `SymbolUtilities.isNumber`: whether `parse_double` would accept
/// the text (hexadecimal floats aside).
pub(crate) fn is_number(text: &str) -> bool {
    parse_double(text).is_ok()
}
