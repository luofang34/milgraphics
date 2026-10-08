use super::*;

#[test]
fn each_symbol_is_declared_once_per_standard() {
    let mut keys: Vec<_> = all().iter().map(|s| (s.standard, s.entity)).collect();
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
            s.name
        );
        for r in s.required {
            assert!(s.modifiers.contains(r), "{} requires undrawn {r:?}", s.name);
        }
        let doc_matches = match s.standard {
            StandardVersion::Mil2525Dch1 => s.reference.document == "mil-std-2525d-ch1",
            StandardVersion::Mil2525Ech1 => s.reference.document == "mil-std-2525e-ch1",
            _ => false,
        };
        assert!(doc_matches, "{} cites {}", s.name, s.reference.document);
    }
}

#[test]
fn lookup_by_symbol_id() {
    let pl = SymbolId::parse("15032510001403000000").unwrap();
    assert_eq!(spec(&pl).unwrap().draw_rule, "Line1");
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

#[test]
fn draw_rules_agree_with_the_catalog_or_declare_why_not() {
    use crate::catalog::{CatalogDrawRule, lookup};
    for s in all() {
        let entry = lookup(s.standard.code(), 25, s.entity).unwrap_or_else(|| {
            panic!(
                "{} {} missing from the upstream catalog",
                s.standard, s.entity
            )
        });
        let upstream = match entry.draw_rule {
            CatalogDrawRule::Standard(rule) => rule.name(),
            CatalogDrawRule::Metoc(rule) => rule.name(),
        };
        assert!(
            upstream == s.draw_rule || s.catalog_divergence.is_some(),
            "{} {}: standard says {}, upstream catalog says {upstream}, and no divergence is declared",
            s.standard,
            s.name,
            s.draw_rule
        );
        if let Some(reason) = s.catalog_divergence {
            assert_ne!(
                upstream, s.draw_rule,
                "{}: divergence declared but none exists: {reason}",
                s.name
            );
        }
    }
}
