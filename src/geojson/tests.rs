use super::*;
use crate::definition::{ControlPoint, GraphicDefinition, GraphicId};
use crate::family::{Config, construct};
use crate::geo::GeoPoint;
use crate::render::{FixedAdvanceMetrics, LocalEquirectangular, View, render};
use crate::sidc::SymbolId;

fn geojson_for(sidc: &str, points: &[(f64, f64)]) -> Value {
    let d = GraphicDefinition::new(
        GraphicId::new("graphic-7").unwrap(),
        SymbolId::parse(sidc).unwrap(),
        points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect(),
    );
    let c = construct(&d, &Config::default()).unwrap();
    let view = View::new(0, 0);
    let frame = LocalEquirectangular::new(points[0].0 - 1.0, 60.0, 50_000.0, 96.0);
    to_geojson(&render(&c, &view, &frame, &FixedAdvanceMetrics::default()).unwrap())
}

#[test]
fn line_features_carry_pick_and_style() {
    let g = geojson_for("11032510001403000000", &[(20.0, 50.0), (20.1, 50.02)]);
    let features = g["features"].as_array().unwrap();
    assert_eq!(features.len(), 3, "one line and two labels");
    let line = &features[0];
    assert_eq!(line["geometry"]["type"], "MultiLineString");
    assert_eq!(line["properties"]["graphic"], "graphic-7");
    assert_eq!(line["properties"]["part"], 0);
    assert_eq!(line["properties"]["role"], "line");
    assert_eq!(line["properties"]["dash"], "dashed");
    assert_eq!(line["properties"]["dasharray"], json!([2.0, 2.0]));
    assert_eq!(features[1]["geometry"]["type"], "Point");
    let label = &features[1]["properties"];
    assert_eq!(label["label"], "PL");
    for key in ["offset-em", "font-size-px", "may-hide"] {
        assert!(label.get(key).is_some(), "{key}");
    }
}

#[test]
fn style_names_are_lower_case_and_distinct() {
    let names: Vec<&str> = crate::style::DashPattern::ALL
        .iter()
        .map(|d| d.name())
        .collect();
    assert_eq!(names, ["solid", "dashed", "dotted"]);
}

#[test]
fn antimeridian_crossings_are_split() {
    let g = geojson_for("11032500001403000000", &[(179.95, 10.0), (-179.95, 10.02)]);
    let lines = g["features"][0]["geometry"]["coordinates"]
        .as_array()
        .unwrap();
    assert_eq!(lines.len(), 2);
    for line in lines {
        for p in line.as_array().unwrap() {
            assert!(p[0].as_f64().unwrap().abs() <= 180.0);
        }
    }
}

#[test]
fn areas_are_closed_polygons() {
    let g = geojson_for(
        "11032500001202000000",
        &[(20.0, 50.0), (20.08, 50.0), (20.08, 50.05), (20.0, 50.05)],
    );
    let area = &g["features"][0];
    assert_eq!(area["geometry"]["type"], "MultiPolygon");
    let ring = area["geometry"]["coordinates"][0][0].as_array().unwrap();
    assert_eq!(ring.first(), ring.last());
}
