//! Port of `MoveChannelPixels` and `moveSingleCPixels` from lineutility.java:
//! nudges coincident points apart so slopes stay defined.

use crate::engine::base::{At, EngineError, Pt};
use crate::engine::tactical_lines as lt;

/// Upstream `MoveChannelPixels`: while two consecutive points coincide, moves
/// the second one to the next whole pixel in x.
pub(crate) fn move_channel_pixels(pts: &mut [Pt]) -> Result<(), EngineError> {
    loop {
        let mut repeated = None;
        for j in 0..pts.len().saturating_sub(1) {
            let (p1, p2) = (pts.at(j)?, pts.at(j + 1)?);
            if p1.x == p2.x && p1.y == p2.y {
                repeated = Some(j + 1);
                break;
            }
        }
        match repeated {
            Some(k) => {
                let p = pts.at_mut(k)?;
                p.x = (p.x as i64).wrapping_add(1) as f64;
            }
            None => return Ok(()),
        }
    }
}

/// Upstream `moveSingleCPixels`: for SINGLEC, separates the first two points
/// vertically when they share a y.
pub(crate) fn move_single_c_pixels(linetype: i32, pts: &mut [Pt]) -> Result<(), EngineError> {
    if linetype != lt::SINGLEC || pts.len() <= 1 {
        return Ok(());
    }
    if pts.at(1)?.y == pts.at(0)?.y {
        pts.at_mut(1)?.y += 1.0;
    }
    Ok(())
}
