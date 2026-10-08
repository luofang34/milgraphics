//! Point counts and required amplifiers per draw rule, as mil-sym-java's
//! `MSInfo.getMinMaxPointsFromDrawRule`, `getMinMaxPointsFromMODrawRule` and
//! `MultiPointHandler.hasRequiredModifiers` give them.

/// Upper bound written for "any number of points".
pub(super) const MANY: &str = "MANY";

/// `(min, max)` control points for a control-measure draw rule; `max` is a
/// Rust expression.
pub(super) fn cm_points(rule: &str, version: u8) -> (u32, &'static str) {
    match rule {
        "Area1" | "Area2" | "Area3" | "Area4" | "Area9" | "Area20" | "Area23" => (3, MANY),
        "Area5" | "Area7" | "Area11" | "Area12" | "Area17" | "Area21" | "Area24" | "Area25"
        | "Area27" | "Point12" | "Line3" | "Line6" | "Line10" | "Line12" | "Line17" | "Line22"
        | "Line23" | "Line24" | "Line29" | "Line30" | "Line32" | "Line33" | "Line50"
        | "Polyline1" => (3, "3"),
        "Area6" | "Area13" | "Area15" | "Area16" | "Area19" | "Line4" | "Line5" | "Line9"
        | "Line14" | "Line18" | "Line19" | "Line20" | "Line25" | "Line28" | "Rectangular1"
        | "Rectangular3" => (2, "2"),
        "Area8" | "Area18" | "Line11" | "Line16" | "Line31" => (4, "4"),
        "Area10" => (3, "6"),
        // Upstream allows a fourth point only in base 2525D (code 10).
        "Area14" if version == 10 => (3, "4"),
        "Area14" => (3, "3"),
        "Line1" | "Line2" | "Line7" | "Line13" | "Line21" | "Corridor1" => (2, MANY),
        "Area26" => (6, MANY),
        "Line8" => (2, "300"),
        "Line26" | "Line27" if version >= 13 => (4, "4"),
        "Line26" | "Line27" => (3, "4"),
        "Axis1" | "Axis2" => (3, "50"),
        _ => (1, "1"),
    }
}

/// `(min, max)` control points for a METOC draw rule.
pub(super) fn metoc_points(rule: &str) -> (u32, &'static str) {
    match rule {
        "Area1" | "Area2" | "Line5" => (3, MANY),
        "Point5" | "Line1" | "Line2" | "Line3" | "Line4" | "Line6" | "Line7" | "Line8" => (2, MANY),
        _ => (1, "1"),
    }
}

/// Fewest `AM` and `AN` values a control-measure draw rule needs.
pub(super) fn required_counts(rule: &str) -> (u32, u32) {
    match rule {
        "Circular1" | "Circular2" | "Rectangular1" | "Rectangular3" | "Corridor1" => (1, 0),
        "Rectangular2" | "Ellipse1" | "Point17" => (2, 1),
        "Arc1" => (1, 2),
        "Point18" => (2, 2),
        _ => (0, 0),
    }
}
