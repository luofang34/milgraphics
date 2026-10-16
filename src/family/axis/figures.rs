//! The figures some axes of advance add to the arrow, sized from the body's
//! half-width so they scale with it on the ground. Proportions follow the
//! standard's templates as mil-sym-java draws them (`clsChannelUtility`).

use crate::plane::{Xy, intersect};

/// Growth of the feint chevron over the arrowhead, about the centre of the
/// arrowhead's base.
const FEINT_SCALE: f64 = 1.5;
/// Rotor stem half-length, in body half-widths.
const ROTOR_STEM: f64 = 2.0;
/// Rotor arrowhead: depth across the stem and half-width along the axis.
const ROTOR_HEAD_DEPTH: f64 = 1.0;
const ROTOR_HEAD_HALF: f64 = 0.5;
/// Rotor ticks: distance along the axis from the stem, and half-length.
const ROTOR_TICK_AT: f64 = 1.3;
const ROTOR_TICK_HALF: f64 = 0.4;

/// A dashed chevron ahead of the arrowhead: the head's outline grown about
/// the centre of its base.
pub(super) fn feint(tip: Xy, base: Xy, barb_l: Xy, barb_u: Xy) -> Vec<Xy> {
    let grow = |p: Xy| base.add(p.sub(base).scale(FEINT_SCALE));
    vec![grow(barb_l), grow(tip), grow(barb_u)]
}

/// The attack-helicopter rotor at the crossing of the two sides: a stem
/// across the axis with an arrowhead on the left of travel and a bar on the
/// right, and a tick either side of the stem. `centre` is the body
/// centreline from the tail to the arrowhead base.
pub(super) fn rotor(lower: &[Xy], upper: &[Xy], centre: &[Xy], half: f64) -> Option<Vec<Vec<Xy>>> {
    let (&base, rest) = centre.split_last()?;
    let &before = rest.last()?;
    let travel = base.sub(before).unit()?;
    let left = travel.left();
    let last_edge = |side: &[Xy]| -> Option<(Xy, Xy)> {
        let (&end, rest) = side.split_last()?;
        Some((*rest.last()?, end))
    };
    let ((a0, a1), (b0, b1)) = (last_edge(lower)?, last_edge(upper)?);
    // The sides' crossing; their midpoint where rounding leaves them parallel.
    let at =
        intersect(a0, a1.sub(a0), b0, b1.sub(b0)).unwrap_or_else(|| before.add(base).scale(0.5));
    let along = |k: f64| travel.scale(k * half);
    let across = |k: f64| left.scale(k * half);
    let top = at.add(across(ROTOR_STEM));
    let bottom = at.sub(across(ROTOR_STEM));
    let shoulder = at.add(across(ROTOR_STEM - ROTOR_HEAD_DEPTH));
    let tick = |k: f64| {
        let c = at.add(along(k));
        vec![
            c.sub(across(ROTOR_TICK_HALF)),
            c.add(across(ROTOR_TICK_HALF)),
        ]
    };
    Some(vec![
        vec![bottom, top],
        vec![
            shoulder.sub(along(ROTOR_HEAD_HALF)),
            top,
            shoulder.add(along(ROTOR_HEAD_HALF)),
        ],
        vec![
            bottom.sub(along(ROTOR_HEAD_HALF)),
            bottom.add(along(ROTOR_HEAD_HALF)),
        ],
        tick(-ROTOR_TICK_AT),
        tick(ROTOR_TICK_AT),
    ])
}
