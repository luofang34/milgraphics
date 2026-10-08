//! ESCORT, the feint/decoy/dummy indicator shapes and AMBUSH from
//! DISMSupport.java.

use super::support::{MAX_LENGTH, MIN_LENGTH, arc_approximation_double, clamp_size, put};
use crate::engine::base::{At, EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::EXTEND_ABOVE;
use crate::engine::lineutility::basics::{calc_distance_double, mid_point_double};
use crate::engine::lineutility::extend::{
    extend_along_line_double, extend_directed_line, extend_line_double,
};
use crate::engine::lineutility::relative::find_closest_point_on_line;
use crate::engine::settings::Settings;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::shape_properties::get_line_stroke;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

/// Upstream `GetDISMEscortDouble`: ESCORT, 6 points. The line is offset to
/// the third control point and broken around the icon when it is long
/// enough to leave room for it.
pub(crate) fn get_dism_escort_double(tg: &Tg, points: &mut Vec<Pt>) -> Result<i32, EngineError> {
    let pt0 = points.at(1)?;
    let pt5 = points.at(2)?;
    let offset = points.at(0)?;
    let intersect = find_closest_point_on_line(pt0, pt5, offset);
    let x_offset = offset.x - intersect.x;
    let y_offset = offset.y - intersect.y;
    let pt1 = Pt::new(pt0.x + x_offset, pt0.y + y_offset);
    let pt4 = Pt::new(pt5.x + x_offset, pt5.y + y_offset);
    let pixel_size = tg.icon_size;
    let font_size = tg.font.size;
    let distance = calc_distance_double(pt1, pt4);
    let center = mid_point_double(pt1, pt4, 0);
    let gap = f64::from(pixel_size as f32 / 2.0 + font_size as f32);
    let collapsed = f64::from(font_size * 2 + pixel_size) >= distance;
    let (pt2, pt3) = if collapsed {
        // Upstream shares one point object here, so its last style (0) wins.
        (styled(center, 0), styled(center, 0))
    } else {
        (
            styled(extend_along_line_double(center, pt1, gap), 5),
            styled(extend_along_line_double(center, pt4, gap), 0),
        )
    };
    put(points, 0, styled(pt0, 0));
    put(points, 1, styled(pt1, 1));
    put(points, 2, pt2);
    put(points, 3, pt3);
    put(points, 4, styled(pt4, 1));
    put(points, 5, styled(pt5, 1));
    Ok(6)
}

fn fdi_shape(tg: &Tg, a: Pt, b: Pt, c: Pt) -> Shape {
    let mut shape = Shape::new(shape_type::POLYLINE);
    shape.move_to(a);
    shape.line_to(b);
    shape.line_to(c);
    shape.style = 1;
    shape.line_color = tg.line_color;
    shape.stroke = get_line_stroke(tg.line_thickness, shape.style);
    shape
}

/// Upstream `getFDIShape(tg, ptA, ptC)`: the feint, decoy or dummy
/// indicator, a triangle apex over the chord from the bottom-left `a` to the
/// bottom-right `c`. It does not check whether the graphic should have one.
pub(crate) fn get_fdi_shape(tg: &Tg, a: Pt, c: Pt) -> Shape {
    let mid = mid_point_double(a, c, 0);
    let len = calc_distance_double(a, mid);
    let b = extend_directed_line(a, c, mid, EXTEND_ABOVE, len);
    fdi_shape(tg, a, b, c)
}

/// Upstream `getFDIShape(tg, ptA, ptB, ptC)`: the indicator for symbols with
/// an arrowhead, pushed out from the arrow by at least 1.5 line widths.
pub(crate) fn get_fdi_shape_arrow(tg: &Tg, a: Pt, b: Pt, c: Pt) -> Shape {
    let min_gap = f64::from(tg.line_thickness) * 1.5;
    let mut w = calc_distance_double(a, c) * 0.25;
    if w < min_gap {
        w = min_gap;
    }
    let c = extend_line_double(a, c, w);
    let a = extend_line_double(c, a, w);
    let mid = mid_point_double(a, c, 0);
    let mut w = calc_distance_double(mid, b) * 0.5;
    if w < min_gap {
        w = min_gap;
    }
    let b = extend_line_double(mid, b, w);
    fdi_shape(tg, a, b, c)
}

/// Upstream `AmbushPointsDouble`: AMBUSH, an arc with an arrow out of its
/// middle and tick marks toward the back; 17 + 2 + 3 + 16 points.
pub(crate) fn ambush_points_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let mut sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    let mid = Pt::new((sp[1].x + sp[2].x) / 2.0, (sp[1].y + sp[2].y) / 2.0);
    let radius =
        ((mid.x - sp[2].x) * (mid.x - sp[2].x) + (mid.y - sp[2].y) * (mid.y - sp[2].y)).sqrt();
    let mut angle1 = (sp[1].y - sp[2].y).atan2(sp[1].x - sp[2].x);
    let mut center = Pt::new(
        mid.x + (angle1 - PI / 2.0).cos() * radius,
        mid.y + (angle1 - PI / 2.0).sin() * radius,
    );
    let angle2 = (sp[2].y - sp[1].y).atan2(sp[2].x - sp[1].x);
    let center2 = Pt::new(
        mid.x + (angle2 - PI / 2.0).cos() * radius,
        mid.y + (angle2 - PI / 2.0).sin() * radius,
    );
    if calc_distance_double(sp[0], center2) > calc_distance_double(sp[0], center) {
        sp.swap(1, 2);
        center = center2;
        angle1 = angle2;
    }
    let radius = ((sp[1].x - center.x) * (sp[1].x - center.x)
        + (sp[1].y - center.y) * (sp[1].y - center.y))
        .sqrt();
    let bounds = (
        center.x - radius,
        center.y - radius,
        center.x + radius,
        center.y + radius,
    );
    let arc = arc_approximation_double(bounds, (sp[2].x, sp[2].y), (sp[1].x, sp[1].y));
    let mut counter = 0;
    for (k, p) in arc.iter().enumerate() {
        put(points, counter, styled(*p, if k == 16 { 5 } else { 0 }));
        counter += 1;
    }

    let angle1c = (center.y - sp[1].y).atan2(center.x - sp[1].x);
    let angle2c = (center.y - sp[2].y).atan2(center.x - sp[2].x);
    let angle12c = (angle1c + angle2c) / 2.0;
    let out = if angle1c > 0.0 && angle2c < 0.0 {
        Pt::new(
            center.x + angle12c.cos() * radius,
            center.y + angle12c.sin() * radius,
        )
    } else {
        Pt::new(
            center.x - angle12c.cos() * radius,
            center.y - angle12c.sin() * radius,
        )
    };
    put(points, counter, styled(sp[0], 0));
    put(points, counter + 1, styled(out, 5));
    counter += 2;

    let dpi = settings.dpi_scale_factor();
    for (k, p) in ambush_arrow_head(&sp, mid, out, dpi).iter().enumerate() {
        put(points, counter, styled(*p, if k == 2 { 5 } else { 0 }));
        counter += 1;
    }
    push_ambush_ticks(points, &mut counter, center, radius, angle1, angle1c, dpi);
    Ok(counter as i32)
}

