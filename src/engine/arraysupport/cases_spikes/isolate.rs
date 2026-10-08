//! Port of `GetIsolatePointsDouble` (pixel path): the arc with an arrowhead
//! that ISOLATE, CORDON, DENY, AREA DEFENSE, OCCUPY, CONTROL, LOCATE, SECURE,
//! RETAIN and TURN share, and the decoration each adds.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::lineutility::arc::{arc_array_double, calc_clockwise_center_double};
use crate::engine::lineutility::arrow::get_arrow_head4_double;
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::bounds::mbr_distance;
use crate::engine::lineutility::extend::{extend_along_line_double, extend_line_double};
use crate::engine::tactical_lines as lt;

use crate::engine::arraysupport::work::{MAX_LENGTH, MIN_LENGTH, set, set_style};

/// The values every decoration reads.
struct Arc {
    pt0: Pt,
    pt1: Pt,
    pt2: Pt,
    radius: f64,
    length: f64,
    d: f64,
    arc: [Pt; 26],
    arrow: [Pt; 3],
}

/// Writes `src` into `p` from `at` with the given style.
fn put_all(p: &mut [Pt], at: i32, src: &[Pt], style: Option<i32>) -> Result<(), EngineError> {
    for (i, s) in src.iter().enumerate() {
        let i = i32::try_from(i).map_err(|_| EngineError::Degenerate("index"))?;
        let mut s = *s;
        if let Some(st) = style {
            s.style = st;
        }
        set(p, at + i, s)?;
    }
    Ok(())
}

/// A triangle point: copy of `p` with the arc's style (0) or `style`.
fn tri(p: Pt, style: i32) -> Pt {
    Pt { style, ..p }
}

/// Upstream `IsTurnArcReversed`.
fn is_turn_arc_reversed(pp: [Pt; 3]) -> Result<bool, EngineError> {
    let mut seize = [pp.at(0)?, pp.at(1)?];
    calc_clockwise_center_double(&mut seize)?;
    let d = calc_distance_double(seize.at(0)?, pp.at(2)?);
    let mut seize = [pp.at(1)?, pp.at(0)?];
    calc_clockwise_center_double(&mut seize)?;
    let reversed = calc_distance_double(seize.at(0)?, pp.at(2)?);
    Ok(reversed > d)
}

/// Upstream `GetIsolatePointsDouble` for no converter: rewrites `p` (at
/// least 26 slots plus the decoration's) for the given line type.
pub(super) fn isolate_points(p: &mut [Pt], line_type: i32, dpi: f64) -> Result<(), EngineError> {
    let p0 = p.at(0)?;
    let p1 = p.at(1)?;
    let mut pt1 = p1;
    let pt0 = p0;
    if pt0.x == pt1.x && pt0.y == pt1.y {
        pt1.x += 1.0;
    }
    let radius = calc_distance_double(pt0, pt1);
    let mut length = (radius - 20.0).abs();
    if radius < 40.0 {
        length = radius / 1.5;
    }
    let mut d = mbr_distance(p, 2)?;
    let saved = [
        p0,
        p1,
        if p.len() >= 3 {
            p.at(2)?
        } else {
            Pt::default()
        },
    ];
    if d / 7.0 > MAX_LENGTH * dpi {
        d = 7.0 * MAX_LENGTH * dpi;
    }
    if d / 7.0 < MIN_LENGTH * dpi {
        d = 7.0 * MIN_LENGTH * dpi;
    }
    if d > 140.0 * dpi {
        d = 140.0 * dpi;
    }
    let e = Pt::new(2.0 * pt1.x - pt0.x, 2.0 * pt1.y - pt0.y);
    let mut arc = [Pt::default(); 26];
    *arc.at_mut(0)? = p1;
    *arc.at_mut(1)? = e;
    arc_array_double(&mut arc, radius, line_type)?;
    for (j, a) in arc.iter_mut().enumerate() {
        a.style = 0;
        set(p, i32::try_from(j).unwrap_or(0), *a)?;
    }
    let mut arrow = [Pt::default(); 3];
    let base = if line_type == lt::OCCUPY {
        (1.75 * d) as i32 / 7
    } else {
        d as i32 / 7
    };
    get_arrow_head4_double(arc.at(24)?, arc.at(25)?, d as i32 / 7, base, &mut arrow, 0)?;
    let mut arrow2 = [Pt::default(); 3];
    if line_type == lt::CONTROL || line_type == lt::LOCATE {
        get_arrow_head4_double(
            arc.at(1)?,
            arc.at(0)?,
            d as i32 / 7,
            d as i32 / 7,
            &mut arrow2,
            0,
        )?;
    }
    set_style(p, 25, 5)?;
    let a = Arc {
        pt0,
        pt1,
        pt2: p0,
        radius,
        length,
        d,
        arc,
        arrow,
    };
    decorate(p, a, (&arrow2, saved), line_type)
}

