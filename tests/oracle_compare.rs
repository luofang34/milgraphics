//! Compares rendered graphics with the pinned mil-sym-java oracle under the
//! comparison policy in AGENTS.md: both sides are placed in mil-sym's pixel
//! frame, geometry is compared by Hausdorff distance, and labels by text,
//! position and angle.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through a `cfg(test)` module.
#[cfg(test)]
mod compare {
    use milgraphics::render::{
        FixedAdvanceMetrics, Font, LocalEquirectangular, Projection, ScreenPoint, ScreenShape,
        TextAlign,
    };
    use milgraphics::style::DashPattern;
    use milgraphics::{
        Altitude, Budget, Config, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, SymbolId,
        VerticalDatum, View, construct, render,
    };
    use serde_json::Value;

    const RECORDS: &str = include_str!("fixtures/oracle/six.jsonl");

    fn record(case: &str) -> Value {
        RECORDS
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|r| r["case"] == case)
            .unwrap_or_else(|| panic!("no fixture {case}"))
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
        let m = &r["modifiers"];
        d.modifiers.designation = m["T_UNIQUE_DESIGNATION_1"].as_str().map(str::to_owned);
        let list = |key: &str| -> Vec<f64> {
            m[key].as_str().map_or_else(Vec::new, |v| {
                v.split(',').map(|x| x.parse().unwrap()).collect()
            })
        };
        d.modifiers.distances_m = list("AM_DISTANCE");
        d.modifiers.azimuths_deg = list("AN_AZIMUTH");
        d.modifiers.altitudes = list("X_ALTITUDE_DEPTH")
            .into_iter()
            .map(|metres| Altitude {
                metres,
                datum: VerticalDatum::MeanSeaLevel,
            })
            .collect();
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
        frame.project(p, 0.0).unwrap()
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

    /// Symmetric Hausdorff distance between two sets of polylines (vertex to
    /// polyline in both directions).
    fn hausdorff(a: &[Vec<ScreenPoint>], b: &[Vec<ScreenPoint>]) -> f64 {
        let one = |x: &[Vec<ScreenPoint>], y: &[Vec<ScreenPoint>]| {
            x.iter()
                .flatten()
                .map(|&p| dist_to(p, y))
                .fold(0.0, f64::max)
        };
        one(a, b).max(one(b, a))
    }

    struct Ours {
        lines: Vec<Vec<ScreenPoint>>,
        dash: Vec<DashPattern>,
        labels: Vec<(String, ScreenPoint, f64)>,
    }

    fn ours(r: &Value) -> Ours {
        let c = construct(&definition(r), &Config::default()).unwrap();
        let view = View {
            view_revision: 0,
            surface_revision: 0,
            label_font: Font::default(),
        };
        let plan = render(
            &c,
            &view,
            &frame(r),
            &FixedAdvanceMetrics::default(),
            &Budget::default(),
        )
        .unwrap();
        let lines = plan
            .screen
            .iter()
            .map(|i| match &i.shape {
                ScreenShape::Polyline(p) => p.clone(),
                ScreenShape::Polygon(p) => p.iter().chain(p.first()).copied().collect(),
            })
            .collect();
        let dash = plan
            .screen
            .iter()
            .filter_map(|i| i.stroke.map(|s| s.dash))
            .collect();
        // Where the text centre line is: the anchor moved by the label's
        // offset in its rotated frame (x along the text, y down).
        let labels = plan
            .labels
            .iter()
            .map(|l| {
                let s = l.screen.unwrap();
                let (sin, cos) = l.rotation_deg.to_radians().sin_cos();
                let [ox, oy] = l.offset_em.map(|v| v * l.font.size_px);
                let ox = if l.align == TextAlign::Center {
                    ox
                } else {
                    0.0
                };
                let at = ScreenPoint {
                    x: s.x + ox * cos - oy * sin,
                    y: s.y + ox * sin + oy * cos,
                };
                (l.text.clone(), at, l.rotation_deg)
            })
            .collect();
        Ours {
            lines,
            dash,
            labels,
        }
    }

    fn oracle_lines(r: &Value) -> Vec<Vec<ScreenPoint>> {
        let f = frame(r);
        r["symbol_shapes"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["polylines"].as_array().unwrap().iter())
            .map(|l| l.as_array().unwrap().iter().map(|p| px(&f, p)).collect())
            .collect()
    }

