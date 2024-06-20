use crate::math::projection::coo_space::XYClip;

use cgmath::Vector2;
pub struct Triangle {
    pub p0: Vector2<f64>,
    pub p1: Vector2<f64>,
    pub p2: Vector2<f64>,
}

use super::super::sdf::ProjDef;
use cgmath::InnerSpace;
impl ProjDef for Triangle {
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let e0 = self.p1 - self.p0;
        let e1 = self.p2 - self.p1;
        let e2 = self.p0 - self.p2;

        let v0 = xy - self.p0;
        let v1 = xy - self.p1;
        let v2 = xy - self.p2;

        let pq0 = v0 - e0 * (v0.dot(e0) / e0.dot(e0)).clamp(0.0, 1.0);
        let pq1 = v1 - e1 * (v1.dot(e1) / e1.dot(e1)).clamp(0.0, 1.0);
        let pq2 = v2 - e2 * (v2.dot(e2) / e2.dot(e2)).clamp(0.0, 1.0);

        let s = e0.x * e2.y - e0.y * e2.x;

        let d1 = Vector2::new(pq0.dot(pq0), s * (v0.x * e0.y - v0.y * e0.x));
        let d2 = Vector2::new(pq1.dot(pq1), s * (v1.x * e1.y - v1.y * e1.x));
        let d3 = Vector2::new(pq2.dot(pq2), s * (v2.x * e2.y - v2.y * e2.x));

        let d = Vector2::new(d1.x.min(d2.x.min(d3.x)), d1.y.min(d2.y.min(d3.y)));

        -d.x.sqrt() * (d.y.signum())
    }
}
