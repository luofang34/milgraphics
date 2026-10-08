//! Every declared MIL-STD-2525E change 1 and APP-6(E)(2) graphic, drawn with
//! all of its declared amplifiers filled, is compared with a golden SVG reviewed
//! against the standard's template. `UPDATE_GOLDEN=1` rewrites
//! `tests/golden/standard/`; a golden without a declared graphic fails.

use std::collections::BTreeSet;

use milgraphics::{ModifierKind, ModifierValue};

use super::*;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/standard");

/// Editions checked against their standard, by case suffix.
const SUFFIXES: [&str; 2] = ["-e", "-app6e"];

/// A text for every declared field the oracle case leaves empty: the field's
/// letters, so each one is visible where the graphic places it.
fn filled(r: &Value) -> Option<(String, GraphicDefinition)> {
    let case = r["case"].as_str()?;
    if !SUFFIXES.iter().any(|s| case.ends_with(s)) {
        return None;
    }
    let symbol = SymbolId::parse(r["symbol"].as_str()?).ok()?;
    let spec = support::spec(&symbol).ok()?;
    let mut d = definition(r);
    // A case listed as differing from the oracle may give a field the
    // template does not show, which the graphic then does not declare.
    let listed = STANDARD
        .lines()
        .any(|l| l.split_whitespace().next() == Some(case));
    for field in ModifierField::ALL {
        if listed && !spec.modifiers.iter().any(|m| m.field == field) {
            d.modifiers.clear(field);
        }
    }
    for m in spec.modifiers {
        if d.modifiers.is_set(m.field) {
            continue;
        }
        let value = match m.field.kind() {
            ModifierKind::Text | ModifierKind::DateTime => {
                ModifierValue::Text(m.field.name().to_owned())
            }
            ModifierKind::SymbolCode => ModifierValue::Text("10031000001211000000".to_owned()),
            _ => continue,
        };
        d.modifiers.set(m.field, value).unwrap();
    }
    Some((case.to_owned(), d))
}

fn svg(r: &Value, d: &GraphicDefinition) -> String {
    let f = frame(r);
    let c = construct(d, &Config::default()).unwrap();
    let plan = render(
        &c,
        &View::new(0, 0),
        &f,
        &FixedAdvanceMetrics::default(),
        &Budget::default(),
    )
    .unwrap();
    let anchors = plan.labels.iter().filter_map(|l| l.screen);
    let points = plan
        .screen
        .iter()
        .flat_map(|i| i.shape.points().iter().copied());
    let (mut w, mut h) = (400.0_f64, 300.0_f64);
    for p in points.chain(anchors) {
        w = w.max((p.x + 80.0).ceil());
        h = h.max((p.y + 80.0).ceil());
    }
    milgraphics::svg::to_svg(&plan, w, h)
}

#[test]
fn every_graphic_matches_its_reviewed_golden() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut drawn = BTreeSet::new();
    let mut failures = Vec::new();
    for r in records() {
        let Some((case, d)) = filled(&r) else {
            continue;
        };
        let out = svg(&r, &d);
        let path = format!("{DIR}/{case}.svg");
        drawn.insert(format!("{case}.svg"));
        if update {
            std::fs::create_dir_all(DIR).unwrap();
            std::fs::write(&path, &out).unwrap();
        } else if std::fs::read_to_string(&path).ok().as_deref() != Some(out.as_str()) {
            failures.push(case);
        }
    }
    let stale: Vec<String> = std::fs::read_dir(DIR)
        .map(|it| {
            it.filter_map(|e| e.ok()?.file_name().into_string().ok())
                .filter(|n| !drawn.contains(n))
                .collect()
        })
        .unwrap_or_default();
    if update {
        for name in &stale {
            std::fs::remove_file(format!("{DIR}/{name}")).unwrap();
        }
        return;
    }
    assert!(
        stale.is_empty(),
        "goldens without a declared graphic: {stale:?}"
    );
    assert!(
        failures.is_empty(),
        "{} graphics differ from their reviewed golden (UPDATE_GOLDEN=1 after review): {failures:?}",
        failures.len()
    );
}