    /// The dash patterns used, as a set: mil-sym dashes at 6 px on 3 px lines.
    fn oracle_dashes(r: &Value) -> Vec<DashPattern> {
        let mut d: Vec<DashPattern> = r["symbol_shapes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                if s["dash"] == serde_json::json!([6, 6]) {
                    DashPattern::Dashed
                } else {
                    DashPattern::Solid
                }
            })
            .collect();
        d.sort();
        d.dedup();
        d
    }

    fn check_geometry(case: &str, tolerance_px: f64) -> (Value, Ours) {
        let r = record(case);
        let o = ours(&r);
        let h = hausdorff(&o.lines, &oracle_lines(&r));
        assert!(
            h <= tolerance_px,
            "{case}: Hausdorff {h:.3} px > {tolerance_px}"
        );
        let mut ours = o.dash.clone();
        ours.sort();
        ours.dedup();
        assert_eq!(ours, oracle_dashes(&r), "{case}: dash patterns");
        (r, o)
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn phase_lines_match_the_oracle() {
        for case in [
            "phase-line-d",
            "phase-line-e",
            "phase-line-4pt-d",
            "phase-line-anticipated-d",
            "phase-line-antimeridian-d",
        ] {
            let (r, o) = check_geometry(case, 0.5);
            let f = frame(&r);
            let expected = r["modifier_shapes"].as_array().unwrap();
            assert_eq!(expected.len(), o.labels.len(), "{case}");
            for (m, (text, at, angle)) in expected.iter().zip(&o.labels) {
                assert_eq!(m["text"], text.as_str(), "{case}");
                // mil-sym rounds label anchors to whole pixels.
                let want = px(&f, &m["position"]);
                assert!(
                    (want.x - at.x).hypot(want.y - at.y) <= 0.75,
                    "{case}: label at {at:?}, oracle {want:?}"
                );
                let da = (m["angle"].as_f64().unwrap() - angle).abs();
                assert!(da <= 1.0, "{case}: angle {angle} vs {}", m["angle"]);
            }
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn named_areas_match_the_oracle() {
        for case in ["nai-d", "nai-e", "nai-anticipated-d"] {
            let (r, o) = check_geometry(case, 0.5);
            let f = frame(&r);
            let m = &r["modifier_shapes"][0];
            let (text, at, angle) = &o.labels[0];
            assert_eq!(m["text"], text.as_str());
            assert_eq!(*angle, 0.0);
            // mil-sym's position is the text baseline, half an em (6 px)
            // below the centre our anchor marks.
            let want = px(&f, &m["position"]);
            let d = (want.x - at.x).hypot(want.y - 6.0 - at.y);
            assert!(
                d <= 1.5,
                "{case}: label centre {at:?}, oracle baseline {want:?}"
            );
        }
    }

    /// Every non-empty oracle label must have a label of ours with the same
    /// text, within `along_px` along the text and `across_px` across it.
    /// mil-sym gives centred area labels by their baseline, half an em
    /// below the centre ours marks; `baseline` moves its anchor up to match.
    fn check_labels(r: &Value, o: &Ours, along_px: f64, across_px: f64, baseline: bool) {
        let f = frame(r);
        let case = r["case"].as_str().unwrap();
        let mut expected = 0;
        for m in r["modifier_shapes"].as_array().unwrap() {
            let text = m["text"].as_str().unwrap();
            if text.trim_end().ends_with(':') {
                continue;
            }
            expected += 1;
            let mut want = px(&f, &m["position"]);
            if baseline {
                want.y -= 6.0;
            }
            let fits = o
                .labels
                .iter()
                .filter(|(t, _, _)| t == text)
                .any(|(_, at, rot)| {
                    let (s, c) = rot.to_radians().sin_cos();
                    let (dx, dy) = (want.x - at.x, want.y - at.y);
                    let (along, across) = (dx * c + dy * s, -dx * s + dy * c);
                    if std::env::var("ORACLE_REPORT").is_ok() {
                        eprintln!("{case}: {text:?} along {along:.2} across {across:.2}");
                    }
                    along.abs() <= along_px && across.abs() <= across_px
                });
            assert!(fits, "{case}: no label {text:?} near the oracle's");
        }
        assert_eq!(expected, o.labels.len(), "{case}: label count");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn main_attacks_match_the_oracle() {
        for case in [
            "main-attack-3pt-d",
            "main-attack-5pt-d",
            "main-attack-3pt-e",
            "main-attack-3pt-anticipated-d",
        ] {
            let (r, o) = check_geometry(case, 1.5);
            // Accepted difference (UPSTREAM.md): mil-sym anchors the label on
            // its own adjusted pixel path, up to ~10 px further along.
            check_labels(&r, &o, 12.0, 2.5, false);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn air_corridors_match_the_oracle() {
        for case in [
            "air-corridor-d",
            "air-corridor-e",
            "air-corridor-anticipated-d",
        ] {
            let (r, o) = check_geometry(case, 1.5);
            check_labels(&r, &o, 1.0, 1.0, false);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn range_fans_match_the_oracle() {
        for case in [
            "range-fan-sector-d",
            "range-fan-sector-2-d",
            "range-fan-sector-e",
            "range-fan-sector-anticipated-d",
        ] {
            // mil-sym measures ranges on a sphere; ours are on the ellipsoid.
            let (r, o) = check_geometry(case, 1.5);
            check_labels(&r, &o, 1.5, 1.5, true);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn bypasses_match_the_oracle() {
        for case in [
            "bypass-easy-d",
            "bypass-easy-e",
            "bypass-easy-anticipated-d",
        ] {
            let (r, o) = check_geometry(case, 0.5);
            check_labels(&r, &o, 0.0, 0.0, false);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn refused_cases_are_refused_here_too() {
        for case in ["phase-line-1pt-d", "range-fan-no-an-d"] {
            let r = record(case);
            assert_ne!(r["can_render"], "true");
            assert!(
                construct(&definition(&r), &Config::default()).is_err(),
                "{case}"
            );
        }
    }
}
