use serde_json::Value;

use super::fdi::{add_fdi, has_fdi, shapes_mbr};
use super::fills::{add_abatis_fill, lines_with_fill_shapes, resolve_post_clipped_shapes};
use super::hatch::add_hatch_fills;
use super::interpolate::interpolate_pixels;
use super::msr::{get_autoshape_fill_shape, get_msr_shapes};
use super::polylines::{set_spline_linetype, shapes_to_polylines};
use crate::engine::base::{PathOp, Pt, Shape, Stroke, shape_type};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines::*;
use crate::engine::tg::Tg;
use crate::style::Rgba;

fn tg(line_type: i32, pts: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = line_type;
    tg.line_thickness = 3;
    tg.pixels = pts.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

fn poly(shape_kind: i32, pts: &[(f64, f64)]) -> Shape {
    let mut s = Shape::new(shape_kind);
    for (i, &(x, y)) in pts.iter().enumerate() {
        if i == 0 {
            s.move_to(Pt::new(x, y));
        } else {
            s.line_to(Pt::new(x, y));
        }
    }
    s
}

#[test]
fn interpolation_matches_recorded_upstream_output() {
    let cases = include_str!("../cpof/tests/probe_cases.tsv");
    let out = include_str!("../cpof/tests/probe_out.jsonl");
    let mut checked = 0;
    for line in cases.lines().filter(|l| l.contains("\tinterp\t")) {
        let f: Vec<&str> = line.split('\t').collect();
        let lt = match f[2] {
            "FLOT" => FLOT,
            "LC" => LC,
            "ZONE" => ZONE,
            "PL" => PL,
            other => panic!("{other}"),
        };
        let pts: Vec<(f64, f64)> = f[3]
            .split(';')
            .map(|p| {
                let n: Vec<f64> = p.split(',').map(|v| v.parse().unwrap()).collect();
                (n[0], n[1])
            })
            .collect();
        let mut t = tg(lt, &pts);
        interpolate_pixels(&mut t);
        let want: Value = out
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|v| v["id"] == f[0])
            .unwrap();
        let want: Vec<(f64, f64)> = want["pixels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap()))
            .collect();
        let got: Vec<(f64, f64)> = t.pixels.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(got, want, "{}", f[0]);
        checked += 1;
    }
    assert_eq!(checked, 5);
}

#[test]
fn interpolation_is_off_without_the_flag() {
    let mut t = tg(FLOT, &[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]);
    t.use_line_interpolation = false;
    interpolate_pixels(&mut t);
    assert_eq!(t.pixels.len(), 3);
}

#[test]
fn spline_types_switch_to_their_twins() {
    let mut t = tg(ISOBAR, &[]);
    set_spline_linetype(&mut t);
    assert_eq!(t.line_type, ISOBAR_GE);
    let mut t = tg(PL, &[]);
    set_spline_linetype(&mut t);
    assert_eq!(t.line_type, PL);
}

#[test]
fn msr_without_colours_thins_close_points() {
    let mut t = tg(
        MSR,
        &[(0.0, 0.0), (4.0, 0.0), (8.0, 0.0), (30.0, 0.0), (60.0, 0.0)],
    );
    t.line_color = Some(Rgba::BLACK);
    let mut shapes = Vec::new();
    get_msr_shapes(&t, &mut shapes);
    assert_eq!(shapes.len(), 1);
    // 4 and 8 are within 10 px of the last kept point (0); the last pair is
    // always drawn as a piece of its own.
    assert_eq!(
        shapes[0].polylines(),
        vec![
            vec![(0.0, 0.0), (30.0, 0.0)],
            vec![(30.0, 0.0), (60.0, 0.0)]
        ]
    );
}

#[test]
fn msr_coloured_segment_gets_its_own_shape() {
    let mut t = tg(MSR, &[(0.0, 0.0), (50.0, 0.0), (100.0, 0.0), (150.0, 0.0)]);
    t.h = "1:FF0000".to_owned();
    t.line_color = Some(Rgba::BLACK);
    let mut shapes = Vec::new();
    get_msr_shapes(&t, &mut shapes);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].line_color, Some(Rgba::opaque(255, 0, 0)));
    assert_eq!(shapes[0].polylines(), vec![vec![(50.0, 0.0), (100.0, 0.0)]]);
    // Segments next to the coloured one stay in the main shape.
    assert_eq!(shapes[1].line_color, Some(Rgba::BLACK));
    assert_eq!(shapes[1].path.len(), 4);
}

