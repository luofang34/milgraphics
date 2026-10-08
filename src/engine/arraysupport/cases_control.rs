//! The control-measure line types of `GetLineArray2Double`'s first switch
//! that build small fixed shapes from two to four control points: relative
//! lines, road blocks, fords, convoys, supply routes and ditches.

mod convoy;
mod ford;
mod lines;
mod routes;

use super::work::Work;
use crate::engine::base::EngineError;
use crate::engine::tactical_lines as lt;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    match w.line_type {
        lt::IL | lt::PLANNED | lt::ESR1 | lt::ESR2 => lines::relative_line(w)?,
        lt::FORDSITE => lines::ford_site(w)?,
        lt::ROADBLK => lines::road_block(w)?,
        lt::PNO | lt::PLD | lt::CFL => lines::dashed(w)?,
        lt::FENCED => lines::fenced(w)?,
        lt::FOXHOLE => lines::foxhole(w)?,
        lt::MINED
        | lt::UXO
        | lt::ACOUSTIC
        | lt::ACOUSTIC_AMB
        | lt::BEARING
        | lt::BEARING_J
        | lt::BEARING_RDF
        | lt::ELECTRO
        | lt::BEARING_EW
        | lt::TORPEDO
        | lt::OPTICAL => w.ac = w.vbl,
        lt::MSDZ => convoy::msdz(w)?,
        lt::CONVOY => convoy::convoy(w)?,
        lt::HCONVOY => convoy::hconvoy(w)?,
        lt::MSR_ONEWAY
        | lt::MSR_TWOWAY
        | lt::MSR_ALT
        | lt::ASR_ONEWAY
        | lt::ASR_TWOWAY
        | lt::ASR_ALT
        | lt::TRAFFIC_ROUTE_ONEWAY
        | lt::TRAFFIC_ROUTE_ALT => routes::supply_route(w)?,
        lt::FORDIF => ford::ford_difficult(w)?,
        lt::ATDITCH | lt::ATDITCHC | lt::ATDITCHM => ford::ditch(w)?,
        _ => return Ok(false),
    }
    Ok(true)
}
