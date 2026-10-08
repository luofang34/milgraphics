//! Unit tests for the clsMETOC port. Expected values are worked out by hand
//! from the upstream formulas.

use super::bezier::{draw_cubic_bezier2, point_on_segment};
use super::get_shape::get_me_toc_shape;
use super::ice_openings::extrapolate_point_from_curve;
use super::parallel_lines::parallel_lines2;
use super::path::GeneralPath;
use super::properties::set_metoc_properties;
use super::shape_properties::set_shape_properties;
use super::{MetocSupport, Partition, scaled_size};
use crate::engine::base::{EngineError, PathOp, Pt, Shape, shape_type};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::style::Rgba;

/// Stands in for the code clsMETOC calls outside its file.
struct Support {
    partitions: Vec<Partition>,
}

impl MetocSupport for Support {
    fn line_array(&self, tg: &mut Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
        let mut shape = Shape::new(shape_type::POLYLINE);
        for p in &tg.pixels {
            shape.path.push(PathOp::LineTo(p.x, p.y));
        }
        shapes.push(shape);
        Ok(())
    }

    fn partitions(&self, _tg: &Tg) -> Result<Vec<Partition>, EngineError> {
        Ok(self.partitions.clone())
    }

    /// One edge 5 pixels below the line, then one 5 above, as `x, y, 0`.
    fn channel_points(
        &self,
        line: &[f64],
        out: &mut [f64],
        _channel_width: i32,
    ) -> Result<(), EngineError> {
        let n = line.len() / 2;
        for (j, xy) in line.chunks_exact(2).enumerate() {
            let (x, y) = (
                xy.first().copied().unwrap_or(0.0),
                xy.get(1).copied().unwrap_or(0.0),
            );
            for (slot, dy) in [(j, 5.0), (n + j, -5.0)] {
                if let Some(o) = out.get_mut(3 * slot..3 * slot + 3) {
                    o.copy_from_slice(&[x, y + dy, 0.0]);
                }
            }
        }
        Ok(())
    }
}

fn no_partitions() -> Support {
    Support { partitions: vec![] }
}

fn tg_with(line_type: i32, pixels: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = line_type;
    tg.line_thickness = 3;
    tg.pixels = pixels.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

#[test]
fn scaled_size_grows_above_width_three_and_caps_at_one_hundred() {
    assert_eq!(scaled_size(10.0, 3.0, 1.0), 10.0);
    assert_eq!(scaled_size(10.0, 5.0, 1.0), 20.0);
    assert_eq!(scaled_size(10.0, 200.0, 1.0), 10.0 * (1.0 + 48.5));
}

#[test]
fn point_on_segment_interpolates() {
    let p = point_on_segment(Pt::new(0.0, 0.0), Pt::new(8.0, 4.0), 0.25);
    assert_eq!((p.x, p.y), (2.0, 1.0));
}

#[test]
fn path_replaces_a_move_before_a_move_and_rounds_to_single_precision() {
    let mut path = GeneralPath::new();
    assert!(path.line_to(1.0, 1.0).is_err());
    path.move_to(1.0, 1.0);
    path.move_to(0.1, 0.2);
    path.line_to(2.0, 3.0).ok();
    assert_eq!(
        path.into_path_ops(),
        vec![
            PathOp::MoveTo(f64::from(0.1_f32), f64::from(0.2_f32)),
            PathOp::LineTo(2.0, 3.0),
        ]
    );
}

#[test]
fn solid_curve_types_draw_only_the_cubic() {
    let tg = tg_with(ISOBAR, &[(0.0, 0.0), (120.0, 0.0)]);
    let mut path = GeneralPath::new();
    let p = |x| Pt::new(x, 0.0);
    let samples = draw_cubic_bezier2(&tg, &mut path, p(0.0), p(30.0), p(90.0), p(120.0));
    assert_eq!(samples.map(|s| s.len()), Ok(0));
    assert_eq!(path.into_path_ops(), vec![PathOp::MoveTo(0.0, 0.0)]);
}

#[test]
fn bezier_samples_a_straight_curve_at_the_increment() {
    // Collinear controls at 0, 30, 90, 120 give the anchors
    // Pa_1 = 27.1875, Pa_2 = 60, Pa_3 = 92.8125 and pieces of 27.1875,
    // 32.8125, 32.8125 and 27.1875 pixels: 2, 3, 3 and 2 samples of 10.
    let tg = tg_with(ISOBAR_GE, &[(0.0, 0.0), (120.0, 0.0)]);
    let mut path = GeneralPath::new();
    let p = |x| Pt::new(x, 0.0);
    let samples =
        draw_cubic_bezier2(&tg, &mut path, p(0.0), p(30.0), p(90.0), p(120.0)).unwrap_or_default();
    assert_eq!(samples.len(), 10);
    let xs: Vec<f64> = samples.iter().map(|s| s.x).collect();
    assert_eq!(xs.first().copied(), Some(0.0));
    assert_eq!(xs.get(2).copied(), Some(27.1875));
    assert_eq!(xs.get(5).copied(), Some(60.0));
    assert_eq!(xs.get(8).copied(), Some(92.8125));
    assert!(xs.windows(2).all(|w| w[0] < w[1]));
    assert!(samples.iter().all(|s| s.y == 0.0));
}

#[test]
fn extrapolation_finds_the_level_point_on_the_other_curve() {
    let spline = [Pt::new(0.0, 5.0), Pt::new(10.0, 5.0)];
    let hit = extrapolate_point_from_curve(&spline, Pt::new(4.0, 0.0), 0.0);
    assert_eq!(hit.map(|p| (p.x, p.y)), Some((4.0, 5.0)));
    assert_eq!(
        extrapolate_point_from_curve(&spline, Pt::new(20.0, 0.0), 0.0),
        None
    );
}

#[test]
fn parallel_lines_split_into_two_edges_and_survive_a_failing_builder() {
    let pixels = [Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)];
    let edges = parallel_lines2(&pixels, 20, &no_partitions());
    let ys: Vec<f64> = edges.iter().map(|p| p.y).collect();
    assert_eq!(ys, vec![5.0, 5.0, -5.0, -5.0]);

    struct Failing;
    impl MetocSupport for Failing {
        fn line_array(&self, _: &mut Tg, _: &mut Vec<Shape>) -> Result<(), EngineError> {
            Ok(())
        }
        fn partitions(&self, _: &Tg) -> Result<Vec<Partition>, EngineError> {
            Ok(vec![])
        }
        fn channel_points(&self, _: &[f64], _: &mut [f64], _: i32) -> Result<(), EngineError> {
            Err(EngineError::Degenerate("no channel"))
        }
    }
    assert_eq!(parallel_lines2(&pixels, 20, &Failing).len(), 4);
}