#[test]
fn autoshape_fill_goes_first_and_strips_other_fills() {
    let pts: Vec<(f64, f64)> = (0..30)
        .map(|i| (f64::from(i), f64::from(i * i % 7)))
        .collect();
    let mut t = tg(OCCUPY, &pts);
    t.fill_color = Some(Rgba::opaque(1, 2, 3));
    let mut line = poly(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)]);
    line.fill_color = t.fill_color;
    let mut shapes = vec![line];
    get_autoshape_fill_shape(&t, &mut shapes);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].shape_type, shape_type::FILL);
    assert_eq!(shapes[1].fill_color, None);
    // Points 0 to n-4, then back to 0.
    assert_eq!(shapes[0].path.len(), 1 + 26 + 1);
}

#[test]
fn separate_fill_for_lines_with_glyphs() {
    let mut t = tg(FLOT, &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
    t.fill_color = Some(Rgba {
        r: 1,
        g: 2,
        b: 3,
        a: 100,
    });
    let shapes = lines_with_fill_shapes(&t).unwrap();
    assert_eq!(shapes.len(), 1);
    // The ring is closed.
    assert_eq!(shapes[0].path.len(), 4);
    t.fill_color = Some(Rgba {
        r: 1,
        g: 2,
        b: 3,
        a: 1,
    });
    assert!(lines_with_fill_shapes(&t).is_none());
    t.fill_color = Some(Rgba::RED);
    t.line_type = MAIN;
    assert!(lines_with_fill_shapes(&t).is_none());
}

#[test]
fn abatis_fill_uses_the_last_three_points() {
    let mut t = tg(ABATIS, &[(0.0, 0.0), (1.0, 0.0), (2.0, 5.0), (3.0, 0.0)]);
    t.fill_color = Some(Rgba::RED);
    let mut shapes = vec![Shape::new(shape_type::POLYLINE)];
    add_abatis_fill(&t, &mut shapes);
    assert_eq!(shapes.len(), 2);
    assert_eq!(
        shapes[0].polylines(),
        vec![vec![(1.0, 0.0), (2.0, 5.0), (3.0, 0.0), (1.0, 0.0)]]
    );
}

#[test]
fn buffer_shapes_split_fill_and_hatch_style() {
    let mut t = tg(PBS_SQUARE, &[]);
    t.fill_color = Some(Rgba::RED);
    t.fill_style = 3;
    let mut shapes = vec![Shape::new(shape_type::FILL), Shape::new(shape_type::FILL)];
    shapes[0].fill_style = 9;
    shapes[1].fill_color = Some(Rgba::BLACK);
    resolve_post_clipped_shapes(&t, &mut shapes);
    assert_eq!(shapes[0].fill_color, Some(Rgba::RED));
    assert_eq!(shapes[0].fill_style, 0);
    assert_eq!(shapes[1].fill_color, None);
    assert_eq!(shapes[1].fill_style, 3);
}

#[test]
fn hatch_parameters_follow_the_line_width() {
    let mut t = tg(NFA, &[]);
    t.line_color = Some(Rgba::BLACK);
    t.use_hatch_fill = true;
    let mut shapes = vec![poly(shape_type::FILL, &[(0.0, 0.0), (1.0, 1.0)])];
    shapes[0].fill_style = 3;
    add_hatch_fills(&t, &mut shapes).unwrap();
    let hatch = shapes[0].pattern_fill.unwrap();
    // 3 * 0.75 = 2.25 px lines (2 as an int), spacing int(2.25 * 6).
    assert_eq!((hatch.style, hatch.spacing, hatch.thickness), (3, 13, 2));
    t.use_hatch_fill = false;
    assert!(add_hatch_fills(&t, &mut shapes).is_err());
}

#[test]
fn hatch_is_skipped_for_lines_and_unstyled_areas() {
    let t = tg(PL, &[]);
    let mut shapes = vec![poly(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)])];
    add_hatch_fills(&t, &mut shapes).unwrap();
    assert!(shapes[0].pattern_fill.is_none());
    let t = tg(GENERAL, &[]);
    add_hatch_fills(&t, &mut shapes).unwrap();
    assert!(shapes[0].pattern_fill.is_none());
}

