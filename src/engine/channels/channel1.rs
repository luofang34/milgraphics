//! `GetChannel1Double` of Channels.java: builds the polyline points of a
//! channel type (the two edges plus the type's end feature) and turns them
//! into shapes. The work is split by stage and by line-type family in
//! the submodules.

mod arrow_glyphs;
mod arrows;
mod edges;
mod features;
mod fence;
mod lc;
mod rotary;
mod shapes;

#[cfg(test)]
mod tests;

use super::ChannelExternals;
use super::fill::get_axad_fill_shapes;
use super::point_index::{at_i, mut_i, new_pts, set_i};
use super::scaled_size::get_scaled_size;
use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::lineutility::extend::extend_along_line_double;
use crate::engine::settings::{SHIFT_LINES, Settings};
use crate::engine::tactical_lines as lt;
use crate::engine::tg::Tg;

/// The inputs of `GetChannel1Double`. The upper and lower client points are
/// x,y pairs in pixels.
#[derive(Debug)]
pub(crate) struct ChannelRequest<'a> {
    /// The graphic being drawn.
    pub(crate) tg: &'a Tg,
    /// Renderer settings (DPI scale).
    pub(crate) settings: &'a Settings,
    /// The line type to draw, which may differ from the graphic's for
    /// partitions of a channel.
    pub(crate) line_type: i32,
    /// The upper client points.
    pub(crate) upper: &'a [f64],
    /// The lower client points.
    pub(crate) lower: &'a [f64],
    /// The number of upper points.
    pub(crate) upper_counter: i32,
    /// The number of lower points.
    pub(crate) lower_counter: i32,
    /// Half the channel width in pixels.
    pub(crate) channel_width: i32,
    /// The distance in pixels from the arrow tip to the back of the arrowhead.
    pub(crate) useptr: i32,
}

/// The working state of one `GetChannel1Double` call, the locals upstream
/// threads through its single large function.
#[derive(Debug)]
pub(super) struct Chan<'a> {
    pub(super) req: &'a ChannelRequest<'a>,
    pub(super) draw: i32,
    pub(super) width: i32,
    pub(super) dist: f64,
    pub(super) lower: Vec<Pt>,
    pub(super) upper: Vec<Pt>,
    pub(super) arrow: Pt,
    pub(super) last_point: Pt,
    pub(super) next_to_last: Pt,
    pub(super) original: Vec<Pt>,
    pub(super) l_count: i32,
    pub(super) u_count: i32,
    pub(super) counter: i32,
    pub(super) shift_lines: bool,
    pub(super) reverse_upper: i32,
    pub(super) offset_factor: f64,
    pub(super) points: Vec<Pt>,
    pub(super) origin: Pt,
}

impl<'a> Chan<'a> {
    pub(super) fn scaled(&self, size: f64) -> f64 {
        get_scaled_size(
            size,
            f64::from(self.req.tg.line_thickness),
            self.req.tg.pattern_scale,
        )
    }

    /// Loads the client points, applies the minimum width and, for the arrow
    /// types, pulls the edge ends back to the back of the arrowhead. `None`
    /// when either edge has fewer than two points.
    fn load(req: &'a ChannelRequest<'a>) -> Result<Option<Self>, EngineError> {
        let draw = req.line_type;
        let offset_factor = f64::from(req.channel_width / 4);
        let mut width = req.channel_width;
        if width < 5 && draw != lt::BBS_LINE {
            width = 5;
        }
        if req.lower_counter < 2 || req.upper_counter < 2 {
            return Ok(None);
        }
        let mut lower = new_pts(req.lower_counter)?;
        for (k, slot) in (0..req.lower_counter).zip(lower.iter_mut()) {
            let i = usize::try_from(k).unwrap_or(usize::MAX);
            *slot = Pt::new(req.lower.at(2 * i)?, req.lower.at(2 * i + 1)?);
        }
        let mut upper = new_pts(req.upper_counter)?;
        for (k, slot) in (0..req.upper_counter).zip(upper.iter_mut()) {
            let i = usize::try_from(k).unwrap_or(usize::MAX);
            *slot = Pt::new(req.upper.at(2 * i)?, req.upper.at(2 * i + 1)?);
        }
        let last_point = at_i(&lower, req.lower_counter - 1)?;
        let next_to_last = at_i(&lower, req.lower_counter - 2)?;
        let arrow = at_i(&upper, req.upper_counter - 1)?;
        let mut ch = Self {
            req,
            draw,
            width,
            dist: f64::from(req.useptr),
            lower,
            upper,
            arrow,
            last_point,
            next_to_last,
            original: Vec::new(),
            l_count: req.lower_counter,
            u_count: req.upper_counter,
            counter: 0,
            shift_lines: SHIFT_LINES
                && matches!(
                    draw,
                    lt::LC
                        | lt::UNSP
                        | lt::LWFENCE
                        | lt::HWFENCE
                        | lt::SINGLEC
                        | lt::DOUBLEC
                        | lt::TRIPLE
                ),
            reverse_upper: 0,
            offset_factor,
            points: Vec::new(),
            origin: Pt::new(req.upper.at(0)?, req.upper.at(1)?),
        };
        ch.pull_back_arrow_ends()?;
        Ok(Some(ch))
    }

