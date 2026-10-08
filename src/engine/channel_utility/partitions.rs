//! `GetPartitions` and `GetPartitions2` of clsChannelUtility.java.

use crate::engine::base::{At, EngineError};
use crate::engine::channels::scaled_size::get_scaled_size;
use crate::engine::partition::Partition;
use crate::engine::tg::Tg;
use crate::engine::tg_utility::segments::get_segments;

/// Upstream `GetPartitions`: groups consecutive good segments. A bad
/// segment ends the partition before it and begins the next one after it.
/// Empty when the first segment is bad.
pub(crate) fn get_partitions(segments: &[bool]) -> Result<Vec<Partition>, EngineError> {
    let mut partitions = Vec::new();
    if !segments.at(0)? {
        return Ok(partitions);
    }
    let n = i32::try_from(segments.len()).unwrap_or(i32::MAX);
    let mut current = Partition { start: 0, end: 0 };
    for j in 0..n - 1 {
        let next = segments.at(usize::try_from(j + 1).unwrap_or(usize::MAX))?;
        if !next {
            current.end = j;
            partitions.push(current);
            current = Partition {
                start: j + 1,
                end: 0,
            };
        }
    }
    current.end = (n - 1).max(0);
    partitions.push(current);
    Ok(partitions)
}

/// Upstream `GetPartitions2`: the partitions of a graphic's own pixels, for
/// METOC lines that handle double-backed segments. `None` when there is no
/// segment.
pub(crate) fn get_partitions2(tg: &Tg) -> Result<Option<Vec<Partition>>, EngineError> {
    if tg.pixels.len() < 2 {
        return Ok(None);
    }
    let pixels: Vec<f64> = tg.pixels.iter().flat_map(|p| [p.x, p.y]).collect();
    let factor = get_scaled_size(3.0, f64::from(tg.line_thickness), tg.pattern_scale);
    let segments = get_segments(&pixels, factor)?;
    Ok(Some(get_partitions(&segments)?))
}
