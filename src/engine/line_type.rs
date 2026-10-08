//! Port of mil-sym-java JavaTacticalRenderer/clsUtility.GetLinetypeFromString and
//! clsMETOC.IsWeather: the dispatch from a symbol's set and entity code to the
//! line type the pipeline draws, plus predicates on line types.

pub(crate) mod classes;
pub(crate) mod control_measures;
pub(crate) mod weather;

#[cfg(test)]
mod tests;

use control_measures::cm_line_type;
use weather::weather_line_type;

/// Symbol set of control measures.
const SYMBOL_SET_CONTROL_MEASURES: u8 = 25;
/// Symbol sets of METOC graphics (atmospheric, oceanographic).
const SYMBOL_SET_METOC: [u8; 2] = [45, 46];

/// The line type of a graphic, or `None` where upstream returns -1 (the
/// symbol set or entity is not drawn as a line type). `version` is the
/// symbol-code version (11 for 2525D change 1, 15 for 2525E change 1).
pub(crate) fn line_type(version: u8, symbol_set: u8, entity: u32) -> Option<i32> {
    if symbol_set == SYMBOL_SET_CONTROL_MEASURES {
        cm_line_type(version, entity)
    } else {
        is_weather(symbol_set, entity)
    }
}

/// Upstream `clsMETOC.IsWeather`: the METOC line type, or `None` for any
/// other symbol set or an unknown entity.
pub(crate) fn is_weather(symbol_set: u8, entity: u32) -> Option<i32> {
    if SYMBOL_SET_METOC.contains(&symbol_set) {
        weather_line_type(entity)
    } else {
        None
    }
}
