//! Port of clsMETOC.SetMeTOCProperties: the colours, dash style, cap and
//! width each weather and oceanographic type is drawn with (MIL-STD-2525
//! Appendix C). The types have no user-defined fills, so any fill colour
//! comes from here.

use crate::engine::modifier::center_label::symbol_version;
use crate::engine::settings::{BASE_DPI, Settings};
use crate::engine::tactical_lines::*;
use crate::engine::tg::{CAP_BUTT, CAP_ROUND, Tg};
use crate::style::Rgba;

const BLACK: Rgba = Rgba::opaque(0, 0, 0);
const WHITE: Rgba = Rgba::opaque(255, 255, 255);
const RED: Rgba = Rgba::opaque(255, 0, 0);
const GREEN: Rgba = Rgba::opaque(0, 255, 0);
const BLUE: Rgba = Rgba::opaque(0, 0, 255);
const YELLOW: Rgba = Rgba::opaque(255, 255, 0);
const MAGENTA: Rgba = Rgba::opaque(255, 0, 255);
const GRAY: Rgba = Rgba::opaque(128, 128, 128);
const PURPLE: Rgba = Rgba::opaque(160, 32, 240);
const ORANGE: Rgba = Rgba::opaque(255, 128, 0);

/// How a type changes the line width.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Width {
    Keep,
    Double,
    /// Turbulence draws at least six pixels per 96 dpi.
    TurbulenceMinimum,
}

/// What a type sets on the graphic. `fill` is `Some(None)` to clear it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    line: Option<Rgba>,
    fill: Option<Option<Rgba>>,
    style: Option<i32>,
    cap: Option<i32>,
    width: Width,
}

fn line(c: Rgba) -> Look {
    Look {
        line: Some(c),
        fill: None,
        style: None,
        cap: None,
        width: Width::Keep,
    }
}

/// Line and fill in one colour.
fn same(c: Rgba) -> Look {
    Look {
        fill: Some(Some(c)),
        ..line(c)
    }
}

impl Look {
    fn style(self, style: i32) -> Self {
        Self {
            style: Some(style),
            ..self
        }
    }

    fn cap(self, cap: i32) -> Self {
        Self {
            cap: Some(cap),
            ..self
        }
    }

    fn fill(self, fill: Option<Rgba>) -> Self {
        Self {
            fill: Some(fill),
            ..self
        }
    }

