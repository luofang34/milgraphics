//! Renders the six first-milestone graphics (2525D change 1) to SVG files.
//!
//! Usage: `cargo run --example render_svg [out_dir]` (default `target/svg`).

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use milgraphics::render::{FixedAdvanceMetrics, Font, LocalEquirectangular, ScreenShape};
use milgraphics::{
    Altitude, Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, SymbolId,
    VerticalDatum, View, construct, render,
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
    d.modifiers.designation = case.designation.map(str::to_owned);
    d.modifiers.distances_m = case.distances_m.to_vec();
    d.modifiers.azimuths_deg = case.azimuths_deg.to_vec();
    d.modifiers.altitudes = case
        .altitudes_m
        .iter()
        .map(|&metres| Altitude {
            metres,
            datum: VerticalDatum::MeanSeaLevel,
        })
        .collect();
    Ok(d)
}

/// Renders one case. The canvas covers every drawn point and label anchor
/// with a margin and is never smaller than 1100 x 900.
fn svg(case: &Case) -> Result<String, Box<dyn Error>> {
    let construction = construct(&definition(case)?, &Config::default())?;
    let view = View {
        view_revision: 0,
        surface_revision: 0,
        label_font: Font::default(),
    };
    let frame = LocalEquirectangular::new(case.west_north.0, case.west_north.1, SCALE, 96.0);
    let plan = render(
        &construction,
        &view,
        &frame,
        &FixedAdvanceMetrics::default(),
        &Budget::default(),
    )?;
    let anchors = plan.labels.iter().filter_map(|l| l.screen);
    let shapes = plan.screen.iter().flat_map(|i| match &i.shape {
        ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.iter().copied(),
    });
    let (mut w, mut h) = (1100.0_f64, 900.0_f64);
    for p in shapes.chain(anchors) {
        w = w.max((p.x + 60.0).ceil());
        h = h.max((p.y + 60.0).ceil());
    }
    Ok(milgraphics::svg::to_svg(&plan, w, h))
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
