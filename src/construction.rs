//! Geographic construction: the view-independent form of a graphic.

use crate::definition::GraphicId;
use crate::edit::HandleId;
use crate::geo::GeoPoint;
use crate::style::{Fill, Stroke};

/// Index of a part within one graphic's construction, stable for a given
/// symbol and control-point count, so picks and styling can refer to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartId(pub(crate) u32);

impl PartId {
    /// The part's index.
    pub const fn get(self) -> u32 {
        self.0
    }
}

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
    /// A line of a hatched area's fill.
    Hatch,
    /// A figure of a pattern-filled area's fill.
    Pattern,
}

impl PartRole {
    /// The role's name in lower snake case (`"line"`, `"boundary"`,
    /// `"arrowhead"`, `"orientation"`, `"decoration"`, `"hatch"`,
    /// `"pattern"`). Names are stable; GeoJSON output and map styles key on
    /// them.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Boundary => "boundary",
            Self::Arrowhead => "arrowhead",
            Self::Orientation => "orientation",
            Self::Decoration => "decoration",
            Self::Hatch => "hatch",
            Self::Pattern => "pattern",
        }
    }
}

/// Geometry in WGS84 degrees. Edges are geodesics and are already densified,
/// so an engine that draws straight segments in its own projection shows the
/// geodesic within the construction tolerance.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub(crate) enum GeoGeometry {
    /// An open polyline.
    Line(Vec<GeoPoint>),
    /// A closed ring, listed without repeating the first point.
    Ring(Vec<GeoPoint>),
}

impl GeoGeometry {
    /// The vertices, without the closing repeat for rings.
    pub(crate) fn points(&self) -> &[GeoPoint] {
        match self {
            Self::Line(p) | Self::Ring(p) => p,
        }
    }
}

/// A drawn part in geographic space.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub(crate) struct GeoPart {
    /// Index of the part.
    pub(crate) id: PartId,
    /// What it represents.
    pub(crate) role: PartRole,
    /// Where it is.
    pub(crate) geometry: GeoGeometry,
    /// Outline, if stroked.
    pub(crate) stroke: Option<Stroke>,
    /// Interior, for rings.
    pub(crate) fill: Fill,
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
pub(crate) struct ScreenDecoration(pub(crate) Decoration);

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
        style: crate::engine::api::Style,
        geographic: Vec<bool>,
        shape_count: usize,
        part: PartId,
        /// Ground size the graphic spans, in metres, so a view can tell how
        /// large it appears when it has no geographic parts to measure.
        reach_m: f64,
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
    /// A small figure drawn in pixels around a point: `points` are offsets
    /// from the anchor's screen position moved by `offset_px`, y downward.
    Glyph {
        /// Index of the part.
        id: PartId,
        /// Where the figure is drawn.
        anchor: GeoPoint,
        /// Shift from the anchor's screen position, in pixels.
        offset_px: [f64; 2],
        /// The outline, in pixels.
        points: Vec<[f64; 2]>,
        /// Whether the last point joins back to the first.
        closed: bool,
        /// Outline style.
        stroke: Stroke,
    },
}

/// How a label is positioned relative to its anchor.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub(crate) enum LabelPlacement {
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
pub(crate) struct LabelSpec {
    /// The part the label belongs to.
    pub(crate) part: PartId,
    /// Full text, prefixes included.
    pub(crate) text: String,
    /// Geographic anchor.
    pub(crate) anchor: GeoPoint,
    /// How the text sits on the anchor.
    pub(crate) placement: LabelPlacement,
    /// Offset perpendicular to the text in ems, positive downward on screen;
    /// stacked labels use whole ems.
    pub(crate) line_offset: f64,
    /// Whether map label collision may hide it.
    pub(crate) may_hide: bool,
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
pub(crate) struct HandleSpec {
    /// Stable identity, used to apply an edit.
    pub(crate) id: HandleId,
    /// What it changes.
    pub(crate) kind: HandleKind,
    /// Where it is drawn.
    pub(crate) at: GeoPoint,
}

/// A graphic constructed in geographic space. Depends only on the
/// definition and configuration, so it is cached across view changes.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Construction {
    /// [`crate::RENDERER_VERSION`] that built it.
    pub(crate) renderer_version: &'static str,
    /// The definition it was built from.
    pub(crate) definition: GraphicId,
    /// The definition's revision.
    pub(crate) revision: u64,
    /// Drawn parts, in drawing order.
    pub(crate) parts: Vec<GeoPart>,
    /// Pixel-sized decorations.
    pub(crate) decorations: Vec<ScreenDecoration>,
    /// Labels.
    pub(crate) labels: Vec<LabelSpec>,
    /// Edit handles.
    pub(crate) handles: Vec<HandleSpec>,
    /// Single-point symbols the graphic embeds, for the host to draw.
    pub(crate) symbols: Vec<EmbeddedSymbol>,
}

impl Construction {
    /// The definition it was built from.
    pub fn definition(&self) -> &GraphicId {
        &self.definition
    }

    /// The definition's revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The [`crate::RENDERER_VERSION`] that built it.
    pub fn renderer_version(&self) -> &'static str {
        self.renderer_version
    }
}

/// A single-point symbol drawn as part of a graphic: the unit assigned a
/// task (amplifier `A`) or an icon the standard puts inside an area.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub(crate) struct EmbeddedSymbol {
    /// The symbol to draw.
    pub(crate) symbol: crate::sidc::SymbolId,
    /// Where its centre goes.
    pub(crate) anchor: GeoPoint,
    /// Pixels to move it from the anchor once projected, y downward.
    pub(crate) offset_px: [f64; 2],
    /// How large it is drawn.
    pub(crate) size: SymbolSize,
}

/// How large an embedded symbol is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub(crate) enum SymbolSize {
    /// A fixed size in pixels.
    Pixels(f64),
    /// Fitted inside the circle centred on the anchor through `edge`.
    WithinCircle {
        /// A point on the circle.
        edge: GeoPoint,
    },
}
