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
        // The APP-6(D) text is not available: its symbols are declared on
        // agreement with the oracle alone and cite nothing.
        let document = match s.standard {
            StandardVersion::Mil2525Dch1 => Some("mil-std-2525d-ch1"),
            StandardVersion::Mil2525Ech1 => Some("mil-std-2525e-ch1"),
            StandardVersion::App6Ech2 => Some("app-06-e-v2"),
            _ => None,
        };
        assert_eq!(
            s.reference().map(|r| r.document),
            document,
            "{} {} cites the wrong standard",
            s.standard,
            s.name()
        );
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
    let app6d = SymbolId::parse("10032500001403000000").unwrap();
    assert_eq!(spec(&app6d).unwrap().reference(), None);
    let app6e = SymbolId::parse("16032500001412000000").unwrap();
    let app6e = spec(&app6e).unwrap();
    assert_eq!(app6e.reference().map(|r| r.table), Some("Table 8-9"));
    // The standard's name, where upstream's catalog has another graphic's.
    assert_eq!(app6e.name(), "Probable Line of Deployment");
    let avenue = SymbolId::parse("16032500001523000000").unwrap();
    assert_eq!(spec(&avenue).unwrap().name(), "Avenue of Approach");
    // Fighting Position has no row in APP-6(E)(2).
    let undefined = SymbolId::parse("16032500002910000000").unwrap();
    assert!(matches!(spec(&undefined), Err(Unsupported::Symbol { .. })));
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

#[test]
fn point_counts_follow_the_standards_text_where_upstream_differs() {
    let points = |code: &str| {
        let s = spec(&SymbolId::parse(code).unwrap()).unwrap();
        (s.min_points, s.max_points)
    };
    // Withdraw and Withdraw Under Pressure: three, as Line24.
    for code in [
        "15032500003424000000",
        "16032500003424000000",
        "15032500003425000000",
        "16032500003425000000",
    ] {
        assert_eq!(points(code), (3, 3), "{code}");
    }
    // Trip Wire (2525D Line15, APP-6(E)), Bearing Line and Linear Target
    // (APP-6(E)): two.
    for code in [
        "11032500002905000000",
        "16032500002905000000",
        "16032500002201000000",
        "16032500002407010000",
    ] {
        assert_eq!(points(code), (2, 2), "{code}");
    }
}
