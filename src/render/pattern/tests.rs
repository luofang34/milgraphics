use super::*;
use crate::pick::{PickRef, PickTarget};
use crate::style::Rgba;

fn ring(points: &[(f64, f64)]) -> Vec<ScreenPoint> {
    points.iter().map(|&(x, y)| ScreenPoint { x, y }).collect()
}

fn square(size: f64) -> Vec<ScreenPoint> {
    ring(&[(0.0, 0.0), (size, 0.0), (size, size), (0.0, size)])
}

fn dots(spacing: f64, staggered: bool) -> Pattern {
    Pattern::new(Motif::Dot, Rgba::BLACK, 4.0, [spacing, spacing], staggered)
}

#[test]
fn dots_fill_a_square_on_the_grid_from_its_first_vertex() {
    let centres = centres(&square(100.0), &dots(10.0, false));
    // Grid points 10..=90 on both axes: a dot on the edge would stick out.
    assert_eq!(centres.len(), 81);
    for c in &centres {
        assert!((c.x / 10.0).fract().abs() < 1e-9 && (c.y / 10.0).fract().abs() < 1e-9);
    }
}

#[test]
fn a_staggered_pattern_adds_a_figure_in_each_cell() {
    let plain = centres(&square(100.0), &dots(20.0, false)).len();
    let staggered = centres(&square(100.0), &dots(20.0, true));
    assert_eq!(staggered.len(), plain + 25);
    assert!(
        staggered
            .iter()
            .any(|c| (c.x - 10.0).abs() < 1e-9 && (c.y - 10.0).abs() < 1e-9)
    );
}

#[test]
fn figures_stay_whole_inside_a_concave_area() {
    // A U shape, open at the top between x = 30 and 70.
    let u = ring(&[
        (0.0, 0.0),
        (30.0, 0.0),
        (30.0, 70.0),
        (70.0, 70.0),
        (70.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
    ]);
    let hash = Pattern::new(Motif::Hash, Rgba::BLACK, 8.0, [10.0, 10.0], false);
    let centres = centres(&u, &hash);
    assert!(!centres.is_empty());
    for c in &centres {
        let in_gap = c.x > 26.0 && c.x < 74.0 && c.y < 74.0;
        assert!(!in_gap, "{c:?} lies in the notch");
    }
}

#[test]
fn a_degenerate_pattern_or_area_draws_nothing() {
    assert!(centres(&square(100.0), &dots(0.0, false)).is_empty());
    assert!(centres(&square(100.0), &dots(f64::NAN, false)).is_empty());
    assert!(centres(&ring(&[(0.0, 0.0), (10.0, 0.0)]), &dots(1.0, false)).is_empty());
}

#[test]
fn a_huge_area_is_capped() {
    let centres = centres(&square(1.0e6), &dots(2.0, true));
    assert_eq!(centres.len(), MAX_FIGURES);
}

fn area(fill: Fill) -> ScreenItem {
    ScreenItem {
        pick: PickRef {
            definition: crate::GraphicId::new("g").unwrap(),
            target: PickTarget::Part(crate::construction::PartId(0)),
        },
        role: PartRole::Boundary,
        shape: ScreenShape::Polygon(square(100.0)),
        stroke: None,
        fill,
        decoration: false,
    }
}

#[test]
fn stroked_figures_are_lines_and_dots_are_filled_rings() {
    let kelp = Pattern::new(Motif::Kelp, Rgba::RED, 20.0, [30.0, 30.0], false);
    let lines = items([area(Fill::Pattern(kelp))].iter(), &[]);
    assert!(!lines.is_empty());
    for l in &lines {
        assert_eq!(l.role, PartRole::Pattern);
        assert!(matches!(l.shape, ScreenShape::Polyline(_)));
        assert_eq!(l.stroke.map(|s| s.color), Some(Rgba::RED));
    }
    let filled = items([area(Fill::Pattern(dots(10.0, false)))].iter(), &[]);
    assert_eq!(filled.len(), 81);
    assert!(
        filled
            .iter()
            .all(|d| d.fill == Fill::Solid(Rgba::BLACK) && d.stroke.is_none())
    );
}

#[test]
fn figures_under_a_label_are_left_out() {
    let label = [
        ScreenPoint { x: 35.0, y: 35.0 },
        ScreenPoint { x: 65.0, y: 35.0 },
        ScreenPoint { x: 65.0, y: 65.0 },
        ScreenPoint { x: 35.0, y: 65.0 },
    ];
    let all = items([area(Fill::Pattern(dots(10.0, false)))].iter(), &[]);
    let clear = items([area(Fill::Pattern(dots(10.0, false)))].iter(), &[label]);
    // The 3 x 3 dots at 40..=60 overlap the box.
    assert_eq!(all.len() - clear.len(), 9);
}
