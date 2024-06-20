use crate::math::projection::coo_space::XYClip;
use cgmath::Vector2;

pub struct FullScreen;

use super::{basic::rect::Rect, sdf::ProjDef};
impl ProjDef for FullScreen {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        Rect {
            dim: Vector2::new(1.0, 1.0),
        }
        .sdf(xy)
    }
}
