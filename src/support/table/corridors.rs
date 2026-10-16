//! Air corridors and routes. Editions whose template draws only the two sides
//! are declared `open`; the others put a circle at each control point.

use super::{MANY, cm, list, opt, req};
use crate::family::Family;
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::{
    self, App6D as D6, App6Ech2 as E6, Mil2525Dch1 as D1, Mil2525Ech1 as E1,
};
use crate::support::SymbolSpec;

const AMPLIFIERS: &[crate::support::ModifierSpec] = &[
    opt(M::T),
    req(M::AM),
    list(M::X, false, 1, 2),
    opt(M::W),
    opt(M::W1),
];

/// Base MIL-STD-2525D, which shares version code 10, allows at most 99 points
/// for these (TABLE H-XIII).
const BASE_2525D_MAX: usize = 99;

const fn route(standard: StandardVersion, entity: u32, prefix: &'static str) -> SymbolSpec {
    cm(standard, entity, Family::Corridor { prefix, open: true })
        .points(2, 99)
        .amplifiers(AMPLIFIERS)
}

const fn circled(
    standard: StandardVersion,
    entity: u32,
    prefix: &'static str,
    max: usize,
) -> SymbolSpec {
    cm(
        standard,
        entity,
        Family::Corridor {
            prefix,
            open: false,
        },
    )
    .points(2, max)
    .amplifiers(AMPLIFIERS)
}

pub(super) static SPECS: &[SymbolSpec] = &[
    circled(D6, 170100, "AC", BASE_2525D_MAX),
    circled(D6, 170200, "LLTR", BASE_2525D_MAX),
    circled(D6, 170300, "MRR", MANY),
    circled(D6, 170400, "SL", MANY),
    circled(D6, 170500, "SAAFR", MANY),
    circled(D6, 170600, "TC", MANY),
    circled(D6, 170700, "SC", MANY),
    circled(D1, 170200, "LLTR", MANY),
    circled(D1, 170300, "MRR", MANY),
    circled(D1, 170400, "SL", MANY),
    circled(D1, 170500, "SAAFR", MANY),
    circled(D1, 170600, "TC", MANY),
    circled(D1, 170700, "SC", MANY),
    route(E1, 170200, "LLTR"),
    route(E1, 170300, "MRR"),
    route(E1, 170400, "SL"),
    route(E1, 170500, "SAAFR"),
    route(E1, 170600, "TC"),
    route(E1, 170700, "SC"),
    route(E6, 170100, "AC"),
    route(E6, 170200, "LLTR"),
    route(E6, 170300, "MRR"),
    route(E6, 170400, "SL"),
    route(E6, 170500, "SAAFR"),
    route(E6, 170600, "TC"),
    route(E6, 170700, "SC"),
];
