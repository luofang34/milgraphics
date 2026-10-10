//! Geographic positions and altitudes on WGS84.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

/// `[lon, lat]` in degrees, the coordinate order of GeoJSON. Output
/// geometry uses it where a cut at the antimeridian must be written as
/// +180, which [`GeoPoint`] normalizes away.
pub type LonLat = [f64; 2];

/// A position on the WGS84 ellipsoid, in degrees.
///
/// Construction through [`GeoPoint::new`] guarantees finite coordinates,
/// latitude within ±90° and longitude normalized to [-180°, 180°).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawPoint", into = "RawPoint")]
pub struct GeoPoint {
    lon: f64,
    lat: f64,
}

#[derive(Serialize, Deserialize)]
struct RawPoint {
    lon: f64,
    lat: f64,
}

/// Why a coordinate was rejected.
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum GeoError {
    /// A coordinate is NaN or infinite.
    #[error("coordinate ({lon}, {lat}) is not finite")]
    NotFinite {
        /// Longitude as given.
        lon: f64,
        /// Latitude as given.
        lat: f64,
    },
    /// Latitude outside ±90°.
    #[error("latitude {lat} is outside -90..=90")]
    Latitude {
        /// Latitude as given.
        lat: f64,
    },
}

impl GeoPoint {
    /// A validated point; longitude is wrapped into [-180°, 180°).
    pub fn new(lon: f64, lat: f64) -> Result<Self, GeoError> {
        if !lon.is_finite() || !lat.is_finite() {
            return Err(GeoError::NotFinite { lon, lat });
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(GeoError::Latitude { lat });
        }
        Ok(Self {
            lon: wrap_longitude(lon),
            lat,
        })
    }

    /// Longitude in degrees, in [-180, 180).
    pub fn lon(self) -> f64 {
        self.lon
    }

    /// Latitude in degrees, in [-90, 90].
    pub fn lat(self) -> f64 {
        self.lat
    }
}

impl TryFrom<RawPoint> for GeoPoint {
    type Error = GeoError;

    fn try_from(raw: RawPoint) -> Result<Self, GeoError> {
        Self::new(raw.lon, raw.lat)
    }
}

impl From<GeoPoint> for RawPoint {
    fn from(p: GeoPoint) -> Self {
        Self {
            lon: p.lon,
            lat: p.lat,
        }
    }
}

/// Wraps a finite longitude into [-180, 180), leaving in-range values
/// untouched so stored coordinates round-trip exactly.
pub(crate) fn wrap_longitude(lon: f64) -> f64 {
    if (-180.0..180.0).contains(&lon) {
        return lon;
    }
    let wrapped = lon.rem_euclid(360.0);
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else {
        wrapped
    }
}

/// The surface an altitude is measured from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VerticalDatum {
    /// Height above the terrain at the point.
    #[serde(rename = "agl")]
    AboveGround,
    /// Height above mean sea level.
    #[serde(rename = "msl")]
    MeanSeaLevel,
    /// Height above the WGS84 ellipsoid.
    #[serde(rename = "hae")]
    Ellipsoid,
}

/// An altitude in metres with an explicit vertical datum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Altitude {
    /// Metres above the datum; negative below it.
    pub metres: f64,
    /// What the altitude is measured from.
    pub datum: VerticalDatum,
    /// Fields this version does not model, preserved as JSON content.
    #[serde(flatten)]
    pub unknown: BTreeMap<String, Value>,
}

impl Altitude {
    /// An altitude with no extra fields.
    pub fn new(metres: f64, datum: VerticalDatum) -> Self {
        Self {
            metres,
            datum,
            unknown: BTreeMap::new(),
        }
    }
}
