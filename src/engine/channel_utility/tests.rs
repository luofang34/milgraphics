//! Partitioning and drawing of channels along simple lines.

use super::draw::draw_channel;
use super::partitions::{get_partitions, get_partitions2};
use crate::engine::base::{Pt, Shape};
use crate::engine::channels::externals::StubExternals;
use crate::engine::partition::Partition;
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::{CAP_BUTT, Tg};

fn tg_with(line_type: i32, points: &[(f64, f64)]) -> Tg {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = line_type;
    tg.pixels = points.iter().map(|&(x, y)| Pt::new(x, y)).collect();
    tg
}

#[test]
fn bad_segments_split_the_partitions() {
    assert_eq!(
        get_partitions(&[true, true, true]).unwrap(),
        vec![Partition { start: 0, end: 2 }]
    );
    assert_eq!(
        get_partitions(&[true, false, true]).unwrap(),
        vec![
            Partition { start: 0, end: 0 },
            Partition { start: 1, end: 2 }
        ]
    );
    assert!(get_partitions(&[false, true]).unwrap().is_empty());
    assert!(get_partitions(&[]).is_err());
}

#[test]
fn partitions_of_a_graphic_follow_its_doubled_back_segments() {
    let tg = tg_with(lt::PL, &[(0.0, 0.0), (100.0, 0.0), (0.0, 0.0)]);
    let parts = get_partitions2(&tg).unwrap().unwrap();
    assert_eq!(
        parts,
        vec![
            Partition { start: 0, end: 0 },
            Partition { start: 1, end: 1 }
        ]
    );
    let single = tg_with(lt::PL, &[(0.0, 0.0)]);
    assert!(get_partitions2(&single).unwrap().is_none());
}

#[test]
fn fence_channels_use_a_butt_cap() {
    let mut tg = tg_with(lt::DOUBLEA, &[(0.0, 0.0), (200.0, 0.0)]);
    let mut shapes: Vec<Shape> = Vec::new();
    draw_channel(
        &mut tg,
        lt::DOUBLEA,
        &Settings::default(),
        &StubExternals,
        &mut shapes,
    )
    .unwrap();
    assert_eq!(tg.line_cap, CAP_BUTT);
    assert!(!shapes.is_empty());
}

#[test]
fn unknown_line_type_is_an_error() {
    let mut tg = tg_with(lt::PL, &[(0.0, 0.0), (200.0, 0.0)]);
    let mut shapes: Vec<Shape> = Vec::new();
    assert!(
        draw_channel(
            &mut tg,
            lt::PL,
            &Settings::default(),
            &StubExternals,
            &mut shapes
        )
        .is_err()
    );
}

#[test]
fn arrow_with_fewer_than_three_points_draws_nothing() {
    let mut tg = tg_with(lt::MAIN, &[(0.0, 0.0), (100.0, 0.0)]);
    let mut shapes: Vec<Shape> = Vec::new();
    draw_channel(
        &mut tg,
        lt::MAIN,
        &Settings::default(),
        &StubExternals,
        &mut shapes,
    )
    .unwrap();
    assert!(shapes.is_empty());
}

#[test]
fn counterattack_by_fire_control_point_is_pulled_back_in_place() {
    // The first segment is longer than 45 pixels, so the control point moves
    // 45 pixels further from the point opposite the second point.
    let mut tg = tg_with(lt::CATKBYFIRE, &[(0.0, 0.0), (200.0, 0.0), (100.0, 60.0)]);
    let mut shapes: Vec<Shape> = Vec::new();
    draw_channel(
        &mut tg,
        lt::CATKBYFIRE,
        &Settings::default(),
        &StubExternals,
        &mut shapes,
    )
    .unwrap();
    let control = tg.pixels.last().copied().unwrap();
    assert!((control.x - 55.0).abs() < 1e-9 && (control.y - 60.0).abs() < 1e-9);
}

#[test]
fn small_line_of_contact_becomes_a_horizontal_stub() {
    let mut tg = tg_with(lt::LC, &[(0.0, 0.0), (5.0, 5.0)]);
    let mut shapes: Vec<Shape> = Vec::new();
    draw_channel(
        &mut tg,
        lt::LC,
        &Settings::default(),
        &StubExternals,
        &mut shapes,
    )
    .unwrap();
    // The graphic's own pixels are kept; the stub is only drawn.
    assert_eq!(tg.pixels.len(), 2);
    assert!(!shapes.is_empty());
}
