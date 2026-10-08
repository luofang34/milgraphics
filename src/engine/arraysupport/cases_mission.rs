//! The mission-task and arrow-tail line types of `GetLineArray2Double`'s
//! first switch: the cases that hand their points to the mission-task
//! builders, plus CLUSTER, NAVIGATION, FOLLA, FOLSP and FERRY.

mod cluster;
mod dism_calls;
mod mobile_defense;
mod tails;

use super::work::Work;
use crate::engine::base::EngineError;
use crate::engine::tactical_lines as lt;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    match w.line_type {
        lt::CLUSTER => cluster::cluster(w)?,
        lt::NAVIGATION => cluster::navigation(w)?,
        lt::FOLLA => tails::folla(w)?,
        lt::FOLSP => tails::folsp(w)?,
        lt::FERRY => tails::ferry(w)?,
        lt::MOBILE_DEFENSE => mobile_defense::mobile_defense(w)?,
        _ => return dism_calls::build(w),
    }
    Ok(true)
}
