//! Where a standard's template and upstream's catalog disagree on the
//! amplifiers of a symbol, per edition: fields the template shows that the
//! catalog does not list are declared in addition, and fields the catalog
//! lists that the template does not show are left undeclared.

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
    // MIL-STD-2525E change 1: the counterattack's name.
    (15, 25, 340_600, &["T"]),
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
    // APP-6(E)(2): fields the labels of the template carry.
    (16, 25, 151_000, &["T"]),
    (16, 25, 151_100, &["H"]),
    (16, 25, 240_102, &["H"]),
    (16, 25, 240_103, &["H"]),
    (16, 25, 272_200, &["H"]),
    (16, 25, 300_100, &["T1"]),
    (16, 25, 340_600, &["T"]),
    (16, 25, 340_700, &["T"]),
    (16, 25, 344_100, &["W"]),
    (16, 25, 344_200, &["W"]),
];

/// `(version code, symbol set, entity, fields)` the template does not show.
pub(super) const REMOVED: &[(u32, u8, u32, &[&str])] = &[
    // APP-6(E)(2): the template shows other fields in their place.
    (16, 25, 151_000, &["H"]),
    (16, 25, 240_102, &["Y"]),
    (16, 25, 240_103, &["Y"]),
    (16, 25, 272_200, &["T"]),
    (16, 25, 300_100, &["T"]),
];

/// The fields to declare in addition to the catalog's for a symbol.
pub(super) fn added(version: u32, set: u8, entity: u32) -> &'static [&'static str] {
    ADDED
        .iter()
        .find(|(v, s, e, _)| (*v, *s, *e) == (version, set, entity))
        .map_or(&[], |(_, _, _, f)| f)
}

/// The catalog's fields for a symbol that are not declared.
pub(super) fn removed(version: u32, set: u8, entity: u32) -> &'static [&'static str] {
    REMOVED
        .iter()
        .find(|(v, s, e, _)| (*v, *s, *e) == (version, set, entity))
        .map_or(&[], |(_, _, _, f)| f)
}
