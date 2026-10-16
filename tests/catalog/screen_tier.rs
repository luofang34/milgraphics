//! Declared graphics that draw anything in the screen tier. A host that
//! drapes only the geographic tier draws those parts flat on the screen, where
//! they bend and ignore terrain in low perspective views, so the list records
//! each one and changes only on purpose.

use super::*;

const SCREEN_TIER: &str = include_str!("../fixtures/oracle/screen_tier.txt");

#[test]
fn only_listed_graphics_draw_in_the_screen_tier() {
    let listed: BTreeSet<(String, String)> = SCREEN_TIER
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| {
            let mut words = l.split_whitespace();
            Some((words.next()?.to_owned(), words.next()?.to_owned()))
        })
        .collect();
    let mut found = BTreeSet::new();
    for r in records() {
        let case = r["case"].as_str().unwrap();
        // One case per declared (version, symbol); the others vary modifiers.
        if case.matches('-').count() != 1 {
            continue;
        }
        let d = definition(&r);
        if support::spec(&d.symbol).is_err() {
            continue;
        }
        // Some oracle cases carry modifiers the standard's template refuses;
        // `catalog::standard` checks those against their goldens.
        let Ok(c) = construct(&d, &Config::default()) else {
            continue;
        };
        let ground = !geographic(&c, &View::new(0, 0)).unwrap().is_empty();
        let plan = render(
            &c,
            &View::new(0, 0),
            &frame(&r),
            &FixedAdvanceMetrics::default(),
        )
        .unwrap();
        let screen = plan.screen.iter().any(|i| i.decoration);
        let tier = match (ground, screen) {
            (false, _) => "screen-only",
            (true, true) => "mixed",
            (true, false) => continue,
        };
        found.insert((case.to_owned(), tier.to_owned()));
    }
    let stale: Vec<_> = listed.difference(&found).collect();
    let missing: Vec<_> = found.difference(&listed).collect();
    assert!(
        stale.is_empty() && missing.is_empty(),
        "fixtures/oracle/screen_tier.txt is out of date.\n\
         listed but not drawn so, remove: {stale:?}\nnot listed, add: {missing:?}"
    );
}
