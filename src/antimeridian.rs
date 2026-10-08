//! Splitting geometry at the ±180° meridian.
//!
//! Map engines that index geometry in longitude/latitude draw a segment from
//! 179.9° to -179.9° the long way round the world. Output geometry is
//! therefore cut at the antimeridian. Coordinates are `[lon, lat]` pairs
//! because a cut point on the eastern side must be written as +180.

use crate::geo::GeoPoint;

#[cfg(test)]
mod tests;

/// `[lon, lat]` in degrees.
pub type LonLat = [f64; 2];

/// Longitudes made continuous along the path: each step takes the shorter
/// way round, so values may leave [-180, 180].
fn unwrap(points: &[GeoPoint]) -> Vec<LonLat> {
    let mut out: Vec<LonLat> = Vec::with_capacity(points.len());
    let mut prev: Option<f64> = None;
    for p in points {
        let lon = match prev {
            Some(prev) => prev + crate::geo::wrap_longitude(p.lon() - prev),
            None => p.lon(),
        };
        out.push([lon, p.lat()]);
        prev = Some(lon);
    }
    out
}

/// Which 360°-wide copy of the world a longitude falls in; copy 0 is
/// [-180, 180).
fn world(lon: f64) -> i64 {
    ((lon + 180.0) / 360.0).floor() as i64
}

fn shift(p: LonLat, copies: i64) -> LonLat {
    [p[0] - 360.0 * copies as f64, p[1]]
}

/// Splits an open polyline into pieces that each stay within [-180, 180].
pub fn split_line(points: &[GeoPoint]) -> Vec<Vec<LonLat>> {
    let path = unwrap(points);
    let mut pieces: Vec<Vec<LonLat>> = Vec::new();
    let mut current: Vec<LonLat> = Vec::new();
    let mut copy = path.first().map_or(0, |p| world(p[0]));
    for pair in path.windows(2) {
        let [a, b] = match pair {
            [a, b] => [*a, *b],
            _ => continue,
        };
        if current.is_empty() {
            current.push(shift(a, copy));
        }
        let next = world(b[0]);
        if next != copy {
            let edge = if next > copy { 180.0 } else { -180.0 } + 360.0 * copy as f64;
            let t = (edge - a[0]) / (b[0] - a[0]);
            let lat = a[1] + t * (b[1] - a[1]);
            current.push(shift([edge, lat], copy));
            pieces.push(core::mem::take(&mut current));
            current.push(shift([edge, lat], next));
            copy = next;
        }
        current.push(shift(b, copy));
    }
    if current.len() >= 2 {
        pieces.push(current);
    } else if pieces.is_empty() {
        if let Some(p) = path.first() {
            pieces.push(vec![shift(*p, copy)]);
        }
    }
    pieces
}

/// Splits a closed ring into rings that each stay within [-180, 180].
/// Each output ring repeats its first point at the end.
///
/// Rings that enclose a pole cannot be represented this way; they are
/// returned unsplit, wrapped into [-180, 180] point by point.
pub fn split_ring(points: &[GeoPoint]) -> Vec<Vec<LonLat>> {
    let ring = unwrap(points);
    let span = ring
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p[0]), hi.max(p[0]))
        });
    let closes = match (ring.first(), ring.last()) {
        (Some(f), Some(l)) => (l[0] + crate::geo::wrap_longitude(f[0] - l[0]) - f[0]).abs() < 1.0,
        _ => true,
    };
    if !closes {
        let mut wrapped: Vec<LonLat> = points.iter().map(|p| [p.lon(), p.lat()]).collect();
        close(&mut wrapped);
        return vec![wrapped];
    }
    let (first, last) = (world(span.0), world(span.1));
    let mut out = Vec::new();
    for copy in first..=last {
        let lo = -180.0 + 360.0 * copy as f64;
        let clipped = clip_band(&ring, lo, lo + 360.0);
        if clipped.len() >= 3 {
            let mut piece: Vec<LonLat> = clipped.into_iter().map(|p| shift(p, copy)).collect();
            close(&mut piece);
            out.push(piece);
        }
    }
    out
}

fn close(ring: &mut Vec<LonLat>) {
    if let (Some(&first), Some(&last)) = (ring.first(), ring.last()) {
        if first != last {
            ring.push(first);
        }
    }
}

/// Sutherland–Hodgman clip of a ring to `lo <= lon <= hi`.
fn clip_band(ring: &[LonLat], lo: f64, hi: f64) -> Vec<LonLat> {
    let left = clip_half(ring, |p| p[0] >= lo, lo);
    clip_half(&left, |p| p[0] <= hi, hi)
}

fn clip_half(ring: &[LonLat], inside: impl Fn(LonLat) -> bool, edge: f64) -> Vec<LonLat> {
    let mut out = Vec::with_capacity(ring.len() + 2);
    let Some(&last) = ring.last() else {
        return out;
    };
    let mut prev = last;
    for &cur in ring {
        let crossing = |a: LonLat, b: LonLat| {
            let t = (edge - a[0]) / (b[0] - a[0]);
            [edge, a[1] + t * (b[1] - a[1])]
        };
        match (inside(prev), inside(cur)) {
            (true, true) => out.push(cur),
            (true, false) => out.push(crossing(prev, cur)),
            (false, true) => {
                out.push(crossing(prev, cur));
                out.push(cur);
            }
            (false, false) => {}
        }
        prev = cur;
    }
    out
}
