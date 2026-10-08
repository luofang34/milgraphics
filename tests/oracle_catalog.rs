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

    use milgraphics::render::{
        FixedAdvanceMetrics, LocalEquirectangular, Projection, ScreenPoint, ScreenShape,
    };
    use milgraphics::{
        Altitude, Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId,
        ModifierField, ModifierValue, SymbolId, VerticalDatum, View, construct, render, support,
    };
    use serde_json::Value;

    const RECORDS: &str = include_str!("fixtures/oracle/all.jsonl");
    const UNIMPLEMENTED: &str = include_str!("fixtures/oracle/unimplemented.txt");

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
        let ours: Vec<Vec<ScreenPoint>> = plan
            .screen
            .iter()
            .map(|i| match &i.shape {
                ScreenShape::Polyline(p) => p.clone(),
                ScreenShape::Polygon(p) => p.iter().chain(p.first()).copied().collect(),
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

    /// `ORACLE_TIMING=1 cargo test --release --test oracle_catalog timing -- --nocapture`
    /// prints construction and render times per case.
    #[test]
    fn timing() {
        if std::env::var("ORACLE_TIMING").is_err() {
            return;
        }
        let records = records();
        let defs: Vec<_> = records.iter().map(definition).collect();
        let start = std::time::Instant::now();
        let built: Vec<_> = defs
            .iter()
            .filter_map(|d| construct(d, &Config::default()).ok())
            .collect();
        let construct_time = start.elapsed();
        let start = std::time::Instant::now();
        for (c, r) in built.iter().zip(&records) {
            render(
                c,
                &View::new(0, 0),
                &frame(r),
                &FixedAdvanceMetrics::default(),
                &Budget::default(),
            )
            .ok();
        }
        let render_time = start.elapsed();
        let n = built.len() as u32;
        eprintln!(
            "{n} graphics: construct {:?} each, render {:?} each",
            construct_time / n,
            render_time / n
        );
    }

    /// `CONTACT_SHEET_DIR=<dir> cargo test --test oracle_catalog contact_sheet`
    /// writes pages of every base case drawn by milgraphics over the
    /// oracle's lines (faint red), for review by eye.
    #[test]
    fn contact_sheet() {
        let Ok(dir) = std::env::var("CONTACT_SHEET_DIR") else {
            return;
        };
        let cells: Vec<String> = records()
            .iter()
            .filter(|r| {
                let case = r["case"].as_str().unwrap_or_default();
                case.ends_with("-d") || case.ends_with("-e")
            })
            .filter_map(cell)
            .collect();
        for (page, chunk) in cells.chunks(48).enumerate() {
            let mut svg = String::from(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1800"><rect width="1600" height="1800" fill="white"/>"#,
            );
            for (i, c) in chunk.iter().enumerate() {
                let (x, y) = ((i % 8) as f64 * 200.0, (i / 8) as f64 * 300.0);
                svg.push_str(&format!(r#"<g transform="translate({x} {y})">{c}</g>"#));
            }
            svg.push_str("</svg>");
            std::fs::write(format!("{dir}/sheet-{page:02}.svg"), svg).unwrap();
        }
    }

    /// One 200×300 cell: the case drawn in a 200×260 box with its name.
    fn cell(r: &Value) -> Option<String> {
        let case = r["case"].as_str()?;
        let symbol = SymbolId::parse(r["symbol"].as_str()?).ok()?;
        let name = support::spec(&symbol).map_or("undeclared", |s| s.name());
        let f = frame(r);
        let oracle = oracle_lines(r, &f);
        let c = construct(&definition(r), &Config::default()).ok()?;
        let plan = render(
            &c,
            &View::new(0, 0),
            &f,
            &FixedAdvanceMetrics::default(),
            &Budget::default(),
        )
        .ok()?;
        let pts = plan.screen.iter().flat_map(|i| match &i.shape {
            ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.clone(),
        });
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in pts.chain(oracle.iter().flatten().copied()) {
            (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
        }
        let (w, h) = ((x1 - x0).max(1.0) + 60.0, (y1 - y0).max(1.0) + 60.0);
        let inner = milgraphics::svg::to_svg(&plan, 4000.0, 4000.0);
        let inner = inner.split_once('\n')?.1.trim_end_matches("</svg>\n");
        let under: String = oracle
            .iter()
            .map(|l| {
                let pts: Vec<String> = l.iter().map(|p| format!("{:.1},{:.1}", p.x, p.y)).collect();
                format!(
                    r##"<polyline points="{}" fill="none" stroke="#ff000060" stroke-width="7"/>"##,
                    pts.join(" ")
                )
            })
            .collect();
        Some(format!(
            r#"<svg width="200" height="260" viewBox="{} {} {w} {h}" preserveAspectRatio="xMidYMid meet">{under}{inner}</svg><text x="100" y="275" font-size="10" text-anchor="middle" font-family="sans-serif">{case}</text><text x="100" y="290" font-size="9" text-anchor="middle" font-family="sans-serif">{}</text>"#,
            x0 - 30.0,
            y0 - 30.0,
            name.replace('&', "&amp;")
                .chars()
                .take(40)
                .collect::<String>()
        ))
    }

    /// `ORACLE_CASE=<id> cargo test --test oracle_catalog dump -- --nocapture`
    /// prints both sides of one case.
    #[test]
    fn dump() {
        let Ok(case) = std::env::var("ORACLE_CASE") else {
            return;
        };
        let r = records()
            .into_iter()
            .find(|r| r["case"] == case.as_str())
            .unwrap();
        let f = frame(&r);
        for l in oracle_lines(&r, &f) {
            let pts: Vec<(i64, i64)> = l
                .iter()
                .map(|p| ((p.x * 10.0) as i64, (p.y * 10.0) as i64))
                .collect();
            eprintln!("oracle {pts:?}");
        }
        let c = construct(&definition(&r), &Config::default()).unwrap();
        let plan = render(
            &c,
            &View::new(0, 0),
            &f,
            &FixedAdvanceMetrics::default(),
            &Budget::default(),
        )
        .unwrap();
        for i in &plan.screen {
            let (ScreenShape::Polyline(p) | ScreenShape::Polygon(p)) = &i.shape;
            let pts: Vec<(i64, i64)> = p
                .iter()
                .map(|p| ((p.x * 10.0) as i64, (p.y * 10.0) as i64))
                .collect();
            eprintln!("ours{} {pts:?}", if i.decoration { "*" } else { "" });
        }
    }

    #[test]
    fn every_oracle_graphic_matches_or_is_listed_unimplemented() {
        let listed: BTreeSet<&str> = UNIMPLEMENTED
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();
        let mut unimplemented = BTreeSet::new();
        let mut failures = Vec::new();
        for r in records() {
            let case = r["case"].as_str().unwrap();
            assert_eq!(r["can_render"], "true", "{case}: oracle cases must render");
            let symbol = SymbolId::parse(r["symbol"].as_str().unwrap()).unwrap();
            if support::spec(&symbol).is_err() {
                unimplemented.insert(case.to_owned());
            } else if let Some(why) = mismatch(&r) {
                failures.push(format!("{case}: {why}"));
            }
        }
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
