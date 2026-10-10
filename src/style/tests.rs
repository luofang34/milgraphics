use super::*;

fn sid(code: &str) -> SymbolId {
    SymbolId::parse(code).unwrap()
}

#[test]
fn hex_colours_round_trip() {
    assert_eq!(Rgba::parse_hex("#ff8000"), Some(Rgba::opaque(255, 128, 0)));
    assert_eq!(Rgba::parse_hex("#ff800080").map(|c| c.a), Some(128));
    assert_eq!(Rgba::opaque(1, 2, 3).to_hex(), "#010203ff");
    for bad in ["ff8000", "#ff80", "#gg8000", "#ff8000801", ""] {
        assert_eq!(Rgba::parse_hex(bad), None, "{bad}");
    }
}

#[test]
fn identity_and_status_choose_colour_and_dash() {
    let none = StyleOverrides::default();
    let friend = palette(&sid("11032500001403000000"), &none);
    assert_eq!(friend.line.color, Rgba::BLACK);
    assert_eq!(friend.line.dash, DashPattern::Solid);
    assert_eq!(
        palette(&sid("11062500001403000000"), &none).line.color,
        Rgba::RED
    );
    let planned = palette(&sid("11032510001403000000"), &none);
    assert_eq!(planned.line.dash, DashPattern::Dashed);
    assert_eq!(planned.solid_line.dash, DashPattern::Solid);
}

#[test]
fn overrides_win() {
    let style = StyleOverrides {
        line_color: Some(Rgba::opaque(0, 255, 0)),
        fill_color: Rgba::parse_hex("#0000ff40"),
        ..StyleOverrides::default()
    };
    let p = palette(&sid("11032500001202000000"), &style);
    assert_eq!(p.line.color, Rgba::opaque(0, 255, 0));
    assert_eq!(
        p.fill,
        Some(Rgba {
            r: 0,
            g: 0,
            b: 255,
            a: 64
        })
    );
}

#[test]
fn stored_hex_is_six_digits_when_opaque() {
    assert_eq!(Rgba::opaque(0x11, 0x22, 0x33).to_stored_hex(), "#112233");
    let translucent = Rgba::parse_hex("#11223340").unwrap();
    assert_eq!(translucent.to_stored_hex(), "#11223340");
    for text in ["#112233", "#11223340"] {
        assert_eq!(Rgba::parse_hex(text).unwrap().to_stored_hex(), text);
    }
}
