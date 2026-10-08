//! Catalog invariants and the draw rules of the six first-milestone graphics.

use super::*;

fn rule(version: u8, set: u8, entity: u32) -> CatalogDrawRule {
    lookup(version, set, entity)
        .expect("catalog row present")
        .draw_rule
}

fn standard(rule: DrawRule) -> CatalogDrawRule {
    CatalogDrawRule::Standard(rule)
}

#[test]
fn phase_line_differs_by_edition() {
    assert_eq!(rule(11, 25, 140300), standard(DrawRule::Line2));
    assert_eq!(rule(15, 25, 140300), standard(DrawRule::Line1));
}

#[test]
fn named_area_of_interest_is_area1() {
    assert_eq!(rule(11, 25, 120200), standard(DrawRule::Area1));
}

#[test]
fn main_attack_follows_upstream_in_both_editions() {
    // The 2525E change 1 standard prints Axis1; upstream's mse.txt says Axis2.
    // The catalog records upstream (see "Divergences under review" in UPSTREAM.md).
    assert_eq!(rule(11, 25, 151403), standard(DrawRule::Axis2));
    assert_eq!(rule(15, 25, 151403), standard(DrawRule::Axis2));
}

#[test]
fn air_corridor_takes_width() {
    let entry = lookup(11, 25, 170100).expect("air corridor");
    assert_eq!(entry.draw_rule, standard(DrawRule::Corridor1));
    assert!(entry.modifiers.contains(&ModifierField::AM));
}

#[test]
fn range_fan_is_an_arc() {
    assert_eq!(rule(11, 25, 242200), standard(DrawRule::Arc1));
}

#[test]
fn bypass_easy_is_a_fixed_three_point_task() {
    assert_eq!(rule(11, 25, 270601), standard(DrawRule::Point12));
}

#[test]
fn metoc_rows_carry_metoc_rules_and_no_modifiers() {
    let front = lookup(11, 45, 110301).expect("cold front");
    assert_eq!(front.geometry, GeometryKind::Line);
    assert_eq!(front.draw_rule, CatalogDrawRule::Metoc(MoDrawRule::Line1));
    assert!(front.modifiers.is_empty());
}

#[test]
fn later_upstream_row_wins_a_duplicated_version() {
    let probable = lookup(15, 25, 141200).expect("v15");
    let nai = lookup(16, 25, 141200).expect("v16");
    assert_ne!(probable.name, nai.name);
    assert!(!probable.versions.contains(16));
}

#[test]
fn hierarchy_names_are_outermost_first() {
    let entry = lookup(11, 25, 140300).expect("phase line");
    assert_eq!(entry.path.first().copied(), Some("Maneuver Lines"));
    assert_ne!(entry.path.last().copied(), Some(entry.name));
}

#[test]
fn entries_are_sorted_and_keys_unique() {
    let all = entries();
    assert!(all.len() > 1000);
    let key = |e: &CatalogEntry| (e.symbol_set, e.entity, e.versions.iter().next());
    for pair in all.windows(2) {
        assert!(
            key(&pair[0]) < key(&pair[1]),
            "{:?} then {:?}",
            pair[0],
            pair[1]
        );
    }
    let mut keys: Vec<_> = all
        .iter()
        .flat_map(|e| e.versions.iter().map(move |v| (e.symbol_set, e.entity, v)))
        .collect();
    let total = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), total);
}

#[test]
fn versions_are_known_codes_and_geometry_matches_family() {
    for e in entries() {
        assert!(e.versions.iter().all(|v| (10..=16).contains(&v)));
        assert!(e.versions.iter().next().is_some());
        match e.draw_rule {
            CatalogDrawRule::Standard(_) => assert_eq!(e.symbol_set, 25),
            CatalogDrawRule::Metoc(_) => {
                assert!(matches!(e.symbol_set, 45 | 46));
                assert_ne!(e.geometry, GeometryKind::Point);
            }
        }
    }
}

#[test]
fn unknown_entities_are_absent() {
    assert!(lookup(11, 25, 999_999).is_none());
    assert!(lookup(11, 99, 140_300).is_none());
    assert!(lookup(12, 25, 140_300).is_none());
    assert!(lookup(255, 25, 140_300).is_none());
}

#[test]
fn names_round_trip() {
    for rule in DrawRule::ALL {
        assert_eq!(DrawRule::from_name(rule.name()), Some(*rule));
    }
    for rule in MoDrawRule::ALL {
        assert_eq!(MoDrawRule::from_name(rule.name()), Some(*rule));
    }
    assert_eq!(DrawRule::from_name("Line2"), Some(DrawRule::Line2));
    assert_eq!(DrawRule::from_name("line2"), None);
    assert_eq!(DrawRule::ALL.first(), Some(&DrawRule::DoNotDraw));
    assert_eq!(DrawRule::ALL.last(), Some(&DrawRule::Arc1));
}

#[test]
fn version_set_membership() {
    let set = VersionSet::from_bits((1 << 11) | (1 << 15));
    assert!(set.contains(11) && set.contains(15));
    assert!(!set.contains(10) && !set.contains(200));
    assert_eq!(set.iter().collect::<Vec<_>>(), [11, 15]);
}
