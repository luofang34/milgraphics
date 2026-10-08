//! Port of clsMETOC.ParallelLines2: the two channel edges beside a run of
//! points.

use super::MetocSupport;
use crate::engine::base::Pt;

/// `ParallelLines2`: the edges of a channel `channel_width` pixels wide
/// along `pixels`, the first half of the result being one edge and the
/// second half the other. Upstream catches a failing channel builder and
/// carries on with the zeros it had not yet filled; so does this.
pub(crate) fn parallel_lines2(
    pixels: &[Pt],
    channel_width: i32,
    support: &dyn MetocSupport,
) -> Vec<Pt> {
    let line: Vec<f64> = pixels.iter().flat_map(|p| [p.x, p.y]).collect();
    let mut channel = vec![0.0; 6 * pixels.len()];
    support
        .channel_points(&line, &mut channel, channel_width)
        .ok();
    channel
        .chunks_exact(3)
        .map(|c| {
            Pt::new(
                c.first().copied().unwrap_or(0.0),
                c.get(1).copied().unwrap_or(0.0),
            )
        })
        .collect()
}
