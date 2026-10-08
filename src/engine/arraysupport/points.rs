//! The first switch of `GetLineArray2Double`: dispatch of a line type to the
//! builder that rewrites the point array. Each group of builders owns the
//! labels of its line types.

use super::work::Work;
use super::{cases_arrows, cases_control, cases_mission, cases_spikes, cases_weather};
use crate::engine::base::EngineError;

/// Rewrites `w.p` into the symbol's points and sets `w.ac` to their count.
/// Line types no group claims keep their control points (`default`).
pub(crate) fn build_points(w: &mut Work<'_>) -> Result<(), EngineError> {
    if cases_weather::build(w)?
        || cases_spikes::build(w)?
        || cases_control::build(w)?
        || cases_arrows::build(w)?
        || cases_mission::build(w)?
    {
        return Ok(());
    }
    w.ac = w.save;
    Ok(())
}
