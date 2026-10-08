//! Port of mil-sym-java JavaLineArray/DISMSupport.java: the point builders
//! for the mission-task and DISM tactical graphics (canalize, breach, cover,
//! screen, guard, escort, disrupt, contain, fix, clear, seize, bypass, block,
//! support/attack by fire, gap, linear targets, minefields, delay, withdraw,
//! and the rest).
//!
//! Upstream passes a pre-sized `POINT2[]` that holds the control points in
//! its leading slots and receives the drawn points. Here the array is a
//! `Vec<Pt>`: reads beyond its length fail with an index error, writes grow
//! it with default points, and each builder returns the number of drawn
//! points. Functions that read upstream's `points.length` use `Vec::len`, so
//! the caller sizes the vector as upstream's point-count code would.
//!
//! The pixel constants (`MAX_LENGTH`, `MIN_LENGTH`, the DPI scale factor) and
//! Java's `(int)` truncations are kept as upstream has them. Branches that
//! depend on a clip rectangle implement only the unclipped path.

pub(crate) mod bypass;
pub(crate) mod cover;
pub(crate) mod delay;
pub(crate) mod disrupt;
pub(crate) mod escort;
pub(crate) mod fire;
pub(crate) mod fix;
pub(crate) mod rip;
pub(crate) mod seize;
pub(crate) mod support;
pub(crate) mod target;

#[cfg(test)]
mod tests;
