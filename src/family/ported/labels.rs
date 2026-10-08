//! Amplifiers the MIL-STD-2525E change 1 notes place along METOC lines that
//! upstream draws without text: the value of an isopleth "at each end of the
//! line and once in the middle" (TABLE M-II), and of a depth curve or
//! contour "at each end of the line and at regular intervals" (TABLE
//! M-III), placed here at the ends and the middle.

use crate::construction::{LabelPlacement, LabelSpec, PartId};
use crate::definition::GraphicDefinition;
use crate::family::Ctx;
use crate::geo::GeoPoint;
use crate::modifier::ModifierField;
use crate::plane::{LocalPlane, Xy};

/// `(version, symbol set, entity, field)` of the labelled lines; the field
/// is declared for the symbol in the same edition.
const LABELLED: &[(u8, u8, u32, ModifierField)] = &[
    (15, 45, 180_100, ModifierField::T),
    (15, 45, 180_200, ModifierField::T),
    (15, 45, 180_300, ModifierField::T),
    (15, 45, 180_400, ModifierField::T),
    (15, 45, 180_500, ModifierField::T),
    (15, 45, 180_600, ModifierField::T),
    (15, 45, 180_700, ModifierField::T),
    (15, 46, 120_102, ModifierField::T),
    (15, 46, 120_103, ModifierField::T),
];

/// Labels `line`, the drawn line of `def`, with its value.
pub(super) fn add(ctx: &mut Ctx<'_>, def: &GraphicDefinition, part: PartId, line: &[GeoPoint]) {
    let s = &def.symbol;
    let Some(&(_, _, _, field)) = LABELLED
        .iter()
        .find(|l| (l.0, l.1, l.2) == (s.version_code(), s.symbol_set(), s.entity().get()))
    else {
        return;
    };
    let text = match field {
        ModifierField::T => def.modifiers.designation().map(str::to_owned),
        _ => None,
    };
    let (Some(text), Some(spots)) = (text, spots(ctx, line)) else {
        return;
    };
    for (anchor, placement) in spots {
        ctx.add_label(LabelSpec {
            part,
            text: text.clone(),
            anchor,
            placement,
            line_offset: 0.0,
            may_hide: false,
        });
    }
}

/// Each end, with the text running outward, and the middle, along the line.
fn spots(ctx: &Ctx<'_>, line: &[GeoPoint]) -> Option<[(GeoPoint, LabelPlacement); 3]> {
    let (&start, &end) = (line.first()?, line.last()?);
    let after = *line.iter().find(|&&p| p != start)?;
    let before = *line.iter().rev().find(|&&p| p != end)?;
    let (middle, toward) = middle(ctx, line)?;
    Some([
        (start, LabelPlacement::LineEnd { inward: after }),
        (middle, LabelPlacement::Along { toward }),
        (end, LabelPlacement::LineEnd { inward: before }),
    ])
}

/// The point halfway along `line` and the end of the segment it lies on,
/// measured in a plane around the first point.
fn middle(ctx: &Ctx<'_>, line: &[GeoPoint]) -> Option<(GeoPoint, GeoPoint)> {
    let plane = LocalPlane::new(ctx.earth, *line.first()?);
    let xy: Vec<Xy> = line.iter().map(|&p| plane.to_xy(p)).collect();
    let pairs = || xy.iter().zip(xy.iter().skip(1));
    let mut left = pairs().map(|(a, b)| b.sub(*a).len()).sum::<f64>() / 2.0;
    for (a, b) in pairs() {
        let len = b.sub(*a).len();
        if len > 0.0 && left <= len {
            let t = left / len;
            let at = Xy::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
            return Some((plane.to_geo(at), plane.to_geo(*b)));
        }
        left -= len;
    }
    None
}
