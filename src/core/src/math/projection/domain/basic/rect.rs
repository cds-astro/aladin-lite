use crate::math::projection::coo_space::XYClip;

use cgmath::Vector2;
pub struct Rect {
    pub dim: Vector2<f64>,
}

use super::super::sdf::ProjDef;

use cgmath::InnerSpace;
impl ProjDef for Rect {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let d = Vector2::new(xy.x.abs() - self.dim.x, xy.y.abs() - self.dim.y);

        let a = Vector2::new(d.x.max(0.0), d.y.max(0.0));
        let b = (d.x.max(d.y)).min(0.0);

        a.magnitude() + b
    }
}
