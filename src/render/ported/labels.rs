//! Placing the ported renderer's labels on screen.
//!
//! Upstream stacks several modifiers line under line, a font size apart, at
//! one place on the graphic. Carrying each line's position to the ground on
//! its own would let a tilted camera foreshorten the spacing while the text
//! stays its size in pixels, and the lines would overprint. Labels set
//! close together at one angle are therefore placed as a block: the block's
//! middle is on the ground, and each line keeps its offset from it in
//! pixels, turned as the graphic runs on screen there.

use crate::engine::api::{self, Justify};
use crate::pick::PickRef;
use crate::render::label::{self, Label, TextAlign};
use crate::render::screen::ScreenCtx;
use crate::render::{Font, FontMetrics};

use super::frame::Frame;

/// Labels whose positions are at most this many font sizes apart, at the
/// same angle, belong to one block: upstream stacks lines one size apart.
const BLOCK_GAP_EM: f64 = 2.0;
/// Angles, in degrees, that count as the same.
const SAME_ANGLE_DEG: f64 = 1.0;

/// Labels placed in blocks; a block whose middle is hidden is not drawn.
pub(super) fn placed(
    ctx: &mut ScreenCtx<'_>,
    frame: &Frame<'_>,
    labels: Vec<api::Label>,
    font: &Font,
    metrics: &dyn FontMetrics,
    pick: &dyn Fn() -> PickRef,
) -> Vec<Label> {
    let mut out = Vec::with_capacity(labels.len());
    for block in blocks(&labels, font.size_px) {
        let n = block.len() as f64;
        let middle = block
            .iter()
            .fold((0.0, 0.0), |(x, y), &i| match labels.get(i) {
                Some(l) => (x + l.x / n, y + l.y / n),
                None => (x, y),
            });
        let Some(first) = block.first().and_then(|&i| labels.get(i)) else {
            continue;
        };
        let anchor = frame.to_geo(middle);
        let Some(screen) = ctx.project(anchor) else {
            continue;
        };
        let turn = frame
            .screen_angle(ctx, middle, first.angle_deg)
            .map_or(0.0, |a| a - first.angle_deg);
        for &i in &block {
            let Some(l) = labels.get(i) else { continue };
            // Kept upright: a label turned past vertical reads the other
            // way, so it extends from its anchor to the other side.
            let (rotation_deg, flipped) = upright(l.angle_deg + turn);
            let align = match (l.justify, flipped) {
                (Justify::Left, false) | (Justify::Right, true) => TextAlign::Left,
                (Justify::Center, _) => TextAlign::Center,
                (Justify::Right, false) | (Justify::Left, true) => TextAlign::Right,
            };
            let on_screen = rotate((l.x - middle.0, l.y - middle.1), turn);
            out.push(label::at_screen(
                label::PlacedText {
                    text: l.text.clone(),
                    screen,
                    anchor,
                    offset_px: rotate(on_screen, -rotation_deg),
                    rotation_deg,
                    align,
                    knockout: l.knockout,
                },
                font,
                metrics,
                pick(),
            ));
        }
    }
    out
}

/// Indices of `labels` grouped into blocks, each in upstream's order:
/// labels chained by neighbours within [`BLOCK_GAP_EM`] at the same angle.
fn blocks(labels: &[api::Label], size_px: f64) -> Vec<Vec<usize>> {
    let reach = BLOCK_GAP_EM * size_px;
    let mut block_of: Vec<usize> = (0..labels.len()).collect();
    let near = |a: &api::Label, b: &api::Label| {
        let turn = (a.angle_deg - b.angle_deg + 180.0).rem_euclid(360.0) - 180.0;
        turn.abs() <= SAME_ANGLE_DEG && (a.x - b.x).hypot(a.y - b.y) <= reach
    };
    for (i, a) in labels.iter().enumerate() {
        for (j, b) in labels.iter().enumerate().take(i) {
            if !near(a, b) {
                continue;
            }
            let (from, to) = (block_of.get(i).copied(), block_of.get(j).copied());
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
                    let (keep, merge) = (from.min(to), from.max(to));
                    for b in &mut block_of {
                        if *b == merge {
                            *b = keep;
                        }
                    }
                }
            }
        }
    }
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut ids: Vec<usize> = Vec::new();
    for (i, &b) in block_of.iter().enumerate() {
        match ids.iter().position(|&id| id == b) {
            Some(k) => {
                if let Some(block) = out.get_mut(k) {
                    block.push(i);
                }
            }
            None => {
                ids.push(b);
                out.push(vec![i]);
            }
        }
    }
    out
}

/// `v` turned clockwise on screen by `deg`.
fn rotate((x, y): (f64, f64), deg: f64) -> (f64, f64) {
    let (s, c) = deg.to_radians().sin_cos();
    (x * c - y * s, x * s + y * c)
}

/// `deg` brought within (-90, 90], and whether that turned it half round.
pub(super) fn upright(deg: f64) -> (f64, bool) {
    let d = (deg + 180.0).rem_euclid(360.0) - 180.0;
    if d > 90.0 {
        (d - 180.0, true)
    } else if d <= -90.0 {
        (d + 180.0, true)
    } else {
        (d, false)
    }
}
