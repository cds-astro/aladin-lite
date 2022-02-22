use cgmath::Rad;
use cgmath::{BaseFloat, InnerSpace};
use cgmath::{Vector2, Vector3, Vector4};

#[inline]
pub fn angle<S: BaseFloat>(ab: &Vector2<S>, bc: &Vector2<S>) -> Angle<S> {
    Angle((ab.dot(*bc)).acos())
}

use num_traits::Float;
#[inline]
pub fn asinc_positive<T: Float>(mut x: f64) -> f64 {
    assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.asin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        x *= x;
        1.0 + x * (1.0 + x * 9.0 / 20.0) / 6.0
    }
}

#[inline]
pub fn sinc_positive(mut x: f64) -> f64 {
    assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.sin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        x *= x;
        1.0 - x * (1.0 - x / 20.0) / 6.0
    }
}

#[inline]
pub fn ang_between_vect<S: BaseFloat>(x: &Vector3<S>, y: &cgmath::Vector3<S>) -> Angle<S> {
    let rad = Rad(x.cross(*y).magnitude().atan2(x.dot(*y)));
    Angle::new(rad)
}

#[inline]
pub fn _ang_between_lonlat<S: BaseFloat>(
    lon1: Angle<S>,
    lat1: Angle<S>,
    lon2: Angle<S>,
    lat2: Angle<S>,
) -> Angle<S> {
    let abs_diff_lon = (lon1 - lon2).abs();
    Angle((lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * abs_diff_lon.cos()).acos())
}

pub trait LonLat<S: BaseFloat> {
    fn lon(&self) -> Angle<S>;
    fn lat(&self) -> Angle<S>;
    fn lonlat(&self) -> LonLatT<S>;
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self;
}

use crate::angle::Angle;
#[derive(Clone, Copy, Debug)]
pub struct LonLatT<S: BaseFloat>(pub Angle<S>, pub Angle<S>);

impl<S> LonLatT<S>
where
    S: BaseFloat,
{
    /// LonLat constructor
    ///
    /// # Arguments
    ///
    /// * ``lon`` - Longitude
    /// * ``lat`` - Latitude
    pub fn new(lon: Angle<S>, lat: Angle<S>) -> LonLatT<S> {
        LonLatT(lon, lat)
    }

    pub fn from_radians(lon: Rad<S>, lat: Rad<S>) -> LonLatT<S> {
        LonLatT(lon.into(), lat.into())
    }

    #[inline]
    pub fn lon(&self) -> Angle<S> {
        self.0
    }

    #[inline]
    pub fn lat(&self) -> Angle<S> {
        self.1
    }

    pub fn vector<VectorT: LonLat<S>>(&self) -> VectorT {
        VectorT::from_lonlat(&self)
    }
}

impl<S> LonLat<S> for Vector3<S>
where
    S: BaseFloat,
{
    #[inline]
    fn lon(&self) -> Angle<S> {
        let rad = Rad(self.x.atan2(self.z));
        Angle::new(rad)
    }

    #[inline]
    fn lat(&self) -> Angle<S> {
        let rad = Rad(self.y.atan2((self.x * self.x + self.z * self.z).sqrt()));
        Angle::new(rad)
    }

    #[inline]
    fn lonlat(&self) -> LonLatT<S> {
        let lon = Rad(self.x.atan2(self.z));
        let lat = Rad(self.y.atan2((self.x * self.x + self.z * self.z).sqrt()));

        LonLatT(Angle::new(lon), Angle::new(lat))
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        let theta = lonlat.lon();
        let delta = lonlat.lat();

        Vector3::<S>::new(
            delta.cos() * theta.sin(),
            delta.sin(),
            delta.cos() * theta.cos(),
        )
    }
}

impl<S> LonLat<S> for Vector4<S>
where
    S: BaseFloat,
{
    #[inline]
    fn lon(&self) -> Angle<S> {
        let rad = Rad(self.x.atan2(self.z));
        Angle::new(rad)
    }

    #[inline]
    fn lat(&self) -> Angle<S> {
        let rad = Rad(self.y.atan2((self.x * self.x + self.z * self.z).sqrt()));
        Angle::new(rad)
    }

    #[inline]
    fn lonlat(&self) -> LonLatT<S> {
        let lon = Rad(self.x.atan2(self.z));
        let lat = Rad(self.y.atan2((self.x * self.x + self.z * self.z).sqrt()));

        LonLatT(Angle::new(lon), Angle::new(lat))
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        let theta = lonlat.lon();
        let delta = lonlat.lat();
        Vector4::<S>::new(
            delta.cos() * theta.sin(),
            delta.sin(),
            delta.cos() * theta.cos(),
            S::one(),
        )
    }
}

#[inline]
pub fn xyz_to_radec<S: BaseFloat>(v: &cgmath::Vector3<S>) -> (Angle<S>, Angle<S>) {
    let lon = Angle(v.x.atan2(v.z));
    let lat = Angle(v.y.atan2((v.x * v.x + v.z * v.z).sqrt()));

    (lon, lat)
}

#[inline]
pub fn xyzw_to_radec<S: BaseFloat>(v: &cgmath::Vector4<S>) -> (Angle<S>, Angle<S>) {
    let lon = Angle(v.x.atan2(v.z));
    let lat = Angle(v.y.atan2((v.x * v.x + v.z * v.z).sqrt()));

    (lon, lat)
}

#[inline]
pub fn radec_to_xyz<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Vector3<S> {
    Vector3::<S>::new(
        delta.cos() * theta.sin(),
        delta.sin(),
        delta.cos() * theta.cos(),
    )
}

#[inline]
pub fn radec_to_xyzw<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Vector4<S> {
    Vector4::<S>::new(
        delta.cos() * theta.sin(),
        delta.sin(),
        delta.cos() * theta.cos(),
        S::one(),
    )
}

#[inline]
const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

use num::traits::PrimInt;
use num::traits::Zero;
#[inline]
pub fn log_2_checked<T>(x: T) -> u32
where
    T: PrimInt + Zero
{
    assert!(x > T::zero());
    num_bits::<T>() as u32 - x.leading_zeros() - 1
}

#[inline]
pub fn log_2_unchecked<T>(x: T) -> u32
where
    T: PrimInt
{
    num_bits::<T>() as u32 - x.leading_zeros() - 1
}

use std::ops::BitAnd;
use std::ops::Sub;
use std::cmp::PartialEq;
use num::One;
#[inline]
pub fn is_power_of_two<T>(x: T) -> bool
where
    T: BitAnd<Output=T> + One + Zero + Sub<Output=T> + PartialEq + Copy
{
    x.bitand(x - T::one()) == T::zero()
}

/// Compute the negative branch of the lambert fonction (W_{-1})
/// defined for x in [-1/e; 0[
/// This paper: https://doi.org/10.1016/S0378-4754(00)00172-5
/// gives an analytical approximation with a relative error of 0.025%
#[inline]
#[allow(dead_code)]
pub fn lambert_wm1(x: f32) -> f32 {
    assert!(x < 0.0 && x >= -1.0 / std::f32::consts::E);
    let m1 = 0.3361;
    let m2 = -0.0042;
    let m3 = -0.0201;

    let s = -1.0 - (-x).ln();
    let s_root = s.sqrt();
    let s_div_2_root = (s * 0.5).sqrt();

    -1.0 - s
        - (2.0 / m1)
            * (1.0 - 1.0 / (1.0 + ((m1 * s_div_2_root) / (1.0 + m2 * s * (m3 * s_root).exp()))))
}
