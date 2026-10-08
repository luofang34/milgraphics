//! Geographic construction: the view-independent form of a graphic.

use crate::definition::GraphicId;
use crate::edit::HandleId;
use crate::geo::GeoPoint;
use crate::style::{Fill, Stroke};

/// Index of a part within one graphic's construction, stable for a given
/// symbol and control-point count, so picks and styling can refer to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartId(pub u16);

/// What a part represents in the symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PartRole {
    /// The main line of a line graphic.
    Line,
    /// The outline of an area.
    Boundary,
    /// An arrowhead.
    Arrowhead,
    /// The orientation (centre of sector) indicator of a range fan.
    Orientation,
    /// Any other drawn element.
    Decoration,
}

/// Geometry in WGS84 degrees. Edges are geodesics and are already densified,
/// so an engine that draws straight segments in its own projection shows the
/// geodesic within the construction tolerance.
#[derive(Clone, Debug, PartialEq)]
pub enum GeoGeometry {
    /// An open polyline.
    Line(Vec<GeoPoint>),
    /// A closed ring, listed without repeating the first point.
    Ring(Vec<GeoPoint>),
}

impl GeoGeometry {
    /// The vertices, without the closing repeat for rings.
    pub fn points(&self) -> &[GeoPoint] {
        match self {
            Self::Line(p) | Self::Ring(p) => p,
        }
    }
}

/// A drawn part in geographic space.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct GeoPart {
    /// Index of the part.
    pub id: PartId,
    /// What it represents.
    pub role: PartRole,
    /// Where it is.
    pub geometry: GeoGeometry,
    /// Outline, if stroked.
    pub stroke: Option<Stroke>,
    /// Interior, for rings.
    pub fill: Fill,
}

/// How large a pixel-sized decoration is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DecorationSize {
    /// A fraction of the summed on-screen lengths of two geographic
    /// segments, clamped to a pixel range.
    Proportional {
        /// The segments, as (start, end) pairs.
        segments: [(GeoPoint, GeoPoint); 2],
        /// Fraction of the summed length.
        fraction: f64,
        /// Smallest size in pixels.
        min_px: f64,
        /// Largest size in pixels.
        max_px: f64,
    },
}

/// A decoration whose size is set in screen pixels, so it can only be
/// resolved once a projection is known. Rendering turns it into screen-tier
/// items marked as decorations; its contents are internal.
#[derive(Clone, Debug, PartialEq)]
pub struct ScreenDecoration(pub(crate) Decoration);

/// The kinds of pixel-sized decoration.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Decoration {
    /// Shapes and labels the ported upstream renderer draws in pixels for
    /// line type `line_type`, from the control points projected for the
    /// view. Shapes at the indices in `geographic` are already parts of the
    /// construction and are not drawn again, as long as the renderer gives
    /// `shape_count` shapes.
    Engine {
        line_type: i32,
        anchors: Vec<GeoPoint>,
        symbol: crate::sidc::SymbolId,
        modifiers: Box<crate::modifier::Modifiers>,
        geographic: Vec<bool>,
        shape_count: usize,
        part: PartId,
    },
    /// A two-winged arrowhead at `tip`, opening back toward `toward`.
    Arrowhead {
        /// Index of the part.
        id: PartId,
        /// The arrow tip.
        tip: GeoPoint,
        /// A point the wings open toward; only its screen direction matters.
        toward: GeoPoint,
        /// Wing length.
        size: DecorationSize,
        /// Angle between each wing and the axis, in degrees.
        half_angle_deg: f64,
        /// Solid triangle rather than two strokes.
        filled: bool,
        /// Outline style.
        stroke: Stroke,
    },
    /// A line from `from` through `through`, continued a short pixel
    /// distance past it to an open arrowhead: the orientation indicator of a
    /// range fan.
    Pointer {
        /// Index of the part.
        id: PartId,
        /// Start of the line.
        from: GeoPoint,
        /// Where the arrowhead starts.
        through: GeoPoint,
        /// Arrowhead length and half-width in pixels.
        head_px: f64,
        /// Outline style.
        stroke: Stroke,
    },
}

/// How a label is positioned relative to its anchor.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum LabelPlacement {
    /// Horizontal text centred on the anchor.
    Centered,
    /// At the end of a line, rotated along the end segment (kept upright)
    /// and extending outward, away from the line.
    LineEnd {
        /// The neighbouring vertex inside the line, giving its direction.
        inward: GeoPoint,
    },
    /// Centred on the anchor and rotated along the direction toward
    /// `toward` (kept upright).
    Along {
        /// A point giving the text direction.
        toward: GeoPoint,
    },
    /// Like [`LabelPlacement::Along`], but anchored at whichever of `edges`
    /// is uppermost on screen, so text stacked upward (negative offsets)
    /// stays outside the symbol whatever the view's rotation.
    OutsideEdge {
        /// A point giving the text direction.
        toward: GeoPoint,
        /// The two edge points either side of the anchor.
        edges: [GeoPoint; 2],
    },
}

/// A label in geographic terms.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct LabelSpec {
    /// The part the label belongs to.
    pub part: PartId,
    /// Full text, prefixes included.
    pub text: String,
    /// Geographic anchor.
    pub anchor: GeoPoint,
    /// How the text sits on the anchor.
    pub placement: LabelPlacement,
    /// Offset perpendicular to the text in ems, positive downward on screen;
    /// stacked labels use whole ems.
    pub line_offset: f64,
    /// Whether map label collision may hide it.
    pub may_hide: bool,
}

/// What dragging a handle changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HandleKind {
    /// A control point.
    Vertex,
    /// A width (point N of an axis, `AM` of a corridor).
    Width,
    /// A range (`AM` of a range fan).
    Range,
    /// An azimuth (`AN` of a range fan).
    Azimuth,
}

/// An edit handle in geographic terms.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct HandleSpec {
    /// Stable identity, used to apply an edit.
    pub id: HandleId,
    /// What it changes.
    pub kind: HandleKind,
    /// Where it is drawn.
    pub at: GeoPoint,
}

/// A graphic constructed in geographic space. Depends only on the
/// definition and configuration, so it is cached across view changes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Construction {
    /// [`crate::RENDERER_VERSION`] that built it.
    pub renderer_version: &'static str,
    /// The definition it was built from.
    pub definition: GraphicId,
    /// The definition's revision.
    pub revision: u64,
    /// Drawn parts, in drawing order.
    pub parts: Vec<GeoPart>,
    /// Pixel-sized decorations.
    pub decorations: Vec<ScreenDecoration>,
    /// Labels.
    pub labels: Vec<LabelSpec>,
    /// Edit handles.
    pub handles: Vec<HandleSpec>,
}
