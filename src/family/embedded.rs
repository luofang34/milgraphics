//! Single-point symbols a graphic embeds: the unit a task is assigned to
//! (amplifier `A`), placed where the standard's template puts it, the event
//! icon of a contaminated area, and the anchor of an anchorage.

use crate::construction::{EmbeddedSymbol, SymbolSize};
use crate::definition::GraphicDefinition;
use crate::family::{ConstructError, Ctx};
use crate::geo::GeoPoint;
use crate::modifier::ModifierField;
use crate::plane::{LocalPlane, Xy};
use crate::sidc::SymbolId;

/// Size of an embedded symbol not fitted to a circle, in pixels.
const SYMBOL_PX: f64 = 30.0;

/// Where a symbol goes, by control point index.
#[derive(Clone, Copy)]
enum Anchor {
    Point(usize),
    Between(usize, usize),
    /// The second-to-last point: the rear of an axis.
    Rear,
    /// The centre of the area the points outline.
    Centre,
    /// Halfway along the line through the points.
    Middle,
}

#[derive(Clone, Copy)]
enum Size {
    Fixed,
    /// Within the circle through this point around the anchor.
    Circle(usize),
}

/// Where the unit symbol of amplifier `A` goes, per entity, as the templates
/// show it; whether a symbol carries `A` is declared per edition.
const UNIT: &[(u32, Anchor, Size, [f64; 2])] = &[
    // Below the area's label.
    (151_600, Anchor::Centre, Size::Fixed, [0.0, SYMBOL_PX]),
    // Over point 1.
    (152_200, Anchor::Point(0), Size::Fixed, [0.0, 0.0]),
    // Under the apex, between the arms' ends.
    (230_100, Anchor::Between(1, 2), Size::Fixed, [0.0, 0.0]),
    // Cover, Guard, Screen: centred between points 2 and 3.
    (342_201, Anchor::Between(1, 2), Size::Fixed, [0.0, 0.0]),
    (342_202, Anchor::Between(1, 2), Size::Fixed, [0.0, 0.0]),
    (342_203, Anchor::Between(1, 2), Size::Fixed, [0.0, 0.0]),
    // Seize, Capture, Evacuate, Recover: in the circle around point 1.
    (342_300, Anchor::Point(0), Size::Circle(1), [0.0, 0.0]),
    (343_000, Anchor::Point(0), Size::Circle(1), [0.0, 0.0]),
    (344_500, Anchor::Point(0), Size::Circle(1), [0.0, 0.0]),
    (344_600, Anchor::Point(0), Size::Circle(1), [0.0, 0.0]),
    // Movement (Advance) to Contact: at the rear of the axis.
    (342_900, Anchor::Rear, Size::Fixed, [0.0, 0.0]),
    // Escort: at the centre point.
    (343_600, Anchor::Point(0), Size::Fixed, [0.0, 0.0]),
];

/// Contaminated areas and the event symbol drawn in them.
const EVENTS: &[(u32, u32)] = &[
    (271_700, 281_400),
    (271_701, 281_401),
    (271_800, 281_300),
    (271_801, 281_301),
    (271_900, 281_500),
    (272_000, 281_700),
    (272_001, 281_701),
];

/// Editions whose templates put the event symbol in the area.
const EVENT_VERSIONS: [u8; 2] = [15, 16];

/// METOC graphics drawn with a symbol of their own set, per edition: the
/// Anchorage - Point symbol (46 120304) halfway along an Anchorage - Line and
/// in an Anchorage - Area (MIL-STD-2525E change 1, TABLE M-III).
const METOC: &[(u8, u8, u32, u32, Anchor)] = &[
    (15, 46, 120_305, 120_304, Anchor::Middle),
    (15, 46, 120_306, 120_304, Anchor::Centre),
];

