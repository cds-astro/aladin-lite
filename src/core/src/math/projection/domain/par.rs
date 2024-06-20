use crate::math::projection::coo_space::XYClip;
use cgmath::Vector2;

pub struct Par;

use super::{
    basic::parabola::Parabola,
    op::{Inter, Translate},
    sdf::ProjDef,
};
impl ProjDef for Par {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let xy = Vector2::new(xy.y, xy.x);

        let p1 = Translate {
            off: Vector2::new(0.0, -1.0),
            def: Parabola { k: 1.0 },
        };
        let p2 = Translate {
            off: Vector2::new(0.0, 1.0),
            def: Parabola { k: -1.0 },
        };

        Inter::new(p1, p2).sdf(&xy)
    }
}
