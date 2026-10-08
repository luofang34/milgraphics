//! Compares the channel family with the checked-in oracle fixtures: the same
//! control points, converted to pixels the way the oracle's point converter
//! does, go through the pipeline stages upstream runs before `DrawChannel`,
//! and every shape and vertex must land where the oracle's back-projected
//! polylines do (to well under a hundredth of a pixel).
//!
//! The flot and DISM cover glyph come from their own ports; the small-angle
//! FLOT of a line of contact is not exercised by the fixtures.

use super::externals::ChannelExternals;
use crate::engine::base::EngineError;
use crate::engine::base::{Pt, Shape};
use crate::engine::channel_utility::draw::draw_channel;
use crate::engine::dism::cover::get_dism_cover_double_rev_c;
use crate::engine::flot::flot_line::{get_flot_count_double, get_flot_double};
use crate::engine::line_type::classes::is_channel;
use crate::engine::line_type::line_type;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::axad_filter::filter_axad_points;
use crate::engine::tg_utility::lc_points::{reverse_usas_lc_points_by_quadrant, segment_lc_points};
use crate::engine::tg_utility::points::{filter_vertical_segments, reverse_points_rev_d};

const FIXTURES: &str = include_str!("../../../tests/fixtures/oracle/all.jsonl");

/// Meters per degree of longitude at `lat`, as the oracle's converter.
fn meters_per_degree_of_longitude(lat: f64) -> f64 {
    let l = lat.to_radians();
    111_412.84 * l.cos() - 93.5 * (3.0 * l).cos() + 0.118 * (5.0 * l).cos()
}

/// Geographic to pixel, with the bounding box's upper left as the origin.
struct Converter {
    left: f64,
    top: f64,
    meters_per_pixel: f64,
}

impl Converter {
    fn from_case(case: &serde_json::Value) -> Option<Self> {
        let bbox: Vec<f64> = case["bbox"]
            .as_str()?
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        let scale = case["scale"].as_f64()?;
        Some(Self {
            left: *bbox.first()?,
            top: *bbox.get(3)?,
            meters_per_pixel: scale / 96.0 / 39.370_078_7,
        })
    }

    fn to_pixels(&self, lon: f64, lat: f64) -> Pt {
        let y = -(lat - self.top) * (40_075_017.0 / 360.0) / self.meters_per_pixel;
        let x = (lon - self.left) * meters_per_degree_of_longitude(lat) / self.meters_per_pixel;
        Pt::new(x, y)
    }
}

fn control_points(case: &serde_json::Value, conv: &Converter) -> Vec<Pt> {
    let text = case["control_points"].as_str().unwrap_or_default();
    text.split(' ')
        .filter_map(|p| {
            let (lon, lat) = p.split_once(',')?;
            Some(conv.to_pixels(lon.parse().ok()?, lat.parse().ok()?))
        })
        .collect()
}

/// A numeric field of a symbol id.
fn symbol_field<T: std::str::FromStr>(symbol: &str, range: std::ops::Range<usize>) -> Option<T> {
    symbol.get(range)?.parse().ok()
}

/// Runs the stages upstream applies to the pixels before `DrawChannel`.
fn render(symbol: &str, line_type: i32, pixels: Vec<Pt>) -> Vec<Shape> {
    let settings = Settings::default();
    let mut tg = Tg::new(&settings);
    tg.line_type = line_type;
    tg.line_thickness = 3;
    tg.symbol_id = symbol.to_owned();
    tg.pixels = pixels;
    reverse_points_rev_d(&mut tg);
    // clsRenderer2.GetLineArray reverses a single concertina a second time.
    if line_type == lt::SINGLEC {
        tg.pixels.reverse();
    }
    if line_type == lt::LC {
        reverse_usas_lc_points_by_quadrant(&mut tg).unwrap();
        segment_lc_points(&mut tg).unwrap();
    }
    filter_vertical_segments(&mut tg);
    filter_axad_points(&mut tg).unwrap();
    let mut shapes = Vec::new();
    draw_channel(&mut tg, line_type, &settings, &Real, &mut shapes).unwrap();
    shapes
}

#[test]
fn channel_shapes_match_the_oracle() {
    let mut compared = 0;
    for line in FIXTURES.lines() {
        let case: serde_json::Value = serde_json::from_str(line).unwrap();
        let symbol = case["symbol"].as_str().unwrap();
        let (Some(version), Some(set), Some(entity)) = (
            symbol_field::<u8>(symbol, 2..4),
            symbol_field::<u8>(symbol, 4..6),
            symbol_field::<u32>(symbol, 10..16),
        ) else {
            continue;
        };
        let Some(line_type) = line_type(version, set, entity) else {
            continue;
        };
        if !is_channel(line_type) {
            continue;
        }
        let conv = Converter::from_case(&case).unwrap();
        let shapes = render(symbol, line_type, control_points(&case, &conv));
        let oracle = case["symbol_shapes"].as_array().unwrap();
        let name = &case["case"];
        assert_eq!(shapes.len(), oracle.len(), "{name}: shape count");
        for (mine, theirs) in shapes.iter().zip(oracle) {
            // The oracle drops one-point subpaths (a trailing pen-up).
            let mine: Vec<_> = mine
                .polylines()
                .into_iter()
                .filter(|p| p.len() > 1)
                .collect();
            let theirs = theirs["polylines"].as_array().unwrap();
            assert_eq!(mine.len(), theirs.len(), "{name}: polyline count");
            for (a, b) in mine.iter().zip(theirs) {
                let b = b.as_array().unwrap();
                assert_eq!(a.len(), b.len(), "{name}: point count");
                for (p, q) in a.iter().zip(b) {
                    let q = q.as_array().unwrap();
                    let q = conv.to_pixels(q[0].as_f64().unwrap(), q[1].as_f64().unwrap());
                    let miss = ((p.0 - q.x).powi(2) + (p.1 - q.y).powi(2)).sqrt();
                    assert!(miss < 0.01, "{name}: vertex off by {miss} px");
                }
            }
        }
        compared += 1;
    }
    assert!(compared >= 30, "only {compared} channel cases compared");
}

/// The channel externals backed by the flot and DISM ports.
#[derive(Debug)]
struct Real;

impl ChannelExternals for Real {
    fn flot_count(&self, pts: &[Pt], len: f64, n: i32) -> Result<i32, EngineError> {
        get_flot_count_double(pts, len, n)
    }

    fn flot(&self, pts: &mut Vec<Pt>, len: f64, n: i32) -> Result<i32, EngineError> {
        get_flot_double(pts, len, n)
    }

    fn dism_cover_rev_c(
        &self,
        pts: &mut Vec<Pt>,
        line_type: i32,
        n: i32,
        settings: &Settings,
    ) -> Result<i32, EngineError> {
        get_dism_cover_double_rev_c(pts, line_type, n, settings)
    }

    fn lc_flot_shapes(&self, _parent: &Tg, _pixels: Vec<Pt>) -> Result<Vec<Shape>, EngineError> {
        Err(EngineError::Degenerate(
            "the fixtures have no small-angle line of contact",
        ))
    }
}
