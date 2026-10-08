//! Port of mil-sym-java JavaTacticalRenderer/clsUtility.getMSRSegmentColors and
//! getMSRSegmentColorStrings, with RendererUtilities.getColorFromHexString:
//! per-segment colours carried in modifier H as "index:color,index:color".

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::engine::tactical_lines::{ASR, BOUNDARY, MSR, TRAFFIC_ROUTE};
use crate::engine::tg::Tg;
use crate::style::Rgba;

/// Java `String.split(regex)` for a literal separator: trailing empty
/// strings are dropped, and a string without the separator comes back whole.
pub(crate) fn java_split(text: &str, separator: char) -> Vec<&str> {
    let mut parts: Vec<&str> = text.split(separator).collect();
    if parts.len() > 1 {
        while parts.last().is_some_and(|p| p.is_empty()) {
            parts.pop();
        }
    }
    parts
}

/// Upstream `RendererUtilities.getColorFromHexString`: "RRGGBB" or
/// "AARRGGBB", optionally prefixed by `#` or `0x`; `None` for anything else.
pub(crate) fn color_from_hex_string(hex: &str) -> Option<Rgba> {
    let stripped = hex.strip_prefix('#').unwrap_or(hex);
    // Upstream reads two characters here, so a shorter string fails.
    let prefix = stripped.get(0..2)?;
    let digits = if prefix == "0x" || prefix == "0X" {
        stripped.get(2..)?
    } else {
        stripped
    };
    if !(digits.len() == 6 || digits.len() == 8) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| {
        digits
            .get(i..i + 2)
            .and_then(|h| u8::from_str_radix(h, 16).ok())
    };
    if digits.len() == 8 {
        Some(Rgba {
            r: byte(2)?,
            g: byte(4)?,
            b: byte(6)?,
            a: byte(0)?,
        })
    } else {
        Some(Rgba::opaque(byte(0)?, byte(2)?, byte(4)?))
    }
}

/// The "index:value" pairs of modifier H, for the line types that carry
/// segment colours; `None` when the type does not or H is empty. A pair that
/// does not parse ends the scan and keeps the pairs read so far, as upstream's
/// exception handler does.
fn segment_values(tg: &Tg) -> Option<BTreeMap<i32, String>> {
    if !matches!(tg.line_type, MSR | ASR | TRAFFIC_ROUTE | BOUNDARY) || tg.h.is_empty() {
        return None;
    }
    let mut map = BTreeMap::new();
    for pair in java_split(&tg.h, ',') {
        if !pair.contains(':') {
            continue;
        }
        let seg = java_split(pair, ':');
        let (Some(index), Some(value)) =
            (seg.first().and_then(|i| i.parse::<i32>().ok()), seg.get(1))
        else {
            break;
        };
        map.insert(index, (*value).to_owned());
    }
    Some(map)
}

/// Upstream `getMSRSegmentColors`: segment index to colour (`None` where the
/// colour text does not parse).
pub(crate) fn msr_segment_colors(tg: &Tg) -> Option<BTreeMap<i32, Option<Rgba>>> {
    segment_values(tg).map(|map| {
        map.into_iter()
            .map(|(index, text)| (index, color_from_hex_string(&text)))
            .collect()
    })
}
