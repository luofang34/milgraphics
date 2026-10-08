//! Phase Line, Named Area of Interest, Main Attack, Air Corridor, Range Fan
//! Sector and Bypass Easy.

use super::{MANY, cm, list, opt, req};
use crate::family::Family;
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::{Mil2525Dch1 as D1, Mil2525Ech1 as E1};
use crate::support::SymbolSpec;

const NAI: Family = Family::LabelledArea { prefix: "NAI" };
const T: &[crate::support::ModifierSpec] = &[opt(M::T)];
const AXIS: &[crate::support::ModifierSpec] = &[opt(M::T), opt(M::W), opt(M::W1)];
const CORRIDOR: &[crate::support::ModifierSpec] = &[
    opt(M::T),
    req(M::AM),
    list(M::X, false, 1, 2),
    opt(M::W),
    opt(M::W1),
];
/// Ranges, and a left and right azimuth per sector (see `family::range_fan`).
const FAN: &[crate::support::ModifierSpec] = &[
    list(M::AM, true, 1, MANY),
    list(M::AN, true, 2, MANY),
    list(M::X, false, 1, MANY),
];

pub(super) static SPECS: &[SymbolSpec] = &[
    cm(D1, 140300, Family::PhaseLine)
        .points(2, MANY)
        .amplifiers(T),
    cm(E1, 140300, Family::PhaseLine)
        .points(2, MANY)
        .amplifiers(T),
    cm(D1, 120200, NAI).points(3, MANY).amplifiers(T),
    cm(E1, 120200, NAI).points(3, MANY).amplifiers(T),
    cm(D1, 151403, Family::Axis).points(3, 50).amplifiers(AXIS),
    cm(E1, 151403, Family::Axis).points(3, 50).amplifiers(AXIS),
    cm(D1, 170100, Family::Corridor)
        .points(2, 99)
        .amplifiers(CORRIDOR),
    cm(E1, 170100, Family::Corridor)
        .points(2, 99)
        .amplifiers(CORRIDOR),
    cm(D1, 242200, Family::RangeFanSector).amplifiers(FAN),
    cm(E1, 242200, Family::RangeFanSector).amplifiers(FAN),
    cm(D1, 270601, Family::Bypass).points(3, 3),
    cm(E1, 270601, Family::Bypass).points(3, 3),
];
