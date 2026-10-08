//! Compares the planar ports with recorded output of the upstream functions.
//! The recording drives upstream's geodesic code at the equator with a linear
//! pixel converter, 10 m per pixel, so planar and spherical results agree to
//! a small fraction of a pixel.

use serde_json::Value;

use super::{areas::change1_tactical_areas, filter};
use crate::engine::base::{EngineError, PathOp, Pt, Shape};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;

const CASES: &str = include_str!("tests/probe_cases.tsv");
const OUT: &str = include_str!("tests/probe_out.jsonl");
const MPP: f64 = 10.0;
const TOLERANCE: f64 = 0.05;

fn line_type(name: &str) -> i32 {
    match name {
        "CIRCULAR" => CIRCULAR,
        "PBS_CIRCLE" => PBS_CIRCLE,
        "BBS_POINT" => BBS_POINT,
        "RECTANGULAR" => RECTANGULAR,
        "CUED_ACQUISITION" => CUED_ACQUISITION,
        "LAUNCH_AREA" => LAUNCH_AREA,
        "FSA_RECTANGULAR" => FSA_RECTANGULAR,
        "RECTANGULAR_TARGET" => RECTANGULAR_TARGET,
        "BS_ORBIT" => BS_ORBIT,
        "BS_ROUTE" => BS_ROUTE,
        "BS_TRACK" => BS_TRACK,
        "RANGE_FAN" => RANGE_FAN,
        "RANGE_FAN_SECTOR" => RANGE_FAN_SECTOR,
        "RADAR_SEARCH" => RADAR_SEARCH,
        "RANGE_FAN_FILL" => RANGE_FAN_FILL,
        "BS_POLYARC" => BS_POLYARC,
        "PL" => PL,
        "FLOT" => FLOT,
        "LC" => LC,
        "ZONE" => ZONE,
        "SC" => SC,
        other => panic!("unmapped line type {other}"),
    }
}

struct Case {
    id: String,
    kind: String,
    tg: Tg,
    control: Vec<Pt>,
}

fn cases() -> Vec<Case> {
    CASES
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            let mut tg = Tg::new(&Settings::default());
            tg.line_type = line_type(f[2]);
            tg.line_thickness = 3;
            let control: Vec<Pt> = f[3]
                .split(';')
                .map(|p| {
                    let n: Vec<f64> = p.split(',').map(|v| v.parse().unwrap()).collect();
                    Pt::styled(n[0], n[1], n.get(2).map_or(0, |s| *s as i32))
                })
                .collect();
            tg.pixels = control.clone();
            for kv in &f[4..] {
                let (k, v) = kv.split_once('=').unwrap();
                match k {
                    "AM" => tg.am = v.to_owned(),
                    "AM1" => tg.am1 = v.to_owned(),
                    "AN" => tg.an = v.to_owned(),
                    "LRMM" => tg.lrmm = v.to_owned(),
                    other => panic!("key {other}"),
                }
            }
            Case {
                id: f[0].to_owned(),
                kind: f[1].to_owned(),
                tg,
                control,
            }
        })
        .collect()
}

fn recorded(id: &str) -> Value {
    OUT.lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["id"] == id)
        .unwrap()
}

fn assert_points(id: &str, got: &[Pt], want: &Value) {
    let want = want.as_array().unwrap();
    assert_eq!(got.len(), want.len(), "{id}: point count");
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        let (wx, wy, ws) = (
            w[0].as_f64().unwrap(),
            w[1].as_f64().unwrap(),
            w[2].as_i64().unwrap(),
        );
        assert!(
            (g.x - wx).abs() < TOLERANCE && (g.y - wy).abs() < TOLERANCE,
            "{id}[{i}]: got ({}, {}) want ({wx}, {wy})",
            g.x,
            g.y
        );
        assert_eq!(i64::from(g.style), ws, "{id}[{i}] style");
    }
}

fn assert_shapes(id: &str, got: &[Shape], want: &Value) {
    let want = want.as_array().unwrap();
    assert_eq!(got.len(), want.len(), "{id}: shape count");
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert_eq!(
            i64::from(g.shape_type),
            w["type"].as_i64().unwrap(),
            "{id} shape {i} type"
        );
        assert_eq!(
            i64::from(g.style),
            w["style"].as_i64().unwrap(),
            "{id} shape {i} style"
        );
        let ops = w["path"].as_array().unwrap();
        assert_eq!(g.path.len(), ops.len(), "{id} shape {i} path length");
        for (j, (op, w)) in g.path.iter().zip(ops).enumerate() {
            let (kind, x, y) = match *op {
                PathOp::MoveTo(x, y) => (0, x, y),
                PathOp::LineTo(x, y) => (1, x, y),
            };
            assert_eq!(kind, w[0].as_i64().unwrap(), "{id} shape {i} op {j} kind");
            assert!(
                (x - w[1].as_f64().unwrap()).abs() < TOLERANCE
                    && (y - w[2].as_f64().unwrap()).abs() < TOLERANCE,
                "{id} shape {i} op {j}: got ({x}, {y}) want ({}, {})",
                w[1],
                w[2]
            );
        }
    }
}

fn no_exterior(_: Pt, _: Pt, _: &[Pt], _: i32, _: i32, _: i32) -> Result<i32, EngineError> {
    Err(EngineError::LineType(0))
}

#[test]
fn change1_areas_match_upstream() {
    for mut case in cases().into_iter().filter(|c| c.kind == "change1") {
        let want = recorded(&case.id);
        let mut shapes = Vec::new();
        let lt = case.tg.line_type;
        let ok = change1_tactical_areas(
            &mut case.tg,
            lt,
            &case.control,
            MPP,
            &mut shapes,
            &no_exterior,
        );
        assert_eq!(ok, want["ok"].as_bool().unwrap(), "{}: result", case.id);
        if ok {
            assert_points(&case.id, &case.tg.pixels, &want["pixels"]);
            assert_shapes(&case.id, &shapes, &want["shapes"]);
        }
        if case.id == "sector_pts" {
            let got: Vec<f64> = case
                .tg
                .lrmm
                .split(',')
                .map(|v| v.parse().unwrap())
                .collect();
            let rec = want["lrmm"].as_str().unwrap();
            let rec: Vec<f64> = rec.split(',').map(|v| v.parse().unwrap()).collect();
            assert_eq!(got.len(), rec.len());
            for (g, r) in got.iter().zip(&rec) {
                assert!((g - r).abs() < 1e-3, "lrmm {g} vs {r}");
            }
        }
    }
}

#[test]
fn point_filters_match_upstream() {
    for mut case in cases()
        .into_iter()
        .filter(|c| c.kind == "filter" || c.kind == "clear")
    {
        let want = recorded(&case.id);
        match case.kind.as_str() {
            "filter" => filter::filter_points2(&mut case.tg),
            "clear" => filter::clear_pixels_style(&mut case.tg),
            other => panic!("unexpected kind {other}"),
        }
        assert_points(&case.id, &case.tg.pixels, &want["pixels"]);
    }
}
