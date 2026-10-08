//! Property tests: random valid-looking graphics anywhere on Earth.

use milgraphics::{
    Altitude, Budget, Config, Edit, GeoPoint, GraphicDefinition, Modifiers, VerticalDatum,
    apply_edit,
};
use proptest::prelude::*;
use proptest::test_runner::{Config as RunConfig, TestCaseError, TestRunner};

use crate::fixtures::{
    AIR_CORRIDOR, BYPASS_EASY, MAIN_ATTACK, NAMED_AREA, PHASE_LINE, RANGE_FAN, assert_plan_sane,
    built, control_point, projections, rendered, run_everywhere, symbol, vertex_counts,
};

const CASES: u32 = 256;

/// A supported symbol: entity and the control-point range generated for it
/// (the upper bound is kept small for the unbounded line and area symbols).
#[derive(Clone, Copy, Debug)]
struct Sym {
    entity: &'static str,
    min: usize,
    max: usize,
}

const SYMBOLS: [Sym; 6] = [
    Sym {
        entity: PHASE_LINE,
        min: 2,
        max: 12,
    },
    Sym {
        entity: NAMED_AREA,
        min: 3,
        max: 12,
    },
    Sym {
        entity: MAIN_ATTACK,
        min: 3,
        max: 24,
    },
    Sym {
        entity: AIR_CORRIDOR,
        min: 2,
        max: 12,
    },
    Sym {
        entity: RANGE_FAN,
        min: 1,
        max: 1,
    },
    Sym {
        entity: BYPASS_EASY,
        min: 3,
        max: 3,
    },
];

fn runner() -> TestRunner {
    TestRunner::new(RunConfig {
        cases: CASES,
        failure_persistence: None,
        ..RunConfig::default()
    })
}

fn lon() -> impl Strategy<Value = f64> {
    prop_oneof![-180.0..=180.0f64, 179.0..=180.0f64, -180.0..=-179.0f64]
}

fn lat() -> impl Strategy<Value = f64> {
    prop_oneof![-90.0..=90.0f64, 89.0..=90.0f64, -90.0..=-89.0f64]
}

/// Extent of the point cloud in degrees: from sub-metre to hemispheric.
fn spread() -> impl Strategy<Value = f64> {
    prop_oneof![
        1.0e-7..1.0e-4f64,
        1.0e-4..1.0e-2f64,
        0.01..1.0f64,
        1.0..30.0f64,
        30.0..180.0f64,
    ]
}

fn points(min: usize, max: usize) -> impl Strategy<Value = Vec<(f64, f64)>> {
    (
        lon(),
        lat(),
        spread(),
        prop::collection::vec((-1.0..=1.0f64, -1.0..=1.0f64), min..=max),
    )
        .prop_map(|(clon, clat, spread, offsets)| {
            offsets
                .into_iter()
                .map(|(a, b)| (clon + a * spread, (clat + b * spread).clamp(-90.0, 90.0)))
                .collect()
        })
}

fn text() -> impl Strategy<Value = Option<String>> {
    prop::option::of(prop_oneof!["[ -~]{0,20}", "\\PC{0,12}"])
}

fn distance() -> impl Strategy<Value = f64> {
    prop_oneof![Just(0.0), 0.5..100.0f64, 100.0..1.0e5f64, 1.0e5..2.0e7f64]
}

fn modifiers(sym: Sym, count: usize) -> impl Strategy<Value = Modifiers> {
    let altitudes = prop::collection::vec(-200.0..30_000.0f64, 0..=count);
    let ranges = prop::collection::vec(distance(), 1..=4);
    let azimuths = prop::collection::vec(-800.0..800.0f64, 8);
    (
        text(),
        text(),
        text(),
        distance(),
        altitudes,
        ranges,
        azimuths,
    )
        .prop_map(move |(t, w, w1, width, x, am, an)| {
            let mut m = Modifiers::default();
            match sym.entity {
                AIR_CORRIDOR => {
                    m.designation = t;
                    m.dtg_start = w;
                    m.dtg_end = w1;
                    m.distances_m = vec![width];
                    m.altitudes = x
                        .into_iter()
                        .map(|metres| Altitude::new(metres, VerticalDatum::MeanSeaLevel))
                        .collect();
                }
                RANGE_FAN => {
                    let pairs = am.len() * 2;
                    m.distances_m = am;
                    m.azimuths_deg = an.into_iter().take(pairs).collect();
                }
                BYPASS_EASY => {}
                _ => m.designation = t,
            }
            m
        })
}

/// A definition of `sym` with a valid point count and plausible amplifiers.
fn definition(sym: Sym) -> impl Strategy<Value = GraphicDefinition> {
    (
        prop_oneof![Just(11u8), Just(15u8)],
        0u8..2,
        points(sym.min, sym.max),
        any::<u64>(),
    )
        .prop_flat_map(move |(version, status, pts, revision)| {
            let count = pts.len();
            modifiers(sym, count).prop_map(move |m| {
                let mut d = crate::fixtures::definition(sym.entity, &pts);
                d.symbol = symbol(version, status, sym.entity);
                d.points = pts.iter().map(|&(lo, la)| control_point(lo, la)).collect();
                d.modifiers = m;
                d.revision = revision;
                d
            })
        })
}

