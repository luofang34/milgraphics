//! Port of mil-sym-java JavaLineArray/Channels.java and CChannelPoints2.java:
//! the channel family (axes of advance, line of contact, wire and fence
//! channels), in pixel space with the "ge" client and shifted lines.
//!
//! The parts of the pipeline that live in other upstream files (flot
//! counting and drawing, the DISM cover glyph, FLOT rendering of small line
//! of contact angles) are reached through [`ChannelExternals`] so this
//! module depends only on the geometry primitives it ports from.

pub(crate) mod axad;
pub(crate) mod channel1;
pub(crate) mod connect;
pub(crate) mod externals;
pub(crate) mod fill;
pub(crate) mod lines;
pub(crate) mod point_index;
pub(crate) mod scaled_size;
pub(crate) mod true_points;

#[cfg(test)]
mod tests;

pub(crate) use externals::ChannelExternals;

use crate::engine::base::Pt;

/// Largest arrow size, `Channels.maxLength`.
pub(crate) const MAX_LENGTH: f64 = 100.0;
/// Smallest arrow size, `Channels.minLength`.
pub(crate) const MIN_LENGTH: f64 = 5.0;

/// Upstream `CChannelPoints2`: the two channel edge points at one vertex.
/// `line1` is the lower edge, `line2` the upper edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ChannelPoints {
    pub(crate) line1: Pt,
    pub(crate) line2: Pt,
}

#[cfg(test)]
mod oracle_tests;
