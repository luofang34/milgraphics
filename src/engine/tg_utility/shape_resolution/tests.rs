use super::*;
use crate::engine::base::Pt;
use crate::engine::settings::Settings;

fn shape_of(kind: i32, pts: &[(f64, f64)]) -> Shape {
    let mut s = Shape::new(kind);
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
fn closed_area_gets_the_fill() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = NAI;
    tg.fill_color = Some(Rgba::opaque(1, 2, 3));
    tg.line_color = Some(Rgba::RED);
    let mut s = shape_of(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)]);
    resolve_modifier_shape(&mut tg, &mut s).unwrap();
    assert_eq!(s.fill_color, tg.fill_color);
    assert_eq!(s.line_color, Some(Rgba::RED));
}

#[test]
fn range_fan_outline_has_no_fill() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = RANGE_FAN;
    tg.fill_color = Some(Rgba::opaque(1, 2, 3));
    let mut s = shape_of(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)]);
    resolve_modifier_shape(&mut tg, &mut s).unwrap();
    assert_eq!(s.fill_color, None);
}

#[test]
fn area_defense_triangle_is_filled_with_the_line_colour() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = AREA_DEFENSE;
    tg.line_color = Some(Rgba::RED);
    tg.fill_color = Some(Rgba::opaque(9, 9, 9));
    let ring = [(0.0, 0.0), (4.0, 0.0), (2.0, 3.0), (1.0, 1.0), (0.0, 0.0)];
    let mut s = shape_of(shape_type::FILL, &ring);
    resolve_modifier_shape(&mut tg, &mut s).unwrap();
    assert_eq!(s.fill_color, Some(Rgba::RED));
    assert_eq!(s.fill_style, 1);
    let mut open = shape_of(shape_type::FILL, &ring[..4]);
    resolve_modifier_shape(&mut tg, &mut open).unwrap();
    assert_eq!(open.fill_color, Some(Rgba::opaque(9, 9, 9)));
}

#[test]
fn empty_fill_in_area_defense_is_an_error() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = AREA_DEFENSE;
    let mut s = Shape::new(shape_type::FILL);
    assert!(resolve_modifier_shape(&mut tg, &mut s).is_err());
}

#[test]
fn butt_cap_types_set_the_cap() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = EASY;
    let mut s = shape_of(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)]);
    resolve_modifier_shape(&mut tg, &mut s).unwrap();
    assert_eq!(tg.line_cap, CAP_BUTT);
}

#[test]
fn metoc_symbols_are_left_alone() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = CF;
    tg.symbol_id = "110345000011030100000000000000".to_owned();
    tg.line_color = Some(Rgba::RED);
    let mut s = shape_of(shape_type::POLYLINE, &[(0.0, 0.0), (1.0, 1.0)]);
    resolve_modifier_shape(&mut tg, &mut s).unwrap();
    assert_eq!(s.line_color, None);
}

#[test]
fn lc_red_side_follows_affiliation() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = LC;
    tg.line_color = Some(Rgba::opaque(0, 0, 255));
    let mut red = shape_of(shape_type::POLYLINE, &[(0.0, 0.0)]);
    red.line_color = Some(Rgba::RED);
    let mut other = red.clone();
    other.line_color = Some(Rgba::BLACK);
    set_lc_color(&tg, &mut red);
    set_lc_color(&tg, &mut other);
    assert_eq!(red.line_color, Some(Rgba::RED));
    assert_eq!(other.line_color, tg.line_color);
    tg.standard_identity = "06".to_owned();
    let mut red2 = shape_of(shape_type::POLYLINE, &[(0.0, 0.0)]);
    red2.line_color = Some(Rgba::RED);
    set_lc_color(&tg, &mut red2);
    assert_eq!(red2.line_color, tg.line_color);
}
