//! Renders the six first-milestone graphics (2525D change 1) to SVG files.
//!
//! Usage: `cargo run --example render_svg [out_dir]` (default `target/svg`).
//! `docs/images` holds three of them for the README.

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use milgraphics::render::{FixedAdvanceMetrics, LocalEquirectangular};
use milgraphics::{
    Altitude, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, ModifierField,
    ModifierValue, SymbolId, VerticalDatum, View, construct, render,
};

/// One graphic: name, entity digits, control points (lon, lat), frame
/// origin (west, north) and the modifiers the graphic needs.
struct Case {
    name: &'static str,
    entity: &'static str,
    points: &'static [(f64, f64)],
    west_north: (f64, f64),
    designation: Option<&'static str>,
    distances_m: &'static [f64],
    azimuths_deg: &'static [f64],
    altitudes_m: &'static [f64],
}

const SCALE: f64 = 50000.0;
/// Space around the drawing, in pixels, for label text past its anchor.
const MARGIN: f64 = 80.0;

const CASES: [Case; 6] = [
    Case {
        name: "phase-line",
        entity: "140300",
        points: &[(20.0, 50.0), (20.1, 50.02)],
        west_north: (19.95, 50.07),
        designation: Some("ALPHA"),
        distances_m: &[],
        azimuths_deg: &[],
        altitudes_m: &[],
    },
    Case {
        name: "nai",
        entity: "120200",
        points: &[(20.0, 50.0), (20.08, 50.0), (20.08, 50.05), (20.0, 50.05)],
        west_north: (19.95, 50.1),
        designation: Some("1"),
        distances_m: &[],
        azimuths_deg: &[],
        altitudes_m: &[],
    },
    Case {
        name: "main-attack",
        entity: "151403",
        points: &[(20.0, 50.0), (20.1, 50.03), (20.0, 50.01)],
        west_north: (19.95, 50.08),
        designation: Some("1"),
        distances_m: &[],
        azimuths_deg: &[],
        altitudes_m: &[],
    },
    Case {
        name: "air-corridor",
        entity: "170100",
        points: &[(20.0, 50.0), (20.05, 50.03), (20.1, 50.02)],
        west_north: (19.95, 50.08),
        designation: Some("AC1"),
        distances_m: &[2000.0],
        azimuths_deg: &[],
        altitudes_m: &[1000.0, 3000.0],
    },
    Case {
        name: "range-fan-sector",
        entity: "242200",
        points: &[(20.0, 50.0)],
        west_north: (19.9, 50.1),
        designation: None,
        distances_m: &[1000.0, 5000.0],
        azimuths_deg: &[30.0, 90.0],
        altitudes_m: &[],
    },
    Case {
        name: "bypass-easy",
        entity: "270601",
        points: &[(20.0, 50.0), (20.04, 50.03), (20.08, 50.0)],
        west_north: (19.95, 50.08),
        designation: None,
        distances_m: &[],
        azimuths_deg: &[],
        altitudes_m: &[],
    },
];

fn definition(case: &Case) -> Result<GraphicDefinition, Box<dyn Error>> {
    let sidc = format!("1103250000{}0000", case.entity);
    let points = case
        .points
        .iter()
        .map(|&(lon, lat)| Ok(ControlPoint::ground(GeoPoint::new(lon, lat)?)))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let mut d = GraphicDefinition::new(GraphicId::new(case.name)?, SymbolId::parse(&sidc)?, points);
    if let Some(t) = case.designation {
        d.modifiers
            .set(ModifierField::T, ModifierValue::Text(t.to_owned()))?;
    }
    let altitudes = case
        .altitudes_m
        .iter()
        .map(|&metres| Altitude::new(metres, VerticalDatum::MeanSeaLevel))
        .collect();
    for (field, value) in [
        (
            ModifierField::AM,
            ModifierValue::Numbers(case.distances_m.to_vec()),
        ),
        (
            ModifierField::AN,
            ModifierValue::Numbers(case.azimuths_deg.to_vec()),
        ),
        (ModifierField::X, ModifierValue::Altitudes(altitudes)),
    ] {
        d.modifiers.set(field, value)?;
    }
    Ok(d)
}

/// Renders one case, cropped to what is drawn with a margin for label text,
/// on a white background so it reads on light and dark pages.
fn svg(case: &Case) -> Result<String, Box<dyn Error>> {
    let construction = construct(&definition(case)?, &Config::default())?;
    let view = View::new(0, 0);
    let frame = LocalEquirectangular::new(case.west_north.0, case.west_north.1, SCALE, 96.0);
    let plan = render(
        &construction,
        &view,
        &frame,
        &FixedAdvanceMetrics::default(),
    )?;
    let anchors = plan.labels.iter().filter_map(|l| l.screen);
    let shapes = plan
        .screen
        .iter()
        .flat_map(|i| i.shape.points().iter().copied());
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in shapes.chain(anchors) {
        (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
    }
    let (x0, y0) = ((x0 - MARGIN).floor(), (y0 - MARGIN).floor());
    let (w, h) = ((x1 + MARGIN).ceil() - x0, (y1 + MARGIN).ceil() - y0);
    let full = milgraphics::svg::to_svg(
        &plan,
        &milgraphics::svg::SvgOptions::new(x1 + MARGIN, y1 + MARGIN),
    );
    let body = full.split_once('\n').map_or("", |(_, body)| body);
    Ok(format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" "#,
            r#"viewBox="{x0} {y0} {w} {h}">"#,
            "\n",
            r#"<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="white"/>"#,
            "\n{body}"
        ),
        w = w,
        h = h,
        x0 = x0,
        y0 = y0,
        body = body,
    ))
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "target/svg".to_owned()),
    );
    fs::create_dir_all(&out_dir)?;
    for case in &CASES {
        fs::write(out_dir.join(format!("{}.svg", case.name)), svg(case)?)?;
    }
    Ok(())
}
