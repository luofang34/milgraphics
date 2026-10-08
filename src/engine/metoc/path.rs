//! The subset of `java.awt.geom.GeneralPath` that clsMETOC builds paths with.

use crate::engine::base::{EngineError, PathOp, Pt, Shape, shape_type};

/// A path of move, line and cubic segments. Like `GeneralPath`, it stores
/// single-precision coordinates, replaces a move that is immediately
/// followed by another move, and refuses a line or curve before the first
/// move.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GeneralPath {
    segments: Vec<Segment>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Segment {
    Move(f32, f32),
    Line(f32, f32),
    Cubic([f32; 6]),
}

const NO_INITIAL_MOVE: EngineError = EngineError::Degenerate("path segment before the first move");

impl GeneralPath {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// `moveTo`. Narrowing to `f32` is what `GeneralPath` does.
    pub(crate) fn move_to(&mut self, x: f64, y: f64) {
        let seg = Segment::Move(x as f32, y as f32);
        match self.segments.last_mut() {
            Some(last @ Segment::Move(..)) => *last = seg,
            _ => self.segments.push(seg),
        }
    }

    /// `lineTo`.
    pub(crate) fn line_to(&mut self, x: f64, y: f64) -> Result<(), EngineError> {
        if self.segments.is_empty() {
            return Err(NO_INITIAL_MOVE);
        }
        self.segments.push(Segment::Line(x as f32, y as f32));
        Ok(())
    }

    /// `curveTo`.
    pub(crate) fn curve_to(&mut self, c1: Pt, c2: Pt, end: Pt) -> Result<(), EngineError> {
        if self.segments.is_empty() {
            return Err(NO_INITIAL_MOVE);
        }
        self.segments.push(Segment::Cubic([
            c1.x as f32,
            c1.y as f32,
            c2.x as f32,
            c2.y as f32,
            end.x as f32,
            end.y as f32,
        ]));
        Ok(())
    }

    /// The path as the engine's shapes hold it. Upstream turns paths into
    /// polylines by walking move and line segments only; its conversion has
    /// no case for cubic segments, so they contribute nothing there and are
    /// left out here as well.
    pub(crate) fn into_path_ops(self) -> Vec<PathOp> {
        self.segments
            .into_iter()
            .filter_map(|s| match s {
                Segment::Move(x, y) => Some(PathOp::MoveTo(f64::from(x), f64::from(y))),
                Segment::Line(x, y) => Some(PathOp::LineTo(f64::from(x), f64::from(y))),
                Segment::Cubic(_) => None,
            })
            .collect()
    }

    /// The path as an open polyline shape of the given line style.
    pub(crate) fn into_polyline(self, style: i32) -> Shape {
        Shape {
            style,
            path: self.into_path_ops(),
            ..Shape::new(shape_type::POLYLINE)
        }
    }
}
