//! Deterministic SVG of a render plan's screen tier, for golden tests,
//! examples and debugging. Not the internal representation.

use std::fmt::Write as _;

use crate::render::{RenderPlan, ScreenItem, ScreenPoint, ScreenShape, TextAlign};
use crate::style::{Fill, Stroke};

#[cfg(test)]
mod tests;

/// Writes the plan's screen items and labels as an SVG document of the
/// given pixel size. Coordinates are rounded to 0.01 px so output is stable.
pub fn to_svg(plan: &RenderPlan, width_px: f64, height_px: f64) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = num(width_px),
        h = num(height_px)
    )
    .ok();
    for item in &plan.screen {
        item_svg(&mut out, item);
    }
    for label in &plan.labels {
        let Some(at) = label.screen else { continue };
        let anchor = match label.align {
            TextAlign::Left => "start",
            TextAlign::Center => "middle",
            TextAlign::Right => "end",
        };
        let em = label.font.size_px;
        writeln!(
            out,
            r#"<text transform="translate({x} {y}) rotate({r})" x="{dx}" y="{dy}" text-anchor="{anchor}" dominant-baseline="central" font-family="{family}" font-size="{size}" font-weight="{weight}">{text}</text>"#,
            x = num(at.x),
            y = num(at.y),
            r = num(label.rotation_deg),
            dx = num(label.offset_em[0] * em),
            dy = num(label.offset_em[1] * em),
            family = escape(&label.font.family),
            size = num(em),
            weight = if label.font.bold { "bold" } else { "normal" },
            text = escape(&label.text),
        )
        .ok();
    }
    out.push_str("</svg>\n");
    out
}

fn item_svg(out: &mut String, item: &ScreenItem) {
    let (tag, points) = match &item.shape {
        ScreenShape::Polyline(p) => ("polyline", p),
        ScreenShape::Polygon(p) => ("polygon", p),
    };
    let fill = match item.fill {
        Fill::Solid(c) => c.to_hex(),
        // Hatch lines are items of their own.
        _ => "none".to_owned(),
    };
    writeln!(
        out,
        r#"<{tag} points="{points}" fill="{fill}"{stroke}/>"#,
        points = point_list(points),
        stroke = stroke_attrs(item.stroke),
    )
    .ok();
}

fn stroke_attrs(stroke: Option<Stroke>) -> String {
    let Some(s) = stroke else {
        return r#" stroke="none""#.to_owned();
    };
    let mut attrs = format!(
        r#" stroke="{}" stroke-width="{}" stroke-linejoin="miter" stroke-linecap="butt""#,
        s.color.to_hex(),
        num(s.width_px)
    );
    let dash = s.dash.array();
    if !dash.is_empty() {
        let lengths: Vec<String> = dash.iter().map(|d| num(d * s.width_px)).collect();
        attrs.push_str(&format!(r#" stroke-dasharray="{}""#, lengths.join(" ")));
    }
    attrs
}

fn point_list(points: &[ScreenPoint]) -> String {
    let parts: Vec<String> = points
        .iter()
        .map(|p| format!("{},{}", num(p.x), num(p.y)))
        .collect();
    parts.join(" ")
}

/// A number rounded to 0.01, without trailing zeros or negative zero.
fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    let r = if r == 0.0 { 0.0 } else { r };
    let s = format!("{r:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' => {}
            c => out.push(c),
        }
    }
    out
}
