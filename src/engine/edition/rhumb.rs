//! The version 16 Navigational Rhumb Line: AN at the middle of the line,
//! along it, and T upright in a box on the other side. AN sits on the
//! left of the line's drawn direction; the construction orders the line so
//! that its left is its north or west side (`family::ported::rhumb`).

use super::{centred, font_px, lines_like, rotated, text, unit, upright_deg};
use crate::engine::api::{Input, Output};

/// Space between the line and the nearest edge of each label, and between
/// T and its box, in pixels.
const CLEARANCE: f64 = 3.0;

pub(super) fn labels(input: &Input<'_>, out: &mut Output) {
    out.labels.clear();
    let Some(line) = out.shapes.first().cloned() else {
        return;
    };
    let Some(points) = line.polylines().into_iter().next() else {
        return;
    };
    let Some((mid, d)) = halfway(&points) else {
        return;
    };
    let size = font_px();
    let left = (d.1, -d.0);
    if let Some(&course) = input.modifiers.azimuths_deg.first() {
        let off = size / 2.0 + CLEARANCE;
        let at = (mid.0 + off * left.0, mid.1 + off * left.1);
        out.labels
            .push(rotated(&course_text(course), at, upright_deg(d), false));
    }
    let Some(t) = text(&input.modifiers.designation) else {
        return;
    };
    let half = (
        (input.text_width)(t) / 2.0 + CLEARANCE,
        size / 2.0 + CLEARANCE,
    );
    let right = (-left.0, -left.1);
    // How far the box reaches toward the line from its centre.
    let reach = right.0.abs() * half.0 + right.1.abs() * half.1;
    let off = reach + CLEARANCE;
    let c = (mid.0 + off * right.0, mid.1 + off * right.1);
    out.labels.push(centred(t, c));
    let corners = vec![
        (c.0 - half.0, c.1 - half.1),
        (c.0 + half.0, c.1 - half.1),
        (c.0 + half.0, c.1 + half.1),
        (c.0 - half.0, c.1 + half.1),
        (c.0 - half.0, c.1 - half.1),
    ];
    let mut boxed = lines_like(&line, &[corners]);
    boxed.stroke.dash = None;
    out.shapes.push(boxed);
}

/// A course in whole degrees as three digits ("060"), and any other value
/// as entered.
pub(crate) fn course_text(deg: f64) -> String {
    if deg.fract() == 0.0 && (0.0..1000.0).contains(&deg) {
        format!("{:03}", deg as u32)
    } else {
        format!("{deg}")
    }
}

/// The point halfway along `points` and the unit direction of the line
/// there.
fn halfway(points: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
    let pairs = || points.iter().zip(points.iter().skip(1));
    let total: f64 = pairs().map(|(a, b)| (b.0 - a.0).hypot(b.1 - a.1)).sum();
    let mut left = total / 2.0;
    for (a, b) in pairs() {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        let Some(d) = unit((b.0 - a.0, b.1 - a.1)) else {
            continue;
        };
        if left <= len {
            return Some(((a.0 + d.0 * left, a.1 + d.1 * left), d));
        }
        left -= len;
    }
    None
}
