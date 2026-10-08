//! Port of mil-sym-java JavaLineArray/TacticalLines.java: every line-type
//! constant, under upstream's names. The values are the keys the whole
//! pipeline dispatches on.

mod part_one;
mod part_two;

pub(crate) use part_one::*;
pub(crate) use part_two::*;

#[cfg(test)]
mod tests;
