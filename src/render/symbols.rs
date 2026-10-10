//! Embedded single-point symbols, placed for a view.

use crate::construction::{Construction, SymbolSize};
use crate::geo::GeoPoint;
use crate::pick::{PickRef, PickTarget};
use crate::render::ScreenPoint;
use crate::render::screen::ScreenCtx;
use crate::sidc::SymbolId;

/// Share of a circle's diameter a symbol drawn inside it takes.
const WITHIN_CIRCLE: f64 = 0.7;

/// A single-point symbol for the host to draw over the graphic, for example
/// with milsymbol.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SymbolPlacement {
    /// What it is.
    pub pick: PickRef,
    /// The symbol to draw.
    pub symbol: SymbolId,
    /// Where its centre is on the ground.
    pub anchor: GeoPoint,
    /// Where its centre is on screen, if the anchor is visible: the
    /// anchor's projection moved by `offset_px`.
    pub screen: Option<ScreenPoint>,
    /// Pixels from the anchor's screen position to the symbol's centre,
    /// y downward, for engines that place icons at `anchor` themselves.
    pub offset_px: [f64; 2],
    /// Its size in pixels (the height a single-point renderer is asked for).
    pub size_px: f64,
    /// Rotation in degrees clockwise on screen; 0 draws it upright.
    pub rotation_deg: f64,
}

pub(crate) fn resolve(
    ctx: &mut ScreenCtx<'_>,
    construction: &Construction,
    pick: &impl Fn(PickTarget) -> PickRef,
) -> Vec<SymbolPlacement> {
    construction
        .symbols
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let centre = ctx.project(s.anchor);
            let size_px = match s.size {
                SymbolSize::Pixels(px) => px,
                SymbolSize::WithinCircle { edge } => match (centre, ctx.project(edge)) {
                    (Some(c), Some(e)) => {
                        let (dx, dy) = e.sub(c);
                        2.0 * dx.hypot(dy) * WITHIN_CIRCLE
                    }
                    _ => 0.0,
                },
            };
            SymbolPlacement {
                pick: pick(PickTarget::Symbol(u32::try_from(i).unwrap_or(u32::MAX))),
                symbol: s.symbol.clone(),
                anchor: s.anchor,
                screen: centre.map(|c| ScreenPoint {
                    x: c.x + s.offset_px[0],
                    y: c.y + s.offset_px[1],
                }),
                offset_px: s.offset_px,
                size_px,
                rotation_deg: 0.0,
            }
        })
        .collect()
}