#[test]
fn fdi_on_a_plain_shape_sits_on_top_of_its_bounds() {
    let t = tg(PL, &[]);
    let mut shapes = vec![poly(
        shape_type::POLYLINE,
        &[(0.0, 10.0), (20.0, 10.0), (20.0, 30.0)],
    )];
    let (ul, ur, lr, ll) = shapes_mbr(&shapes).unwrap();
    assert_eq!((ul.x, ul.y, ur.x, ur.y), (0.0, 10.0, 20.0, 10.0));
    assert_eq!((lr.x, lr.y, ll.x, ll.y), (20.0, 30.0, 0.0, 30.0));
    add_fdi(&t, &mut shapes, None, 0);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[1].path.len(), 3);
    assert!(has_fdi(1) && has_fdi(7) && !has_fdi(2));
}

#[test]
fn dashed_area_fill_becomes_its_own_shape() {
    let mut t = tg(GENERAL, &[]);
    t.use_dash_array = true;
    let mut outline = poly(shape_type::FILL, &[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
    outline.fill_color = Some(Rgba::RED);
    outline.stroke = Stroke {
        width: 3.0,
        dash: Some(vec![6.0, 6.0]),
    };
    let mut shapes = vec![outline];
    shapes_to_polylines(&t, &mut shapes);
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].shape_type, shape_type::FILL);
    assert_eq!(shapes[0].fill_color, Some(Rgba::RED));
    assert_eq!(shapes[1].shape_type, shape_type::POLYLINE);
    assert_eq!(shapes[1].fill_color, None);
    // Both are flattened while the shape is still a fill, so both rings are
    // closed.
    assert_eq!(shapes[0].path.len(), 4);
    assert_eq!(shapes[1].path.len(), 4);
}

#[test]
fn polylines_break_at_moves_and_drop_single_points() {
    let t = tg(PL, &[]);
    let mut s = poly(shape_type::POLYLINE, &[(0.0, 0.0), (5.0, 0.0)]);
    s.move_to(Pt::new(9.0, 9.0));
    s.move_to(Pt::new(20.0, 0.0));
    s.line_to(Pt::new(30.0, 0.0));
    let mut shapes = vec![s];
    shapes_to_polylines(&t, &mut shapes);
    assert_eq!(
        shapes[0].path,
        vec![
            PathOp::MoveTo(0.0, 0.0),
            PathOp::LineTo(5.0, 0.0),
            PathOp::MoveTo(20.0, 0.0),
            PathOp::LineTo(30.0, 0.0)
        ]
    );
}

#[test]
fn dashes_are_cut_only_when_the_graphic_does_not_use_a_dash_array() {
    let mut t = tg(PL, &[]);
    let mut s = poly(shape_type::POLYLINE, &[(0.0, 0.0), (20.0, 0.0)]);
    s.line_color = Some(Rgba::BLACK);
    s.stroke = Stroke {
        width: 1.0,
        dash: Some(vec![6.0, 4.0]),
    };
    let mut kept = vec![s.clone()];
    t.use_dash_array = true;
    shapes_to_polylines(&t, &mut kept);
    assert_eq!(kept[0].path.len(), 2);
    let mut cut = vec![s];
    t.use_dash_array = false;
    shapes_to_polylines(&t, &mut cut);
    // 0-6, 10-16 and the part of the next dash up to 20.
    assert_eq!(
        cut[0].polylines(),
        vec![vec![(0.0, 0.0), (6.0, 0.0)], vec![(10.0, 0.0), (16.0, 0.0)],]
    );
}
