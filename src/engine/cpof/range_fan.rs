//! Ports of the range-fan helpers of `clsUtilityCPOF` and
//! `clsUtility.GetSectorRadiiFromPoints`, on pixel coordinates.

use super::planar;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::java_text::{double_to_string, parse_double};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::{extend_along_line_double, extend_directed_line};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segment_colors::java_split;

/// Upstream `GetConcentricCircles`: one circle per non-zero radius in AM,
/// about the single anchor point. Upstream's exception handler leaves the
/// pixels untouched when there is not exactly one anchor point or AM does not
/// parse; so does this.
pub(super) fn get_concentric_circles(tg: &mut Tg, control: &[Pt], mpp: f64) {
    let Some(center) = control.first().copied().filter(|_| control.len() == 1) else {
        return;
    };
    let parsed: Result<Vec<f64>, EngineError> = java_split(&tg.am, ',')
        .into_iter()
        .map(parse_double)
        .collect();
    let Ok(radii) = parsed else {
        return;
    };
    for radius in radii {
        if radius == 0.0 {
            continue;
        }
        let edge = planar::coordinate(center, radius, 90.0, mpp);
        let pts = planar::geodesic_arc(center, edge, edge, mpp);
        let last = pts.len().saturating_sub(1);
        for (j, mut p) in pts.into_iter().enumerate() {
            p.style = if j == last { 5 } else { 0 };
            tg.pixels.push(p);
        }
    }
    if let Some(last) = tg.pixels.last_mut() {
        last.style = 5;
    }
}

/// Upstream `GetMaxSector`: the "left,right,min,max" group of LRMM with the
/// largest max range, or `None` when LRMM is not whole groups of four numbers.
pub(super) fn get_max_sector(lrmm: &str) -> Option<String> {
    let parts = java_split(lrmm, ',');
    let sectors = parts.len() / 4;
    if sectors < 1 || sectors * 4 != parts.len() {
        return None;
    }
    let mut max_index: Option<usize> = None;
    let mut best = -f64::MAX;
    for k in 0..sectors {
        let max = parse_double(parts.get(4 * k + 3)?).ok()?;
        if max > best {
            best = max;
            max_index = Some(k);
        }
    }
    let k = max_index?;
    let group: Vec<&str> = (0..4)
        .filter_map(|i| parts.get(4 * k + i).copied())
        .collect();
    Some(group.join(","))
}

/// Upstream `GetSectorRadiiFromPoints`: with more than two anchor points,
/// LRMM is rebuilt from the points as bearing and distance from the first
/// point, two points per sector.
pub(super) fn sector_radii_from_points(tg: &mut Tg, control: &[Pt], mpp: f64) {
    if tg.line_type == RANGE_FAN_FILL || control.len() <= 2 {
        return;
    }
    let Some(center) = control.first().copied() else {
        return;
    };
    let sectors = (control.len() - 2) / 2;
    let mut groups: Vec<String> = Vec::new();
    for k in 0..sectors {
        let (Some(left_min), Some(right_max)) = (control.get(2 * k + 2), control.get(2 * k + 3))
        else {
            break;
        };
        let min = planar::distance_m(center, *left_min, mpp);
        let left = planar::azimuth(center, *left_min);
        let max = planar::distance_m(center, *right_max, mpp);
        let right = planar::azimuth(center, *right_max);
        groups.push(format!(
            "{},{},{},{}",
            double_to_string(left),
            double_to_string(right),
            double_to_string(min),
            double_to_string(max)
        ));
    }
    if !groups.is_empty() {
        tg.lrmm = groups.join(",");
    }
}

/// Upstream `GetSectorRangeFan`: the outline of each sector (inner arc,
/// outer arc reversed, closing point) appended to `tg.pixels`. Returns false
/// when LRMM is unusable, leaving the pixels as they were.
pub(super) fn get_sector_range_fan(tg: &mut Tg, control: &[Pt], mpp: f64) -> bool {
    let Some(center) = control.first().copied() else {
        return false;
    };
    sector_radii_from_points(tg, control, mpp);
    let parts = java_split(&tg.lrmm, ',');
    let sectors = parts.len() / 4;
    if sectors < 1 || sectors * 4 != parts.len() {
        return false;
    }
    let numbers: Result<Vec<f64>, EngineError> = parts.iter().map(|t| parse_double(t)).collect();
    let Ok(numbers) = numbers else {
        return false;
    };
    let mut all: Vec<Pt> = Vec::new();
    for group in numbers.chunks(4) {
        let [left, right, min, max] = *group else {
            return false;
        };
        let (inner, _) = sector_arc(center, min, left, right, mpp);
        let (outer, _) = sector_arc(center, max, left, right, mpp);
        all.extend(inner.iter().copied());
        all.extend(outer.iter().rev().copied());
        if let Some(first) = inner.first() {
            let mut close = *first;
            close.style = 5;
            all.push(close);
        }
    }
    let mut last: Option<Pt> = None;
    for p in all {
        if last.is_some_and(|l| l.x == p.x && l.y == p.y) {
            continue;
        }
        tg.pixels.push(p);
        last = Some(p);
    }
    true
}

