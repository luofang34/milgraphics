//! FIX, MNFLDFIX and CLEAR from DISMSupport.java.
//!
//! FIX only implements the path where no clip rectangle bounds the segment,
//! so the zigzag is always drawn.

use super::support::{calc_endpiece_deltas_double, clamp_size, draw_endpiece_deltas_double, put};
use crate::engine::base::{At, EngineError, Pt};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use std::f64::consts::PI;

fn styled(p: Pt, style: i32) -> Pt {
    let mut q = p;
    q.style = style;
    q
}

fn push_segment(points: &mut Vec<Pt>, counter: &mut usize, a: Pt, b: Pt) {
    put(points, *counter, styled(a, 0));
    put(points, *counter + 1, styled(b, 5));
    *counter += 2;
}

/// The zigzag deltas of a jagged line: out from the centre line and along it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Jaggy {
    pub(super) out_x: f64,
    pub(super) out_y: f64,
    pub(super) along_x: f64,
    pub(super) along_y: f64,
}

/// Draws the jagged line from `end` back to `start` (the shared body of FIX
/// and BYDIF) and returns its deltas.
pub(super) fn push_jaggy_line(
    points: &mut Vec<Pt>,
    counter: &mut usize,
    start: Pt,
    end: Pt,
    settings: &Settings,
) -> Jaggy {
    let angle = (start.y - end.y).atan2(start.x - end.x);
    let length =
        ((end.x - start.x) * (end.x - start.x) + (end.y - start.y) * (end.y - start.y)).sqrt();
    let half_amp = clamp_size(length / 15.0, settings.dpi_scale_factor());
    let half_period = half_amp / 1.5;
    let j = Jaggy {
        out_x: (angle + PI / 2.0).cos() * half_amp,
        out_y: (angle + PI / 2.0).sin() * half_amp,
        along_x: angle.cos() * half_period,
        along_y: angle.sin() * half_period,
    };
    let num_jaggies = (length / half_period) as i32 - 3;
    let mut i: i32 = 2;
    let mut pts = [Pt::default(); 3];
    pts[0] = end;
    pts[1].x = end.x + j.along_x * 1.5;
    pts[1].y = end.y + j.along_y * 1.5;
    push_segment(points, counter, pts[0], pts[1]);

    pts[0].x = end.x + j.out_x + j.along_x * f64::from(i);
    pts[0].y = end.y + j.out_y + j.along_y * f64::from(i);
    i += 1;
    push_segment(points, counter, pts[0], pts[1]);

    while i <= num_jaggies {
        pts[1].x = end.x - j.out_x + j.along_x * f64::from(i);
        pts[1].y = end.y - j.out_y + j.along_y * f64::from(i);
        i += 1;
        pts[2].x = end.x + j.out_x + j.along_x * f64::from(i);
        pts[2].y = end.y + j.out_y + j.along_y * f64::from(i);
        i += 1;
        for (k, p) in pts.iter().enumerate() {
            put(points, *counter, styled(*p, if k == 2 { 5 } else { 0 }));
            *counter += 1;
        }
        pts[0] = pts[2];
    }

    pts[1] = pts[0];
    pts[0].x = end.x + j.along_x * f64::from(i);
    pts[0].y = end.y + j.along_y * f64::from(i);
    push_segment(points, counter, pts[0], pts[1]);
    pts[1] = start;
    push_segment(points, counter, pts[0], pts[1]);
    j
}

/// Upstream `GetDISMFixDouble`: a zigzag from the second to the first
/// control point ending in an arrowhead.
pub(crate) fn get_dism_fix_double(
    points: &mut Vec<Pt>,
    line_type: i32,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let sp = [points.at(0)?, points.at(1)?];
    let mut counter = 0;
    let j = push_jaggy_line(points, &mut counter, sp[0], sp[1], settings);
    let head = [
        Pt::new(
            sp[0].x + j.out_x / 1.5 - j.along_x,
            sp[0].y + j.out_y / 1.5 - j.along_y,
        ),
        sp[0],
        Pt::new(
            sp[0].x - j.out_x / 1.5 - j.along_x,
            sp[0].y - j.out_y / 1.5 - j.along_y,
        ),
    ];
    let mine = line_type == lt::MNFLDFIX;
    for (k, p) in head.iter().enumerate() {
        let style = match (mine, k == 2) {
            (true, true) => 10,
            (true, false) => 9,
            (false, true) => 5,
            (false, false) => 0,
        };
        put(points, counter, styled(*p, style));
        counter += 1;
    }
    Ok(counter as i32)
}

/// Upstream `GetDISMClearDouble`: the CLEAR figure, 20 points.
pub(crate) fn get_dism_clear_double(
    points: &mut Vec<Pt>,
    settings: &Settings,
) -> Result<i32, EngineError> {
    let sp = [points.at(0)?, points.at(1)?, points.at(2)?];
    let ctr = Pt::new((sp[0].x + sp[1].x) / 2.0, (sp[0].y + sp[1].y) / 2.0);
    let mut counter = 0;
    push_segment(points, &mut counter, sp[0], sp[1]);
    push_segment(points, &mut counter, ctr, sp[2]);
    let mut arrows = vec![ctr];
    for from in [sp[0], sp[1]] {
        let start = Pt::new((from.x + ctr.x) / 2.0, (from.y + ctr.y) / 2.0);
        let end = Pt::new(sp[2].x + from.x - start.x, sp[2].y + from.y - start.y);
        arrows.push(start);
        push_segment(points, &mut counter, start, end);
    }
    let dpi = settings.dpi_scale_factor();
    let (dx1, dy1) = calc_endpiece_deltas_double(&sp, PI / 6.0, dpi)?;
    let (dx2, dy2) = calc_endpiece_deltas_double(&sp, -PI / 6.0, dpi)?;
    for tip in arrows {
        for p in draw_endpiece_deltas_double(tip, dx1, dy1, dx2, dy2) {
            put(points, counter, p);
            counter += 1;
        }
    }
    Ok(counter as i32)
}
