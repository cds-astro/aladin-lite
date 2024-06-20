use crate::math::projection::coo_space::XYClip;
use cgmath::Vector2;

pub struct Hpx;

use super::sdf::ProjDef;
use super::{
    basic::{rect::Rect, triangle::Triangle},
    op::Union,
};
impl ProjDef for Hpx {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let rect = Rect {
            dim: Vector2::new(1.0, 0.5),
        };

        let t1 = Triangle {
            p0: Vector2::new(1.0, 0.5),
            p1: Vector2::new(0.5, 0.5),
            p2: Vector2::new(0.75, 1.0),
        };
        let t2 = Triangle {
            p0: Vector2::new(0.5, 0.5),
            p1: Vector2::new(0.0, 0.5),
            p2: Vector2::new(0.25, 1.0),
        };

        let t3 = Triangle {
            p0: Vector2::new(-1.0, 0.5),
            p1: Vector2::new(-0.5, 0.5),
            p2: Vector2::new(-0.75, 1.0),
        };
        let t4 = Triangle {
            p0: Vector2::new(-0.5, 0.5),
            p1: Vector2::new(-0.0, 0.5),
            p2: Vector2::new(-0.25, 1.0),
        };

        let t5 = Triangle {
            p0: Vector2::new(-1.0, -0.5),
            p1: Vector2::new(-0.5, -0.5),
            p2: Vector2::new(-0.75, -1.0),
        };
        let t6 = Triangle {
            p0: Vector2::new(-0.5, -0.5),
            p1: Vector2::new(-0.0, -0.5),
            p2: Vector2::new(-0.25, -1.0),
        };

        let t7 = Triangle {
            p0: Vector2::new(1.0, -0.5),
            p1: Vector2::new(0.5, -0.5),
            p2: Vector2::new(0.75, -1.0),
        };
        let t8 = Triangle {
            p0: Vector2::new(0.5, -0.5),
            p1: Vector2::new(0.0, -0.5),
            p2: Vector2::new(0.25, -1.0),
        };

        let t12 = Union::new(t1, t2);
        let t34 = Union::new(t3, t4);
        let t56 = Union::new(t5, t6);
        let t78 = Union::new(t7, t8);

        let t_all = Union::new(Union::new(t12, t34), Union::new(t56, t78));
        Union::new(t_all, rect).sdf(xy)
    }
}