fn sector_arc(center: Pt, radius: f64, left: f64, right: f64, mpp: f64) -> (Vec<Pt>, bool) {
    let p1 = planar::coordinate(center, radius, left, mpp);
    let p2 = planar::coordinate(center, radius, right, mpp);
    planar::geodesic_arc2(center, p1, p2, mpp)
}

/// Upstream `RangeFanOrientation`: the arrow beyond the largest range that
/// shows the fan's direction, appended to `tg.pixels`.
pub(super) fn range_fan_orientation(tg: &mut Tg, control: &[Pt], mpp: f64) {
    let Some(pt0) = control.first().copied() else {
        return;
    };
    let (dist_m, orientation) = if let Some(pt1) = control.get(1) {
        (
            planar::distance_m(pt0, *pt1, mpp),
            planar::azimuth(pt0, *pt1),
        )
    } else {
        let Some(sector) = get_max_sector(&tg.lrmm) else {
            return;
        };
        let nums: Vec<f64> = java_split(&sector, ',')
            .into_iter()
            .filter_map(|t| parse_double(t).ok())
            .collect();
        let (Some(&left), Some(&right), Some(&max)) = (nums.first(), nums.get(1), nums.get(3))
        else {
            return;
        };
        (max, sector_orientation(left, right))
    };
    let radius = dist_m * 1.1;
    let pt1 = planar::coordinate(pt0, radius, orientation, mpp);
    let dist = calc_distance_double(pt0, pt1);
    let mut base = 10.0;
    if dist < 100.0 {
        base = dist / 10.0;
    }
    if base < 5.0 {
        base = 5.0;
    }
    let base_pt = extend_along_line_double(pt0, pt1, dist + base);
    let mut tip = extend_along_line_double(pt0, pt1, dist + 2.0 * base);
    let left = extend_directed_line(pt0, base_pt, base_pt, 0, base);
    let right = extend_directed_line(pt0, base_pt, base_pt, 1, base);
    // Upstream appends the same point object twice and changes its style
    // between the appends, so both entries end with the later style.
    tip.style = 0;
    tg.pixels.extend([pt0, tip, left, tip, right]);
}

/// The middle of the sector in 0..360 as upstream's loops normalise it.
fn sector_orientation(left: f64, right: f64) -> f64 {
    let wrap = |mut v: f64| {
        while v > 360.0 {
            v -= 360.0;
        }
        while v < 0.0 {
            v += 360.0;
        }
        v
    };
    let (left, right) = (wrap(left), wrap(right));
    if left > right {
        (left - 360.0 + right) / 2.0
    } else {
        (left + right) / 2.0
    }
}

/// Upstream `GetCircularRangeFanFillTG`: a stand-in graphic of type
/// `RANGE_FAN_FILL` that draws the sector fills of a range fan. `control`
/// are the range fan's anchor points; only the first is kept. The returned
/// control list is what the stand-in's own change 1 pass must be given.
pub(crate) fn circular_range_fan_fill_tg(
    tg: &Tg,
    settings: &Settings,
    control: &[Pt],
) -> Result<(Tg, Vec<Pt>), EngineError> {
    let mut fill = Tg::new(settings);
    fill.line_thickness = 0;
    fill.fill_color = tg.fill_color;
    fill.fill_style = tg.fill_style;
    fill.pixels = vec![tg.pixels.at(0)?, tg.pixels.at(1)?];
    fill.line_type = RANGE_FAN_FILL;
    let center = control.at(0)?;
    if tg.line_type == RANGE_FAN_SECTOR || tg.line_type == RADAR_SEARCH {
        fill.lrmm.clone_from(&tg.lrmm);
    } else if tg.line_type == RANGE_FAN {
        let radii = java_split(&tg.am, ',');
        let groups: Vec<String> = radii
            .windows(2)
            .map(|w| {
                format!(
                    "0,0,{},{}",
                    w.first().unwrap_or(&""),
                    w.get(1).unwrap_or(&"")
                )
            })
            .collect();
        fill.lrmm = groups.join(",");
    }
    Ok((fill, vec![center]))
}
