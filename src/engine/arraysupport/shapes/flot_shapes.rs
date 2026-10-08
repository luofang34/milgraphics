//! Outline shapes of the weather fronts and the line types drawn with dots:
//! ITD, the stationary fronts, WFG, CFG and PIPE, and the mined ditch.

use crate::engine::arraysupport::shape_path::{BLUE, GREEN, RED, line_to, move_to};
use crate::engine::arraysupport::work::{Work, get};
use crate::engine::base::{EngineError, Pt, Shape, shape_type};
use crate::engine::lineutility::basics::calc_distance_double;
use crate::engine::lineutility::circle::calc_circle_shape;
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::tactical_lines as lt;
use crate::engine::tg::CAP_BUTT;
use crate::style::Rgba;

/// Appends the shapes of ITD, SFY, SFG, USF, SF, WFG, CFG and PIPE.
pub(crate) fn build(w: &mut Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    match w.line_type {
        lt::ITD => itd(w, shapes),
        lt::SFY | lt::SFG => sfy_sfg(w, shapes),
        lt::USF => usf(w, shapes),
        lt::SF => sf(w, shapes),
        lt::WFG => {
            let s = polyline_between(w, 0, 5)?;
            shapes.push(s);
            dots(w, 20, 3.0, shapes)
        }
        lt::CFG => cfg(w, shapes),
        _ => {
            dots(w, 20, 5.0, shapes)?;
            let s = polyline_between(w, 0, 5)?;
            shapes.push(s);
            Ok(())
        }
    }
}

fn colored(color: Rgba) -> Shape {
    let mut s = Shape::new(shape_type::POLYLINE);
    s.line_color = Some(color);
    s
}

fn filled(color: Rgba) -> Shape {
    let mut s = Shape::new(shape_type::FILL);
    s.fill_color = Some(color);
    s
}

/// Segment pairs `p[k]`-`p[k+1]` with the given styles, as one polyline.
fn polyline_between(w: &Work<'_>, from: i32, to: i32) -> Result<Shape, EngineError> {
    let mut s = Shape::new(shape_type::POLYLINE);
    for k in 0..w.ac - 1 {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        if a.style == from && b.style == to {
            move_to(&mut s, a);
            line_to(&mut s, b)?;
        }
    }
    Ok(s)
}

/// A filled circle shape of `size` (scaled) at every point of `style`.
fn dots(w: &Work<'_>, style: i32, size: f64, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    for k in 0..w.ac {
        let pk = get(&w.p, k)?;
        if pk.style == style {
            let mut pts = [Pt::default(); 8];
            shapes.push(calc_circle_shape(pk, w.scaled(size), 8, &mut pts, 9)?);
        }
    }
    Ok(())
}

fn itd(w: &mut Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut red = colored(RED);
    let mut blue = colored(GREEN);
    for k in 0..w.ac - 1 {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        if a.style == 19 && b.style == 5 {
            move_to(&mut red, a);
            line_to(&mut red, b)?;
        } else if a.style == 25 && b.style == 5 {
            move_to(&mut blue, a);
            line_to(&mut blue, b)?;
        }
    }
    shapes.push(red);
    shapes.push(blue);
    w.tg.line_cap = CAP_BUTT;
    Ok(())
}

/// The flots (red fills) and spikes (blue fills) of SFY and SFG.
fn flot_fills(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    for k in 0..w.ac - 1 {
        let style = get(&w.p, k)?.style;
        if style == 23 {
            let mut f = filled(RED);
            move_to(&mut f, get(&w.p, k - 9)?);
            for l in k - 8..=k {
                line_to(&mut f, get(&w.p, l)?)?;
            }
            shapes.push(f);
        }
        if style == 24 {
            let mut f = filled(BLUE);
            move_to(&mut f, get(&w.p, k - 2)?);
            line_to(&mut f, get(&w.p, k - 1)?)?;
            line_to(&mut f, get(&w.p, k)?)?;
            shapes.push(f);
        }
    }
    Ok(())
}

/// The distance of a corner leg: 50 scaled, shortened to the segment.
fn leg(w: &Work<'_>, a: Pt, b: Pt) -> f64 {
    let d = w.scaled(50.0);
    let d1 = calc_distance_double(a, b);
    if d1 < d { d1 } else { d }
}

