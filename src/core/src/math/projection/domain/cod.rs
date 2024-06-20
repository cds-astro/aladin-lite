use crate::math::projection::coo_space::XYClip;
use cgmath::Vector2;

use super::{
    basic::{ellipse::Ellipse, triangle::Triangle},
    op::{Diff, Translate},
    sdf::ProjDef,
};
use crate::math::angle::PI;
use crate::math::HALF_PI;

pub struct Cod {
    pub r_max: f64,
    pub r_min: f64,
    pub negative_ta: bool,
    // Angle is defined by c * PI
    pub c: f64,
    pub y0: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub x_min: f64,
    pub x_max: f64,
}

impl Cod {
    pub const fn new() -> Self {
        Self {
            r_min: 0.2146018366,
            r_max: 3.35619449019,
            negative_ta: false,
            c: 0.7071067811865475,
            y0: 1.0000000000000002,

            x_min: -3.356194490192345,
            x_max: 3.356194490192345,
            y_min: -2.356194490192345,
            y_max: 3.0328465566001492,
        }
    }

    fn to_clip(&self, xy: &Vector2<f64>) -> XYClip<f64> {
        let x = (xy.x - self.x_min) / (self.x_max - self.x_min);
        let y = (xy.y - self.y_min) / (self.y_max - self.y_min);

        XYClip::new((x - 0.5) * 2.0, (y - 0.5) * 2.0)
    }
}

impl ProjDef for Cod {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let y_mean = (self.y_min + self.y_max) * 0.5;
        let center_ellipse = self.to_clip(&Vector2::new(0.0, self.y0 + y_mean));

        // Big frontier ellipse
        let a = 1.0;
        let b = 2.0 * (2.356194490192345 + self.y0) / (2.356194490192345 + 3.0328465566001492);
        let e = b / a;
        let ext_ellipse = Translate {
            off: center_ellipse,
            def: Ellipse { a: a, b: b },
        };

        // Small ellipse where projection is not defined
        let b_int = 2.0 * self.r_min / (2.356194490192345 + 3.0328465566001492);
        let a_int = b_int / e;
        let int_ellipse = Translate {
            off: center_ellipse,
            def: Ellipse { a: a_int, b: b_int },
        };

        // The top edges
        let gamma = PI * self.c - HALF_PI;
        let (s_gam, c_gam) = gamma.sin_cos();

        let b = Vector2::new(c_gam * self.r_max, self.y0 + y_mean + s_gam * self.r_max);
        let c = Vector2::new(-c_gam * self.r_max, self.y0 + y_mean + s_gam * self.r_max);

        let tri = Triangle {
            p0: center_ellipse,
            p1: self.to_clip(&b),
            p2: self.to_clip(&c),
        };

        Diff::new(Diff::new(ext_ellipse, int_ellipse), tri).sdf(xy)
    }
}
