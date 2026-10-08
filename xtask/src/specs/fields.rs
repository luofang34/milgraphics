//! Amplifiers a standard's template shows that upstream's catalog does not
//! list for the symbol, per edition: they are declared in addition.

/// `(version code, symbol set, entity, fields)`.
pub(super) const ADDED: &[(u32, u8, u32, &[&str])] = &[
    // MIL-STD-2525E change 1: the unit assigned a task.
    (15, 25, 151_600, &["A"]),
    (15, 25, 342_201, &["A"]),
    (15, 25, 342_202, &["A"]),
    (15, 25, 342_203, &["A"]),
    (15, 25, 342_300, &["A"]),
    (15, 25, 342_900, &["A"]),
    (15, 25, 344_500, &["A"]),
    // APP-6(E)(2): the unit assigned a task.
    (16, 25, 152_200, &["A"]),
    (16, 25, 230_100, &["A"]),
    (16, 25, 342_201, &["A"]),
    (16, 25, 342_202, &["A"]),
    (16, 25, 342_203, &["A"]),
    (16, 25, 342_300, &["A"]),
    (16, 25, 342_900, &["A"]),
    (16, 25, 343_000, &["A"]),
    (16, 25, 343_600, &["A"]),
    (16, 25, 344_500, &["A"]),
];

/// The fields to declare in addition to the catalog's for a symbol.
pub(super) fn added(version: u32, set: u8, entity: u32) -> &'static [&'static str] {
    ADDED
        .iter()
        .find(|(v, s, e, _)| (*v, *s, *e) == (version, set, entity))
        .map_or(&[], |(_, _, _, f)| f)
}
