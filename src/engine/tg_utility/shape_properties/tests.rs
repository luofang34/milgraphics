use super::*;
use crate::engine::base::{PathOp, Pt};
use crate::engine::settings::Settings;
use crate::style::Rgba;

fn polyline() -> Shape {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.move_to(Pt::new(0.0, 0.0));
    s.line_to(Pt::new(10.0, 0.0));
    s
}

#[test]
fn stroke_dash_patterns_scale_with_width() {
    assert_eq!(get_line_stroke(3, 0).dash, None);
    assert_eq!(get_line_stroke(3, 1).dash, Some(vec![6.0, 6.0]));
    assert_eq!(get_line_stroke(3, 2).dash, Some(vec![1.0, 6.0]));
    assert_eq!(get_line_stroke(3, 3).dash, Some(vec![12.0, 6.0, 1.0, 6.0]));
    assert_eq!(
        get_line_stroke(2, 4).dash,
        Some(vec![4.0, 4.0, 1.0, 4.0, 1.0, 4.0])
    );
    assert_eq!(get_line_stroke(2, 9).dash, None);
}

#[test]
fn plain_line_takes_graphic_properties() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = PL;
    tg.line_style = 1;
    tg.line_thickness = 0;
    tg.line_color = Some(Rgba::RED);
    let mut shapes = vec![polyline()];
    set_shape_properties(&mut tg, &mut shapes);
    let s = &shapes[0];
    assert_eq!(s.line_color, Some(Rgba::RED));
    assert_eq!(s.style, 1);
    // Zero thickness is drawn as 1 px.
    assert_eq!(s.stroke, get_line_stroke(1, 1));
    assert_eq!(s.path.first(), Some(&PathOp::MoveTo(0.0, 0.0)));
}

#[test]
fn unfilled_axis_of_advance_keeps_its_fill_shapes_unstyled() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = MAIN;
    tg.line_thickness = 2;
    let mut fill = polyline();
    fill.shape_type = shape_type::FILL;
    let untouched = fill.clone();
    let mut shapes = vec![fill, polyline()];
    set_shape_properties(&mut tg, &mut shapes);
    // Upstream filters them into a list only the rest of the method sees.
    assert_eq!(shapes.len(), 2);
    assert_eq!(shapes[0], untouched);
    assert_eq!(shapes[1].stroke.width, 2.0);
}

#[test]
fn direction_of_attack_arrowhead_is_solid() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = DIRATKGND;
    tg.line_style = 1;
    tg.line_thickness = 2;
    tg.fill_color = Some(Rgba::RED);
    let mut shapes = vec![polyline(), polyline()];
    set_shape_properties(&mut tg, &mut shapes);
    assert_eq!(shapes[1].style, 0);
    assert_eq!(shapes[1].stroke.dash, None);
    assert_eq!(shapes[0].stroke.dash, Some(vec![4.0, 4.0]));
}