/// The red legs of the control polyline's corners.
fn corners(w: &Work<'_>, red: &mut Shape) -> Result<(), EngineError> {
    let o = &w.orig;
    let n = w.save;
    for k in 0..n {
        let ok = get(o, k)?;
        if k == 0 {
            let o1 = get(o, 1)?;
            move_to(red, ok);
            let d = leg(w, ok, o1);
            line_to(red, extend_along_line_double(ok, o1, d))?;
        } else if k < n - 1 {
            let (prev, next) = (get(o, k - 1)?, get(o, k + 1)?);
            let pt0 = extend_along_line_double(ok, prev, leg(w, ok, prev));
            let pt2 = extend_along_line_double(ok, next, leg(w, ok, next));
            move_to(red, pt0);
            line_to(red, ok)?;
            line_to(red, pt2)?;
        } else {
            let prev = get(o, n - 2)?;
            let d = leg(w, ok, prev);
            move_to(red, ok);
            line_to(red, extend_along_line_double(ok, prev, d))?;
        }
    }
    Ok(())
}

fn sfy_sfg(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut red = colored(RED);
    let mut blue = colored(BLUE);
    flot_fills(w, shapes)?;
    corners(w, &mut red)?;
    if w.line_type == lt::SFY {
        for k in 0..w.vbl - 1 {
            let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
            if a.style == 19 && b.style == 5 {
                move_to(&mut red, a);
                line_to(&mut red, b)?;
            } else if a.style == 25 && b.style == 5 {
                move_to(&mut blue, a);
                line_to(&mut blue, b)?;
            }
        }
        shapes.push(red);
        shapes.push(blue);
        return Ok(());
    }
    shapes.push(red);
    for k in 0..w.ac {
        let pk = get(&w.p, k)?;
        let color = match pk.style {
            22 => RED,
            20 => BLUE,
            _ => continue,
        };
        let mut pts = [Pt::default(); 8];
        let mut c = calc_circle_shape(pk, w.scaled(3.0), 8, &mut pts, 9)?;
        c.fill_color = Some(color);
        shapes.push(c);
    }
    Ok(())
}

fn usf(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut red = colored(RED);
    let mut blue = colored(BLUE);
    for k in 0..w.ac - 1 {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        let to_red =
            (a.style == 19 && (b.style == 5 || b.style == 19)) || (a.style == 0 && b.style == 5);
        let to_blue = a.style == 25 && (b.style == 5 || b.style == 25);
        if to_red {
            move_to(&mut red, a);
            line_to(&mut red, b)?;
        }
        if to_blue {
            move_to(&mut blue, a);
            line_to(&mut blue, b)?;
        }
    }
    shapes.push(red);
    shapes.push(blue);
    Ok(())
}

fn solid_fill(color: Rgba) -> Shape {
    let mut s = filled(color);
    s.line_color = Some(color);
    s
}

/// SF: flots and spikes become fills, the rest red and blue lines. `k`
/// jumps over the points a fill consumed, and the later tests read the new
/// `k`, as upstream does.
fn sf(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut red = colored(RED);
    let mut blue = colored(BLUE);
    let mut red_fill = solid_fill(RED);
    let mut blue_fill = solid_fill(BLUE);
    let mut k = 0;
    while k < w.ac - 1 {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        if a.style == 19 && b.style == 5 {
            move_to(&mut red, a);
            line_to(&mut red, b)?;
        }
        if a.style == 19 && b.style == 19 {
            if red_fill.path.is_empty() {
                move_to(&mut red_fill, get(&w.p, k + 9)?);
                for l in (k..=k + 9).rev() {
                    line_to(&mut red_fill, get(&w.p, l)?)?;
                }
            } else {
                move_to(&mut red_fill, a);
                for l in k..k + 10 {
                    line_to(&mut red_fill, get(&w.p, l)?)?;
                }
            }
            k += 9;
            shapes.push(std::mem::replace(&mut red_fill, solid_fill(RED)));
        }
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        if a.style == 25 && b.style == 5 {
            move_to(&mut blue, a);
            line_to(&mut blue, b)?;
        }
        if a.style == 25 && b.style == 25 {
            if blue_fill.path.is_empty() {
                move_to(&mut blue_fill, get(&w.p, k + 2)?);
                line_to(&mut blue_fill, b)?;
                line_to(&mut blue_fill, a)?;
            } else {
                move_to(&mut blue_fill, a);
                line_to(&mut blue_fill, b)?;
                line_to(&mut blue_fill, get(&w.p, k + 2)?)?;
            }
            shapes.push(std::mem::replace(&mut blue_fill, solid_fill(BLUE)));
        }
        if a.style == 0 && b.style == 5 {
            move_to(&mut red, a);
            line_to(&mut red, b)?;
        }
        k += 1;
    }
    shapes.push(red);
    shapes.push(blue);
    Ok(())
}

