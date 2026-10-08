//! Port of `arraysupport.getScaledSize`: base pixel sizes grow with the line
//! width.

/// Upstream `arraysupport.getScaledSize`: `original_size` unchanged for line
/// widths up to 3, otherwise scaled by `1 + (width - 3) / 2 * pattern_scale`
/// with the width capped at 100.
pub(crate) fn get_scaled_size(original_size: f64, line_width: f64, pattern_scale: f64) -> f64 {
    if line_width <= 3.0 {
        return original_size;
    }
    let width = if line_width > 100.0 {
        100.0
    } else {
        line_width
    };
    original_size * (1.0 + ((width - 3.0) / 2.0) * pattern_scale)
}
