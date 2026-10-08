//! A local plane for constructions that are defined by planar offsets.

use crate::geo::GeoPoint;
use crate::geodesy::Earth;

#[cfg(test)]
mod tests;

/// A point in a local plane: metres east (`x`) and north (`y`) of the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Xy {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Xy {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub(crate) fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y)
    }
    pub(crate) fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y)
    }
    pub(crate) fn scale(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k)
    }
    pub(crate) fn len(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub(crate) fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y
    }
    pub(crate) fn cross(self, o: Self) -> f64 {
        self.x * o.y - self.y * o.x
    }
    /// Unit vector, or `None` for a zero or non-finite vector.
    pub(crate) fn unit(self) -> Option<Self> {
        let l = self.len();
        (l.is_finite() && l > 0.0).then(|| self.scale(1.0 / l))
    }
    /// Rotated 90° counter-clockwise (to the left of the direction).
    pub(crate) fn left(self) -> Self {
        Self::new(-self.y, self.x)
    }
}

/// Azimuthal equidistant plane on WGS84 centred at `origin`: distances and
/// azimuths from the origin are exact, and over tactical extents (tens of
/// kilometres) the plane is conformal to well under a metre.
#[derive(Debug)]
pub(crate) struct LocalPlane<'e> {
    earth: &'e Earth,
    origin: GeoPoint,
}

impl<'e> LocalPlane<'e> {
    pub(crate) fn new(earth: &'e Earth, origin: GeoPoint) -> Self {
        Self { earth, origin }
    }

    pub(crate) fn to_xy(&self, p: GeoPoint) -> Xy {
        let inv = self.earth.inverse(self.origin, p);
        let (s, c) = inv.azimuth1.to_radians().sin_cos();
        Xy::new(inv.distance_m * s, inv.distance_m * c)
    }

    pub(crate) fn to_geo(&self, p: Xy) -> GeoPoint {
        let d = p.len();
        if d == 0.0 {
            return self.origin;
        }
        self.earth
            .direct(self.origin, p.x.atan2(p.y).to_degrees(), d)
    }
}

/// Intersection of the lines `a + s·u` and `b + t·v`, or `None` when they
/// are parallel.
pub(crate) fn intersect(a: Xy, u: Xy, b: Xy, v: Xy) -> Option<Xy> {
    let den = u.cross(v);
    if den.abs() < 1e-12 * u.len() * v.len() {
        return None;
    }
    Some(a.add(u.scale(b.sub(a).cross(v) / den)))
}

/// Offset of an open polyline by `distance` to the left (negative: right),
/// with mitred joins and square ends. Consecutive duplicate points are
/// skipped; returns `None` if no segment has length.
pub(crate) fn offset_polyline(points: &[Xy], distance: f64) -> Option<Vec<Xy>> {
    let mut pts: Vec<Xy> = Vec::with_capacity(points.len());
    for &p in points {
        if pts.last().is_none_or(|q| q.sub(p).len() > 1e-9) {
            pts.push(p);
        }
    }
    let dirs: Vec<Xy> = pts
        .windows(2)
        .filter_map(|w| match w {
            [a, b] => b.sub(*a).unit(),
            _ => None,
        })
        .collect();
    let (first, last) = (*dirs.first()?, *dirs.last()?);
    let mut out = Vec::with_capacity(pts.len());
    out.push(pts.first()?.add(first.left().scale(distance)));
    for (i, (d0, d1)) in dirs.iter().zip(dirs.iter().skip(1)).enumerate() {
        let p = *pts.get(i + 1)?;
        let a = p.add(d0.left().scale(distance));
        let b = p.add(d1.left().scale(distance));
        out.push(intersect(a, *d0, b, *d1).unwrap_or(a));
    }
    out.push(pts.last()?.add(last.left().scale(distance)));
    Some(out)
}