pub(crate) fn add(ctx: &mut Ctx<'_>, def: &GraphicDefinition) -> Result<(), ConstructError> {
    let points: Vec<GeoPoint> = def.positions().collect();
    let symbol = &def.symbol;
    let metoc = METOC.iter().find(|m| {
        (m.0, m.1, m.2)
            == (
                symbol.version_code(),
                symbol.symbol_set(),
                symbol.entity().get(),
            )
    });
    if let Some(&(_, set, _, icon, anchor)) = metoc {
        let placed = SymbolId::parse(&same_kind(symbol, set, icon))
            .ok()
            .and_then(|icon| place(ctx, &points, icon, anchor, Size::Fixed, [0.0, 0.0]));
        if let Some(placed) = placed {
            ctx.add_symbol(placed);
        }
    }
    if symbol.symbol_set() != 25 {
        return Ok(());
    }
    let entity = symbol.entity().get();
    if let (Some(icon), Some(_)) = (
        &def.modifiers.symbol_icon,
        ctx.spec.modifier(ModifierField::A),
    ) {
        let unit = SymbolId::parse(icon).map_err(|source| ConstructError::InvalidSymbolIcon {
            symbol: ctx.spec.name(),
            source,
        })?;
        if let Some(&(_, anchor, size, offset)) = UNIT.iter().find(|r| r.0 == entity) {
            if let Some(placed) = place(ctx, &points, unit, anchor, size, offset) {
                ctx.add_symbol(placed);
            }
        }
    }
    let event = EVENTS.iter().find(|e| e.0 == entity).map(|e| e.1);
    if let (Some(event), true) = (event, EVENT_VERSIONS.contains(&symbol.version_code())) {
        let placed = SymbolId::parse(&same_kind(symbol, 25, event))
            .ok()
            .and_then(|icon| place(ctx, &points, icon, Anchor::Centre, Size::Fixed, [0.0, 0.0]));
        if let Some(placed) = placed {
            ctx.add_symbol(placed);
        }
    }
    Ok(())
}

/// The code of `entity` in `set`, with the edition, context, identity and
/// status of `symbol`.
fn same_kind(symbol: &SymbolId, set: u8, entity: u32) -> String {
    format!(
        "{:02}{}{}{:02}{}000{:06}0000",
        symbol.version_code(),
        symbol.context(),
        symbol.identity(),
        set,
        symbol.status(),
        entity
    )
}

fn place(
    ctx: &Ctx<'_>,
    points: &[GeoPoint],
    symbol: SymbolId,
    anchor: Anchor,
    size: Size,
    offset_px: [f64; 2],
) -> Option<EmbeddedSymbol> {
    let at = |i: usize| points.get(i).copied();
    let anchor = match anchor {
        Anchor::Point(i) => at(i)?,
        Anchor::Between(i, j) => ctx.earth.interpolate(at(i)?, at(j)?, 0.5),
        Anchor::Rear => at(points.len().checked_sub(2)?)?,
        Anchor::Centre => centre(ctx, points)?,
        Anchor::Middle => middle(ctx, points)?,
    };
    let size = match size {
        Size::Fixed => SymbolSize::Pixels(SYMBOL_PX),
        Size::Circle(i) => SymbolSize::WithinCircle { edge: at(i)? },
    };
    Some(EmbeddedSymbol {
        symbol,
        anchor,
        offset_px,
        size,
    })
}

/// The mean of the points in a plane around the first, which holds across
/// the antimeridian.
fn centre(ctx: &Ctx<'_>, points: &[GeoPoint]) -> Option<GeoPoint> {
    let origin = *points.first()?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let n = points.len() as f64;
    let sum = points
        .iter()
        .map(|&p| plane.to_xy(p))
        .fold(Xy::new(0.0, 0.0), |a, p| Xy::new(a.x + p.x, a.y + p.y));
    Some(plane.to_geo(Xy::new(sum.x / n, sum.y / n)))
}

/// The point halfway along the line through `points`, measured in a plane
/// around the first.
fn middle(ctx: &Ctx<'_>, points: &[GeoPoint]) -> Option<GeoPoint> {
    let origin = *points.first()?;
    let plane = LocalPlane::new(ctx.earth, origin);
    let xy: Vec<Xy> = points.iter().map(|&p| plane.to_xy(p)).collect();
    let total: f64 = xy.windows(2).map(|w| segment(w).len()).sum();
    let mut left = total / 2.0;
    for w in xy.windows(2) {
        let (Some(&a), d) = (w.first(), segment(w)) else {
            continue;
        };
        let len = d.len();
        if len > 0.0 && left <= len {
            let t = left / len;
            return Some(plane.to_geo(Xy::new(a.x + d.x * t, a.y + d.y * t)));
        }
        left -= len;
    }
    points.last().copied()
}

/// The vector along a two-point window.
fn segment(w: &[Xy]) -> Xy {
    match w {
        [a, b] => b.sub(*a),
        _ => Xy::new(0.0, 0.0),
    }
}

#[cfg(test)]
mod tests;
