//! Air corridors and routes whose template draws only their two sides.

use super::{cm, list, opt, req};
use crate::family::Family;
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::{App6Ech2 as E6, Mil2525Ech1 as E1};
use crate::support::SymbolSpec;

const AMPLIFIERS: &[crate::support::ModifierSpec] = &[
    opt(M::T),
    req(M::AM),
    list(M::X, false, 1, 2),
    opt(M::W),
    opt(M::W1),
];

const fn route(
    standard: crate::standard::StandardVersion,
    entity: u32,
    prefix: &'static str,
) -> SymbolSpec {
    cm(standard, entity, Family::Corridor { prefix, open: true })
        .points(2, 99)
        .amplifiers(AMPLIFIERS)
}

pub(super) static SPECS: &[SymbolSpec] = &[
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
