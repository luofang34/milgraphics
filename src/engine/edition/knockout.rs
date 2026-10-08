//! Labels the template sets into a gap in the outline.

use crate::engine::api::Output;

/// Marks the labels whose text `on_outline` picks, so the view leaves the
/// outline out under them.
pub(super) fn mark(out: &mut Output, on_outline: impl Fn(&str) -> bool) {
    for label in &mut out.labels {
        if on_outline(&label.text) {
            label.knockout = true;
        }
    }
}
