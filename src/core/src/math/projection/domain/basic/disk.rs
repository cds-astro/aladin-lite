use crate::math::projection::coo_space::XYClip;

pub struct Disk {
    pub radius: f64,
}

use super::super::sdf::ProjDef;
use cgmath::InnerSpace;

impl ProjDef for Disk {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        xy.magnitude() - self.radius
    }
}
