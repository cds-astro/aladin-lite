use crate::math;
use crate::math::angle::ToAngle;
use cgmath::{BaseFloat, InnerSpace};
use cgmath::{Euler, Quaternion};
use cgmath::Vector3;

#[derive(Clone, Copy, Debug)]
// Internal structure of a rotation, a quaternion
// All operations are done on it
pub struct Rotation<S: BaseFloat>(pub Quaternion<S>);

use cgmath::{Matrix3};
impl<S> From<&Matrix3<S>> for Rotation<S>
where
    S: BaseFloat,
{
    fn from(m: &Matrix3<S>) -> Self {
        /*let m: [[S; 3]; 3] = (*m).into();

        let t = Matrix3::new(
            m[0][0], m[0][1], m[0][2],
            m[1][0], m[1][1], m[1][2],
            m[2][0], m[2][1], m[2][2],
        );*/

        Rotation((*m).into())
    }
}
impl<S> From<&Rotation<S>> for Matrix3<S>
where
    S: BaseFloat,
{
    fn from(s: &Rotation<S>) -> Self {
        s.0.into()
    }
}
use crate::math::angle::Angle;
use cgmath::Matrix;
use cgmath::Rad;
impl<S> Rotation<S>
where
    S: BaseFloat,
{
    pub fn slerp(&self, other: &Rotation<S>, alpha: S) -> Rotation<S> {
        // Check if the dot of the two quaternions is negative
        // If so, negative one:
        // This comes from https://www.gamedev.net/forums/topic/551663-quaternion-rotation-issue-sometimes-take-longest-path/
        let d = self.0.dot(other.0);
        let q = if d < S::zero() {
            self.0.slerp(-other.0, alpha)
        } else {
            self.0.slerp(other.0, alpha)
        };

        Rotation(q)
    }

    pub fn get_rot_x(&self) -> Matrix3<S> {
        let mut q = self.0;

        q.v.z = S::zero();
        q.v.y = S::zero();

        let norm = (q.v.x * q.v.x + q.s * q.s).sqrt();
        q.v.x /= norm;
        q.s /= norm;

        q.into()
    }

    pub fn get_rot_y(&self) -> Matrix3<S> {
        let mut q = self.0;

        q.v.x = S::zero();
        q.v.z = S::zero();

        let norm = (q.v.y * q.v.y + q.s * q.s).sqrt();
        q.v.y /= norm;
        q.s /= norm;

        q.into()
    }

    pub fn get_rot_z(&self) -> Matrix3<S> {
        let mut q = self.0;

        q.v.x = S::zero();
        q.v.y = S::zero();

        let norm = (q.v.z * q.v.z + q.s * q.s).sqrt();
        q.v.z /= norm;
        q.s /= norm;

        q.into()
    }

    pub fn zero() -> Rotation<S> {
        let q = Quaternion::new(S::one(), S::zero(), S::zero(), S::zero());
        Rotation(q)
    }

    // Define a rotation from a quaternion
    pub fn from_quaternion(q: &Quaternion<S>) -> Rotation<S> {
        Rotation(*q)
    }

    pub fn from_matrix3(mat: &Matrix3<S>) -> Rotation<S> {
        mat.into()
    }

    // Define a rotation from an axis and a angle
    pub fn from_axis_angle(axis: &Vector3<S>, angle: Angle<S>) -> Rotation<S> {
        let angle: Rad<S> = angle.into();
        let mat = Matrix3::from_axis_angle(axis.normalize(), angle);
        (&mat).into()
    }

    // Define a rotation from a normalized vector
    pub fn from_sky_position(pos: &Vector3<S>) -> Rotation<S> {
        let (lon, lat) = math::lonlat::xyz_to_radec(&pos);

        let rot_y = Matrix3::from_angle_y(lon);
        let rot_x = Matrix3::from_angle_x(-lat);

        let mat = rot_y * rot_x;
        (&(mat)).into()
    }

    // Apply a rotation to a position
    pub fn rotate(&self, pos_world_space: &Vector3<S>) -> Vector3<S> {
        let w2m: &Matrix3<S> = &self.into();

        w2m * pos_world_space
    }
    pub fn inv_rotate(&self, pos_model_space: &Vector3<S>) -> Vector3<S> {
        let w2m: &Matrix3<S> = &self.into();
        let m2w = w2m.transpose();

        m2w * pos_model_space
    }

    pub fn euler(&self) -> Euler<Rad<S>> {
        self.0.into()
    }

    /// Extract the 3 euler angles from the quaternion
    /// Aladin Lite rotation basis is formed by Z, X, Y axis:
    /// * Z axis is pointing towards us
    /// * Y is pointing upward
    /// * X is defined from the right-hand rule to form a basis
    ///
    /// The first euler angle describes the longitude (rotation around the Y axis) <=> pitch
    /// The second euler angle describes the latitude (rotation around the X' modified axis) <=> yaw
    /// The third euler angle describes a rotation deviation from the north pole (rotation around the Z'' modified axis) <=> roll
    ///
    /// Equations come from this paper (Appendix 6):
    /// https://ntrs.nasa.gov/api/citations/19770024290/downloads/19770024290.pdf
    pub fn euler_yxz(&self) -> (Angle<S>, Angle<S>, Angle<S>) {
        let m: Matrix3<S> = self.0.into();

        let a = m.x.z.atan2(m.z.z);
        let b = (-m.z.y).atan2((S::one() - m.z.y * m.z.y).sqrt());
        let c = m.x.y.atan2(m.y.y);
        (a.to_angle(), b.to_angle(), c.to_angle())
    }
}

use std::ops::Mul;
impl<S> Mul<Rotation<S>> for Rotation<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn mul(self, rhs: Rotation<S>) -> Self::Output {
        let q = self.0 * rhs.0;
        Rotation(q)
    }
}
