//! The calls between the ported files that upstream makes directly: the
//! channels' flot and cover glyphs, the METOC shapes' line arrays and
//! partitions, and the line of contact's hostile FLOT.

use crate::engine::arraysupport::{ArraySupport, get_line_array2};
use crate::engine::base::{At, EngineError, Pt, Shape};
use crate::engine::channel_utility::partitions::get_partitions2;
use crate::engine::channels::channel1::{ChannelRequest, get_channel1_points};
use crate::engine::channels::externals::ChannelExternals;
use crate::engine::dism::cover::get_dism_cover_double_rev_c;
use crate::engine::flot::flot_line::{get_flot_count_double, get_flot_double};
use crate::engine::metoc::{MetocSupport, Partition};
use crate::engine::settings::Settings;
use crate::engine::tactical_lines as tl;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::shape_properties::set_shape_properties;

/// The ported files, wired to each other.
#[derive(Debug)]
pub(super) struct Wiring<'a> {
    pub(super) settings: &'a Settings,
}

impl ChannelExternals for Wiring<'_> {
    fn flot_count(&self, pts: &[Pt], len: f64, n: i32) -> Result<i32, EngineError> {
        get_flot_count_double(pts, len, n)
    }

    fn flot(&self, pts: &mut Vec<Pt>, len: f64, n: i32) -> Result<i32, EngineError> {
        get_flot_double(pts, len, n)
    }

    fn dism_cover_rev_c(
        &self,
        pts: &mut Vec<Pt>,
        line_type: i32,
        n: i32,
        settings: &Settings,
    ) -> Result<i32, EngineError> {
        get_dism_cover_double_rev_c(pts, line_type, n, settings)
    }

    /// A hostile line of contact's short segments are drawn as a FLOT.
    fn lc_flot_shapes(&self, parent: &Tg, pixels: Vec<Pt>) -> Result<Vec<Shape>, EngineError> {
        let mut flot = parent.clone();
        flot.line_type = tl::FLOT;
        flot.pixels = pixels.clone();
        let mut shapes = Vec::new();
        get_line_array2(&mut flot, &pixels, &mut shapes, self.settings)?;
        set_shape_properties(&mut flot, &mut shapes);
        Ok(shapes)
    }
}

impl MetocSupport for Wiring<'_> {
    fn line_array(&self, tg: &mut Tg, shapes: &mut Vec<Shape>) -> Result<(), EngineError> {
        ArraySupport {
            settings: self.settings,
        }
        .line_array(tg, shapes)
    }

    fn partitions(&self, tg: &Tg) -> Result<Vec<Partition>, EngineError> {
        let parts = get_partitions2(tg)?.ok_or(EngineError::Degenerate("no partitions"))?;
        Ok(parts
            .into_iter()
            .map(|p| Partition {
                start: p.start,
                end: p.end,
            })
            .collect())
    }

    fn channel_points(
        &self,
        line: &[f64],
        out: &mut [f64],
        channel_width: i32,
    ) -> Result<(), EngineError> {
        let tg = Tg::new(self.settings);
        let count = i32::try_from(line.len() / 2).unwrap_or(i32::MAX);
        let request = ChannelRequest {
            tg: &tg,
            settings: self.settings,
            line_type: tl::CHANNEL,
            upper: line,
            lower: line,
            upper_counter: count,
            lower_counter: count,
            channel_width,
            useptr: 0,
        };
        let Some(points) = get_channel1_points(&request, self)? else {
            return Ok(());
        };
        // Upstream writes point by point and fails on the first slot past
        // the end of `out`, leaving the earlier ones written.
        for (j, p) in points.iter().enumerate() {
            let base = 3 * j;
            *out.at_mut(base)? = p.x;
            *out.at_mut(base + 1)? = p.y;
            *out.at_mut(base + 2)? = f64::from(p.style);
        }
        Ok(())
    }
}
