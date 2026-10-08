//! Explicit degenerate and boundary inputs. Each must give a typed error or
//! finite output; the ones named as errors must be errors.

use milgraphics::render::GeoShape;
use milgraphics::{
    Altitude, Budget, BudgetError, Config, ConstructError, GraphicDefinition, VerticalDatum,
    construct,
};

use crate::fixtures::{
    AIR_CORRIDOR, BYPASS_EASY, MAIN_ATTACK, NAMED_AREA, PHASE_LINE, RANGE_FAN, definition,
    projections, rendered, run_everywhere, vertex_counts,
};

const SCALE: f64 = 250_000.0;

fn corridor(points: &[(f64, f64)], width_m: f64) -> GraphicDefinition {
    let mut d = definition(AIR_CORRIDOR, points);
    d.modifiers.distances_m = vec![width_m];
    d.modifiers.altitudes = vec![Altitude::new(500.0, VerticalDatum::MeanSeaLevel)];
    d.modifiers.designation = Some("ROUTE".to_owned());
    d
}

fn fan(at: (f64, f64), ranges: &[f64], azimuths: &[f64]) -> GraphicDefinition {
    let mut d = definition(RANGE_FAN, &[at]);
    d.modifiers.distances_m = ranges.to_vec();
    d.modifiers.azimuths_deg = azimuths.to_vec();
    d
}

/// Runs the whole pipeline; any panic or non-finite output fails the test.
fn survives(d: &GraphicDefinition) -> bool {
    run_everywhere(d, &Config::default(), SCALE).is_ok()
}

