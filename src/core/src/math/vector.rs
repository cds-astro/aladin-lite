use crate::math::angle::Angle;
use cgmath::{
    Vector3, Vector2,
    BaseFloat,
    InnerSpace
};

#[inline]
pub fn angle2<S: BaseFloat>(ab: &Vector2<S>, bc: &Vector2<S>) -> Angle<S> {
    Angle((ab.dot(*bc)).acos())
}

#[inline]
pub fn angle3<S: BaseFloat>(x: &Vector3<S>, y: &cgmath::Vector3<S>) -> Angle<S> {
    Angle(x.cross(*y).magnitude().atan2(x.dot(*y)))
}