#[test]
fn properties_follow_appendix_c() {
    let settings = Settings::default();
    let mut tg = tg_with(BEACH, &[]);
    set_metoc_properties(&mut tg, &settings);
    assert_eq!(tg.fill_color.map(|c| c.a), Some(30));

    let mut tg = tg_with(SWEPT_AREA, &[]);
    tg.fill_color = Some(Rgba::RED);
    set_metoc_properties(&mut tg, &settings);
    assert_eq!((tg.line_color, tg.fill_color), (None, Some(Rgba::RED)));

    let mut tg = tg_with(CANAL, &[]);
    set_metoc_properties(&mut tg, &settings);
    assert_eq!(tg.line_thickness, 6);

    let mut tg = tg_with(TURBULENCE, &[]);
    tg.line_thickness = 1;
    set_metoc_properties(&mut tg, &settings);
    assert_eq!((tg.line_thickness, tg.line_style), (6, 2));
    let hi_dpi = Settings {
        dpi: 192,
        ..Settings::default()
    };
    let mut tg = tg_with(TURBULENCE, &[]);
    tg.line_thickness = 1;
    set_metoc_properties(&mut tg, &hi_dpi);
    assert_eq!(tg.line_thickness, 12);
}

fn arc_shape() -> Shape {
    let mut shape = Shape::new(shape_type::POLYLINE);
    for i in 0..7 {
        shape.path.push(PathOp::LineTo(f64::from(i) * 10.0, 0.0));
    }
    shape
}

fn stroked(line_type: i32) -> Shape {
    let tg = tg_with(line_type, &[]);
    let mut shapes = vec![arc_shape()];
    set_shape_properties(&tg, &mut shapes);
    shapes.into_iter().next().unwrap_or_default()
}

#[test]
fn trough_dashes_are_capped_by_the_arc() {
    // Arc of 60 pixels, width 3: min(2 * 3, 60 / 4) = 6.
    assert_eq!(stroked(TROUGH).stroke.dash, Some(vec![6.0, 6.0]));
}

