//! A port of mil-sym-java's multipoint renderer, run in pixel space: control
//! points and sizes in pixels go in, shapes and labels in pixels come out.
//! milgraphics drives it in a local frame for the geographic tier and in the
//! view's projection for the screen tier.

// Items become reachable as milgraphics' construction dispatches line types
// to the pipeline.
#![allow(dead_code)]

pub(crate) mod api;
pub(crate) mod arraysupport;
pub(crate) mod base;
pub(crate) mod dism;
pub(crate) mod flot;
pub(crate) mod line_type;
pub(crate) mod lineutility;
pub(crate) mod metoc;
pub(crate) mod settings;
pub(crate) mod tactical_lines;
pub(crate) mod tg;
pub(crate) mod tg_utility;
