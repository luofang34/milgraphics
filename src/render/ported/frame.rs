//! The frame the ported renderer draws in for one view: the graphic's local
//! plane, scaled and turned to match the screen at one point of the graphic.
//!
//! The renderer lays its patterns out in pixels along straight segments
//! between the control points. Drawing it in this frame and carrying every
//! vertex back to the ground keeps those patterns at their pixel size where
//! the reference point is, while each vertex is projected like the rest of
//! the graphic: on terrain, through a tilted camera and cut at the horizon.
//! A top-down view of a small graphic matches the frame to within the
//! projection's variation over the graphic, so its output is the renderer's
//! pixel output on screen.

use crate::engine::visible::PixelBox;
use crate::geo::GeoPoint;
use crate::plane::{LocalPlane, Xy};
use crate::render::ScreenPoint;
use crate::render::screen::ScreenCtx;

/// Ground distance over which the view's scale at a graphic is measured.
const SCALE_PROBE_M: f64 = 100.0;
/// Points sampled on each edge of a segment whose ends are both hidden,
/// when looking for a visible point of the graphic.
const HIDDEN_PROBES: usize = 8;
/// Rows and columns of the grid of screen points the near box is found from.
const NEAR_GRID: usize = 5;
/// Points sampled along the control points' lines to measure the scale.
const SCALE_SAMPLES: usize = 64;

pub(super) struct Frame<'e> {
    plane: LocalPlane<'e>,
    /// Plane position of the reference point.
    at: Xy,
    /// Screen position of the reference point.
    screen: ScreenPoint,
    /// Frame pixels per plane metre.
    scale: f64,
    /// Cosine and sine of the frame's turn from the north-up plane.
    turn: (f64, f64),
}

impl<'e> Frame<'e> {
    /// The frame for the graphic with control points `anchors`, whose local
    /// plane is centred on the first, as at construction; `None` when no
    /// point of the graphic is visible.
    pub(super) fn new(ctx: &mut ScreenCtx<'e>, anchors: &[GeoPoint]) -> Option<Self> {
        let origin = *anchors.first()?;
        let (reference, screen) = reference(ctx, anchors)?;
        let plane = LocalPlane::new(ctx.earth(), origin);
        let probe = ctx.earth().direct(reference, 90.0, SCALE_PROBE_M);
        let probe_screen = ctx.project(probe)?;
        let at = plane.to_xy(reference);
        let east = plane.to_xy(probe).sub(at);
        let (ex, ey) = (east.x, -east.y);
        let (wx, wy) = probe_screen.sub(screen);
        let probed = wx.hypot(wy) / ex.hypot(ey);
        let scale = scale_along(ctx, anchors).unwrap_or(probed);
        let angle = wy.atan2(wx) - ey.atan2(ex);
        (scale.is_finite() && scale > 0.0 && angle.is_finite()).then(|| Self {
            plane,
            at,
            screen,
            scale,
            turn: (angle.cos(), angle.sin()),
        })
    }

    /// Ground metres per frame pixel.
    pub(super) fn meters_per_pixel(&self) -> f64 {
        1.0 / self.scale
    }

    /// Frame position of a ground point.
    pub(super) fn to_px(&self, p: GeoPoint) -> ScreenPoint {
        let d = self.plane.to_xy(p).sub(self.at);
        let (x, y) = (d.x * self.scale, -d.y * self.scale);
        let (c, s) = self.turn;
        ScreenPoint {
            x: self.screen.x + c * x - s * y,
            y: self.screen.y + s * x + c * y,
        }
    }

    /// Ground point at a frame position.
    pub(super) fn to_geo(&self, (x, y): (f64, f64)) -> GeoPoint {
        let (dx, dy) = (x - self.screen.x, y - self.screen.y);
        let (c, s) = self.turn;
        let (rx, ry) = (c * dx + s * dy, -s * dx + c * dy);
        let d = Xy::new(rx / self.scale, -ry / self.scale);
        self.plane.to_geo(self.at.add(d))
    }

    /// The near box (the viewport grown by its size on every side) in the
    /// frame, from the ground under a grid of its screen points; `None`
    /// without a viewport or when too little of it reaches the ground.
    pub(super) fn near(&self, ctx: &ScreenCtx<'_>) -> Option<PixelBox> {
        let b = super::clip::near(ctx.viewport())?;
        let mut found: Option<PixelBox> = None;
        let steps = (NEAR_GRID - 1) as f64;
        for i in 0..NEAR_GRID {
            for j in 0..NEAR_GRID {
                let s = ScreenPoint {
                    x: b.min_x + (b.max_x - b.min_x) * i as f64 / steps,
                    y: b.min_y + (b.max_y - b.min_y) * j as f64 / steps,
                };
                let Some(g) = ctx.unproject(s) else { continue };
                let p = self.to_px(g);
                found = Some(match found {
                    None => PixelBox {
                        min_x: p.x,
                        min_y: p.y,
                        max_x: p.x,
                        max_y: p.y,
                    },
                    Some(f) => PixelBox {
                        min_x: f.min_x.min(p.x),
                        min_y: f.min_y.min(p.y),
                        max_x: f.max_x.max(p.x),
                        max_y: f.max_y.max(p.y),
                    },
                });
            }
        }
        found.filter(|f| f.max_x > f.min_x || f.max_y > f.min_y)
    }

