//! Axes of advance, in every edition that defines them. Each is drawn by the
//! axis family, so its outline is sized on the ground and draped by the map.

use super::{cm, opt};
use crate::family::{AxisKind, Family};
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::{
    self, App6D as D6, App6Ech2 as E6, Mil2525Dch1 as D1, Mil2525Ech1 as E1,
};
use crate::support::SymbolSpec;

const AMPLIFIERS: &[crate::support::ModifierSpec] = &[opt(M::T), opt(M::W), opt(M::W1)];

const fn axis(standard: StandardVersion, entity: u32, kind: AxisKind) -> SymbolSpec {
    cm(standard, entity, Family::Axis(kind))
        .points(3, 50)
        .amplifiers(AMPLIFIERS)
}

pub(super) static SPECS: &[SymbolSpec] = &[
    axis(D6, 151401, AxisKind::Airborne),
    axis(D1, 151401, AxisKind::Airborne),
    axis(E1, 151401, AxisKind::Airborne),
    axis(E6, 151401, AxisKind::Airborne),
    axis(D6, 151402, AxisKind::AttackHelicopter),
    axis(D1, 151402, AxisKind::AttackHelicopter),
    axis(E1, 151402, AxisKind::AttackHelicopter),
    axis(E6, 151402, AxisKind::AttackHelicopter),
    axis(D6, 151403, AxisKind::Main),
    axis(D1, 151403, AxisKind::Main),
    axis(E1, 151403, AxisKind::Main),
    axis(E6, 151403, AxisKind::Main),
    axis(D6, 151404, AxisKind::Supporting),
    axis(D1, 151404, AxisKind::Supporting),
    axis(E1, 151404, AxisKind::Supporting),
    axis(E6, 151404, AxisKind::Supporting),
    axis(D6, 151406, AxisKind::Feint),
    axis(E6, 151406, AxisKind::Feint),
];