/// A coarser geodesic step and a smaller vertex budget than the defaults, so
/// continent-scale cases stay fast; oversized output is a budget error.
fn config() -> Config {
    Config {
        budget: Budget {
            max_vertices: 20_000,
            ..Budget::default()
        },
        geodesic_step_m: 100_000.0,
    }
}

fn check<S: Strategy>(strategy: S, test: impl Fn(S::Value) -> Result<(), TestCaseError>) {
    let result = runner().run(&strategy, test);
    assert!(
        result.is_ok(),
        "{}",
        result.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

fn scale() -> impl Strategy<Value = f64> {
    prop_oneof![1.0e2..1.0e4f64, 1.0e4..1.0e7f64, 1.0e7..1.0e9f64]
}

#[test]
fn construction_and_rendering_never_panic_and_stay_finite() {
    for sym in SYMBOLS {
        check((definition(sym), scale()), |(def, scale)| {
            // A typed construction error is a valid outcome.
            run_everywhere(&def, &config(), scale).ok();
            Ok(())
        });
    }
}

/// Renders every definition under a small vertex budget and hands each plan
/// that fits to `check_counts` with the budget.
fn under_small_budget(check_counts: impl Fn((usize, usize), usize) -> Result<(), TestCaseError>) {
    for sym in SYMBOLS {
        check(
            (definition(sym), 1usize..400, scale()),
            |(def, max, scale)| {
                let budget = Budget {
                    max_vertices: max,
                    ..Budget::default()
                };
                let config = Config { budget, ..config() };
                // Err is a typed budget (or validation) error; Ok must fit.
                let Ok(c) = built(&def, &config) else {
                    return Ok(());
                };
                for p in projections(&def, scale) {
                    if let Ok(plan) = rendered(&c, p.as_ref(), &budget) {
                        assert_plan_sane(&plan);
                        check_counts(vertex_counts(&plan), max)?;
                    }
                }
                Ok(())
            },
        );
    }
}

#[test]
fn small_budgets_give_typed_errors_and_sane_output() {
    under_small_budget(|_, _| Ok(()));
}

#[test]
fn screen_tier_respects_the_vertex_budget() {
    under_small_budget(|(_, screen), max| {
        prop_assert!(screen <= max, "{screen} screen vertices exceed {max}");
        Ok(())
    });
}

#[test]
fn geographic_tier_respects_the_vertex_budget() {
    under_small_budget(|(geo, _), max| {
        prop_assert!(geo <= max, "{geo} geographic vertices exceed {max}");
        Ok(())
    });
}

#[test]
fn construction_and_rendering_are_deterministic() {
    for sym in SYMBOLS {
        check((definition(sym), scale()), |(def, scale)| {
            let config = config();
            let (a, b) = (built(&def, &config), built(&def, &config));
            prop_assert_eq!(&a, &b);
            if let Ok(c) = a {
                // One projection of the three keeps the run short.
                let projections = projections(&def, scale);
                for p in projections.iter().take(1) {
                    let first = rendered(&c, p.as_ref(), &config.budget);
                    let second = rendered(&c, p.as_ref(), &config.budget);
                    prop_assert_eq!(first, second);
                }
            }
            Ok(())
        });
    }
}

fn target() -> impl Strategy<Value = GeoPoint> {
    (lon(), lat()).prop_map(|(lo, la)| GeoPoint::new(lo, la).unwrap_or_else(|_| zero()))
}

fn zero() -> GeoPoint {
    control_point(0.0, 0.0).position
}

#[test]
fn moving_any_handle_is_pure_and_bumps_the_revision() {
    for sym in SYMBOLS {
        check(
            (definition(sym), target(), 0usize..1024),
            |(def, to, pick)| {
                let Ok(c) = built(&def, &config()) else {
                    return Ok(());
                };
                // One random handle per case; across cases every handle kind,
                // widths and azimuths included, is reached.
                let Some(handle) = c.handles.get(pick % c.handles.len().max(1)) else {
                    return Ok(());
                };
                let before = def.clone();
                let result = apply_edit(
                    &def,
                    &Edit::Move {
                        handle: handle.id,
                        to,
                    },
                );
                prop_assert_eq!(&def, &before, "apply_edit mutated its input");
                if let Ok(next) = result {
                    prop_assert_eq!(next.revision, def.revision.wrapping_add(1));
                    // The edited graphic is constructible or refused with a
                    // typed error, and renders sanely.
                    if let Ok(c) = built(&next, &config()) {
                        let projection = &projections(&next, 1.0e5)[0];
                        if let Ok(plan) = rendered(&c, projection.as_ref(), &config().budget) {
                            assert_plan_sane(&plan);
                        }
                    }
                }
                Ok(())
            },
        );
    }
}
