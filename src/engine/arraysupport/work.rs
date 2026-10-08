//! The working state `GetLineArray2Double` threads through its switch: the
//! point array being rewritten, the untouched control points, the counters
//! and the calculation points every case starts from.

use crate::engine::base::{At, EngineError, Pt, idx};
use crate::engine::flot::get_scaled_size;
use crate::engine::settings::Settings;
use crate::engine::tg::Tg;

/// Upstream `arraysupport.maxLength`: the largest arrow size.
pub(crate) const MAX_LENGTH: f64 = 100.0;
/// Upstream `arraysupport.minLength`: the smallest arrow size.
pub(crate) const MIN_LENGTH: f64 = 2.5;

/// Upstream's locals of `GetLineArray2Double` that outlive one case.
#[derive(Debug)]
pub(crate) struct Work<'a> {
    /// The graphic (line type, thickness, pattern scale, modifiers).
    pub(crate) tg: &'a mut Tg,
    /// Renderer settings (DPI).
    pub(crate) settings: &'a Settings,
    /// `tg.get_LineType()`.
    pub(crate) line_type: i32,
    /// `DPIScaleFactor`: device DPI over 96.
    pub(crate) dpi: f64,
    /// `pLinePoints`: sized like upstream's array (`vblCounter`); the
    /// control points come first and the symbol points replace them.
    pub(crate) p: Vec<Pt>,
    /// `pOriginalLinePoints`: copies of the control points.
    pub(crate) orig: Vec<Pt>,
    /// `vblCounter`: the array size, later the symbol point count for the
    /// types that reassign it.
    pub(crate) vbl: i32,
    /// `vblSaveCounter`: the number of control points.
    pub(crate) save: i32,
    /// `dMBR`: the diagonal of the control points' bounding box.
    pub(crate) d_mbr: f64,
    /// `pt0`: copy of the first control point, style 0.
    pub(crate) pt0: Pt,
    /// `pt1`: copy of the second control point, style 0.
    pub(crate) pt1: Pt,
    /// `pt2`: copy of the third control point (the second when there are
    /// fewer than three slots), style 0.
    pub(crate) pt2: Pt,
    /// `acCounter`: how many points of `p` are the symbol.
    pub(crate) ac: i32,
    /// `points`: the list `GetLineArray2` returns (filled by few types).
    pub(crate) points: Vec<Pt>,
}

impl Work<'_> {
    /// `arraysupport.getScaledSize(size, tg.get_LineThickness(),
    /// tg.get_patternScale())`.
    pub(crate) fn scaled(&self, size: f64) -> f64 {
        scaled_size(self.tg, size)
    }
}

/// `arraysupport.getScaledSize` with a graphic's thickness and pattern scale.
pub(crate) fn scaled_size(tg: &Tg, size: f64) -> f64 {
    get_scaled_size(size, f64::from(tg.line_thickness), tg.pattern_scale)
}

/// Element `i` of a point array, failing like Java's array access.
pub(crate) fn get(v: &[Pt], i: i32) -> Result<Pt, EngineError> {
    v.at(idx(i, v.len())?)
}

/// Mutable element `i` of a point array, failing like Java's array access.
pub(crate) fn at_mut(v: &mut [Pt], i: i32) -> Result<&mut Pt, EngineError> {
    let len = v.len();
    v.at_mut(idx(i, len)?)
}

/// `v[i] = p`, failing like Java's array store.
pub(crate) fn set(v: &mut [Pt], i: i32, p: Pt) -> Result<(), EngineError> {
    *at_mut(v, i)? = p;
    Ok(())
}

/// `v[i].style = style`.
pub(crate) fn set_style(v: &mut [Pt], i: i32, style: i32) -> Result<(), EngineError> {
    at_mut(v, i)?.style = style;
    Ok(())
}
