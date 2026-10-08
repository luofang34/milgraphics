//! OVERHEAD_WIRE: towers with cross pieces joined by wires.

use super::super::work::{get, scaled_size, set};
use crate::engine::base::{EngineError, Pt};
use crate::engine::tg::Tg;

/// One tower point: offsets from the pole foot in unscaled pixels and the
/// style it ends the stroke with.
const TOWER: [(f64, f64, Option<i32>); 13] = [
    (0.0, -5.0, None),
    (-5.0, 0.0, None),
    (0.0, -20.0, None),
    (5.0, 0.0, None),
    (0.0, -5.0, Some(5)),
    (-2.0, -10.0, None),
    (2.0, -10.0, Some(5)),
    (-7.0, -17.0, None),
    (-5.0, -20.0, None),
    (5.0, -20.0, None),
    (7.0, -17.0, Some(5)),
    (0.0, -20.0, None),
    (8.0, -12.0, Some(5)),
];

/// Upstream `getOverheadWire`: returns the number of points written.
pub(super) fn get_overhead_wire(tg: &Tg, p: &mut [Pt], vbl: i32) -> Result<i32, EngineError> {
    let s = |size: f64| scaled_size(tg, size);
    let offset = |v: f64| {
        if v < 0.0 {
            -s(-v)
        } else if v > 0.0 {
            s(v)
        } else {
            0.0
        }
    };
    let mut pts: Vec<Pt> = Vec::new();
    for j in 0..vbl {
        let pt = get(p, j)?;
        for (dx, dy, style) in TOWER {
            let mut q = pt;
            q.x += offset(dx);
            q.y += offset(dy);
            if let Some(style) = style {
                q.style = style;
            }
            pts.push(q);
        }
    }
    for j in 0..vbl - 1 {
        let mut pt = get(p, j)?;
        let mut pt2 = get(p, j + 1)?;
        if pt.x < pt2.x {
            pt.x += s(5.0);
            pt2.x -= s(5.0);
        } else {
            pt.x -= s(5.0);
            pt2.x += s(5.0);
        }
        pt.y -= s(10.0);
        pt2.y -= s(10.0);
        pt2.style = 5;
        pts.push(pt);
        pts.push(pt2);
    }
    let mut counter = 0;
    for q in &pts {
        set(p, counter, *q)?;
        counter += 1;
    }
    let pad = get(p, counter - 1)?;
    for j in counter..i32::try_from(p.len()).unwrap_or(0) {
        set(p, j, pad)?;
    }
    Ok(counter)
}
