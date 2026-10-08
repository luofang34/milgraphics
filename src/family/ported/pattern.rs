//! The figures upstream tiles over some METOC areas from raster images
//! (`PatternFillRenderer.MakeMetocPatternFill`), as vector patterns of the
//! same size, spacing and colour as the images.

use crate::engine::tactical_lines::{
    BEACH_SLOPE_MODERATE, BEACH_SLOPE_STEEP, FISH_TRAPS, FOUL_GROUND, KELP, OIL_RIG_FIELD,
    SWEPT_AREA,
};
use crate::style::{Motif, Pattern, Rgba};

const SILVER: Rgba = Rgba::opaque(192, 192, 192);
const GRAY: Rgba = Rgba::opaque(128, 128, 128);

/// The pattern of METOC `line_type`. Each image is one grid cell, with a
/// second figure half a cell across and down where it holds two.
pub(crate) fn metoc(line_type: i32) -> Option<Pattern> {
    Some(match line_type {
        BEACH_SLOPE_MODERATE | BEACH_SLOPE_STEEP => Pattern::new(
            Motif::Dot,
            Rgba::opaque(204, 204, 204),
            6.0,
            [30.0, 30.0],
            false,
        ),
        OIL_RIG_FIELD => Pattern::new(Motif::Dot, SILVER, 20.0, [50.0, 50.0], false),
        SWEPT_AREA => Pattern::new(
            Motif::Dot,
            Rgba::opaque(255, 0, 255),
            38.0,
            [150.0, 150.0],
            true,
        ),
        FOUL_GROUND => Pattern::new(Motif::Hash, GRAY, 42.0, [200.0, 200.0], true),
        KELP => Pattern::new(Motif::Kelp, GRAY, 94.0, [200.0, 250.0], true),
        FISH_TRAPS => Pattern::new(Motif::FishTrap, SILVER, 62.0, [124.0, 104.0], true),
        _ => return None,
    })
}
