//! Projecting geographic parts: adaptive densification and horizon clipping.

use crate::budget::{BudgetError, VertexMeter};
use crate::construction::GeoGeometry;
use crate::geo::GeoPoint;
use crate::geodesy::Earth;
use crate::render::{Projection, ScreenPoint, ScreenShape};
use crate::style::Fill;

/// Deepest subdivision of one geodesic piece (2^12 sub-pieces).
const MAX_DEPTH: u32 = 12;
/// Bisection steps when locating where a piece crosses the horizon.
const HORIZON_STEPS: u32 = 30;

/// Projection state for one render: geodesics, budget and terrain notes.
pub(crate) struct ScreenCtx<'a> {
    earth: &'a Earth,
    projection: &'a dyn Projection,
    meter: &'a mut VertexMeter,
    terrain_missing: bool,
}

impl<'a> ScreenCtx<'a> {
    pub(crate) fn new(
        earth: &'a Earth,
        projection: &'a dyn Projection,
        meter: &'a mut VertexMeter,
    ) -> Self {
        Self {
            earth,
            projection,
            meter,
            terrain_missing: false,
        }
    }

    pub(crate) fn terrain_missing(&self) -> bool {
        self.terrain_missing
    }

    /// Screen position of a ground-clamped point.
    pub(crate) fn project(&mut self, p: GeoPoint) -> Option<ScreenPoint> {
        let height = match self.projection.terrain_height_m(p) {
            Some(h) => h,
            None => {
                self.terrain_missing = true;
                0.0
            }
        };
        self.projection.project(p, height)
    }

    /// Screen pieces of a part. A ring cut by the horizon is returned as
    /// open polylines without fill.
    pub(crate) fn part(
        &mut self,
        geometry: &GeoGeometry,
        fill: Fill,
    ) -> Result<Vec<(ScreenShape, Fill)>, BudgetError> {
        match geometry {
            GeoGeometry::Line(points) => Ok(self
                .pieces(points)?
                .into_iter()
                .map(|p| (ScreenShape::Polyline(p), Fill::None))
                .collect()),
            GeoGeometry::Ring(points) => {
                let mut closed = points.clone();
                if let Some(first) = points.first() {
                    closed.push(*first);
                }
                let mut pieces = self.pieces(&closed)?;
                let whole = pieces.len() == 1
                    && pieces.first().map_or(0, Vec::len) > closed.len().saturating_sub(1);
                if whole {
                    let mut ring = pieces.pop().unwrap_or_default();
                    ring.pop();
                    Ok(vec![(ScreenShape::Polygon(ring), fill)])
                } else {
                    Ok(pieces
                        .into_iter()
                        .map(|p| (ScreenShape::Polyline(p), Fill::None))
                        .collect())
                }
            }
        }
    }

    /// Visible runs of a geodesic path, densified to the projection tolerance.
    fn pieces(&mut self, points: &[GeoPoint]) -> Result<Vec<Vec<ScreenPoint>>, BudgetError> {
        let mut pieces = Vec::new();
        let mut run: Vec<ScreenPoint> = Vec::new();
        let mut prev: Option<(GeoPoint, Option<ScreenPoint>)> = None;
        for &p in points {
            let s = self.project(p);
            if let Some((a, sa)) = prev {
                self.segment(a, sa, p, s, &mut run, &mut pieces)?;
            } else if let Some(s) = s {
                self.meter.take(1)?;
                run.push(s);
            }
            prev = Some((p, s));
        }
        if run.len() >= 2 {
            pieces.push(run);
        }
        Ok(pieces)
    }

    fn segment(
        &mut self,
        a: GeoPoint,
        sa: Option<ScreenPoint>,
        b: GeoPoint,
        sb: Option<ScreenPoint>,
        run: &mut Vec<ScreenPoint>,
        pieces: &mut Vec<Vec<ScreenPoint>>,
    ) -> Result<(), BudgetError> {
        match (sa, sb) {
            (Some(sa), Some(sb)) => self.subdivide(a, sa, b, sb, 0, run),
            (Some(_), None) => {
                let (edge, s) = self.horizon(a, b, true);
                if let Some(s) = s {
                    self.subdivide_to(a, edge, s, run)?;
                }
                if run.len() >= 2 {
                    pieces.push(core::mem::take(run));
                }
                run.clear();
                Ok(())
            }
            (None, Some(sb)) => {
                let (edge, s) = self.horizon(a, b, false);
                run.clear();
                if let Some(s) = s {
                    self.meter.take(1)?;
                    run.push(s);
                    self.subdivide(edge, s, b, sb, 0, run)
                } else {
                    self.meter.take(1)?;
                    run.push(sb);
                    Ok(())
                }
            }
            (None, None) => Ok(()),
        }
    }

    fn subdivide_to(
        &mut self,
        a: GeoPoint,
        b: GeoPoint,
        sb: ScreenPoint,
        run: &mut Vec<ScreenPoint>,
    ) -> Result<(), BudgetError> {
        match (run.last().copied(), self.project(a)) {
            (Some(sa), _) | (None, Some(sa)) => self.subdivide(a, sa, b, sb, 0, run),
            (None, None) => Ok(()),
        }
    }

    /// Appends the screen points after `sa` up to and including `sb`.
    fn subdivide(
        &mut self,
        a: GeoPoint,
        sa: ScreenPoint,
        b: GeoPoint,
        sb: ScreenPoint,
        depth: u32,
        run: &mut Vec<ScreenPoint>,
    ) -> Result<(), BudgetError> {
        if run.is_empty() {
            self.meter.take(1)?;
            run.push(sa);
        }
        if depth < MAX_DEPTH {
            let m = self.earth.interpolate(a, b, 0.5);
            if let Some(sm) = self.project(m) {
                let chord_mid = ScreenPoint {
                    x: (sa.x + sb.x) / 2.0,
                    y: (sa.y + sb.y) / 2.0,
                };
                let (dx, dy) = sm.sub(chord_mid);
                if dx.hypot(dy) > self.projection.tolerance_px() {
                    self.subdivide(a, sa, m, sm, depth + 1, run)?;
                    return self.subdivide(m, sm, b, sb, depth + 1, run);
                }
            }
        }
        self.meter.take(1)?;
        run.push(sb);
        Ok(())
    }

    /// The last visible point on the geodesic from `a` to `b`, searching from
    /// the visible end. `a_visible` says which end is visible.
    fn horizon(
        &mut self,
        a: GeoPoint,
        b: GeoPoint,
        a_visible: bool,
    ) -> (GeoPoint, Option<ScreenPoint>) {
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        let mut best = (if a_visible { a } else { b }, None);
        for _ in 0..HORIZON_STEPS {
            let mid = (lo + hi) / 2.0;
            let p = self.earth.interpolate(a, b, mid);
            match self.project(p) {
                Some(s) => {
                    best = (p, Some(s));
                    if a_visible { lo = mid } else { hi = mid }
                }
                None => {
                    if a_visible {
                        hi = mid
                    } else {
                        lo = mid
                    }
                }
            }
        }
        best
    }
}
