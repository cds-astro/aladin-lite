use crate::math::projection::coo_space::XYClip;
use cgmath::Vector2;

pub struct Ellipse {
    // Semi-major axis length
    pub a: f64,
    // Semi-minor axis length
    pub b: f64,
}

use super::super::sdf::ProjDef;
use cgmath::InnerSpace;

impl ProjDef for Ellipse {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let mut p = Vector2::new(xy.x.abs(), xy.y.abs());
        let mut ab = Vector2::new(self.a, self.b);

        let sdf = if p.x == 0.0 {
            -(self.b - p.y)
        } else if p.y == 0.0 {
            -(self.a - p.x)
        } else {
            if p.x > p.y {
                p = Vector2::new(p.y, p.x);
                ab = Vector2::new(ab.y, ab.x);
            }

            let l = ab.y * ab.y - ab.x * ab.x;
            let m = ab.x * p.x / l;
            let m2 = m * m;
            let n = ab.y * p.y / l;
            let n2 = n * n;
            let c = (m2 + n2 - 1.0) / 3.0;
            let c3 = c * c * c;
            let q = c3 + m2 * n2 * 2.0;
            let d = c3 + m2 * n2;
            let g = m + m * n2;

            let co = if d < 0.0 {
                let p = (q / c3).acos() / 3.0;
                let s = p.cos();
                let t = p.sin() * (3.0_f64).sqrt();
                let rx = (-c * (s + t + 2.0) + m2).sqrt();
                let ry = (-c * (s - t + 2.0) + m2).sqrt();

                (ry + (l).signum() * rx + ((g).abs() / (rx * ry)) - m) / 2.0
            } else {
                let h = 2.0 * m * n * ((d).sqrt());
                let s = (q + h).signum() * ((q + h).abs()).powf(1.0 / 3.0);
                let u = (q - h).signum() * ((q - h).abs()).powf(1.0 / 3.0);
                let rx = -s - u - c * 4.0 + 2.0 * m2;
                let ry = (s - u) * (3.0_f64).sqrt();

                let rm = (rx * rx + ry * ry).sqrt();
                let p = ry / ((rm - rx).sqrt());
                (p + (2.0 * g / rm) - m) / 2.0
            };

            let si = (1.0 - co * co).sqrt();
            let q = Vector2::new(ab.x * co, ab.y * si);

            (q - p).magnitude() * (p.y - q.y).signum()
        };

        sdf
    }
}
