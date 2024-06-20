use crate::math::projection::coo_space::XYClip;

use cgmath::Vector2;
pub struct Parabola {
    // Quadratic coefficient
    pub k: f64,
}

use super::super::sdf::ProjDef;

use cgmath::InnerSpace;
impl ProjDef for Parabola {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let mut xy = *xy;

        // There is a singularity around x == 0
        // I add an offset to avoid handling this case but
        // we should treat it more properly
        if xy.x == 0.0 {
            xy.x += 1e-4;
        }
        xy.x = xy.x.abs();
        let ik = 1.0 / self.k;
        let p = ik * (xy.y - 0.5 * ik) / 3.0;
        let q = 0.25 * ik * ik * xy.x;
        let h = q * q - p * p * p;
        let r = h.abs().sqrt();
        let x = if h > 0.0 {
            (q + r).powf(1.0 / 3.0) - (q - r).abs().powf(1.0 / 3.0) * (r - q).signum()
        } else {
            2.0 * (r.atan2(q) / 3.0).cos() * p.sqrt()
        };
        let a = if xy.x - x < 0.0 { -1.0 } else { 1.0 };
        (xy - Vector2::new(x, self.k * x * x)).magnitude() * a
    }
}
