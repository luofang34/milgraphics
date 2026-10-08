//! Pattern fills: copies of a small figure on a screen grid, inside an area.

use crate::construction::PartRole;
use crate::render::{ScreenItem, ScreenPoint, ScreenShape};
use crate::style::{DashPattern, Fill, Motif, Pattern, Stroke};

/// Most figures one area may draw; a larger area keeps its outline and the
/// figures that fit, so a graphic zoomed far in stays cheap.
const MAX_FIGURES: usize = 2000;
/// Vertices of a dot's outline.
const DOT_VERTICES: usize = 12;

/// A figure's strokes in a frame one figure wide, centred on its grid point,
/// y downward: open polylines, or for a dot one filled ring.
struct Figure {
    lines: Vec<Vec<(f64, f64)>>,
    /// Half its width and height, in figure widths.
    half: (f64, f64),
    /// Stroke width, in figure widths; zero for a filled figure.
    width: f64,
}

fn figure(motif: Motif) -> Figure {
    match motif {
        Motif::Dot => {
            let ring = (0..DOT_VERTICES)
                .map(|i| {
                    let a = i as f64 * core::f64::consts::TAU / DOT_VERTICES as f64;
                    (0.5 * a.cos(), 0.5 * a.sin())
                })
                .collect();
            Figure {
                lines: vec![ring],
                half: (0.5, 0.5),
                width: 0.0,
            }
        }
        Motif::Hash => Figure {
            lines: vec![
                vec![(-0.07, -0.53), (-0.27, 0.53)],
                vec![(0.27, -0.53), (0.07, 0.53)],
                vec![(-0.4, -0.17), (0.5, -0.17)],
                vec![(-0.5, 0.2), (0.4, 0.2)],
            ],
            half: (0.5, 0.53),
            width: 0.14,
        },
        Motif::Kelp => scaled(
            94.0,
            (47.0, 15.5),
            &[
                &[
                    (0.0, 5.0),
                    (13.0, 10.0),
                    (30.0, 16.0),
                    (46.0, 11.0),
                    (62.0, 18.0),
                    (75.0, 17.0),
                    (87.0, 16.0),
                    (93.0, 12.0),
                ],
                &[(10.0, 9.0), (4.0, 14.0)],
                &[(30.0, 16.0), (14.0, 27.0)],
                &[(46.0, 11.0), (39.0, 0.0)],
                &[(62.0, 18.0), (47.0, 30.0)],
            ],
            3.0,
        ),
        Motif::FishTrap => scaled(
            62.0,
            (31.0, 25.5),
            &[
                &[
                    (1.0, 21.0),
                    (61.0, 21.0),
                    (61.0, 50.0),
                    (1.0, 50.0),
                    (1.0, 21.0),
                ],
                &[(25.0, 21.0), (4.0, 1.0)],
            ],
            3.0,
        ),
    }
}

/// A figure drawn in pixels on an image `size` wide, centred on `centre`.
fn scaled(size: f64, centre: (f64, f64), lines: &[&[(f64, f64)]], width: f64) -> Figure {
    let map = |&(x, y): &(f64, f64)| ((x - centre.0) / size, (y - centre.1) / size);
    let lines: Vec<Vec<(f64, f64)>> = lines.iter().map(|l| l.iter().map(map).collect()).collect();
    let half = lines
        .iter()
        .flatten()
        .fold((0.0_f64, 0.0_f64), |(hx, hy), &(x, y)| {
            (hx.max(x.abs()), hy.max(y.abs()))
        });
    Figure {
        lines,
        half,
        width: width / size,
    }
}

