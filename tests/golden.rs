//! Golden SVG renders of the six first-milestone graphics for 2525D and
//! 2525E change 1, each as current and anticipated. Set `UPDATE_GOLDEN=1` on
//! a native target to rewrite `tests/golden/*.svg` instead of comparing.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// Clippy does not recognise `wasm_bindgen_test` functions as tests, so the
// test exemptions in clippy.toml reach them only through a `cfg(test)` module.
#[cfg(test)]
mod golden {
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

    const GOLDEN: [(&str, &str); 24] = [
        (
            "phase-line-2525d-s0",
            include_str!("golden/phase-line-2525d-s0.svg"),
        ),
        (
            "phase-line-2525d-s1",
            include_str!("golden/phase-line-2525d-s1.svg"),
        ),
        (
            "phase-line-2525e-s0",
            include_str!("golden/phase-line-2525e-s0.svg"),
        ),
        (
            "phase-line-2525e-s1",
            include_str!("golden/phase-line-2525e-s1.svg"),
        ),
        ("nai-2525d-s0", include_str!("golden/nai-2525d-s0.svg")),
        ("nai-2525d-s1", include_str!("golden/nai-2525d-s1.svg")),
        ("nai-2525e-s0", include_str!("golden/nai-2525e-s0.svg")),
        ("nai-2525e-s1", include_str!("golden/nai-2525e-s1.svg")),
        (
            "main-attack-2525d-s0",
            include_str!("golden/main-attack-2525d-s0.svg"),
        ),
        (
            "main-attack-2525d-s1",
            include_str!("golden/main-attack-2525d-s1.svg"),
        ),
        (
            "main-attack-2525e-s0",
            include_str!("golden/main-attack-2525e-s0.svg"),
        ),
        (
            "main-attack-2525e-s1",
            include_str!("golden/main-attack-2525e-s1.svg"),
        ),
        (
            "air-corridor-2525d-s0",
            include_str!("golden/air-corridor-2525d-s0.svg"),
        ),
        (
            "air-corridor-2525d-s1",
            include_str!("golden/air-corridor-2525d-s1.svg"),
        ),
        (
            "air-corridor-2525e-s0",
            include_str!("golden/air-corridor-2525e-s0.svg"),
        ),
        (
            "air-corridor-2525e-s1",
            include_str!("golden/air-corridor-2525e-s1.svg"),
        ),
        (
            "range-fan-sector-2525d-s0",
            include_str!("golden/range-fan-sector-2525d-s0.svg"),
        ),
        (
            "range-fan-sector-2525d-s1",
            include_str!("golden/range-fan-sector-2525d-s1.svg"),
        ),
        (
            "range-fan-sector-2525e-s0",
            include_str!("golden/range-fan-sector-2525e-s0.svg"),
        ),
        (
            "range-fan-sector-2525e-s1",
            include_str!("golden/range-fan-sector-2525e-s1.svg"),
        ),
        (
            "bypass-easy-2525d-s0",
            include_str!("golden/bypass-easy-2525d-s0.svg"),
        ),
        (
            "bypass-easy-2525d-s1",
            include_str!("golden/bypass-easy-2525d-s1.svg"),
        ),
        (
            "bypass-easy-2525e-s0",
            include_str!("golden/bypass-easy-2525e-s0.svg"),
        ),
        (
            "bypass-easy-2525e-s1",
            include_str!("golden/bypass-easy-2525e-s1.svg"),
        ),
    ];

    fn definition(case: &Case, version: &str, status: &str) -> GraphicDefinition {
        let sidc = format!("{version}0325{status}000{}0000", case.entity);
        let points = case
            .points
            .iter()
            .map(|&(lon, lat)| ControlPoint::ground(GeoPoint::new(lon, lat).unwrap()))
            .collect();
        let mut d = GraphicDefinition::new(
            GraphicId::new(case.name).unwrap(),
            SymbolId::parse(&sidc).unwrap(),
            points,
        );
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
        d
    }

    /// The canvas covers every drawn point and label anchor with a margin and
    /// is never smaller than 1100 x 900.
    fn svg(case: &Case, version: &str, status: &str) -> String {
        let construction =
            construct(&definition(case, version, status), &Config::default()).unwrap();
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
        )
        .unwrap();
        let anchors = plan.labels.iter().filter_map(|l| l.screen);
        let shapes = plan.screen.iter().flat_map(|i| match &i.shape {
            ScreenShape::Polyline(p) | ScreenShape::Polygon(p) => p.iter().copied(),
        });
        let (mut w, mut h) = (1100.0_f64, 900.0_f64);
        for p in shapes.chain(anchors) {
            w = w.max((p.x + 60.0).ceil());
            h = h.max((p.y + 60.0).ceil());
        }
        milgraphics::svg::to_svg(&plan, w, h)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_requested() -> bool {
        std::env::var("UPDATE_GOLDEN").is_ok_and(|v| v == "1")
    }

    #[cfg(target_arch = "wasm32")]
    fn update_requested() -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn write_golden(name: &str, svg: &str) {
        let path = format!("{}/tests/golden/{name}.svg", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(path, svg).unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    fn write_golden(_name: &str, _svg: &str) {}

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn graphics_match_their_golden_svgs() {
        let update = update_requested();
        let mut mismatches = Vec::new();
        for case in &CASES {
            for (version, edition) in [("11", "2525d"), ("15", "2525e")] {
                for (status, tag) in [("0", "s0"), ("1", "s1")] {
                    let name = format!("{}-{edition}-{tag}", case.name);
                    let actual = svg(case, version, status);
                    if update {
                        write_golden(&name, &actual);
                        continue;
                    }
                    let expected = GOLDEN.iter().find(|(n, _)| *n == name).map(|(_, s)| *s);
                    if expected != Some(actual.as_str()) {
                        mismatches.push(name);
                    }
                }
            }
        }
        assert!(mismatches.is_empty(), "golden mismatch: {mismatches:?}");
    }
}
