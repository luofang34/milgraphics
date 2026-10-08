//! `addSectorModifiers` of Modifier2.java: the range, azimuth and altitude
//! labels of range-fan sectors and the name of a radar search.
//!
//! Upstream measures these on the ellipsoid through the point converter.
//! Here a distance in metres is divided by the ground size of a pixel and
//! an azimuth is the clockwise angle from pixel north, which is the same
//! thing in the local frame the pixels are drawn in.

use super::AREA;
use super::add::add_area_modifier;
use super::layout::{parse_double, remove_decimal_value};
use super::post::SectorFrame;
use super::post_areas::split_commas;
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;

/// The point `meters` from `origin` towards the azimuth `azimuth_deg`.
fn offset_point(origin: Pt, meters: f64, frame: SectorFrame, azimuth_deg: f64) -> Pt {
    let pixels = meters / frame.meters_per_pixel;
    let az = azimuth_deg.to_radians();
    Pt::new(origin.x + pixels * az.sin(), origin.y - pixels * az.cos())
}

/// Azimuth in degrees of `to` seen from `from`, clockwise from pixel north.
fn azimuth(from: Pt, to: Pt) -> f64 {
    (to.x - from.x).atan2(from.y - to.y).to_degrees()
}

/// Labels the sector line types; any other type adds nothing.
pub(super) fn sector_labels(
    tg: &mut Tg,
    line_type: i32,
    frame: SectorFrame,
) -> Result<(), EngineError> {
    if !matches!(line_type, tl::RANGE_FAN_SECTOR | tl::RADAR_SEARCH) {
        return Ok(());
    }
    if !(frame.meters_per_pixel.is_finite() && frame.meters_per_pixel > 0.0) {
        return Err(EngineError::Degenerate("meters per pixel must be positive"));
    }
    // Upstream catches any failure here (a missing or malformed amplifier)
    // and goes on with the labels it has.
    match line_type {
        tl::RANGE_FAN_SECTOR => range_fan_sector(tg, frame).ok(),
        _ => radar_search(tg, frame).ok(),
    };
    Ok(())
}

/// Parses every comma-separated number; `None` where upstream's
/// `NumberFormatException` makes it give up.
fn parse_all(items: &[&str]) -> Option<Vec<f64>> {
    items.iter().map(|s| parse_double(s).ok()).collect()
}

/// `RANGE_FAN_SECTOR`: per sector the range at the middle of the sector's
/// radii, its two azimuths and, when given, the altitude.
fn range_fan_sector(tg: &mut Tg, frame: SectorFrame) -> Result<(), EngineError> {
    let (am_text, an_text, x_text) = (tg.am.clone(), tg.an.clone(), tg.x.clone());
    let (am, an) = (split_commas(&am_text), split_commas(&an_text));
    let sectors = an.len() / 2;
    if sectors < 1 {
        return Ok(());
    }
    let altitudes = if x_text.is_empty() {
        None
    } else {
        Some(split_commas(&x_text))
    };
    let (Some(mut ranges), Some(azimuths)) = (parse_all(&am), parse_all(&an)) else {
        return Ok(());
    };
    if sectors + 1 > ranges.len() && parse_double(am.first().copied().unwrap_or(""))? != 0.0 {
        ranges.insert(0, 0.0);
    }
    let n = tg.pixels.len();
    let origin = tg.pixels.at(n
        .checked_sub(5)
        .ok_or(EngineError::Index { index: -5, len: n })?)?;
    let tip = tg.pixels.at(n - 4)?;
    let az12 = azimuth(origin, tip);
    let (mut middles, mut edges) = (Vec::new(), Vec::new());
    for k in 0..sectors {
        let (Some(inner), Some(outer)) = (ranges.get(k), ranges.get(k + 1)) else {
            break;
        };
        let radius = (inner + outer) / 2.0;
        middles.push(offset_point(origin, radius, frame, az12));
        if tg.hide_optional_labels {
            continue;
        }
        for edge in [2 * k, 2 * k + 1] {
            edges.push(offset_point(origin, radius, frame, azimuths.at(edge)?));
        }
    }
    if let Some(altitudes) = altitudes {
        for (alt, p) in altitudes.iter().zip(&middles) {
            add_area_modifier(tg, &format!("ALT {alt}"), AREA, 0.0, (*p, *p));
        }
    }
    if !tg.hide_optional_labels {
        for k in 0..sectors {
            let p = middles.at(k)?;
            add_area_modifier(
                tg,
                &format!("RG {}", remove_decimal_value(ranges.at(k + 1)?)),
                AREA,
                -1.0,
                (p, p),
            );
            for edge in [2 * k, 2 * k + 1] {
                let (left, text) = (edges.at(edge)?, an.at(edge)?);
                let text = super::layout::remove_decimal(text)?;
                add_area_modifier(tg, &text, AREA, 0.0, (left, left));
            }
        }
    }
    Ok(())
}

/// `RADAR_SEARCH`: the name at the middle of the first range band, along
/// the bisector of the search sector.
fn radar_search(tg: &mut Tg, frame: SectorFrame) -> Result<(), EngineError> {
    let lrmm = tg.lrmm.clone();
    let sector = split_commas(&lrmm);
    let normalize = |mut a: f64| {
        while a > 360.0 {
            a -= 360.0;
        }
        while a < 0.0 {
            a += 360.0;
        }
        a
    };
    let left = normalize(parse_double(sector.at(0)?)?);
    let right = normalize(parse_double(sector.at(1)?)?);
    let orientation = if left > right {
        (left - 360.0 + right) / 2.0
    } else {
        (left + right) / 2.0
    };
    let radius = parse_double(sector.at(3)?)? * 1.1;
    let origin = frame.origin;
    let end = offset_point(origin, radius, frame, orientation);
    let dist = calc_distance_double(origin, end);
    let base = if dist < 100.0 { dist / 10.0 } else { 10.0 }.max(5.0);
    let tip = extend_along_line_double(origin, end, dist + 2.0 * base);
    let am_text = tg.am.clone();
    let am = split_commas(&am_text);
    let mut ranges = am
        .iter()
        .map(|s| parse_double(s))
        .collect::<Result<Vec<_>, _>>()?;
    if ranges.len() < 2 {
        if parse_double(am.first().copied().unwrap_or(""))? == 0.0 {
            return Ok(());
        }
        ranges.insert(0, 0.0);
    }
    let az12 = azimuth(origin, tip);
    let middle = (ranges.at(0)? + ranges.at(1)?) / 2.0;
    let p = offset_point(origin, middle, frame, az12);
    let name = tg.t.clone();
    add_area_modifier(tg, &name, AREA, -1.0, (p, p));
    Ok(())
}
