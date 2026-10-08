//! Every multipoint graphic the pinned mil-sym-java renders for 2525D and
//! 2525E change 1 is either declared here and matches the oracle, or is
//! listed in `fixtures/oracle/unimplemented.txt`. The list must be exact, so
//! implementing a symbol forces its removal and an empty list means the
//! catalog is covered.
//!
//! Comparison follows AGENTS.md: both sides are placed in mil-sym's pixel
//! frame; geometry is compared by Hausdorff distance and labels by their
//! set of texts.

#![cfg(not(target_arch = "wasm32"))]

#[cfg(test)]
mod catalog {
    use std::collections::BTreeSet;

    use milgraphics::construction::PartRole;
    use milgraphics::render::{FixedAdvanceMetrics, LocalEquirectangular, Projection, ScreenPoint};
    use milgraphics::{
        Altitude, Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId,
        ModifierField, ModifierValue, SymbolId, VerticalDatum, View, construct, render, support,
    };
    use serde_json::Value;

    mod review;
    mod standard;

    const RECORDS: &str = include_str!("fixtures/oracle/all.jsonl");
    const UNIMPLEMENTED: &str = include_str!("fixtures/oracle/unimplemented.txt");
    /// Cases where the standard's template differs from what the oracle
    /// draws; they are checked against their reviewed goldens instead
    /// (`catalog::standard`).
    const STANDARD: &str = include_str!("fixtures/oracle/standard.txt");

    /// Hausdorff tolerance in pixels; mil-sym measures on a sphere and in its
    /// own pixel path, ours on the ellipsoid.
    const TOLERANCE_PX: f64 = 2.0;

    /// Accepted differences (UPSTREAM.md, "Ported renderer"): symbol code
    /// prefix of the case, the Hausdorff tolerance in pixels, and why.
    const ACCEPTED: &[(&str, f64, &str)] = &[(
        "46120104",
        7.5,
        "DEPTH_AREA bands are drawn as lines along their centres, not as fills with a hole",
    )];

    /// mil-sym modifier keys and the fields they fill.
    const KEYS: &[(&str, ModifierField)] = &[
        ("A_SYMBOL_ICON", ModifierField::A),
        ("AM_DISTANCE", ModifierField::AM),
        ("AN_AZIMUTH", ModifierField::AN),
        ("AP_TARGET_NUMBER", ModifierField::AP),
        ("AP1_TARGET_NUMBER_EXTENSION", ModifierField::AP1),
        ("AS_COUNTRY", ModifierField::AS),
        ("B_ECHELON", ModifierField::B),
        ("C_QUANTITY", ModifierField::C),
        ("H_ADDITIONAL_INFO_1", ModifierField::H),
        ("H1_ADDITIONAL_INFO_2", ModifierField::H1),
        ("N_HOSTILE", ModifierField::N),
        ("Q_DIRECTION_OF_MOVEMENT", ModifierField::Q),
        ("T_UNIQUE_DESIGNATION_1", ModifierField::T),
        ("T1_UNIQUE_DESIGNATION_2", ModifierField::T1),
        ("T2_UNIQUE_DESIGNATION_3", ModifierField::T2),
        ("V_EQUIP_TYPE", ModifierField::V),
        ("W_DTG_1", ModifierField::W),
        ("W1_DTG_2", ModifierField::W1),
        ("X_ALTITUDE_DEPTH", ModifierField::X),
        ("Y_LOCATION", ModifierField::Y),
    ];