#[test]
fn instability_and_shear_put_dots_on_the_arc_peak() {
    // Arc 60, width 3: spacing = min(6, 12) = 6.
    // Instability: dash = (60 - (2 + 18)) / 2 = 20.
    assert_eq!(
        stroked(INSTABILITY).stroke.dash,
        Some(vec![20.0, 6.0, 1.0, 6.0, 1.0, 6.0, 20.0, 0.0])
    );
    // Shear: dash = (60 - (1 + 12)) / 2 = 23.5.
    assert_eq!(
        stroked(SHEAR).stroke.dash,
        Some(vec![23.5, 6.0, 1.0, 6.0, 23.5, 0.0])
    );
}

#[test]
fn pattern_types_report_their_fill_for_the_host() {
    let tg = tg_with(KELP, &[]);
    let mut shapes = vec![arc_shape()];
    let fill = set_shape_properties(&tg, &mut shapes);
    assert_eq!(fill.map(|f| (f.shape_index, f.line_type)), Some((0, KELP)));
}

#[test]
fn line_array_types_go_to_the_support_code_and_get_their_properties() {
    let mut tg = tg_with(SQUALL, &[(0.0, 0.0), (10.0, 0.0)]);
    let mut shapes = Vec::new();
    let fill = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &no_partitions());
    assert_eq!(fill, Ok(None));
    assert_eq!(shapes.len(), 1);
    assert_eq!(shapes.first().and_then(|s| s.line_color), Some(Rgba::BLACK));
}

#[test]
fn google_earth_spline_ends_at_the_last_control_point() {
    let mut tg = tg_with(ISOBAR_GE, &[(0.0, 0.0), (120.0, 0.0)]);
    let mut shapes = Vec::new();
    let result = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &no_partitions());
    assert_eq!(result, Ok(None));
    // The sampled polyline, then the segment from the last sample to the
    // last control point.
    assert_eq!(shapes.len(), 2);
    let spline = shapes.first().map(|s| s.path.clone()).unwrap_or_default();
    assert_eq!(spline.first(), Some(&PathOp::MoveTo(0.0, 0.0)));
    assert_eq!(spline.last(), Some(&PathOp::LineTo(120.0, 0.0)));
    let closing = shapes.get(1).map(|s| s.path.clone()).unwrap_or_default();
    assert_eq!(closing.len(), 2);
    assert_eq!(closing.last(), Some(&PathOp::LineTo(120.0, 0.0)));
}

#[test]
fn a_spline_through_one_point_fails_like_a_missing_move() {
    let mut tg = tg_with(ISOBAR_GE, &[(0.0, 0.0)]);
    let mut shapes = Vec::new();
    let result = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &no_partitions());
    assert!(result.is_err());
    assert!(shapes.is_empty());
}

#[test]
fn leading_line_is_solid_then_dashed() {
    let mut tg = tg_with(LEADING_LINE, &[(0.0, 0.0), (100.0, 0.0)]);
    let mut shapes = Vec::new();
    let result = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &no_partitions());
    assert_eq!(result, Ok(None));
    assert_eq!(shapes.len(), 3);
    assert_eq!(shapes.first().and_then(|s| s.stroke.dash.clone()), None);
    assert_eq!(
        shapes.get(1).and_then(|s| s.stroke.dash.clone()),
        Some(vec![6.0, 6.0])
    );
}

#[test]
fn ice_opening_lead_draws_an_edge_each_side_and_keeps_the_upper_spline() {
    let support = Support {
        partitions: vec![Partition { start: 0, end: 0 }],
    };
    let mut tg = tg_with(ICE_OPENINGS_LEAD, &[(0.0, 0.0), (100.0, 0.0)]);
    let mut shapes = Vec::new();
    let result = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &support);
    assert_eq!(result, Ok(None));
    // The cubics are not part of the output; each edge keeps its start.
    assert_eq!(shapes.len(), 2);
    assert_eq!(
        shapes.first().map(|s| s.path.clone()),
        Some(vec![PathOp::MoveTo(0.0, -5.0)])
    );
    let ys: Vec<f64> = tg.pixels.iter().map(|p| p.y).collect();
    assert_eq!(ys, vec![5.0, 5.0]);
}

#[test]
fn frozen_openings_add_the_cross_lines_after_both_edges() {
    let support = Support {
        partitions: vec![Partition { start: 0, end: 0 }],
    };
    let mut tg = tg_with(ICE_OPENINGS_FROZEN_GE, &[(0.0, 0.0), (100.0, 0.0)]);
    let mut shapes = Vec::new();
    let result = get_me_toc_shape(&mut tg, &mut shapes, &Settings::default(), &support);
    assert_eq!(result, Ok(None));
    assert_eq!(shapes.len(), 3);
    let cross = shapes.get(2).map(|s| s.path.len()).unwrap_or(0);
    assert!(cross >= 4 && cross % 2 == 0);
}