    fn width(self, width: Width) -> Self {
        Self { width, ..self }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba::opaque(r, g, b)
}

/// Bottom type and bottom category colours.
fn bottom_look(line_type: i32) -> Option<Look> {
    Some(same(match line_type {
        BOTTOM_TYPE_A1 => rgb(48, 255, 0),
        BOTTOM_TYPE_A2 => rgb(127, 255, 0),
        BOTTOM_TYPE_A3 => rgb(175, 255, 0),
        BOTTOM_TYPE_B1 => rgb(207, 255, 0),
        BOTTOM_TYPE_B3 => rgb(255, 207, 0),
        BOTTOM_TYPE_C2 => rgb(255, 80, 0),
        BOTTOM_TYPE_C3 => rgb(255, 48, 0),
        IMPACT_BURIAL_0 => BLUE,
        BOTTOM_TYPE_C1 | IMPACT_BURIAL_75 => rgb(255, 127, 0),
        BOTTOM_CATEGORY_C | IMPACT_BURIAL_100 | CLUTTER_HIGH | BOTTOM_ROUGHNESS_ROUGH => RED,
        BOTTOM_TYPE_B2
        | BOTTOM_CATEGORY_B
        | IMPACT_BURIAL_20
        | CLUTTER_MEDIUM
        | BOTTOM_ROUGHNESS_MODERATE => YELLOW,
        BOTTOM_CATEGORY_A | IMPACT_BURIAL_10 | CLUTTER_LOW | BOTTOM_ROUGHNESS_SMOOTH => GREEN,
        _ => return None,
    }))
}

/// Sediment and vertical-dimension colours.
fn sediment_look(line_type: i32) -> Option<Look> {
    Some(same(match line_type {
        BOTTOM_SEDIMENTS_NO_DATA => rgb(230, 230, 230),
        BOTTOM_SEDIMENTS_LAND => rgb(220, 220, 220),
        SAND_AND_SHELLS => rgb(255, 220, 220),
        PEBBLES => rgb(255, 190, 190),
        OYSTER_SHELLS => rgb(255, 150, 150),
        BOULDERS => RED,
        COARSE_SILT => rgb(200, 255, 105),
        MEDIUM_SILT => GREEN,
        FINE_SILT => rgb(25, 255, 230),
        VERY_FINE_SILT => rgb(0, 215, 255),
        VERY_FINE_SAND => rgb(255, 255, 220),
        FINE_SAND => rgb(255, 255, 140),
        MEDIUM_SAND => rgb(255, 235, 0),
        COARSE_SAND => rgb(255, 215, 0),
        VERY_COARSE_SAND => rgb(255, 180, 0),
        CLAY => rgb(100, 130, 255),
        SOLID_ROCK => MAGENTA,
        VDR_LEVEL_12 => rgb(26, 153, 77),
        VDR_LEVEL_23 => rgb(26, 204, 77),
        VDR_LEVEL_34 => rgb(128, 255, 51),
        VDR_LEVEL_45 => rgb(204, 255, 26),
        VDR_LEVEL_56 => YELLOW,
        VDR_LEVEL_67 => rgb(255, 204, 0),
        VDR_LEVEL_78 => rgb(255, 128, 0),
        VDR_LEVEL_89 => rgb(255, 77, 0),
        VDR_LEVEL_910 => RED,
        _ => return None,
    }))
}

/// Areas and fronts whose line and fill share a colour.
fn area_look(line_type: i32) -> Option<Look> {
    Some(same(match line_type {
        LOADING_FACILITY_AREA | ISLAND => rgb(210, 180, 140),
        FORESHORE_AREA => rgb(173, 255, 47),
        PIPE => GRAY,
        WATER => WHITE,
        OFY | OCCLUDED => PURPLE,
        WFY | WFG | WF => RED,
        CFG | CFY | CF => BLUE,
        _ => return None,
    }))
}

/// Types that only set the line colour.
fn line_colour_look(line_type: i32) -> Option<Look> {
    Some(line(match line_type {
        OPERATOR_DEFINED => ORANGE,
        FORESHORE_LINE => rgb(173, 255, 47),
        RESTRICTED_AREA | TRAINING_AREA | ANCHORAGE_LINE | ANCHORAGE_AREA | CABLE => MAGENTA,
        UOF => PURPLE,
        UWF | IFR | FROZEN | JET | JET_GE => RED,
        UCF | MVFR => BLUE,
        SEAWALL | SEAWALL_GE | FLOOD_TIDE | FLOOD_TIDE_GE | EBB_TIDE | EBB_TIDE_GE
        | JETTY_ABOVE_WATER | JETTY_ABOVE_WATER_GE | DEPTH_CURVE | DEPTH_CURVE_GE
        | DEPTH_CONTOUR | DEPTH_CONTOUR_GE | COASTLINE | COASTLINE_GE | PIER | PIER_GE => GRAY,
        ISODROSOTHERM | ISODROSOTHERM_GE | NON_CONVECTIVE => GREEN,
        SAND => rgb(165, 121, 82),
        ICING => rgb(189, 154, 56),
        FOG => YELLOW,
        _ => return black_look(line_type),
    }))
}

/// Types drawn in black with nothing else changed.
fn black_look(line_type: i32) -> Option<Look> {
    matches!(
        line_type,
        LRO | UNDERCAST
            | LVO
            | RIDGE
            | ICE_OPENINGS_LEAD
            | ICE_OPENINGS_LEAD_GE
            | ICE_OPENINGS_FROZEN
            | ICE_OPENINGS_FROZEN_GE
            | LEADING_LINE
            | STREAM
            | STREAM_GE
            | CRACKS
            | CRACKS_GE
            | CRACKS_SPECIFIC_LOCATION
            | CRACKS_SPECIFIC_LOCATION_GE
            | ISOBAR
            | ISOBAR_GE
            | UPPER_AIR
            | UPPER_AIR_GE
            | ICE_EDGE
            | ICE_EDGE_GE
            | ICE_EDGE_RADAR
            | ICE_EDGE_RADAR_GE
            | REEF
            | RAMP_ABOVE_WATER
            | RAMP_ABOVE_WATER_GE
    )
    .then(|| line(BLACK))
}

/// Types that also set a dash style, cap, width or fill.
fn styled_look(line_type: i32) -> Option<Look> {
    Some(match line_type {
        SQUALL => line(BLACK).cap(CAP_BUTT),
        TROUGH => line(BLACK).style(1).cap(CAP_ROUND),
        UPPER_TROUGH => line(BLACK).cap(CAP_ROUND),
        CANAL => line(BLACK).width(Width::Double),
        MARITIME_LIMIT | MARITIME_AREA => line(MAGENTA).style(1),
        PERCHES | SUBMERGED_CRIB => line(BLACK).style(2).cap(CAP_ROUND).fill(Some(BLUE)),
        DISCOLORED_WATER | UNDERWATER_HAZARD => line(BLACK).style(2).fill(Some(rgb(0, 191, 255))),
        LOADING_FACILITY_LINE => line(GRAY).width(Width::Double),
        DRYDOCK => line(BLACK).fill(Some(rgb(205, 133, 63))).style(1),
        FISH_TRAPS => line(rgb(192, 192, 192)).style(1),
        SWEPT_AREA | OIL_RIG_FIELD | FOUL_GROUND | KELP => Look {
            line: None,
            ..line(BLACK)
        },
        BEACH => line(rgb(206, 158, 140)).fill(Some(Rgba {
            a: (255.0 * 0.12) as u8,
            ..rgb(206, 158, 140)
        })),
        DEPTH_AREA => line(BLUE).fill(Some(WHITE)),
        CONVERGENCE | ITC | ITCZ_LADDER => line(ORANGE).cap(CAP_BUTT),
        TURBULENCE => line(BLUE)
            .style(2)
            .cap(CAP_ROUND)
            .width(Width::TurbulenceMinimum),
        BEACH_SLOPE_MODERATE | BEACH_SLOPE_FLAT => line(rgb(179, 179, 179)).fill(None),
        BEACH_SLOPE_GENTLE | BEACH_SLOPE_STEEP => line(GRAY).fill(None),
        BREAKERS | JETTY_BELOW_WATER | JETTY_BELOW_WATER_GE => line(GRAY).style(1),
        THUNDERSTORMS => line(RED).style(3),
        RAMP_BELOW_WATER | RAMP_BELOW_WATER_GE | ESTIMATED_ICE_EDGE | ESTIMATED_ICE_EDGE_GE => {
            line(BLACK).style(1)
        }
        INSTABILITY => line(BLACK).style(4).cap(CAP_ROUND),
        SHEAR => line(BLACK).style(3).cap(CAP_ROUND),
        ISOPLETHS | ISOPLETHS_GE | ISOTHERM | ISOTHERM_GE => line(RED).style(1),
        ISOTACH | ISOTACH_GE => line(PURPLE).style(1),
        CONVECTIVE => line(GREEN).style(3),
        _ => return None,
    })
}

/// MIL-STD-2525E change 1 (version 15) colours that differ from upstream's:
/// Offshore Loading Facility - Area is the brown of its template (TABLE
/// M-III), not upstream's tan.
fn edition_look(line_type: i32, version: Option<i32>) -> Option<Look> {
    match (version, line_type) {
        (Some(15), LOADING_FACILITY_AREA) => Some(same(rgb(189, 154, 56))),
        (Some(15), ISLAND) => Some(same(rgb(210, 176, 106))),
        _ => None,
    }
}

fn look_for(line_type: i32, version: Option<i32>) -> Option<Look> {
    edition_look(line_type, version)
        .or_else(|| bottom_look(line_type))
        .or_else(|| sediment_look(line_type))
        .or_else(|| area_look(line_type))
        .or_else(|| styled_look(line_type))
        .or_else(|| line_colour_look(line_type))
}

/// `SetMeTOCProperties`.
pub(crate) fn set_metoc_properties(tg: &mut Tg, settings: &Settings) {
    let Some(look) = look_for(tg.line_type, symbol_version(&tg.symbol_id)) else {
        return;
    };
    tg.line_color = look.line;
    if let Some(fill) = look.fill {
        tg.fill_color = fill;
    }
    if let Some(style) = look.style {
        tg.line_style = style;
    }
    if let Some(cap) = look.cap {
        tg.line_cap = cap;
    }
    match look.width {
        Width::Keep => {}
        Width::Double => tg.line_thickness = tg.line_thickness.wrapping_mul(2),
        Width::TurbulenceMinimum => {
            let minimum = (settings.dpi / BASE_DPI).max(1) * 6;
            if tg.line_thickness < minimum {
                tg.line_thickness = minimum;
            }
        }
    }
}
