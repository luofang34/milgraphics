use super::*;
use crate::geo::{Altitude, VerticalDatum};
use crate::modifier::Modifiers;
use crate::sidc::SymbolId;

fn symbol(identity: u8, status: u8, version: u8, entity: u32) -> SymbolId {
    SymbolId::parse(&format!(
        "{version:02}0{identity}25{status}000{entity:06}0000"
    ))
    .unwrap()
}

fn build_with(
    line_type: i32,
    symbol: &SymbolId,
    pixels: &[(f64, f64)],
    modifiers: &Modifiers,
    mpp: f64,
) -> Tg {
    let width = |_: &str| 0.0;
    let input = Input {
        line_type,
        symbol,
        pixels: pixels.iter().map(|&(x, y)| Pt::new(x, y)).collect(),
        modifiers,
        meters_per_pixel: mpp,
        text_width: &width,
        ms_info: None,
        style: crate::engine::api::Style::default(),
    };
    build_tg(&input, &Settings::default(), &Overrides::default()).unwrap()
}

fn pts(n: u8) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| (f64::from(i) * 10.0, f64::from(i % 2) * 5.0))
        .collect()
}

#[test]
fn colours_follow_identity_and_edition() {
    let m = Modifiers::default();
    let color = |identity, version| {
        build_with(PL, &symbol(identity, 0, version, 140_100), &pts(2), &m, 1.0).line_color
    };
    assert_eq!(color(3, 11), Some(Rgba::BLACK));
    assert_eq!(color(6, 11), Some(Rgba::RED));
    assert_eq!(color(5, 11), Some(Rgba::RED));
    assert_eq!(color(5, 15), Some(Rgba::opaque(255, 188, 1)));
    assert_eq!(color(4, 11), Some(Rgba::opaque(0, 255, 0)));
    assert_eq!(color(1, 11), Some(Rgba::opaque(255, 255, 0)));
    let tg = build_with(PL, &symbol(6, 0, 11, 270_300), &pts(2), &m, 1.0);
    assert_eq!(tg.line_color, Some(Rgba::opaque(0, 255, 0)));
    // Text takes the affiliation colour, not the entity's own line colour.
    let tg = build_with(PL, &symbol(3, 0, 11, 200_600), &pts(2), &m, 1.0);
    assert_eq!(tg.line_color, Some(Rgba::opaque(255, 255, 255)));
    assert_eq!(tg.text_color, Some(Rgba::BLACK));
    assert_eq!(
        tg.fill_color,
        Some(Rgba {
            r: 85,
            g: 119,
            b: 136,
            a: 63
        })
    );
}

#[test]
fn status_and_flags() {
    let m = Modifiers::default();
    let tg = build_with(PL, &symbol(3, 1, 11, 140_100), &pts(2), &m, 1.0);
    assert_eq!(tg.line_style, 1);
    assert_eq!(tg.line_thickness, 3);
    assert!(tg.use_dash_array && tg.use_hatch_fill && tg.use_line_interpolation);
    let tg = build_with(PL, &symbol(3, 0, 11, 140_100), &pts(2), &m, 1.0);
    assert_eq!(tg.line_style, 0);
    assert_eq!(tg.fill_color, None);
}

#[test]
fn text_fields_come_from_the_modifiers() {
    let m = Modifiers {
        designation: Some("ALPHA".into()),
        designation2: Some("BRAVO".into()),
        additional_info: Some("h".into()),
        dtg_start: Some("w".into()),
        hostile: Some("ENY".into()),
        location: Some("y".into()),
        ..Modifiers::default()
    };
    let tg = build_with(PL, &symbol(3, 0, 11, 140_100), &pts(2), &m, 1.0);
    assert_eq!(
        (tg.t.as_str(), tg.t1.as_str(), tg.h.as_str()),
        ("ALPHA", "BRAVO", "h")
    );
    assert_eq!(
        (tg.w.as_str(), tg.y.as_str(), tg.n.as_str()),
        ("w", "y", "ENY")
    );
    assert_eq!(tg.h1, "");
}

#[test]
fn areas_are_closed_and_strikwarn_is_two_areas() {
    let m = Modifiers::default();
    let s = symbol(3, 0, 11, 120_200);
    let tg = build_with(AO, &s, &pts(3), &m, 1.0);
    assert_eq!(tg.pixels.len(), 4);
    let tg = build_with(STRIKWARN, &s, &pts(6), &m, 1.0);
    assert_eq!(tg.pixels.len(), 8);
    assert_eq!(tg.pixels.get(3), tg.pixels.first());
    assert_eq!(tg.pixels.get(7), tg.pixels.get(4));
    let tg = build_with(PL, &s, &pts(3), &m, 1.0);
    assert_eq!(tg.pixels.len(), 3);
}

#[test]
fn corridor_widths_become_pixel_half_widths() {
    let m = Modifiers {
        distances_m: vec![1000.0, 2000.0],
        altitudes: vec![Altitude::new(1000.0, VerticalDatum::MeanSeaLevel)],
        ..Modifiers::default()
    };
    let tg = build_with(AC, &symbol(3, 0, 11, 170_100), &pts(3), &m, 10.0);
    let widths: Vec<i32> = tg.pixels.iter().map(|p| p.style).collect();
    // 1000 m and 2000 m at 10 m/px, half each; the third point uses the widest.
    assert_eq!(widths, vec![50, 100, 100]);
    assert_eq!(tg.am, "2000.0 M");
    assert_eq!(tg.x, "3280 FT AMSL");
    assert_eq!(tg.x1, "");
}

