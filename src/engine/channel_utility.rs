//! Port of mil-sym-java JavaTacticalRenderer/clsChannelUtility.java:
//! `DrawChannel`, the partitioning of a channel's client line at
//! double-backed segments, and the drawing of each partition through
//! `Channels.GetChannel1Double`.
//!
//! Upstream's alternate output of channel points as an array (used only when
//! no shape list is given) is not ported: the "ge" client always draws into
//! shapes.

pub(crate) mod draw;
pub(crate) mod good_channel;
pub(crate) mod lc;
pub(crate) mod partitions;

#[cfg(test)]
mod tests;
