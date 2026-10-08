//! Version 16 graphics drawn by the ported renderer whose declaration
//! differs from what upstream's catalog gives for their code.

use super::{MANY, list, opt, ported, req};
use crate::modifier::ModifierField as M;
use crate::standard::StandardVersion::App6Ech2 as E6;
use crate::support::SymbolSpec;

pub(super) static SPECS: &[SymbolSpec] = &[
    // Mobility Corridor: two or more points; B, the size of force the
    // corridor can take, is mandatory.
    ported(E6, 25, 142100)
        .points(2, MANY)
        .amplifiers(&[req(M::B), opt(M::H)]),
    // Restricted and Severely Restricted Terrain: H, the cause of the
    // restriction, must be displayed.
    ported(E6, 25, 152400)
        .points(3, MANY)
        .amplifiers(&[req(M::H)]),
    ported(E6, 25, 152500)
        .points(3, MANY)
        .amplifiers(&[req(M::H)]),
    // Navigational Rhumb Line: two points and one course.
    ported(E6, 25, 220109)
        .points(2, 2)
        .amplifiers(&[list(M::AN, false, 1, 1), opt(M::T)]),
    // Rectangular Target – Single Target: upstream's catalog has no
    // version 16 row; declared as its 2525D change 1 code is.
    ported(E6, 25, 240804)
        .points(2, 2)
        .amplifiers(&[list(M::AM, true, 1, MANY), opt(M::T)]),
];