    fn records() -> Vec<Value> {
        RECORDS
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn definition(r: &Value) -> GraphicDefinition {
        let points = r["control_points"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .map(|p| {
                let (lon, lat) = p.split_once(',').unwrap();
                ControlPoint::ground(
                    GeoPoint::new(lon.parse().unwrap(), lat.parse().unwrap()).unwrap(),
                )
            })
            .collect();
        let mut d = GraphicDefinition::new(
            GraphicId::new(r["case"].as_str().unwrap()).unwrap(),
            SymbolId::parse(r["symbol"].as_str().unwrap()).unwrap(),
            points,
        );
        for (key, value) in r["modifiers"].as_object().unwrap() {
            let field = KEYS
                .iter()
                .find(|(k, _)| k == key)
                .unwrap_or_else(|| panic!("unmapped mil-sym key {key}"))
                .1;
            let text = value.as_str().unwrap();
            let numbers = || text.split(',').map(|v| v.parse().unwrap()).collect();
            let value = match field {
                ModifierField::AM | ModifierField::AN | ModifierField::Q => {
                    ModifierValue::Numbers(numbers())
                }
                ModifierField::X => ModifierValue::Altitudes(
                    numbers()
                        .into_iter()
                        .map(|m: f64| Altitude::new(m, VerticalDatum::MeanSeaLevel))
                        .collect(),
                ),
                _ => ModifierValue::Text(text.to_owned()),
            };
            d.modifiers.set(field, value).unwrap();
        }
        d
    }

    fn frame(r: &Value) -> LocalEquirectangular {
        let b: Vec<f64> = r["bbox"]
            .as_str()
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        LocalEquirectangular::new(b[0], b[3], r["scale"].as_f64().unwrap(), 96.0)
    }

    fn px(frame: &LocalEquirectangular, lonlat: &Value) -> ScreenPoint {
        let p = GeoPoint::new(lonlat[0].as_f64().unwrap(), lonlat[1].as_f64().unwrap()).unwrap();
        frame.project(p).unwrap()
    }

    fn seg_dist(p: ScreenPoint, a: ScreenPoint, b: ScreenPoint) -> f64 {
        let (abx, aby) = (b.x - a.x, b.y - a.y);
        let len2 = abx * abx + aby * aby;
        let t = if len2 > 0.0 {
            (((p.x - a.x) * abx + (p.y - a.y) * aby) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (p.x - a.x - t * abx).hypot(p.y - a.y - t * aby)
    }

    fn dist_to(p: ScreenPoint, lines: &[Vec<ScreenPoint>]) -> f64 {
        lines
            .iter()
            .flat_map(|l| {
                l.windows(2)
                    .map(|w| seg_dist(p, w[0], w[1]))
                    .chain(l.first().map(|&a| seg_dist(p, a, a)))
            })
            .fold(f64::INFINITY, f64::min)
    }

    fn hausdorff(a: &[Vec<ScreenPoint>], b: &[Vec<ScreenPoint>]) -> f64 {
        let one = |x: &[Vec<ScreenPoint>], y: &[Vec<ScreenPoint>]| {
            x.iter()
                .flatten()
                .map(|&p| dist_to(p, y))
                .fold(0.0, f64::max)
        };
        one(a, b).max(one(b, a))
    }

    fn oracle_lines(r: &Value, f: &LocalEquirectangular) -> Vec<Vec<ScreenPoint>> {
        r["symbol_shapes"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["polylines"].as_array().into_iter().flatten())
            .map(|l| l.as_array().unwrap().iter().map(|p| px(f, p)).collect())
            .collect()
    }

    fn oracle_texts(r: &Value) -> Vec<&str> {
        r["modifier_shapes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m["text"].as_str())
            .collect()
    }

    /// Why a declared symbol's case does not match, or `None` when it does.
    fn mismatch(r: &Value) -> Option<String> {
        let c = match construct(&definition(r), &Config::default()) {
            Ok(c) => c,
            Err(e) => return Some(format!("not constructed: {e}")),
        };
        let f = frame(r);
        let plan = match render(
            &c,
            &View::new(0, 0),
            &f,
            &FixedAdvanceMetrics::default(),
            &Budget::default(),
        ) {
            Ok(p) => p,
            Err(e) => return Some(format!("not rendered: {e}")),
        };
        // Upstream paints hatching and pattern figures from images the
        // fixture does not record.
        let ours: Vec<Vec<ScreenPoint>> = plan
            .screen
            .iter()
            .filter(|i| !matches!(i.role, PartRole::Hatch | PartRole::Pattern))
            .map(|i| {
                let p = i.shape.points();
                let close = i.shape.is_closed().then(|| p.first()).flatten();
                p.iter().chain(close).copied().collect()
            })
            .collect();
        let oracle = oracle_lines(r, &f);
        let h = hausdorff(&ours, &oracle);
        let case = r["case"].as_str().unwrap_or_default();
        let tolerance = ACCEPTED
            .iter()
            .find(|(prefix, _, _)| case.starts_with(prefix))
            .map_or(TOLERANCE_PX, |(_, t, _)| *t);
        if h > tolerance {
            let worst =
                |x: &[Vec<ScreenPoint>], y: &[Vec<ScreenPoint>]| {
                    x.iter().flatten().map(|&p| (dist_to(p, y), p)).fold(
                        (0.0, None),
                        |a, (d, p)| if d > a.0 { (d, Some(p)) } else { a },
                    )
                };
            let (ours_far, oracle_far) = (worst(&ours, &oracle), worst(&oracle, &ours));
            return Some(format!(
                "Hausdorff {h:.2} px (ours {:.2} at {:?}, oracle {:.2} at {:?})",
                ours_far.0, ours_far.1, oracle_far.0, oracle_far.1
            ));
        }
        let texts = |labels: Vec<&str>| -> BTreeSet<String> {
            labels
                .into_iter()
                .map(str::trim)
                .filter(|t| !t.is_empty() && !t.ends_with(':'))
                .map(str::to_owned)
                .collect()
        };
        let expected = texts(oracle_texts(r));
        let got = texts(plan.labels.iter().map(|l| l.text.as_str()).collect());
        let missing: Vec<&String> = expected.difference(&got).collect();
        let extra: Vec<&String> = got.difference(&expected).collect();
        (!missing.is_empty() || !extra.is_empty())
            .then(|| format!("labels missing {missing:?}, extra {extra:?}"))
    }

    /// No geographic part strays far beyond the graphic's own extent (the
    /// oracle cases span a few kilometres).
    #[test]
    fn geographic_parts_stay_near_the_graphic() {
        let mut far = Vec::new();
        for r in records() {
            let d = definition(&r);
            let Ok(c) = construct(&d, &Config::default()) else {
                continue;
            };
            let origin = d.points[0].position;
            for p in &c.parts {
                let worst = p
                    .geometry
                    .points()
                    .iter()
                    .map(|q| {
                        ((q.lon() - origin.lon()) * 71.0).hypot((q.lat() - origin.lat()) * 111.0)
                    })
                    .fold(0.0, f64::max);
                if worst > 50.0 {
                    far.push(format!(
                        "{} part {:?} {worst:.0} km",
                        r["case"].as_str().unwrap_or_default(),
                        p.id
                    ));
                }
            }
        }
        assert!(
            far.is_empty(),
            "{} far parts: {:?}",
            far.len(),
            &far[..far.len().min(20)]
        );
    }

    /// Map engines reject lines of fewer than two points and rings of fewer
    /// than three, so the geographic tier never holds them.
    #[test]
    fn geographic_parts_are_drawable() {
        let mut bad = Vec::new();
        for r in records() {
            let Ok(c) = construct(&definition(&r), &Config::default()) else {
                continue;
            };
            for p in &c.parts {
                let distinct = |pts: &[GeoPoint]| {
                    let mut d: Vec<(i64, i64)> = pts
                        .iter()
                        .map(|q| ((q.lon() * 1e7) as i64, (q.lat() * 1e7) as i64))
                        .collect();
                    d.sort_unstable();
                    d.dedup();
                    d.len()
                };
                let needed = if p.geometry.is_closed() { 3 } else { 2 };
                let ok = distinct(p.geometry.points()) >= needed;
                if !ok {
                    bad.push(format!(
                        "{} part {:?}",
                        r["case"].as_str().unwrap_or_default(),
                        p.id
                    ));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "{} degenerate parts: {:?}",
            bad.len(),
            &bad[..bad.len().min(20)]
        );
    }

    #[test]
    fn every_oracle_graphic_matches_or_is_listed_unimplemented() {
        let listed: BTreeSet<&str> = UNIMPLEMENTED
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();
        let standard: BTreeSet<&str> = STANDARD
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| l.split_whitespace().next())
            .collect();
        let mut agreeing = Vec::new();
        let mut unimplemented = BTreeSet::new();
        let mut failures = Vec::new();
        for r in records() {
            let case = r["case"].as_str().unwrap();
            assert_eq!(r["can_render"], "true", "{case}: oracle cases must render");
            let symbol = SymbolId::parse(r["symbol"].as_str().unwrap()).unwrap();
            if support::spec(&symbol).is_err() {
                unimplemented.insert(case.to_owned());
            } else if standard.contains(case) {
                if mismatch(&r).is_none() {
                    agreeing.push(case.to_owned());
                }
            } else if let Some(why) = mismatch(&r) {
                failures.push(format!("{case}: {why}"));
            }
        }
        assert!(
            agreeing.is_empty(),
            "fixtures/oracle/standard.txt lists cases that match the oracle: {agreeing:?}"
        );
        assert!(
            failures.is_empty(),
            "{} mismatches:\n{}",
            failures.len(),
            failures.join("\n")
        );
        let listed: BTreeSet<String> = listed.into_iter().map(str::to_owned).collect();
        let stale: Vec<&String> = listed.difference(&unimplemented).collect();
        let missing: Vec<&String> = unimplemented.difference(&listed).collect();
        assert!(
            stale.is_empty() && missing.is_empty(),
            "fixtures/oracle/unimplemented.txt is out of date.\nimplemented, remove: {stale:?}\nnot implemented, add: {missing:?}"
        );
    }
}
