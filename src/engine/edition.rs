//! Where the templates of version 15 (MIL-STD-2525E change 1) and version 16
//! draw a graphic differently from upstream's renderer. Each change applies
//! to the renderer's finished output, only for the versions whose template
//! shows it, and leaves the output unchanged when its shapes are not the
//! ones upstream draws for the symbol. Every case it changes is listed in
//! `tests/fixtures/oracle/standard.txt`.

use super::api::{Input, Justify, Label, Output};
use super::base::{Pt, Shape, shape_type};
use super::settings::Settings;

mod arrowheads;
mod decision;
mod frontal;
mod knockout;
mod minefield;
mod trip;
mod wire;

#[cfg(test)]
mod tests;

/// Applies the template of `input`'s version to `out`.
pub(crate) fn adjust(input: &Input<'_>, out: &mut Output) {
    let symbol = input.symbol;
    let version = symbol.version_code();
    if symbol.symbol_set() != 25 || !matches!(version, 15 | 16) {
        return;
    }
    match (version, symbol.entity().get()) {
        // Fix: an open arrowhead.
        (_, 270_503) => arrowheads::open_fix(out),
        // Control: solid arrowheads at both ends of the opening.
        (16, 343_200) => arrowheads::fill_control(out),
        // Frontal Attack: the bar at the tip is twice the arrowhead's base.
        (_, 152_700) => frontal::widen_bar(out),
        // Trip Wire: the trip wire glyph at point 1, without the "t".
        (16, 290_500) => trip::glyph(out),
        // Decision Line: a star at each end.
        (16, 110_500) => decision::stars(input, out),
        // Minefield, Dynamic Depiction: H above the area and W below it.
        (16, 270_707) => minefield::above_and_below(input, out),
        // Wire, unspecified: a row of separate X marks.
        (_, 290_301) => wire::spread(input, out),
        // PAA, Mined Area and Minimum Safe Distance Zone: the outline
        // breaks under the labels on it.
        (_, 240_502 | 240_503) => knockout::mark(out, |t| t == "PAA"),
        (_, 270_800 | 270_801) => knockout::mark(out, |t| t == "M"),
        (_, 272_100) => knockout::mark(out, |t| {
            !t.is_empty() && t.chars().all(|c| c.is_ascii_digit())
        }),
        _ => {}
    }
}

/// A unit vector along `(dx, dy)`, or `None` for a zero or non-finite one.
fn unit((dx, dy): (f64, f64)) -> Option<(f64, f64)> {
    let len = dx.hypot(dy);
    (len.is_finite() && len > 0.0).then(|| (dx / len, dy / len))
}

/// A stroked shape styled like `like`, drawing `lines`.
fn lines_like(like: &Shape, lines: &[Vec<(f64, f64)>]) -> Shape {
    let mut shape = like.clone();
    shape.shape_type = shape_type::POLYLINE;
    shape.fill_color = None;
    shape.pattern_fill = None;
    set_lines(&mut shape, lines);
    shape
}

/// Replaces `shape`'s path with `lines`.
fn set_lines(shape: &mut Shape, lines: &[Vec<(f64, f64)>]) {
    shape.path.clear();
    for line in lines {
        let mut points = line.iter();
        if let Some(&(x, y)) = points.next() {
            shape.move_to(Pt::new(x, y));
        }
        for &(x, y) in points {
            shape.line_to(Pt::new(x, y));
        }
    }
}

/// A label centred on `at`. Labels are placed by their baseline, which sits
/// below the middle of the text by 0.3 of the font size.
fn centred(text: &str, at: (f64, f64)) -> Label {
    let size = f64::from(Settings::default().label_font.size);
    Label {
        text: text.to_owned(),
        x: at.0,
        y: at.1 + 0.3 * size,
        angle_deg: 0.0,
        justify: Justify::Center,
        knockout: false,
    }
}
