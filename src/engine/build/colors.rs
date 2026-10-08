//! The colours upstream gives a multipoint graphic from its symbol code:
//! `MilStdSymbol`'s constructor with `SymbolUtilities.getDefaultLineColor`,
//! `getLineColorOfAffiliation`, `hasDefaultFill` and `getFillColorOfAffiliation`.

use crate::sidc::SymbolId;
use crate::style::Rgba;

const SYMBOL_SET_CONTROL_MEASURES: u8 = 25;
/// `SymbolID.Version_2525E` and `Version_2525Ech1`.
const VERSION_2525E: u8 = 13;
const VERSION_2525E_CH1: u8 = 15;

const GREEN: Rgba = Rgba::opaque(0, 255, 0);
const YELLOW: Rgba = Rgba::opaque(255, 255, 0);
const SUSPECT: Rgba = Rgba::opaque(255, 188, 1);
/// `(int) (.25 * 255)`.
const QUARTER_ALPHA: u8 = 63;

/// Upstream `SymbolUtilities.isGreenProtectionGraphic`: protection areas,
/// points and lines (entity group 27 to 29) are drawn green.
fn is_green_protection(entity: u32) -> bool {
    let group = entity / 10_000;
    let kind = (entity / 100) % 100;
    match group {
        27 => matches!(kind, 1..=5 | 7 | 8 | 10 | 12),
        28 => matches!(kind, 1..=7 | 19),
        29 => matches!(kind, 1..=5),
        _ => false,
    }
}

/// Upstream `getLineColorOfAffiliation`.
pub(super) fn line_color_of_affiliation(symbol: &SymbolId) -> Rgba {
    let set = symbol.symbol_set();
    if set == SYMBOL_SET_CONTROL_MEASURES {
        if is_green_protection(symbol.entity().get()) {
            return GREEN;
        }
        let version = symbol.version_code();
        return match symbol.identity() {
            2 | 3 => Rgba::BLACK,
            6 => Rgba::RED,
            5 if (VERSION_2525E..=VERSION_2525E_CH1).contains(&version) => SUSPECT,
            5 => Rgba::RED,
            4 => GREEN,
            _ => YELLOW,
        };
    }
    Rgba::BLACK
}

/// Upstream `getDefaultLineColor`: the colour a few control measures have
/// whatever the affiliation, else the affiliation colour.
pub(super) fn default_line_color(symbol: &SymbolId) -> Rgba {
    if symbol.symbol_set() == SYMBOL_SET_CONTROL_MEASURES {
        let entity = symbol.entity().get();
        let version = symbol.version_code();
        match entity {
            200_600 => return Rgba::opaque(255, 255, 255),
            200_700 => return Rgba::opaque(51, 136, 136),
            200_101 => return Rgba::opaque(255, 155, 0),
            200_201 | 200_202 => return Rgba::opaque(85, 119, 136),
            132_100 | 282_001 | 282_002 | 282_003 if version >= VERSION_2525E => {
                return Rgba::opaque(128, 0, 128);
            }
            _ => {}
        }
    }
    line_color_of_affiliation(symbol)
}

/// `hasDefaultFill` and `getFillColorOfAffiliation` for control measures:
/// only five entities have a fill unless the user gives one. (Other symbol
/// sets are tactical graphics too, so `hasDefaultFill` is false for them.)
pub(super) fn default_fill_color(symbol: &SymbolId) -> Option<Rgba> {
    if symbol.symbol_set() != SYMBOL_SET_CONTROL_MEASURES {
        return None;
    }
    let (r, g, b) = match symbol.entity().get() {
        200_101 => (255, 155, 0),
        200_201 | 200_202 | 200_600 => (85, 119, 136),
        200_700 => (51, 136, 136),
        _ => return None,
    };
    Some(Rgba {
        r,
        g,
        b,
        a: QUARTER_ALPHA,
    })
}
