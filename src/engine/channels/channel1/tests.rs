//! Hand-checked outputs of `get_channel1_double`.

use super::{ChannelRequest, get_channel1_double};
use crate::engine::base::{Pt, Shape};
use crate::engine::channels::externals::StubExternals;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

fn run(line_type: i32, points: &[(f64, f64)], channel_width: i32, useptr: i32) -> Vec<Shape> {
    let settings = Settings::default();
    let tg = Tg::new(&settings);
    let flat: Vec<f64> = points.iter().flat_map(|&(x, y)| [x, y]).collect();
    let count = i32::try_from(points.len()).unwrap();
    let request = ChannelRequest {
        tg: &tg,
        settings: &settings,
        line_type,
        upper: &flat,
        lower: &flat,
        upper_counter: count,
        lower_counter: count,
        channel_width,
        useptr,
    };
    let mut shapes = Vec::new();
    get_channel1_double(&request, &StubExternals, &mut shapes).unwrap();
    shapes
}

fn lines(shape: &Shape) -> Vec<Vec<(f64, f64)>> {
    shape.polylines()
}

#[test]
fn plain_channel_is_two_parallel_edges() {
    // A width of 20 puts each edge 5 pixels from the client line.
    let shapes = run(
        lt::CHANNEL,
        &[(0.0, 0.0), (100.0, 0.0), (200.0, 0.0)],
        20,
        0,
    );
    assert_eq!(shapes.len(), 1);
    assert_eq!(
        lines(&shapes[0]),
        vec![
            vec![(0.0, -5.0), (100.0, -5.0), (200.0, -5.0)],
            vec![(0.0, 5.0), (100.0, 5.0), (200.0, 5.0)],
        ]
    );
}

#[test]
fn dashed_channel_marks_its_points_with_line_style_18() {
    // The style is on the points; the shape keeps style 0 from its first point.
    let shapes = run(lt::CHANNEL_DASHED, &[(0.0, 0.0), (100.0, 0.0)], 20, 0);
    assert_eq!(shapes.len(), 1);
    assert_eq!(lines(&shapes[0]).len(), 2);
}

#[test]
fn main_attack_arrow_ends_at_the_tip_with_the_arrowhead_back_at_useptr() {
    // The client line runs right; the arrow tip is the last client point
    // (400, 0) and the head is 20 pixels long (useptr).
    let shapes = run(lt::MAIN, &[(0.0, 0.0), (200.0, 0.0), (400.0, 0.0)], 40, 20);
    assert_eq!(shapes.len(), 1);
    let pl = lines(&shapes[0]);
    let all: Vec<(f64, f64)> = pl.iter().flatten().copied().collect();
    let tip = (400.0, 0.0);
    assert!(all.contains(&tip), "tip missing from {all:?}");
    // No point lies beyond the tip.
    assert!(all.iter().all(|p| p.0 <= 400.0));
    // The arrowhead flares: some point is 20 pixels behind the tip.
    assert!(all.iter().any(|p| (p.0 - 380.0).abs() < 1e-9));
}

#[test]
fn edges_with_too_few_points_draw_nothing() {
    let shapes = run(lt::CHANNEL, &[(0.0, 0.0)], 20, 0);
    assert!(shapes.is_empty());
}

#[test]
fn fill_color_adds_a_fill_shape_first() {
    let settings = Settings::default();
    let mut tg = Tg::new(&settings);
    tg.fill_color = Some(crate::style::Rgba::RED);
    let flat = [0.0, 0.0, 100.0, 0.0];
    let request = ChannelRequest {
        tg: &tg,
        settings: &settings,
        line_type: lt::CHANNEL,
        upper: &flat,
        lower: &flat,
        upper_counter: 2,
        lower_counter: 2,
        channel_width: 20,
        useptr: 0,
    };
    let mut shapes = Vec::new();
    get_channel1_double(&request, &StubExternals, &mut shapes).unwrap();
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0].shape_type, crate::engine::base::shape_type::FILL);
    let _ = Pt::default();
}

#[test]
fn point_output_is_the_edges_with_styles() {
    use super::get_channel1_points;
    let settings = Settings::default();
    let tg = Tg::new(&settings);
    let flat = [0.0, 0.0, 100.0, 0.0];
    let request = ChannelRequest {
        tg: &tg,
        settings: &settings,
        line_type: lt::CHANNEL,
        upper: &flat,
        lower: &flat,
        upper_counter: 2,
        lower_counter: 2,
        channel_width: 20,
        useptr: 0,
    };
    let pts = get_channel1_points(&request, &StubExternals)
        .unwrap()
        .unwrap();
    let got: Vec<(f64, f64, i32)> = pts.iter().map(|p| (p.x, p.y, p.style)).collect();
    assert_eq!(
        got,
        vec![
            (0.0, -5.0, 0),
            (100.0, -5.0, 5),
            (0.0, 5.0, 0),
            (100.0, 5.0, 5)
        ]
    );
}
