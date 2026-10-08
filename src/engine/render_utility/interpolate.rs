//! Port of `JavaTacticalRenderer/clsUtility.InterpolatePixels`.

use crate::engine::flot::get_scaled_size;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

/// Glyph size of the line types that decorate their line, `None` for the
/// rest.
fn glyph_size(tg: &Tg) -> Option<f64> {
    let scaled = |size: f64| get_scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale);
    Some(match tg.line_type {
        ATDITCH | ATDITCHC => 25.0,
        ATDITCHM => 50.0,
        FLOT | LC | FORT_REVD | FORT | FORTL | ENCIRCLE | ZONE | OBSFAREA | OBSAREA | DOUBLEA
        | LWFENCE | HWFENCE | BBS_LINE | SINGLEC | DOUBLEC | TRIPLE | STRONG => scaled(30.0),
        UNSP | LINE | ATWALL | SFENCE => scaled(40.0),
        DFENCE => scaled(50.0),
        _ => return None,
    })
}

/// Upstream `InterpolatePixels`: thins out points that are closer than the
/// glyph size to the last kept point, keeping the end points and the points
/// where the line turns by more than 20 degrees. Does nothing unless the
/// graphic uses line interpolation. Upstream's try block swallows a failed
/// re-insertion for the area types and leaves the points as they were; so
/// does this.
pub(crate) fn interpolate_pixels(tg: &mut Tg) {
    if !tg.use_line_interpolation {
        return;
    }
    let Some(glyph) = glyph_size(tg) else {
        return;
    };
    let pts = &tg.pixels;
    let n = pts.len();
    let mut keep = vec![false; n];
    let mut current = 0usize;
    for j in 0..n {
        if j == 0 || j + 1 == n {
            if let Some(k) = keep.get_mut(j) {
                *k = true;
            }
            continue;
        }
        let (Some(&c), Some(&p1), Some(&p2)) = (pts.get(current), pts.get(j), pts.get(j + 1))
        else {
            return;
        };
        let dist = calc_distance_double(c, p1);
        let dist2 = calc_distance_double(p1, p2);
        // Division by zero gives infinity or NaN, which compare as upstream.
        let direction1 = (180.0 / std::f64::consts::PI) * ((c.y - p1.y) / (c.x - p1.x)).atan();
        let direction2 = (180.0 / std::f64::consts::PI) * ((p1.y - p2.y) / (p1.x - p2.x)).atan();
        let delta = (direction1 - direction2).abs();
        if dist > glyph || dist2 > glyph || delta > 20.0 {
            if let Some(k) = keep.get_mut(j) {
                *k = true;
            }
            current = j;
        }
    }
    let mut kept: Vec<_> = pts
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(p, _)| *p)
        .collect();
    let area = matches!(
        tg.line_type,
        FORT_REVD | FORT | ENCIRCLE | ZONE | OBSFAREA | OBSAREA | STRONG
    );
    if area && kept.len() == 2 {
        if let Some(j) = keep.iter().position(|k| !k) {
            let (Some(&p), true) = (pts.get(j), j <= kept.len()) else {
                return;
            };
            kept.insert(j, p);
        }
    }
    tg.pixels = kept;
}