/// Adds the decoration of the line type.
fn decorate(
    p: &mut [Pt],
    a: Arc,
    extra: (&[Pt; 3], [Pt; 3]),
    line_type: i32,
) -> Result<(), EngineError> {
    let (arrow2, saved) = extra;
    match line_type {
        lt::CORDONKNOCK | lt::CORDONSEARCH | lt::ISOLATE => cordon(p, a),
        lt::DENY => deny(p, a),
        lt::AREA_DEFENSE => area_defense(p, a),
        lt::OCCUPY => occupy(p, &a),
        lt::SECURE => {
            put_all(p, 26, &a.arrow, Some(0))?;
            set_style(p, 28, 5)
        }
        lt::CONTROL | lt::LOCATE => {
            put_all(p, 26, &a.arrow, Some(0))?;
            set_style(p, 28, 5)?;
            put_all(p, 29, arrow2, Some(0))?;
            set_style(p, 31, 5)
        }
        lt::TURN_REVD | lt::TURN => turn(p, a, saved, line_type),
        lt::RETAIN => retain(p, &a),
        _ => Ok(()),
    }
}

/// Java `(long) x` as a float.
fn trunc(x: f64) -> f64 {
    x as i64 as f64
}

/// The triangle points of the cordon types and the arrowhead.
fn cordon(p: &mut [Pt], a: Arc) -> Result<(), EngineError> {
    let mut length = a.length;
    if a.radius > 100.0 {
        length = 0.8 * a.radius;
    }
    let mut tris = Vec::new();
    for j in (3..=23).step_by(3) {
        let m = Pt::new(
            a.pt0.x - trunc((length / a.radius) * (a.pt0.x - a.arc.at(j)?.x)),
            a.pt0.y - trunc((length / a.radius) * (a.pt0.y - a.arc.at(j)?.y)),
        );
        tris.push(a.arc.at(j - 1)?);
        tris.push(m);
        tris.push(tri(a.arc.at(j + 1)?, 5));
    }
    put_all(p, 26, &tris, None)?;
    set_style(p, 46, 5)?;
    put_all(p, 47, &a.arrow, Some(0))
}

/// DENY: triangles pointing outward to 120 % of the radius.
fn deny(p: &mut [Pt], a: Arc) -> Result<(), EngineError> {
    let mut tris = Vec::new();
    for j in (3..=23).step_by(3) {
        let (start, end) = (a.arc.at(j - 1)?, a.arc.at(j + 1)?);
        let mid_x = (start.x + end.x) / 2.0;
        let mid_y = (start.y + end.y) / 2.0;
        let mut dx = mid_x - a.pt0.x;
        let mut dy = mid_y - a.pt0.y;
        let mut distance = (dx * dx + dy * dy).sqrt();
        let target = a.radius * 1.20;
        if distance < 0.00001 {
            dx = start.x - a.pt0.x;
            dy = start.y - a.pt0.y;
            distance = (dx * dx + dy * dy).sqrt();
        }
        let m = Pt::new(
            a.pt0.x + (dx / distance) * target,
            a.pt0.y + (dy / distance) * target,
        );
        tris.push(start);
        tris.push(m);
        tris.push(tri(end, 5));
    }
    put_all(p, 26, &tris, None)?;
    set_style(p, 46, 5)?;
    put_all(p, 47, &a.arrow, Some(0))
}

