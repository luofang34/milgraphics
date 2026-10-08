use super::*;
use crate::engine::settings::Settings;

#[test]
fn hex_colors() {
    assert_eq!(
        color_from_hex_string("#FF0000"),
        Some(Rgba::opaque(255, 0, 0))
    );
    assert_eq!(
        color_from_hex_string("0x80ff00ff"),
        Some(Rgba {
            r: 255,
            g: 0,
            b: 255,
            a: 128
        })
    );
    assert_eq!(color_from_hex_string("12345"), None);
    assert_eq!(color_from_hex_string("GG0000"), None);
    assert_eq!(color_from_hex_string(""), None);
}

#[test]
fn java_split_drops_trailing_empties() {
    assert_eq!(java_split("a,b,,", ','), vec!["a", "b"]);
    assert_eq!(java_split("", ','), vec![""]);
    assert_eq!(java_split(",a", ','), vec!["", "a"]);
}

#[test]
fn segment_colors_by_index() {
    let mut tg = Tg::new(&Settings::default());
    assert_eq!(msr_segment_color_strings(&tg), None);
    tg.line_type = MSR;
    tg.h = "0:FFBBBB,junk,4:FFAAAA".to_owned();
    let strings = msr_segment_color_strings(&tg).unwrap();
    assert_eq!(strings.get(&0).map(String::as_str), Some("FFBBBB"));
    assert_eq!(strings.get(&4).map(String::as_str), Some("FFAAAA"));
    let colors = msr_segment_colors(&tg).unwrap();
    assert_eq!(colors.get(&0), Some(&Some(Rgba::opaque(255, 187, 187))));
    tg.line_type = crate::engine::tactical_lines::PL;
    assert_eq!(msr_segment_colors(&tg), None);
}

#[test]
fn bad_pair_keeps_the_pairs_before_it() {
    let mut tg = Tg::new(&Settings::default());
    tg.line_type = BOUNDARY;
    tg.h = "0:FF0000,x:00FF00,2:0000FF".to_owned();
    let map = msr_segment_color_strings(&tg).unwrap();
    assert_eq!(map.len(), 1);
}
