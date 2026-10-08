//! The weather, hydrographic and area line types of `GetLineArray2Double`'s
//! first switch (BBS_AREA through CF/UCF, and FLOT), with their point
//! builders from arraysupport.java.

mod anchorage;
mod atwall;
mod fronts;
mod patterns;
mod primitives;
mod ridge;
mod squall;
mod strips;
mod wire;

use super::work::Work;
use crate::engine::base::EngineError;
use crate::engine::tactical_lines as lt;

/// Builds the points of the line types of this group; false when the line
/// type belongs to another group.
pub(crate) fn build(w: &mut Work<'_>) -> Result<bool, EngineError> {
    match w.line_type {
        lt::BBS_AREA => primitives::bbs_area(w)?,
        lt::BS_CROSS => primitives::bs_cross(w)?,
        lt::BS_RECTANGLE => primitives::bs_rectangle(w)?,
        lt::BBS_RECTANGLE => primitives::bbs_rectangle(w)?,
        lt::BS_ELLIPSE => primitives::bs_ellipse(w)?,
        lt::OVERHEAD_WIRE => w.ac = wire::get_overhead_wire(w.tg, &mut w.p, w.save)?,
        lt::BOUNDARY | lt::TRIP => w.ac = i32::try_from(w.p.len()).unwrap_or(i32::MAX),
        lt::REEF => strips::reef(w)?,
        lt::RESTRICTED_AREA => strips::restricted_area(w)?,
        lt::TRAINING_AREA => primitives::training_area(w)?,
        lt::PIPE => strips::pipe(w)?,
        lt::ANCHORAGE_AREA => anchorage::anchorage_area(w)?,
        lt::ANCHORAGE_LINE => anchorage::anchorage_line(w)?,
        lt::LRO => patterns::lro(w)?,
        lt::UNDERCAST => anchorage::undercast(w)?,
        lt::LVO => patterns::lvo(w)?,
        lt::ICING => patterns::icing(w)?,
        lt::MVFR => anchorage::mvfr(w)?,
        lt::ITD => squall::itd(w)?,
        lt::CONVERGENCE => squall::convergence(w)?,
        lt::RIDGE => ridge::ridge(w)?,
        lt::TROUGH | lt::UPPER_TROUGH | lt::INSTABILITY | lt::SHEAR => {
            squall::trough(w, 10.0, 30.0)?;
        }
        lt::CABLE => squall::trough(w, 20.0, 20.0)?,
        lt::SQUALL => squall::severe_squall(w)?,
        lt::SF | lt::USF | lt::SFG | lt::SFY => fronts::sf_family(w)?,
        lt::OFY => fronts::ofy(w)?,
        lt::OCCLUDED | lt::UOF => fronts::occluded(w)?,
        lt::WF | lt::UWF => fronts::wf(w)?,
        lt::WFG | lt::WFY => fronts::wfg(w)?,
        lt::CFG | lt::CFY => fronts::cfg(w)?,
        lt::CF | lt::UCF => fronts::cf(w)?,
        lt::FLOT => fronts::flot(w)?,
        _ => return Ok(false),
    }
    Ok(true)
}
