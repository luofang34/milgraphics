//! Text values upstream builds from numeric amplifiers: comma-joined lists
//! (`Double.toString` per value) and altitude labels
//! (`clsRenderer.createAltitudeLabel`).

use crate::engine::base::java_round;
use crate::engine::java_text::double_to_string;
use crate::geo::{Altitude, VerticalDatum};

/// Feet per metre, upstream `DistanceUnit.FEET`.
const FEET_PER_METER: f64 = 3.28084;
/// Hundreds of feet per metre, upstream `DistanceUnit.FLIGHT_LEVEL`.
const FLIGHT_LEVEL_PER_METER: f64 = 0.0328084;

/// Upstream's default altitude unit label.
const ALTITUDE_UNIT_LABEL: &str = "FT";

/// `Double.toString` of each value joined by commas.
pub(super) fn join_doubles(values: &[f64]) -> String {
    values
        .iter()
        .map(|v| double_to_string(*v))
        .collect::<Vec<_>>()
        .join(",")
}

/// The altitude-mode text upstream's `AltitudeMode` takes for a datum.
pub(super) fn altitude_mode(datum: VerticalDatum) -> &'static str {
    match datum {
        VerticalDatum::AboveGround => "AGL",
        VerticalDatum::MeanSeaLevel => "AMSL",
        VerticalDatum::Ellipsoid => "HAE",
    }
}

/// Upstream `createAltitudeLabel`: the altitude in whole feet with its mode
/// ("3280 FT AMSL"), "GL" or "MSL" for zero, "FL 100" for flight levels.
pub(super) fn create_altitude_label(metres: f64, mode: &str) -> String {
    let factor = if mode == "FL" {
        FLIGHT_LEVEL_PER_METER
    } else {
        FEET_PER_METER
    };
    let scaled = java_round(metres * factor * 10.0);
    // Java int arithmetic: truncating division, saturating cast.
    let whole = (scaled as i32) / 10;
    if whole == 0 {
        if mode == "AGL" || mode == "GL" {
            return "GL".to_owned();
        }
        if matches!(mode, "AMSL" | "BMSL" | "MSL") {
            return "MSL".to_owned();
        }
    }
    if mode == "FL" {
        return format!("FL {whole:03}");
    }
    format!("{whole} {ALTITUDE_UNIT_LABEL} {mode}")
}

/// The label of one altitude amplifier in its own datum's mode.
pub(super) fn altitude_label(altitude: &Altitude) -> String {
    create_altitude_label(altitude.metres, altitude_mode(altitude.datum))
}
