//! Port of mil-sym-java JavaTacticalRenderer/P1.java: a run of consecutive
//! client segments that one channel is drawn along.

/// Upstream `P1`: the first and last segment index of a partition. The end
/// can be `-1` for an empty partition at the start of a line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Partition {
    pub(crate) start: i32,
    pub(crate) end: i32,
}
