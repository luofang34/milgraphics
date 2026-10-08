use super::*;
use crate::modifier::ModifierKind;

#[test]
fn each_symbol_is_declared_once_per_standard() {
    let mut keys: Vec<_> = all()
        .map(|s| (s.standard, s.symbol_set, s.entity))
        .collect();
    let count = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), count);
}

#[test]
fn declarations_are_internally_consistent() {
    for s in all() {
        assert!(
            s.min_points >= 1 && s.min_points <= s.max_points,
            "{}",
            s.name()
        );
        for m in s.modifiers {
            assert!(
                m.min_count >= 1 && m.min_count <= m.max_count,
                "{} {}",
                s.name(),
                m.field
            );
            let single = !matches!(
                m.field.kind(),
                ModifierKind::Distances | ModifierKind::Azimuths | ModifierKind::Altitudes
            );
            assert!(
                !single || m.max_count == 1,
                "{} {} is single-valued",
                s.name(),
                m.field
            );
        }
        let mut fields: Vec<_> = s.modifiers.iter().map(|m| m.field).collect();
        fields.sort();
        fields.dedup();
        assert_eq!(
            fields.len(),
            s.modifiers.len(),
            "{} lists a field twice",
            s.name()
        );
        let reference = s
            .reference()
            .unwrap_or_else(|| panic!("{} {} has no standard reference", s.standard, s.name()));
        let doc_matches = match s.standard {
            StandardVersion::Mil2525Dch1 => reference.document == "mil-std-2525d-ch1",
            StandardVersion::Mil2525Ech1 => reference.document == "mil-std-2525e-ch1",
            _ => false,
        };
        assert!(doc_matches, "{} cites {}", s.name(), reference.document);
    }
}

#[test]
fn lookup_by_symbol_id() {
    let pl = SymbolId::parse("15032510001403000000").unwrap();
    assert_eq!(
        spec(&pl).unwrap().reference().unwrap().draw_rule,
        Some("Line1")
    );
    let unknown_version = SymbolId::parse("12032500001403000000").unwrap();
    assert_eq!(
        spec(&unknown_version),
        Err(Unsupported::Standard { code: 12 })
    );
    let other_set = SymbolId::parse("11031000001403000000").unwrap();
    assert!(matches!(
        spec(&other_set),
        Err(Unsupported::Symbol { symbol_set: 10, .. })
    ));
    let app6 = SymbolId::parse("10032500001403000000").unwrap();
    assert!(
        matches!(spec(&app6), Err(Unsupported::Symbol { .. })),
        "APP-6 is not yet referenced"
    );
}

/// Declared symbols whose standard prints a different draw rule than
/// upstream's catalog; each is listed in UPSTREAM.md.
const DIVERGENCES: &[(StandardVersion, u32)] = &[
    (StandardVersion::Mil2525Dch1, 290500),
    (StandardVersion::Mil2525Ech1, 151403),
    (StandardVersion::Mil2525Ech1, 240802),
    (StandardVersion::Mil2525Ech1, 290700),
    (StandardVersion::Mil2525Ech1, 342400),
    (StandardVersion::Mil2525Ech1, 342500),
];

#[test]
fn draw_rules_agree_with_the_catalog_or_are_listed_divergences() {
    use crate::catalog::CatalogDrawRule;
    let upstream_md = include_str!("../../UPSTREAM.md");
    for s in all() {
        let entry = s.catalog_entry().unwrap_or_else(|| {
            panic!(
                "{} {} missing from the upstream catalog",
                s.standard, s.entity
            )
        });
        let upstream = match entry.draw_rule {
            CatalogDrawRule::Standard(rule) => rule.name(),
            CatalogDrawRule::Metoc(rule) => rule.name(),
        };
        let printed = s.reference().and_then(|r| r.draw_rule);
        let listed = DIVERGENCES.contains(&(s.standard, s.entity));
        let differs = printed.is_some_and(|p| p != upstream);
        assert_eq!(
            differs,
            listed,
            "{} {}: standard prints {printed:?}, upstream catalog {upstream}",
            s.standard,
            s.name()
        );
        if listed {
            assert!(
                upstream_md.contains(&s.entity.to_string()),
                "UPSTREAM.md does not explain {} {}",
                s.standard,
                s.entity
            );
        }
    }
}