fn geo_parts(d: &GraphicDefinition) -> Vec<Vec<[f64; 2]>> {
    let c = construct(d, &Config::default()).unwrap();
    let projection = &projections(d, SCALE)[0];
    let plan = rendered(&c, projection.as_ref(), &Budget::default()).unwrap();
    plan.geo
        .into_iter()
        .flat_map(|i| match i.shape {
            GeoShape::Lines(p) | GeoShape::Polygons(p) => p,
        })
        .collect()
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn coincident_control_points() {
    let p = (10.0, 10.0);
    survives(&definition(PHASE_LINE, &[p, p]));
    survives(&definition(NAMED_AREA, &[p, p, p]));
    survives(&definition(MAIN_ATTACK, &[p, p, p]));
    survives(&definition(BYPASS_EASY, &[p, p, p]));
    survives(&corridor(&[p, p], 1_000.0));
    survives(&definition(NAMED_AREA, &[p, (10.0, 11.0), p]));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn zero_length_segments() {
    let (a, b) = ((0.0, 0.0), (1.0, 1.0));
    assert!(survives(&definition(PHASE_LINE, &[a, a, b, b])));
    survives(&definition(NAMED_AREA, &[a, a, b, b, a]));
    survives(&definition(MAIN_ATTACK, &[a, a, b, b, (2.0, 0.0)]));
    survives(&corridor(&[a, a, b, b], 2_000.0));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn sharp_turn_reversing_on_itself() {
    let (a, b) = ((0.0, 0.0), (0.0, 1.0));
    assert!(survives(&definition(PHASE_LINE, &[a, b, a])));
    survives(&definition(NAMED_AREA, &[a, b, a]));
    survives(&definition(MAIN_ATTACK, &[a, b, a, (1.0, 0.0)]));
    survives(&corridor(&[a, b, a], 5_000.0));
    survives(&corridor(&[a, b, a, b], 5_000.0));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn reversed_point_order() {
    let forward = [(0.0, 0.0), (1.0, 0.5), (2.0, 0.0), (3.0, 1.0)];
    let mut backward = forward;
    backward.reverse();
    for entity in [PHASE_LINE, NAMED_AREA] {
        assert!(survives(&definition(entity, &forward)));
        assert!(survives(&definition(entity, &backward)));
    }
    assert!(survives(&corridor(&forward, 8_000.0)));
    assert!(survives(&corridor(&backward, 8_000.0)));
    let attack = [(0.0, 0.0), (1.0, 0.5), (2.0, 0.0), (1.0, 0.2)];
    assert!(survives(&definition(MAIN_ATTACK, &attack)));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn phase_line_crossing_the_antimeridian_is_split() {
    let d = definition(PHASE_LINE, &[(170.0, 10.0), (-170.0, 10.0)]);
    assert!(survives(&d));
    let parts = geo_parts(&d);
    assert!(parts.len() >= 2, "expected a split line, got {parts:?}");
    assert!(
        parts
            .iter()
            .flatten()
            .all(|p| (-180.0..=180.0).contains(&p[0]))
    );
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn area_crossing_the_antimeridian_is_split() {
    let d = definition(
        NAMED_AREA,
        &[(170.0, 0.0), (-170.0, 0.0), (-170.0, 10.0), (170.0, 10.0)],
    );
    assert!(survives(&d));
    let parts = geo_parts(&d);
    assert!(parts.len() >= 2, "expected a split ring, got {parts:?}");
    assert!(
        parts
            .iter()
            .flatten()
            .all(|p| (-180.0..=180.0).contains(&p[0]))
    );
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn area_around_a_pole() {
    let ring = [(0.0, 89.5), (90.0, 89.5), (180.0, 89.5), (-90.0, 89.5)];
    assert!(survives(&definition(NAMED_AREA, &ring)));
    let south = [(0.0, -89.5), (-90.0, -89.5), (180.0, -89.5), (90.0, -89.5)];
    assert!(survives(&definition(NAMED_AREA, &south)));
    let on_pole = [(0.0, 90.0), (90.0, 90.0), (180.0, 90.0)];
    survives(&definition(NAMED_AREA, &on_pole));
    survives(&definition(PHASE_LINE, &[(0.0, 90.0), (180.0, 89.0)]));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn range_fan_full_circle_and_zero_range() {
    let at = (20.0, 30.0);
    for azimuths in [[0.0, 0.0], [90.0, 90.0], [0.0, 360.0], [-180.0, 180.0]] {
        survives(&fan(at, &[5_000.0], &azimuths));
    }
    survives(&fan(at, &[0.0], &[10.0, 100.0]));
    survives(&fan(at, &[0.0, 5_000.0], &[0.0, 0.0, 10.0, 20.0]));
    assert!(survives(&fan(at, &[5_000.0], &[10.0, 100.0])));
    survives(&fan((0.0, 90.0), &[5_000.0], &[0.0, 90.0]));
    survives(&fan((179.999, 0.0), &[1.0e6], &[0.0, 359.0]));
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn corridor_without_width_is_a_typed_error() {
    let d = corridor(&[(0.0, 0.0), (1.0, 1.0)], 0.0);
    assert!(construct(&d, &Config::default()).is_err());
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn main_attack_width_point_on_the_axis_is_a_typed_error() {
    let on_tip = definition(MAIN_ATTACK, &[(0.0, 0.0), (0.0, 1.0), (0.0, 1.0)]);
    assert!(construct(&on_tip, &Config::default()).is_err(), "on P2");
    let between = definition(MAIN_ATTACK, &[(0.0, 0.0), (0.0, 1.0), (0.0, 0.5)]);
    assert!(construct(&between, &Config::default()).is_err(), "on axis");
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn non_finite_amplifiers_are_typed_errors() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let d = corridor(&[(0.0, 0.0), (1.0, 1.0)], bad);
        assert!(
            construct(&d, &Config::default()).is_err(),
            "corridor AM {bad}"
        );
        let d = fan((0.0, 0.0), &[bad], &[0.0, 90.0]);
        assert!(construct(&d, &Config::default()).is_err(), "fan AM {bad}");
        let d = fan((0.0, 0.0), &[1_000.0], &[0.0, bad]);
        assert!(construct(&d, &Config::default()).is_err(), "fan AN {bad}");
        let mut d = corridor(&[(0.0, 0.0), (1.0, 1.0)], 1_000.0);
        d.modifiers.altitudes[0].metres = bad;
        assert!(construct(&d, &Config::default()).is_err(), "X {bad}");
    }
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn too_many_control_points_is_a_budget_error() {
    let points: Vec<(f64, f64)> = (0..10_001).map(|i| (f64::from(i) * 1.0e-3, 0.0)).collect();
    let d = definition(PHASE_LINE, &points);
    let err = construct(&d, &Config::default()).unwrap_err();
    assert!(
        matches!(
            err,
            ConstructError::Budget(BudgetError::ControlPoints { .. })
        ),
        "{err}"
    );
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn overlong_designation_is_a_budget_error() {
    let mut d = definition(PHASE_LINE, &[(0.0, 0.0), (1.0, 1.0)]);
    d.modifiers.designation = Some("T".repeat(300));
    let err = construct(&d, &Config::default()).unwrap_err();
    assert!(
        matches!(err, ConstructError::Budget(BudgetError::Text { .. })),
        "{err}"
    );
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn antimeridian_cuts_count_against_the_vertex_budget() {
    let d = definition(PHASE_LINE, &[(170.0, 0.0), (-170.0, 0.0), (170.0, 0.0)]);
    let full = construct(&d, &Config::default()).unwrap();
    let built: usize = full.parts.iter().map(|p| p.geometry.points().len()).sum();
    // The tightest budget that still constructs.
    let (max, c) = (built..built + 8)
        .find_map(|max| {
            let budget = budget_of(max);
            construct(&d, &config_of(budget)).ok().map(|c| (max, c))
        })
        .unwrap();
    let budget = budget_of(max);
    let projection = &projections(&d, SCALE)[0];
    if let Ok(plan) = rendered(&c, projection.as_ref(), &budget) {
        let (geo, _) = vertex_counts(&plan);
        assert!(
            geo <= max,
            "{geo} geographic vertices exceed the budget {max}"
        );
    }
}

#[cfg_attr(not(target_arch = "wasm32"), test)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn screen_decorations_count_against_the_vertex_budget() {
    let d = definition(BYPASS_EASY, &[(0.0, 0.0), (0.1, 0.1), (0.2, 0.0)]);
    let (max, c) = (1..64)
        .find_map(|max| {
            let budget = budget_of(max);
            construct(&d, &config_of(budget)).ok().map(|c| (max, c))
        })
        .unwrap();
    let budget = budget_of(max);
    let projection = &projections(&d, SCALE)[0];
    if let Ok(plan) = rendered(&c, projection.as_ref(), &budget) {
        let (_, screen) = vertex_counts(&plan);
        assert!(
            screen <= max,
            "{screen} screen vertices exceed the budget {max}"
        );
    }
}

/// The default budget with at most `max` vertices.
fn budget_of(max: usize) -> Budget {
    let mut budget = Budget::default();
    budget.max_vertices = max;
    budget
}

/// The default configuration with `budget`.
fn config_of(budget: Budget) -> Config {
    let mut config = Config::default();
    config.budget = budget;
    config
}
