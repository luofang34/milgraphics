//! Where the templates of version 15 (MIL-STD-2525E change 1), version 16
//! and, for code 10, base MIL-STD-2525D draw a graphic differently from
//! upstream's renderer. Each change applies to the renderer's finished
//! output, only for the versions whose template shows it, and leaves the
//! output unchanged when its shapes are not the ones upstream draws for the
//! symbol. Every case it changes is listed in
//! `tests/fixtures/oracle/standard.txt`.

use super::api::{Input, Justify, Label, Output};
use super::base::{Pt, Shape, shape_type};
use super::settings::Settings;

mod arrowheads;
mod avenue;
mod decision;
mod frontal;
mod knockout;
mod minefield;
mod mobility;
mod psyops;
mod rhumb;
mod terrain;
mod trip;
mod wire;

#[cfg(test)]
mod tests;

/// The control points upstream is given in place of `input`'s, when a
/// change below replaces its output whatever it draws.
pub(crate) fn upstream_pixels(input: &Input<'_>) -> Option<Vec<Pt>> {
    if spreads_wire(input.symbol) {
        return wire::stub(input);
    }
    None
}

/// Whether `symbol` is drawn as spaced wire marks, which keep to the
/// visible box whatever the line's length.
pub(crate) fn spreads_wire(symbol: &crate::sidc::SymbolId) -> bool {
    symbol.symbol_set() == 25
        && matches!(symbol.version_code(), 15 | 16)
        && symbol.entity().get() == 290_301
}

/// Applies the template of `input`'s version to `out`.
pub(crate) fn adjust(input: &Input<'_>, out: &mut Output) {
    let symbol = input.symbol;
    let version = symbol.version_code();
    // Code 10 is also base MIL-STD-2525D, whose Trip Wire is a glyph on
    // three points.
    if (version, symbol.symbol_set(), symbol.entity().get()) == (10, 25, 290_500) {
        trip::glyph_on_points(input, out);
        return;
    }
    if symbol.symbol_set() != 25 || !matches!(version, 15 | 16) {
        return;
    }
    match (version, symbol.entity().get()) {
        // Fix: an open arrowhead.
        (_, 270_503) => arrowheads::open_fix(out),
        // Control: solid arrowheads at both ends of the opening, and the C
        // outside the circle.
        (16, 343_200) => {
            arrowheads::fill_control(out);
            arrowheads::control_label_outside(input, out);
        }
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
        (16, entity) => app6e_codes(input, out, entity),
        _ => {}
    }
}

/// Version 16 codes upstream has no line type for, drawn with another
/// code's line type (`line_type::control_measures`).
fn app6e_codes(input: &Input<'_>, out: &mut Output, entity: u32) {
    match entity {
        // Artillery Manoeuvre and Reserved Areas: the outline breaks under
        // the label at each side.
        242_400 => knockout::mark(out, |t| t == "AMA"),
        242_500 => knockout::mark(out, |t| t == "ARA"),
        // Human Terrain: H under "HT".
        370_100 => terrain::human(input, out),
        // Restricted Terrain hatched, Severely Restricted cross hatched.
        152_400 => terrain::restricted(input, out, false),
        152_500 => terrain::restricted(input, out, true),
        // PsyOps Zones: a loudspeaker beside H over T.
        242_701..=242_703 => psyops::speaker(input, out),
        // Avenue of Approach: "AA T" in the axis, H and N beside it.
        152_300 => avenue::labels(input, out),
        // Mobility Corridor: forks at the ends, B and H on each segment.
        142_100 => mobility::corridor(input, out),
        // Navigational Rhumb Line: AN along the line, T boxed across it.
        220_109 => rhumb::labels(input, out),
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

/// The text of a field, unless it is unset or empty.
fn text(field: &Option<String>) -> Option<&str> {
    field.as_deref().filter(|t| !t.is_empty())
}

/// Label font size in pixels: also the distance between stacked lines.
fn font_px() -> f64 {
    f64::from(Settings::default().label_font.size)
}

/// The centre of the bounds of `points`.
fn bounds_centre(points: &[Pt]) -> Option<(f64, f64)> {
    let first = points.first()?;
    let (mut left, mut right, mut top, mut bottom) = (first.x, first.x, first.y, first.y);
    for p in points {
        left = left.min(p.x);
        right = right.max(p.x);
        top = top.min(p.y);
        bottom = bottom.max(p.y);
    }
    Some(((left + right) / 2.0, (top + bottom) / 2.0))
}

/// A label whose middle is at `at`, rotated `angle_deg` clockwise; the
/// baseline sits below the middle by 0.3 of the font size, across the text.
fn rotated(text: &str, at: (f64, f64), angle_deg: f64, knockout: bool) -> Label {
    let drop = 0.3 * font_px();
    let a = angle_deg.to_radians();
    Label {
        text: text.to_owned(),
        x: at.0 - drop * a.sin(),
        y: at.1 + drop * a.cos(),
        angle_deg,
        justify: Justify::Center,
        knockout,
    }
}

/// The clockwise screen angle of `(dx, dy)` in degrees, turned so text
/// along it reads upright.
fn upright_deg((dx, dy): (f64, f64)) -> f64 {
    let a = dy.atan2(dx).to_degrees();
    if a > 90.0 {
        a - 180.0
    } else if a <= -90.0 {
        a + 180.0
    } else {
        a
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
