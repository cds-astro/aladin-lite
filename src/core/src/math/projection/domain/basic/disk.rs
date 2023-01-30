use crate::math::projection::coo_space::XYClip;

pub struct Disk {
    pub radius: f64
}

use cgmath::InnerSpace;
use super::super::sdf::ProjDef;

impl ProjDef for Disk {
    fn sdf(&self, xy: &XYClip) -> f64 {
        xy.magnitude() - self.radius
    }
}