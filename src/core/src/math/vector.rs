use crate::math::angle::Angle;
use cgmath::{BaseFloat, InnerSpace, Vector2, Vector3};

#[inline]
pub fn angle2<S: BaseFloat>(ab: &Vector2<S>, bc: &Vector2<S>) -> Angle<S> {
    Angle((ab.dot(*bc)).acos())
}

#[inline]
pub fn angle3<S: BaseFloat>(x: &Vector3<S>, y: &cgmath::Vector3<S>) -> Angle<S> {
    Angle(x.cross(*y).magnitude().atan2(x.dot(*y)))
}

#[inline]
pub fn dist2(a: &Vector2<f64>, b: &Vector2<f64>) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    return  dx*dx + dy*dy;
}

#[inline]
pub fn ccw_tri<S: BaseFloat>(a: &Vector2<S>, b: &Vector2<S>, c: &Vector2<S>) -> bool {
    // From: https://math.stackexchange.com/questions/1324179/how-to-tell-if-3-connected-points-are-connected-clockwise-or-counter-clockwise
    // | x1, y1, 1 |
    // | x2, y2, 1 | > 0 => the triangle is given in anticlockwise order
    // | x3, y3, 1 |

    a.x*b.y + a.y*c.x + b.x*c.y - c.x*b.y - c.y*a.x - b.x*a.y > S::zero()
}
