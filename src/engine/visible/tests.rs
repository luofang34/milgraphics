use super::*;

const BOX: PixelBox = PixelBox {
    min_x: 0.0,
    min_y: 0.0,
    max_x: 100.0,
    max_y: 100.0,
};

#[test]
fn a_crossing_segment_is_visible_between_the_edges() {
    let span = BOX.span(Pt::new(-50.0, 50.0), Pt::new(250.0, 50.0));
    assert_eq!(span, Some((50.0, 150.0)));
    assert_eq!(BOX.span(Pt::new(-50.0, 150.0), Pt::new(250.0, 150.0)), None);
}

#[test]
fn repeats_near_the_box_keep_their_indices() {
    let seg = (Pt::new(-1_000.0, 50.0), Pt::new(1_000.0, 50.0));
    // Repeats every 10 px from 5 px: the box covers 1000..1100 along it.
    let r = PixelBox::repeats(Some(&BOX), seg, (5.0, 10.0, 200), 20.0);
    assert_eq!(r, 97..113);
    assert_eq!(PixelBox::repeats(None, seg, (5.0, 10.0, 200), 20.0), 0..200);
    let away = (Pt::new(-1_000.0, 500.0), Pt::new(1_000.0, 500.0));
    assert!(PixelBox::repeats(Some(&BOX), away, (5.0, 10.0, 200), 20.0).is_empty());
}

fn drawn(
    code: &str,
    line_type: i32,
    points: &[(f64, f64)],
    visible: Option<PixelBox>,
) -> Vec<(f64, f64)> {
    let symbol = crate::sidc::SymbolId::parse(code).unwrap();
    let modifiers = crate::modifier::Modifiers::default();
    let width = |t: &str| 7.0 * t.chars().count() as f64;
    let out = crate::engine::api::draw(&crate::engine::api::Input {
        line_type,
        symbol: &symbol,
        pixels: points.iter().map(|&(x, y)| Pt::new(x, y)).collect(),
        modifiers: &modifiers,
        meters_per_pixel: 10.0,
        text_width: &width,
        ms_info: crate::family::ms_info(&symbol),
        style: crate::engine::api::Style::default(),
        visible,
    })
    .unwrap();
    out.shapes
        .iter()
        .flat_map(crate::engine::base::Shape::polylines)
        .flatten()
        .collect()
}

/// Every point drawn with `visible` is one the whole graphic draws, so the
/// pattern does not move with the view; and far fewer are drawn.
fn assert_subset(code: &str, line_type: i32, points: &[(f64, f64)]) {
    let near = PixelBox {
        min_x: 9_000.0,
        min_y: -500.0,
        max_x: 10_000.0,
        max_y: 500.0,
    };
    let all = drawn(code, line_type, points, None);
    let some = drawn(code, line_type, points, Some(near));
    let close = |p: &(f64, f64)| all.iter().any(|q| (p.0 - q.0).hypot(p.1 - q.1) < 1e-6);
    let off: Vec<_> = some.iter().filter(|p| !close(p)).collect();
    assert!(
        off.is_empty(),
        "{code}: {} points not in the whole drawing: {:?}",
        off.len(),
        &off[..off.len().min(5)]
    );
    assert!(
        some.len() * 5 < all.len(),
        "{code}: {} of {}",
        some.len(),
        all.len()
    );
    assert!(
        some.iter().any(|p| p.0 > 9_000.0 && p.0 < 10_000.0),
        "{code}: nothing in view"
    );
}

#[test]
fn zones_drawn_for_a_view_keep_their_spikes_in_place() {
    let ring = [
        (0.0, 0.0),
        (40_000.0, 0.0),
        (40_000.0, 20_000.0),
        (0.0, 20_000.0),
    ];
    assert_subset(
        "15032500002704000000",
        crate::engine::tactical_lines::OBSAREA,
        &ring,
    );
}

#[test]
fn wire_drawn_for_a_view_keeps_its_marks_in_place() {
    let line = [(0.0, 0.0), (20_000.0, 0.0)];
    assert_subset(
        "15032500002903010000",
        crate::engine::tactical_lines::UNSP,
        &line,
    );
}
