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
/// Depth of the search for a visible span between two hidden points: a
/// span narrower than 1/2^depth of the edge (densified edges are at most
/// `Config::geodesic_step_m` long) may be missed.
const HIDDEN_PROBE_DEPTH: u32 = 6;

/// Visible runs of a path, the run being extended, and whether anything
/// was cut away.
#[derive(Default)]
struct Runs {
    pieces: Vec<Vec<ScreenPoint>>,
    run: Vec<ScreenPoint>,
    clipped: bool,
}

impl Runs {
    /// Ends the current run, keeping it if it has a length.
    fn cut(&mut self) {
        let run = core::mem::take(&mut self.run);
        if run.len() >= 2 {
            self.pieces.push(run);
        }
    }
}

/// Projection state for one render: geodesics, budget and terrain notes.
pub(crate) struct ScreenCtx<'a> {
    earth: &'a Earth,
    projection: &'a dyn Projection,
    meter: &'a mut VertexMeter,
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
        }
    }

    /// Reserves `count` output vertices from the render budget.
    pub(crate) fn take(&mut self, count: usize) -> Result<(), BudgetError> {
        self.meter.take(count)
    }

    /// Screen position of a ground-clamped point.
    pub(crate) fn project(&mut self, p: GeoPoint) -> Option<ScreenPoint> {
        self.projection.project(p)
    }

    /// Screen pieces of a part. A ring cut by the horizon is returned as
    /// open polylines without fill.
    pub(crate) fn part(
        &mut self,
        geometry: &GeoGeometry,
        fill: Fill,
    ) -> Result<Vec<(ScreenShape, Fill)>, BudgetError> {
        let polylines = |pieces: Vec<Vec<ScreenPoint>>| {
            pieces
                .into_iter()
                .map(|p| (ScreenShape::Polyline(p), Fill::None))
                .collect()
        };
        match geometry {
            GeoGeometry::Line(points) => Ok(polylines(self.pieces(points)?.pieces)),
            GeoGeometry::Ring(points) => {
                let mut closed = points.clone();
                if let Some(first) = points.first() {
                    closed.push(*first);
                }
                let Runs {
                    mut pieces,
                    clipped,
                    ..
                } = self.pieces(&closed)?;
                if !clipped {
                    let mut ring = pieces.pop().unwrap_or_default();
                    ring.pop();
                    return Ok(if ring.len() >= 3 {
                        vec![(ScreenShape::Polygon(ring), fill)]
                    } else {
                        vec![]
                    });
                }
                // The first run starts at the ring's first vertex and the last
                // ends there when that vertex is visible: they are one line.
                let start_visible = points.first().is_some_and(|&p| self.project(p).is_some());
                if start_visible && pieces.len() >= 2 {
                    let first = pieces.remove(0);
                    if let Some(last) = pieces.last_mut() {
                        last.extend(first.into_iter().skip(1));
                    }
                }
                Ok(polylines(pieces))
            }
        }
    }

    /// Visible runs of a geodesic path, densified to the projection tolerance.
    fn pieces(&mut self, points: &[GeoPoint]) -> Result<Runs, BudgetError> {
        let mut runs = Runs::default();
        let mut prev: Option<(GeoPoint, Option<ScreenPoint>)> = None;
        for &p in points {
            let s = self.project(p);
            runs.clipped |= s.is_none();
            if let Some((a, sa)) = prev {
                self.segment(a, sa, p, s, &mut runs, 0)?;
            } else if let Some(s) = s {
                self.meter.take(1)?;
                runs.run.push(s);
            }
            prev = Some((p, s));
        }
        runs.cut();
        Ok(runs)
    }

    fn segment(
        &mut self,
        a: GeoPoint,
        sa: Option<ScreenPoint>,
        b: GeoPoint,
        sb: Option<ScreenPoint>,
        runs: &mut Runs,
        depth: u32,
    ) -> Result<(), BudgetError> {
        match (sa, sb) {
            (Some(sa), Some(sb)) => self.subdivide(a, sa, b, sb, 0, &mut runs.run),
            (Some(_), None) => {
                let (edge, s) = self.horizon(a, b, true);
                if let Some(s) = s {
                    self.subdivide_to(a, edge, s, &mut runs.run)?;
                }
                runs.cut();
                Ok(())
            }
            (None, Some(sb)) => {
                let (edge, s) = self.horizon(a, b, false);
                runs.cut();
                self.meter.take(1)?;
                match s {
                    Some(s) => {
                        runs.run.push(s);
                        self.subdivide(edge, s, b, sb, 0, &mut runs.run)
                    }
                    None => {
                        runs.run.push(sb);
                        Ok(())
                    }
                }
            }
            (None, None) if depth == 0 && !self.projection.segment_may_be_visible(a, b) => Ok(()),
            (None, None) if depth < HIDDEN_PROBE_DEPTH => {
                // Both ends hidden, but the middle may cross the view.
                let m = self.earth.interpolate(a, b, 0.5);
                let sm = self.project(m);
                self.segment(a, None, m, sm, runs, depth + 1)?;
                self.segment(m, sm, b, None, runs, depth + 1)
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