/// AREA DEFENSE: filled triangles on the arc and a second arrowhead.
fn area_defense(p: &mut [Pt], mut a: Arc) -> Result<(), EngineError> {
    let mut length = a.length;
    if a.radius > 100.0 {
        length = 0.8 * a.radius;
    }
    let mut tris = Vec::new();
    for j in (3..=23).step_by(3) {
        let m = Pt::new(
            a.pt0.x - trunc((a.radius / length) * (a.pt0.x - a.arc.at(j)?.x)),
            a.pt0.y - trunc((a.radius / length) * (a.pt0.y - a.arc.at(j)?.y)),
        );
        tris.push(tri(a.arc.at(j - 1)?, 9));
        tris.push(tri(m, 9));
        tris.push(tri(a.arc.at(j + 1)?, 9));
        tris.push(tri(a.arc.at(j)?, 9));
        tris.push(tri(a.arc.at(j - 1)?, 10));
    }
    put_all(p, 26, &tris, None)?;
    put_all(p, 61, &a.arrow, Some(0))?;
    let step = a.d as i32 / 7;
    get_arrow_head4_double(a.arc.at(1)?, a.arc.at(0)?, step, step, &mut a.arrow, 0)?;
    set_style(p, 63, 5)?;
    put_all(p, 64, &a.arrow, Some(0))
}

/// OCCUPY: an open arrowhead with extended barbs.
fn occupy(p: &mut [Pt], a: &Arc) -> Result<(), EngineError> {
    let [a0, a1, a2] = a.arrow;
    put_all(p, 26, &a.arrow, None)?;
    let long0 = calc_distance_double(a0, a1) * 2.0;
    set(p, 29, extend_along_line_double(a0, a1, long0))?;
    set(p, 30, a1)?;
    let long2 = calc_distance_double(a2, a1) * 2.0;
    set(p, 31, extend_along_line_double(a2, a1, long2))
}

/// RETAIN: arrowhead and a radial tick at every arc point.
fn retain(p: &mut [Pt], a: &Arc) -> Result<(), EngineError> {
    put_all(p, 26, &a.arrow, Some(0))?;
    set_style(p, 28, 5)?;
    let mut k = 29;
    let tick = ((a.d as i64) / 7) as f64;
    for j in 1..24 {
        set(p, k, tri(a.arc.at(j)?, 0))?;
        k += 1;
        set(p, k, tri(extend_line_double(a.pt0, a.arc.at(j)?, tick), 5))?;
        k += 1;
    }
    Ok(())
}

/// TURN: an arc around the clockwise centre with a filled arrowhead.
fn turn(p: &mut [Pt], a: Arc, saved: [Pt; 3], line_type: i32) -> Result<(), EngineError> {
    let change = is_turn_arc_reversed(saved)?;
    let (mut pt0, mut pt1) = (a.pt0, a.pt1);
    if change {
        pt0.x = pt1.x;
        pt0.y = pt1.y;
        pt1.x = a.pt2.x;
        pt1.y = a.pt2.y;
    }
    let mut seize = [pt0, pt1];
    let radius = calc_clockwise_center_double(&mut seize)?;
    let mut arc = [Pt::default(); 26];
    *arc.at_mut(0)? = pt0;
    *arc.at_mut(1)? = seize.at(1)?;
    arc_array_double(&mut arc, radius, line_type)?;
    for (j, ar) in arc.iter_mut().enumerate() {
        ar.style = 0;
        set(p, i32::try_from(j).unwrap_or(0), *ar)?;
    }
    let step = a.d as i32 / 7;
    let mut arrow = [Pt::default(); 3];
    if change {
        get_arrow_head4_double(arc.at(1)?, pt0, step, step, &mut arrow, 5)?;
    } else {
        get_arrow_head4_double(arc.at(24)?, pt1, step, step, &mut arrow, 5)?;
    }
    set_style(p, 25, 5)?;
    put_all(p, 26, &arrow, Some(9))?;
    set_style(p, 28, 10)
}
