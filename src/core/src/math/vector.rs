use crate::math::angle::Angle;
use cgmath::{BaseFloat, InnerSpace, Vector2, Vector3};

#[inline]
pub fn angle2<S: BaseFloat>(ab: &Vector2<S>, bc: &Vector2<S>) -> Angle<S> {
    Angle((ab.dot(*bc)).acos())
}

#[inline]
pub fn angle3<S: BaseFloat>(x: &Vector3<S>, y: &cgmath::Vector3<S>) -> Angle<S> {
    let theta = x.cross(*y).magnitude().atan2(x.dot(*y));
    Angle(theta)
}

#[inline]
pub fn dist2<S>(a: &[S; 2], b: &[S; 2]) -> S
where
    S: BaseFloat,
{
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[inline]
pub fn ccw_tri<'a, S, V>(a: V, b: V, c: V) -> bool
where
    S: BaseFloat + 'a,
    V: AsRef<[S; 2]>,
{
    let a: &[S; 2] = a.as_ref();
    let b: &[S; 2] = b.as_ref();
    let c: &[S; 2] = c.as_ref();

    // From: https://math.stackexchange.com/questions/1324179/how-to-tell-if-3-connected-points-are-connected-clockwise-or-counter-clockwise
    // | x1, y1, 1 |
    // | x2, y2, 1 | > 0 => the triangle is given in anticlockwise order
    // | x3, y3, 1 |
    a[0] * b[1] + a[1] * c[0] + b[0] * c[1] - c[0] * b[1] - c[1] * a[0] - b[0] * a[1] >= S::zero()
}

#[inline]
pub fn det<S: BaseFloat>(a: &Vector2<S>, b: &Vector2<S>) -> S {
    a.x * b.y - a.y * b.x
}

#[inline]
pub fn dot<S: BaseFloat>(a: &Vector2<S>, b: &Vector2<S>) -> S {
    a.x * b.x + a.y * b.y
}

pub struct NormedVector2(Vector2<f64>);

impl NormedVector2 {
    pub fn new(x: f64, y: f64) -> Self {
        let v = Vector2::new(x, y);
        let normed_v = v.normalize();

        Self(normed_v)
    }

    pub const unsafe fn new_unsafe(x: f64, y: f64) -> Self {
        let v = Vector2::new(x, y);
        Self(v)
    }
}

use std::borrow::Borrow;
use std::ops::Deref;
impl Deref for NormedVector2 {
    type Target = Vector2<f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::ops::Mul;
impl<'a> Mul<f64> for &'a NormedVector2 {
    // The multiplication of rational numbers is a closed operation.
    type Output = Vector2<f64>;

    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}