/// Grid points of `pattern` whose whole figure lies inside `ring` (closed,
/// first point not repeated) by the even-odd rule. The grid starts at the
/// ring's first vertex, so the pattern moves with the graphic.
pub(crate) fn centres(ring: &[ScreenPoint], pattern: &Pattern) -> Vec<ScreenPoint> {
    let (Some(&origin), true) = (ring.first(), ring.len() >= 3) else {
        return Vec::new();
    };
    let [sx, sy] = pattern.spacing_px;
    let size = pattern.size_px;
    if !(sx.is_finite() && sy.is_finite() && size.is_finite() && sx > 1.0 && sy > 1.0) {
        return Vec::new();
    }
    let (lo, hi) = ring.iter().fold(
        ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN)),
        |(lo, hi), p| {
            (
                (lo.0.min(p.x), lo.1.min(p.y)),
                (hi.0.max(p.x), hi.1.max(p.y)),
            )
        },
    );
    let fig = figure(pattern.motif);
    let (hx, hy) = (fig.half.0 * size, fig.half.1 * size);
    let offsets: &[(f64, f64)] = if pattern.staggered {
        &[(0.0, 0.0), (0.5, 0.5)]
    } else {
        &[(0.0, 0.0)]
    };
    let cell = |v: f64, o: f64, s: f64| ((v - o) / s).floor() as i64;
    let mut out = Vec::new();
    for j in cell(lo.1, origin.y, sy)..=cell(hi.1, origin.y, sy) {
        for i in cell(lo.0, origin.x, sx)..=cell(hi.0, origin.x, sx) {
            for &(ox, oy) in offsets {
                if out.len() >= MAX_FIGURES {
                    return out;
                }
                let c = ScreenPoint {
                    x: origin.x + (i as f64 + ox) * sx,
                    y: origin.y + (j as f64 + oy) * sy,
                };
                let corners = [(-hx, -hy), (hx, -hy), (hx, hy), (-hx, hy), (0.0, 0.0)];
                let inside = corners.iter().all(|&(dx, dy)| {
                    contains(
                        ring,
                        ScreenPoint {
                            x: c.x + dx,
                            y: c.y + dy,
                        },
                    )
                });
                if inside {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// Even-odd point in polygon.
fn contains(ring: &[ScreenPoint], p: ScreenPoint) -> bool {
    let mut inside = false;
    for (i, a) in ring.iter().enumerate() {
        let Some(b) = ring.get(i + 1).or(ring.first()) else {
            continue;
        };
        if (a.y <= p.y) != (b.y <= p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if x > p.x {
                inside = !inside;
            }
        }
    }
    inside
}

/// The figures of every pattern-filled area among `items`, as decorations,
/// leaving the label `boxes` clear so their text stays legible.
pub(crate) fn items<'a>(
    items: impl Iterator<Item = &'a ScreenItem>,
    boxes: &[[ScreenPoint; 4]],
) -> Vec<ScreenItem> {
    let mut out = Vec::new();
    for item in items {
        let (Fill::Pattern(pattern), ScreenShape::Polygon(ring)) = (item.fill, &item.shape) else {
            continue;
        };
        let fig = figure(pattern.motif);
        let size = pattern.size_px;
        let (hx, hy) = (fig.half.0 * size, fig.half.1 * size);
        let stroke = (fig.width > 0.0).then_some(Stroke {
            color: pattern.color,
            width_px: fig.width * size,
            dash: DashPattern::Solid,
        });
        let fill = if fig.width > 0.0 {
            Fill::None
        } else {
            Fill::Solid(pattern.color)
        };
        for c in centres(ring, &pattern) {
            if boxes.iter().any(|b| overlaps(b, c, hx, hy)) {
                continue;
            }
            for line in &fig.lines {
                let points: Vec<ScreenPoint> = line
                    .iter()
                    .map(|&(x, y)| ScreenPoint {
                        x: c.x + x * size,
                        y: c.y + y * size,
                    })
                    .collect();
                let shape = if fill == Fill::None {
                    ScreenShape::Polyline(points)
                } else {
                    ScreenShape::Polygon(points)
                };
                out.push(ScreenItem {
                    pick: item.pick.clone(),
                    role: PartRole::Pattern,
                    shape,
                    stroke,
                    fill,
                    decoration: true,
                });
            }
        }
    }
    out
}

/// Whether the box `quad` overlaps the figure's bounds around `c`.
fn overlaps(quad: &[ScreenPoint; 4], c: ScreenPoint, hx: f64, hy: f64) -> bool {
    let (lo, hi) = quad.iter().fold(
        ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN)),
        |(lo, hi), p| {
            (
                (lo.0.min(p.x), lo.1.min(p.y)),
                (hi.0.max(p.x), hi.1.max(p.y)),
            )
        },
    );
    c.x + hx > lo.0 && c.x - hx < hi.0 && c.y + hy > lo.1 && c.y - hy < hi.1
}

#[cfg(test)]
mod tests;