fn cfg(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    dots(w, 20, 3.0, shapes)?;
    let mut s = Shape::new(shape_type::POLYLINE);
    for k in 0..w.ac - 1 {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1)?);
        if a.style == 0 && (b.style == 0 || b.style == 9) {
            move_to(&mut s, a);
            line_to(&mut s, b)?;
        }
        if a.style == 0 && b.style == 5 {
            let d = calc_distance_double(a, b);
            let pt0 = extend_along_line_double(a, b, d - w.scaled(5.0));
            move_to(&mut s, a);
            line_to(&mut s, pt0)?;
        }
        if a.style == 0 && k == w.ac - 2 {
            move_to(&mut s, a);
            line_to(&mut s, b)?;
        }
    }
    shapes.push(s);
    Ok(())
}

/// A shape that may keep growing after it was added to the list, as a
/// shared Java object does.
struct Live<'a> {
    cur: Option<Shape>,
    pushed: Option<usize>,
    shapes: &'a mut Vec<Shape>,
}

impl Live<'_> {
    fn start(&mut self, style: i32) {
        let mut s = Shape::new(shape_type::POLYLINE);
        s.style = style;
        self.cur = Some(s);
        self.pushed = None;
    }

    fn edit(
        &mut self,
        f: impl Fn(&mut Shape) -> Result<(), EngineError>,
    ) -> Result<(), EngineError> {
        let s = self
            .cur
            .as_mut()
            .ok_or(EngineError::Degenerate("no current ditch shape"))?;
        f(s)?;
        if let Some(i) = self.pushed {
            if let Some(t) = self.shapes.get_mut(i) {
                f(t)?;
            }
        }
        Ok(())
    }

    fn publish(&mut self) -> Result<(), EngineError> {
        let s = self
            .cur
            .clone()
            .ok_or(EngineError::Degenerate("no current ditch shape"))?;
        self.shapes.push(s);
        self.pushed = Some(self.shapes.len() - 1);
        Ok(())
    }
}

/// ATDITCHM: the ditch line, spike runs and mine circles.
pub(crate) fn atditchm(w: &Work<'_>, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
    let mut live = Live {
        cur: None,
        pushed: None,
        shapes,
    };
    for k in 0..w.ac {
        let (a, b) = (get(&w.p, k)?, get(&w.p, k + 1).unwrap_or_default());
        if a.style == 20 {
            let mut pts = [Pt::default(); 8];
            live.shapes
                .push(calc_circle_shape(a, w.scaled(4.0), 8, &mut pts, 9)?);
            continue;
        }
        if k < w.ac - 2 {
            ditch_first(&mut live, a, b)?;
            ditch_second(&mut live, a, b)?;
        }
    }
    Ok(())
}

fn ditch_first(live: &mut Live<'_>, a: Pt, b: Pt) -> Result<(), EngineError> {
    if a.style != 0 && b.style == 0 {
        live.start(a.style);
        live.edit(|s| {
            move_to(s, a);
            line_to(s, a)
        })
    } else if a.style == 0 && b.style == 0 {
        live.edit(|s| {
            move_to(s, a);
            line_to(s, b)
        })
    } else if a.style == 0 && b.style == 10 {
        live.edit(|s| {
            move_to(s, a);
            line_to(s, b)
        })?;
        live.publish()
    } else {
        Ok(())
    }
}

fn ditch_second(live: &mut Live<'_>, a: Pt, b: Pt) -> Result<(), EngineError> {
    if a.style == 5 && b.style == 0 {
        live.start(a.style);
        live.edit(|s| {
            move_to(s, a);
            Ok(())
        })
    } else if a.style == 0 && b.style == 0 {
        live.edit(|s| line_to(s, b))
    } else if a.style == 0 && b.style == 5 {
        live.edit(|s| line_to(s, b))?;
        live.publish()
    } else {
        Ok(())
    }
}