    fn pull_back_arrow_ends(&mut self) -> Result<(), EngineError> {
        if !matches!(
            self.draw,
            lt::CATK
                | lt::AIRAOA
                | lt::AAAAA
                | lt::SPT
                | lt::SPT_STRAIGHT
                | lt::FRONTAL_ATTACK
                | lt::TURNING_MOVEMENT
                | lt::MOVEMENT_TO_CONTACT
                | lt::MAIN
                | lt::MAIN_STRAIGHT
                | lt::CATKBYFIRE
        ) {
            return Ok(());
        }
        let u = self.u_count;
        let l = self.l_count;
        let new_upper = extend_along_line_double(
            at_i(&self.upper, u - 1)?,
            at_i(&self.upper, u - 2)?,
            self.dist,
        );
        set_i(&mut self.upper, u - 1, new_upper)?;
        let new_lower = extend_along_line_double(
            at_i(&self.lower, l - 1)?,
            at_i(&self.lower, l - 2)?,
            self.dist,
        );
        set_i(&mut self.lower, l - 1, new_lower)
    }
}

/// Upstream `GetChannel1Double` with `shapes == null`: the channel's points
/// (`x, y, style` per point upstream; here the points themselves). `None`
/// when an edge has fewer than two points or a line of contact has no flot
/// points, where upstream writes nothing.
pub(crate) fn get_channel1_points<E: ChannelExternals>(
    req: &ChannelRequest<'_>,
    ext: &E,
) -> Result<Option<Vec<Pt>>, EngineError> {
    Ok(build_points(req, ext)?.map(|ch| ch.points))
}

/// Loads and builds the point list shared by the shape and point outputs.
fn build_points<'a, E: ChannelExternals>(
    req: &'a ChannelRequest<'a>,
    ext: &E,
) -> Result<Option<Chan<'a>>, EngineError> {
    let Some(mut ch) = Chan::load(req)? else {
        return Ok(None);
    };
    ch.build_edges()?;
    if !ch.load_points(ext)? {
        return Ok(None);
    }
    if ch.draw == lt::CHANNEL_DASHED {
        for k in 0..ch.counter {
            let p = mut_i(&mut ch.points, k)?;
            if p.style != 5 {
                p.style = 18;
            }
        }
    }
    Ok(Some(ch))
}

/// Upstream `GetChannel1Double`: appends the channel's shapes to `shapes`.
/// Nothing is added when an edge has fewer than two points or a line of
/// contact has no flot points.
pub(crate) fn get_channel1_double<E: ChannelExternals>(
    req: &ChannelRequest<'_>,
    ext: &E,
    shapes: &mut Vec<Shape>,
) -> Result<(), EngineError> {
    let Some(ch) = build_points(req, ext)? else {
        return Ok(());
    };
    shapes::points_to_shapes(&ch, shapes)?;
    if req.tg.fill_color.is_some() {
        if let Some(fill) = get_axad_fill_shapes(ch.draw, &ch.points)? {
            if !fill.is_empty() {
                shapes.splice(0..0, fill);
            }
        }
    }
    if ch.draw == lt::BBS_LINE {
        let mut shape = Shape::new(crate::engine::base::shape_type::POLYLINE);
        shape.move_to(at_i(&ch.original, 0)?);
        for j in 1..ch.original.len() {
            shape.line_to(ch.original.at(j)?);
        }
        shapes.push(shape);
    }
    Ok(())
}

impl Chan<'_> {
    /// Loads the channel edge points into `points`. False when a line of
    /// contact has no flot points and nothing is drawn.
    fn load_points<E: ChannelExternals>(&mut self, ext: &E) -> Result<bool, EngineError> {
        match self.draw {
            lt::LC => return self.load_lc_points(ext),
            lt::TRIPLE
            | lt::DOUBLEC
            | lt::SINGLEC
            | lt::HWFENCE
            | lt::LWFENCE
            | lt::UNSP
            | lt::DOUBLEA
            | lt::SFENCE
            | lt::DFENCE
            | lt::CHANNEL
            | lt::CHANNEL_FLARED
            | lt::CHANNEL_DASHED => self.load_fence_points()?,
            lt::BBS_LINE => self.load_bbs_points()?,
            lt::SPT
            | lt::SPT_STRAIGHT
            | lt::FRONTAL_ATTACK
            | lt::TURNING_MOVEMENT
            | lt::MOVEMENT_TO_CONTACT
            | lt::CATK
            | lt::CATKBYFIRE
            | lt::AIRAOA
            | lt::AAAAA
            | lt::MAIN
            | lt::MAIN_STRAIGHT => self.load_arrow_points(ext)?,
            _ => {}
        }
        Ok(true)
    }
}
