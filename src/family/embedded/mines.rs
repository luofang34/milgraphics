//! The mine types of a minefield or mined area: the standard puts the
//! single-point symbol of each type the symbol code's sector 1 modifier
//! names in a row at the graphic's centre.

use crate::construction::{EmbeddedSymbol, SymbolSize};
use crate::family::Ctx;
use crate::geo::GeoPoint;
use crate::sidc::SymbolId;

/// Areas and lines that show their mine types, and whether the row sits at
/// the middle of the line (rather than the centre of the area).
const GRAPHICS: &[(u32, bool)] = &[
    // Minefield, Dynamic Depiction; Mined Area; Mined Area, Fenced.
    (270_707, false),
    (270_800, false),
    (270_801, false),
    // Mineline.
    (290_101, true),
];

/// Editions whose templates show the mine types.
const VERSIONS: [u8; 2] = [15, 16];

/// The single mine types in the order the sector 1 modifier codes combine
/// them: Antipersonnel Mine, with Directional Effects, Antitank Mine, with
/// Anti-handling Device, Wide Area Antitank Mine, Mine Cluster. Mine
/// Cluster is a multipoint graphic of its own, with no single-point symbol
/// to embed, so it is left out of the row.
const TYPES: [Option<u32>; 6] = [
    Some(280_200),
    Some(280_201),
    Some(280_300),
    Some(280_400),
    Some(280_500),
    None,
];
/// Unspecified Mine, the default mine type.
const UNSPECIFIED: u32 = 280_600;
/// Sector 1 codes: single types from 14, then pairs from 20, then triples
/// from 35, each in the lexicographic order of `TYPES`.
const SINGLE: u8 = 14;
const LAST: u8 = 50;

/// Size of a mine symbol and the distance between neighbours, in pixels.
const MINE_PX: f64 = 20.0;
const PITCH_PX: f64 = 24.0;

/// The mine types of sector 1 code `code`, as indices into `TYPES`.
pub(super) fn types(code: u8) -> Vec<usize> {
    let n = TYPES.len();
    let mut combos: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    for i in 0..n {
        for j in i + 1..n {
            combos.push(vec![i, j]);
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                combos.push(vec![i, j, k]);
            }
        }
    }
    match code.checked_sub(SINGLE) {
        Some(index) if code <= LAST => combos.get(usize::from(index)).cloned().unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The entity codes drawn for sector 1 code `code`: one symbol per type,
/// three of a single type as the templates' examples show, and the
/// Unspecified Mine for a code that names no mine type.
pub(super) fn entities(code: u8) -> Vec<u32> {
    let listed = types(code);
    if listed.is_empty() {
        return vec![UNSPECIFIED; 3];
    }
    let drawn: Vec<u32> = listed
        .iter()
        .filter_map(|&i| TYPES.get(i).copied().flatten())
        .collect();
    match drawn.as_slice() {
        [one] if listed.len() == 1 => vec![*one; 3],
        _ => drawn,
    }
}

pub(super) fn add(ctx: &mut Ctx<'_>, symbol: &SymbolId, points: &[GeoPoint]) {
    let entity = symbol.entity().get();
    let Some(&(_, on_line)) = GRAPHICS.iter().find(|g| g.0 == entity) else {
        return;
    };
    if !VERSIONS.contains(&symbol.version_code()) {
        return;
    }
    let anchor = if on_line {
        midline(ctx, points)
    } else {
        super::centre(ctx, points)
    };
    let Some(anchor) = anchor else {
        return;
    };
    let mines = entities(symbol.modifier1());
    let first = -(mines.len().saturating_sub(1) as f64) / 2.0;
    for (i, mine) in mines.into_iter().enumerate() {
        let code = format!(
            "{:02}{}{}25{}000{:06}0000",
            symbol.version_code(),
            symbol.context(),
            symbol.identity(),
            symbol.status(),
            mine
        );
        let Ok(mine) = SymbolId::parse(&code) else {
            continue;
        };
        ctx.add_symbol(EmbeddedSymbol {
            symbol: mine,
            anchor,
            offset_px: [(first + i as f64) * PITCH_PX, 0.0],
            size: SymbolSize::Pixels(MINE_PX),
        });
    }
}

/// The point halfway along the line's length.
fn midline(ctx: &Ctx<'_>, points: &[GeoPoint]) -> Option<GeoPoint> {
    let lengths: Vec<f64> = points
        .windows(2)
        .filter_map(|w| match w {
            [a, b] => Some(ctx.earth.inverse(*a, *b).distance_m),
            _ => None,
        })
        .collect();
    let mut left = lengths.iter().sum::<f64>() / 2.0;
    for (w, &len) in points.windows(2).zip(&lengths) {
        let [a, b] = w else { continue };
        if left <= len && len > 0.0 {
            return Some(ctx.earth.interpolate(*a, *b, left / len));
        }
        left -= len;
    }
    points.first().copied()
}

#[cfg(test)]
mod tests;
