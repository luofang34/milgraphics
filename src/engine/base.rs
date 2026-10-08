//! The types every part of the port shares: upstream's `POINT2`, its shapes
//! (`Shape2`/`ShapeInfo`), the engine's error, and the idioms that keep Java
//! semantics (out-of-range access fails, `Math.round`, integer truncation)
//! without panicking.

use crate::style::Rgba;

#[cfg(test)]
mod tests;

/// Upstream `POINT2`: a pixel position with the line-style marker the
/// renderer threads through point arrays (0 normal, 5 end of a polyline,
/// 9 fill segment, 10 end of a CF polyline, others as upstream uses).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Pt {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) style: i32,
    pub(crate) segment: i32,
}

impl Pt {
    pub(crate) const fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            style: 0,
            segment: 0,
        }
    }

    pub(crate) const fn styled(x: f64, y: f64, style: i32) -> Self {
        Self {
            x,
            y,
            style,
            segment: 0,
        }
    }
}

/// Why the engine could not draw a graphic. Upstream throws (and draws
/// nothing) in the same situations.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub(crate) enum EngineError {
    /// An index outside an array, where Java would throw.
    #[error("index {index} outside {len} items")]
    Index {
        /// The index.
        index: i64,
        /// The array length.
        len: usize,
    },
    /// A modifier value that does not parse, where Java would throw.
    #[error("modifier value {0:?} is not a number")]
    Number(String),
    /// The line type is not one the engine draws.
    #[error("line type {0} is not drawn")]
    LineType(i32),
    /// The input cannot define the shape.
    #[error("{0}")]
    Degenerate(&'static str),
}

/// Checked element access with Java's failure semantics.
pub(crate) trait At<T> {
    /// The element at `i`, or an error where Java would throw.
    fn at(&self, i: usize) -> Result<T, EngineError>;
    /// The element at `i` for update, or an error where Java would throw.
    fn at_mut(&mut self, i: usize) -> Result<&mut T, EngineError>;
}

impl<T: Copy> At<T> for [T] {
    fn at(&self, i: usize) -> Result<T, EngineError> {
        self.get(i).copied().ok_or(EngineError::Index {
            index: i64::try_from(i).unwrap_or(i64::MAX),
            len: self.len(),
        })
    }

    fn at_mut(&mut self, i: usize) -> Result<&mut T, EngineError> {
        let len = self.len();
        self.get_mut(i).ok_or(EngineError::Index {
            index: i64::try_from(i).unwrap_or(i64::MAX),
            len,
        })
    }
}

/// A Java `int` index as `usize`; negative indices fail as Java would.
pub(crate) fn idx(i: i32, len: usize) -> Result<usize, EngineError> {
    usize::try_from(i).map_err(|_| EngineError::Index {
        index: i64::from(i),
        len,
    })
}

/// Java `Math.round(double)`: the nearest integer, halves rounded up.
pub(crate) fn java_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// Upstream `ShapeInfo` shape types.
pub(crate) mod shape_type {
    pub(crate) const POLYLINE: i32 = 0;
    pub(crate) const FILL: i32 = 1;
    pub(crate) const MODIFIER: i32 = 2;
    pub(crate) const MODIFIER_FILL: i32 = 3;
}

/// One step of a shape's path, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PathOp {
    MoveTo(f64, f64),
    LineTo(f64, f64),
}

/// A stroke: width in pixels and an optional dash array (lengths in pixels,
/// as upstream's `BasicStroke`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stroke {
    pub(crate) width: f64,
    pub(crate) dash: Option<Vec<f64>>,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            dash: None,
        }
    }
}

/// Upstream `Shape2`: a path with its shape type, line style, colours and
/// stroke.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Shape {
    pub(crate) shape_type: i32,
    pub(crate) style: i32,
    pub(crate) fill_style: i32,
    pub(crate) line_color: Option<Rgba>,
    pub(crate) fill_color: Option<Rgba>,
    pub(crate) stroke: Stroke,
    pub(crate) path: Vec<PathOp>,
}

impl Shape {
    pub(crate) fn new(shape_type: i32) -> Self {
        Self {
            shape_type,
            ..Self::default()
        }
    }

    pub(crate) fn move_to(&mut self, p: Pt) {
        self.path.push(PathOp::MoveTo(p.x, p.y));
    }

    pub(crate) fn line_to(&mut self, p: Pt) {
        self.path.push(PathOp::LineTo(p.x, p.y));
    }

    /// The path's points, as upstream's `getPoints` lists them.
    pub(crate) fn points(&self) -> Vec<Pt> {
        self.path
            .iter()
            .map(|op| match *op {
                PathOp::MoveTo(x, y) => Pt::styled(x, y, 0),
                PathOp::LineTo(x, y) => Pt::styled(x, y, 1),
            })
            .collect()
    }

    /// The path split into polylines at each move.
    pub(crate) fn polylines(&self) -> Vec<Vec<(f64, f64)>> {
        let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
        for op in &self.path {
            match *op {
                PathOp::MoveTo(x, y) => out.push(vec![(x, y)]),
                PathOp::LineTo(x, y) => match out.last_mut() {
                    Some(line) => line.push((x, y)),
                    None => out.push(vec![(x, y)]),
                },
            }
        }
        out
    }
}