    /// Screen direction, in degrees clockwise from the screen x axis, of the
    /// frame direction `angle_deg` at frame position `at`.
    pub(super) fn screen_angle(
        &self,
        ctx: &mut ScreenCtx<'_>,
        at: (f64, f64),
        angle_deg: f64,
    ) -> Option<f64> {
        let (s, c) = angle_deg.to_radians().sin_cos();
        let ahead = (at.0 + c, at.1 + s);
        let a = ctx.project(self.to_geo(at))?;
        let b = ctx.project(self.to_geo(ahead))?;
        let (dx, dy) = b.sub(a);
        let deg = dy.atan2(dx).to_degrees();
        deg.is_finite().then_some(deg)
    }
}

/// Screen pixels per ground metre along the lines between the control
/// points where they are on screen: their projected length, which follows
/// the ground through hills and a tilted camera, over their ground length.
/// A single short probe would take the slope under one point for the scale
/// of the whole graphic.
fn scale_along(ctx: &mut ScreenCtx<'_>, anchors: &[GeoPoint]) -> Option<f64> {
    let earth = ctx.earth();
    let lengths: Vec<f64> = anchors
        .windows(2)
        .map(|w| match w {
            &[a, b] => earth.inverse(a, b).distance_m,
            _ => 0.0,
        })
        .collect();
    let total: f64 = lengths.iter().sum();
    if !(total.is_finite() && total > 0.0) {
        return None;
    }
    // Samples evenly spaced along each geodesic, each with the ground length to the next:
    // the geodesic solved once per edge, and no length measured again.
    let mut samples = Vec::with_capacity(SCALE_SAMPLES + anchors.len());
    let mut step_m = Vec::with_capacity(SCALE_SAMPLES + anchors.len());
    for (pair, &len) in anchors.windows(2).zip(&lengths) {
        let &[a, b] = pair else { continue };
        let steps = ((len / total) * SCALE_SAMPLES as f64).ceil().max(1.0) as usize;
        let line = earth.line(a, b);
        for k in 0..steps {
            samples.push(line.at(k as f64 / steps as f64));
            step_m.push(len / steps as f64);
        }
    }
    samples.extend(anchors.last());
    let viewport = ctx.viewport();
    let on_screen = |s: ScreenPoint| {
        viewport
            .is_none_or(|v| s.x >= v.min.x && s.x <= v.max.x && s.y >= v.min.y && s.y <= v.max.y)
    };
    let projected: Vec<Option<ScreenPoint>> = samples.iter().map(|&p| ctx.project(p)).collect();
    let (mut screen, mut ground) = (0.0, 0.0);
    for (s, &len) in projected.windows(2).zip(&step_m) {
        if let &[Some(sa), Some(sb)] = s {
            if on_screen(sa) || on_screen(sb) {
                let (dx, dy) = sb.sub(sa);
                screen += dx.hypot(dy);
                ground += len;
            }
        }
    }
    let scale = screen / ground;
    (scale.is_finite() && scale > 0.0).then_some(scale)
}

/// The visible point of the graphic the frame matches the screen at: the
/// control point nearest the middle of the viewport, or, when every control
/// point is hidden, the nearest visible point of the lines between them.
fn reference(ctx: &mut ScreenCtx<'_>, anchors: &[GeoPoint]) -> Option<(GeoPoint, ScreenPoint)> {
    let middle = ctx.viewport().map(|v| ScreenPoint {
        x: (v.min.x + v.max.x) / 2.0,
        y: (v.min.y + v.max.y) / 2.0,
    });
    let mut seen: Vec<(GeoPoint, ScreenPoint)> = anchors
        .iter()
        .filter_map(|&a| Some((a, ctx.project(a)?)))
        .collect();
    if seen.is_empty() {
        for pair in anchors.windows(2) {
            let &[a, b] = pair else { continue };
            for k in 1..=HIDDEN_PROBES {
                let t = k as f64 / (HIDDEN_PROBES + 1) as f64;
                let p = ctx.earth().interpolate(a, b, t);
                if let Some(s) = ctx.project(p) {
                    seen.push((p, s));
                }
            }
        }
    }
    let off_middle = |s: ScreenPoint| {
        middle.map_or(0.0, |m| {
            let (dx, dy) = s.sub(m);
            dx.hypot(dy)
        })
    };
    // The first of equally near points, so the choice is repeatable.
    seen.into_iter().fold(
        None,
        |best: Option<(GeoPoint, ScreenPoint)>, c| match best {
            Some(b) if off_middle(b.1) <= off_middle(c.1) => Some(b),
            _ => Some(c),
        },
    )
}
