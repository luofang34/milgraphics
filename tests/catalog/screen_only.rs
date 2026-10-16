//! Declared graphics with nothing in the geographic tier. A host that drapes
//! only that tier draws them through the screen tier alone, which bends and
//! ignores terrain in low perspective views, so the list may only shrink.

use super::*;

const SCREEN_ONLY: &str = include_str!("../fixtures/oracle/screen_only.txt");

#[test]
fn only_listed_graphics_lack_a_geographic_tier() {
    let listed: BTreeSet<String> = SCREEN_ONLY
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
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
        if geographic(&c, &View::new(0, 0)).unwrap().is_empty() {
            found.insert(case.to_owned());
        }
    }
    let gained: Vec<&String> = listed.difference(&found).collect();
    let lost: Vec<&String> = found.difference(&listed).collect();
    assert!(
        gained.is_empty() && lost.is_empty(),
        "fixtures/oracle/screen_only.txt is out of date.\n\
         now geographic, remove: {gained:?}\nscreen only, not listed: {lost:?}"
    );
}
