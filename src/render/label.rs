//! Placing labels for a view.

use crate::construction::{LabelPlacement, LabelSpec};
use crate::geo::GeoPoint;
use crate::pick::PickRef;
use crate::render::screen::ScreenCtx;
use crate::render::{Font, FontMetrics, ScreenPoint};

/// Horizontal alignment of text on its anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Text starts at the anchor.
    Left,
    /// Text is centred on the anchor.
    Center,
    /// Text ends at the anchor.
    Right,
}

/// A label resolved for a view.
///
/// `offset_em` is expressed in the text's own rotated frame (x along the
/// baseline, y downward), in ems, which is how map engines take symbol
/// text offsets. The anchor is the vertical middle of the text line.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// What it belongs to.
    pub pick: PickRef,
    /// Text to draw.
    pub text: String,
    /// Geographic anchor.
    pub anchor: GeoPoint,
    /// Anchor on screen, if visible.
    pub screen: Option<ScreenPoint>,
    /// Clockwise rotation in degrees from the screen x axis, within (-90, 90].
    pub rotation_deg: f64,
    /// Alignment on the anchor.
    pub align: TextAlign,
    /// Offset from the anchor in ems, in the text frame.
    pub offset_em: [f64; 2],
    /// Font.
    pub font: Font,
    /// Whether collision may hide it.
    pub may_hide: bool,
    /// Text width in pixels, from the host's metrics.
    pub width_px: f64,
    /// Screen corners of the text box (for picking and overlap checks), if
    /// visible.
    pub corners: Option<[ScreenPoint; 4]>,
}

/// Gap between a line end and its label, in ems.
const LINE_END_GAP_EM: f64 = 0.5;

pub(crate) fn place(
    ctx: &mut ScreenCtx<'_>,
    spec: &LabelSpec,
    font: &Font,
    metrics: &dyn FontMetrics,
    pick: PickRef,
) -> Label {
    let screen = ctx.project(spec.anchor);
    let (rotation_deg, align, offset_em) = match spec.placement {
        LabelPlacement::Centered => (0.0, TextAlign::Center, [0.0, spec.line_offset]),
        LabelPlacement::LineEnd { inward } => {
            let direction = screen
                .zip(ctx.project(inward))
                .map(|(end, inner)| end.sub(inner))
                .filter(|(dx, dy)| dx.hypot(*dy) > 0.0);
            line_end(direction, spec.line_offset)
        }
        LabelPlacement::Along { toward } => {
            let direction = screen
                .zip(ctx.project(toward))
                .map(|(a, b)| b.sub(a))
                .filter(|(dx, dy)| dx.hypot(*dy) > 0.0);
            let rotation = direction.map_or(0.0, |(dx, dy)| upright(dy.atan2(dx).to_degrees()));
            (rotation, TextAlign::Center, [0.0, spec.line_offset])
        }
    };
    let width_px = metrics.text_width_px(font, &spec.text);
    let height_px = metrics.line_height_px(font);
    let corners = screen.map(|s| {
        text_box(
            s,
            rotation_deg,
            align,
            offset_em,
            font.size_px,
            width_px,
            height_px,
        )
    });
    Label {
        pick,
        text: spec.text.clone(),
        anchor: spec.anchor,
        screen,
        rotation_deg,
        align,
        offset_em,
        font: font.clone(),
        may_hide: spec.may_hide,
        width_px,
        corners,
    }
}

/// Rotation, alignment and offset for text at a line end extending outward
/// along `outward` (screen direction from the inner vertex to the end),
/// kept upright.
fn line_end(outward: Option<(f64, f64)>, line_offset: f64) -> (f64, TextAlign, [f64; 2]) {
    let Some((dx, dy)) = outward else {
        return (0.0, TextAlign::Left, [LINE_END_GAP_EM, line_offset]);
    };
    let angle = dy.atan2(dx).to_degrees();
    if dx >= 0.0 && angle > -90.0 {
        (angle, TextAlign::Left, [LINE_END_GAP_EM, line_offset])
    } else {
        let upright = if angle > 0.0 {
            angle - 180.0
        } else {
            angle + 180.0
        };
        (upright, TextAlign::Right, [-LINE_END_GAP_EM, line_offset])
    }
}

/// An angle in degrees folded into (-90, 90] so text reads left to right.
fn upright(angle: f64) -> f64 {
    if angle > 90.0 {
        angle - 180.0
    } else if angle <= -90.0 {
        angle + 180.0
    } else {
        angle
    }
}

fn text_box(
    anchor: ScreenPoint,
    rotation_deg: f64,
    align: TextAlign,
    offset_em: [f64; 2],
    em_px: f64,
    width_px: f64,
    height_px: f64,
) -> [ScreenPoint; 4] {
    let x0 = offset_em[0] * em_px
        - match align {
            TextAlign::Left => 0.0,
            TextAlign::Center => width_px / 2.0,
            TextAlign::Right => width_px,
        };
    let y0 = offset_em[1] * em_px - height_px / 2.0;
    let (s, c) = rotation_deg.to_radians().sin_cos();
    let at = |x: f64, y: f64| ScreenPoint {
        x: anchor.x + x * c - y * s,
        y: anchor.y + x * s + y * c,
    };
    [
        at(x0, y0),
        at(x0 + width_px, y0),
        at(x0 + width_px, y0 + height_px),
        at(x0, y0 + height_px),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outward_text_is_upright_and_starts_or_ends_at_the_anchor() {
        // Eastward end: rotation follows the line, text starts at the end.
        let (r, a, o) = line_end(Some((10.0, -5.0)), 0.0);
        assert!((r - (-5.0f64).atan2(10.0).to_degrees()).abs() < 1e-12);
        assert_eq!((a, o[0]), (TextAlign::Left, 0.5));
        // Westward end: flipped upright, text ends at the end.
        let (r, a, o) = line_end(Some((-10.0, -5.0)), 0.0);
        assert!(r > -90.0 && r <= 90.0);
        assert!((r - 5.0f64.atan2(10.0).to_degrees()).abs() < 1e-12);
        assert_eq!((a, o[0]), (TextAlign::Right, -0.5));
        // Straight up and straight down stay within (-90, 90].
        assert_eq!(line_end(Some((0.0, -1.0)), 0.0).0, 90.0);
        assert_eq!(line_end(Some((0.0, 1.0)), 0.0).0, 90.0);
    }

    #[test]
    fn text_box_of_unrotated_left_text() {
        let b = text_box(
            ScreenPoint { x: 10.0, y: 20.0 },
            0.0,
            TextAlign::Left,
            [0.5, 0.0],
            12.0,
            40.0,
            14.0,
        );
        assert_eq!(b[0], ScreenPoint { x: 16.0, y: 13.0 });
        assert_eq!(b[2], ScreenPoint { x: 56.0, y: 27.0 });
    }
}
