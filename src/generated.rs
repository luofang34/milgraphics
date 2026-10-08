//! Data extracted mechanically by `cargo xtask catalog` (from the pinned
//! upstream files) and `cargo xtask references` (from the standards' tables).
//! Never edited by hand; see `UPSTREAM.md`.

// Reflowing generated files would make the CI regeneration check depend on
// the rustfmt version.
#[rustfmt::skip]
pub(crate) mod catalog;
#[rustfmt::skip]
pub(crate) mod draw_rule;
#[rustfmt::skip]
pub(crate) mod references;
