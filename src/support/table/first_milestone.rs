//! Phase Line, Named Area of Interest, Main Attack, Air Corridor, Range Fan
//! Sector and Bypass Easy.

use super::{D, E, MANY, cm, list, opt, r, req};
use crate::family::Family;
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::{Mil2525Dch1 as D1, Mil2525Ech1 as E1};
use crate::support::SymbolSpec;

const NAI: Family = Family::LabelledArea { prefix: "NAI" };
const T: &[crate::support::ModifierSpec] = &[opt(M::T)];
const CORRIDOR: &[crate::support::ModifierSpec] = &[
    opt(M::T),
    req(M::AM),
    list(M::X, false, 1, 2),
    opt(M::W),
    opt(M::W1),
];
/// Ranges, and a left and right azimuth per sector (see `family::range_fan`).
const FAN: &[crate::support::ModifierSpec] =
    &[list(M::AM, true, 1, MANY), list(M::AN, true, 2, MANY)];

pub(super) static SPECS: &[SymbolSpec] = &[
    cm(D1, 140300, "Phase Line", Family::PhaseLine, "Line2", r(D, "TABLE H-VII", 446)).points(2, MANY).amplifiers(T),
    cm(E1, 140300, "Phase Line", Family::PhaseLine, "Line1", r(E, "TABLE L-VII", 478)).points(2, MANY).amplifiers(T),
    cm(D1, 120200, "Named Area of Interest", NAI, "Area1", r(D, "TABLE H-V", 436)).points(3, MANY).amplifiers(T),
    cm(E1, 120200, "Named Area of Interest", NAI, "Area1", r(E, "TABLE L-V", 468)).points(3, MANY).amplifiers(T),
    cm(D1, 151403, "Main Attack", Family::Axis, "Axis2", r(D, "TABLE H-X", 455)).points(3, 50).amplifiers(T),
    cm(E1, 151403, "Main Attack", Family::Axis, "Axis1", r(E, "TABLE L-X", 496)).points(3, 50).amplifiers(T).diverging(
        "mil-sym-java mse.txt files 151403 under Axis2; 2525E change 1 TABLE L-X gives Axis1 and marks Axis2 disused",
    ),
    cm(D1, 170100, "Air Corridor", Family::Corridor, "Corridor1", r(D, "TABLE H-XIII", 463)).points(2, 99).amplifiers(CORRIDOR),
    cm(E1, 170100, "Air Corridor", Family::Corridor, "Corridor1", r(E, "TABLE L-XI", 498)).points(2, 99).amplifiers(CORRIDOR),
    cm(D1, 242200, "Weapon/Sensor Range Fan, Sector", Family::RangeFanSector, "Arc1", r(D, "TABLE H-XVII", 528)).amplifiers(FAN),
    cm(E1, 242200, "Weapon/Sensor Range Fan, Sector", Family::RangeFanSector, "Arc1", r(E, "TABLE L-XV", 573)).amplifiers(FAN),
    cm(D1, 270601, "Obstacle Bypass, Easy", Family::Bypass, "Point12", r(D, "TABLE H-XVIII", 533)).points(3, 3),
    cm(E1, 270601, "Obstacle Bypass, Easy", Family::Bypass, "Point12", r(E, "TABLE L-XVI", 576)).points(3, 3),
];