#[test]
fn altitude_labels() {
    use super::text::create_altitude_label as label;
    assert_eq!(label(0.0, "AGL"), "GL");
    assert_eq!(label(0.0, "AMSL"), "MSL");
    assert_eq!(label(150.0, "AGL"), "492 FT AGL");
    assert_eq!(label(3048.0, "FL"), "FL 100");
    assert_eq!(label(-300.0, "HAE"), "-984 FT HAE");
}

#[test]
fn sector_range_fan_builds_lrmm() {
    let m = Modifiers {
        distances_m: vec![1000.0, 2000.0],
        azimuths_deg: vec![10.0, 50.0, 100.0, 140.0],
        ..Modifiers::default()
    };
    let tg = build_with(
        RANGE_FAN_SECTOR,
        &symbol(3, 0, 11, 242_200),
        &pts(1),
        &m,
        1.0,
    );
    // The sector count is two, so a leading range of 0 was added.
    assert_eq!(tg.am, "0.0,1000.0,2000.0");
    assert_eq!(tg.an, "10.0,50.0,100.0,140.0");
    assert_eq!(tg.lrmm, "10.0,50.0,0.0,1000.0,100.0,140.0,1000.0,2000.0");
}

#[test]
fn sector_range_fan_without_a_sector_stops_building() {
    let m = Modifiers {
        distances_m: vec![0.0],
        azimuths_deg: vec![10.0, 50.0],
        ..Modifiers::default()
    };
    let tg = build_with(
        RANGE_FAN_SECTOR,
        &symbol(3, 0, 11, 242_200),
        &pts(1),
        &m,
        1.0,
    );
    assert_eq!(tg.lrmm, "");
}

#[test]
fn circular_range_fan_limits_to_three_radii() {
    let m = Modifiers {
        distances_m: vec![100.0, 200.0, 300.0, 400.0],
        altitudes: vec![
            Altitude::new(0.0, VerticalDatum::AboveGround),
            Altitude::new(150.0, VerticalDatum::AboveGround),
        ],
        ..Modifiers::default()
    };
    let tg = build_with(RANGE_FAN, &symbol(3, 0, 11, 242_100), &pts(1), &m, 1.0);
    assert_eq!(tg.am, "100.0,200.0,300.0,");
    assert_eq!(tg.x, "GL,492 FT AGL");
}

#[test]
fn sized_areas_take_their_amplifiers() {
    let m = Modifiers {
        distances_m: vec![3000.0, 1500.0],
        azimuths_deg: vec![30.0],
        ..Modifiers::default()
    };
    let s = symbol(3, 0, 11, 242_300);
    let tg = build_with(LAUNCH_AREA, &s, &pts(1), &m, 1.0);
    assert_eq!(
        (tg.am.as_str(), tg.am1.as_str(), tg.an.as_str()),
        ("3000.0", "1500.0", "30.0")
    );
    let tg = build_with(RECTANGULAR, &s, &pts(1), &m, 1.0);
    assert_eq!(
        (tg.am.as_str(), tg.am1.as_str(), tg.an.as_str()),
        ("3000.0", "1500.0", "30.0")
    );
    let tg = build_with(CIRCULAR, &s, &pts(1), &m, 1.0);
    assert_eq!((tg.am.as_str(), tg.am1.as_str()), ("3000.0", ""));
    let none = Modifiers::default();
    let tg = build_with(RECTANGULAR, &s, &pts(1), &none, 1.0);
    assert_eq!(tg.an, "");
}

#[test]
fn overrides_replace_the_symbol_colours() {
    let m = Modifiers::default();
    let s = symbol(6, 0, 11, 140_100);
    let width = |_: &str| 0.0;
    let input = Input {
        line_type: PL,
        symbol: &s,
        pixels: vec![Pt::new(0.0, 0.0), Pt::new(5.0, 0.0)],
        modifiers: &m,
        meters_per_pixel: 1.0,
        text_width: &width,
        ms_info: None,
        style: crate::engine::api::Style::default(),
    };
    let o = Overrides {
        line_color: Some(Rgba::opaque(1, 2, 3)),
        fill_color: Some(Rgba::opaque(4, 5, 6)),
        text_color: None,
        line_width: Some(5),
    };
    let tg = build_tg(&input, &Settings::default(), &o).unwrap();
    assert_eq!(tg.line_color, Some(Rgba::opaque(1, 2, 3)));
    assert_eq!(tg.text_color, Some(Rgba::opaque(1, 2, 3)));
    assert_eq!(tg.fill_color, Some(Rgba::opaque(4, 5, 6)));
    assert_eq!(tg.line_thickness, 5);
}

#[test]
fn planned_status_dash_matches_the_recorded_stroke() {
    // The oracle records width 3 with dash [6, 6] for planned control measures
    // and black for friendly ones (tests/fixtures/oracle/all.jsonl).
    let tg = build_with(
        PL,
        &symbol(3, 1, 11, 110_100),
        &pts(2),
        &Modifiers::default(),
        1.0,
    );
    let stroke = crate::engine::tg_utility::shape_properties::get_line_stroke(
        tg.line_thickness,
        tg.line_style,
    );
    assert_eq!(stroke.width, 3.0);
    assert_eq!(stroke.dash, Some(vec![6.0, 6.0]));
    assert_eq!(tg.line_color, Some(Rgba::BLACK));
}
