/// Some basic Signed distance functions
use crate::math::projection::coo_space::XYClip;

#[enum_dispatch(ProjDefType)]
pub trait ProjDef {
    fn is_in(&self, xy: &XYClip<f64>) -> bool {
        self.sdf(xy) <= 0.0
    }

    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64;
}

use crate::math::vector::NormedVector2;
/// Project a vertex on a valid region defined by a Signed Distance Function (SDF)
///
/// # Arguments
///
/// * `p` - A vertex in the clipping space
/// * `dir` - A direction of the normed vector
/// * `valid_reg` - The projection definition region
pub fn ray_marching<P>(p: &XYClip<f64>, dir: &NormedVector2, valid_reg: &P) -> Option<XYClip<f64>>
where
    P: ProjDef,
{
    // This is done so that we get further a little bit
    let in_clip_space = |p: &XYClip<f64>| -> bool {
        ((-1.0)..=1.0).contains(&p.x) && ((-1.0)..=1.0).contains(&p.y)
    };

    let mut v = *p;
    let mut is_in = valid_reg.is_in(&v);
    const N_MAX_ITER: usize = 20;
    let mut i = 0;
    while in_clip_space(&v) && !is_in && i < N_MAX_ITER {
        let d = valid_reg.sdf(&v);

        // Perform the ray marching advancement
        v = v + dir * d;
        is_in = valid_reg.is_in(&v);

        i += 1;
    }

    if !in_clip_space(&v) {
        None
    } else {
        Some(v)
    }
}

use super::{basic::disk::Disk, cod::Cod, full::FullScreen, hpx::Hpx, par::Par};

// List of all the footprints
// found in Aladin Lite
#[enum_dispatch]
pub enum ProjDefType {
    Disk,
    Par,
    Cod,
    FullScreen,
    Hpx,
}