/// The eight tick marks leading from the arc toward the back of the figure.
fn push_ambush_ticks(
    points: &mut Vec<Pt>,
    counter: &mut usize,
    center: Pt,
    radius: f64,
    angle1: f64,
    start_angle: f64,
    dpi: f64,
) {
    let d = clamp_size(radius / 3.0, dpi);
    let back = ((angle1 + PI / 2.0).cos() * d, (angle1 + PI / 2.0).sin() * d);
    let mut angle = start_angle;
    for _ in 0..8 {
        angle += PI / 18.0;
        let start = Pt::new(
            center.x - angle.cos() * radius,
            center.y - angle.sin() * radius,
        );
        put(points, *counter, styled(start, 0));
        put(
            points,
            *counter + 1,
            styled(Pt::new(start.x - back.0, start.y - back.1), 5),
        );
        *counter += 2;
    }
}

/// The arrowhead on the line leaving the arc.
fn ambush_arrow_head(sp: &[Pt; 3], mid: Pt, out: Pt, dpi: f64) -> [Pt; 3] {
    let angle0 = (out.y - sp[0].y).atan2(out.x - sp[0].x);
    let height = ((sp[1].x - sp[2].x) * (sp[1].x - sp[2].x)
        + (sp[1].y - sp[2].y) * (sp[1].y - sp[2].y))
        .sqrt();
    let length =
        ((sp[0].x - mid.x) * (sp[0].x - mid.x) + (sp[0].y - mid.y) * (sp[0].y - mid.y)).sqrt();
    let mut arrow = (height + length) / 20.0;
    // Upstream truncates the limits to whole pixels here before scaling.
    if arrow > MAX_LENGTH * dpi {
        arrow = f64::from(MAX_LENGTH as i32) * dpi;
    }
    if arrow < MIN_LENGTH * dpi {
        arrow = f64::from(MIN_LENGTH as i32) * dpi;
    }
    [
        Pt::new(
            sp[0].x + (angle0 + PI / 6.0).cos() * arrow,
            sp[0].y + (angle0 + PI / 6.0).sin() * arrow,
        ),
        sp[0],
        Pt::new(
            sp[0].x + (angle0 - PI / 6.0).cos() * arrow,
            sp[0].y + (angle0 - PI / 6.0).sin() * arrow,
        ),
    ]
}
