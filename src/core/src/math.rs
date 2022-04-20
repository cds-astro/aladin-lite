use cgmath::Rad;
use cgmath::{BaseFloat, InnerSpace};
use cgmath::{Vector2, Vector3, Vector4};

pub const TWICE_PI: f64 = 6.28318530718;
pub const PI: f64 = 3.14159265359;

#[inline]
pub fn angle<S: BaseFloat>(ab: &Vector2<S>, bc: &Vector2<S>) -> Angle<S> {
    Angle((ab.dot(*bc)).acos())
}

use num_traits::Float;
#[inline]
pub fn asinc_positive(x: f64) -> f64 {
    assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.asin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        let x2 = x*x;
        1.0 + x2/6.0 + x2*x2*0.075
    }
}

#[inline]
pub fn sinc_positive(x: f64) -> f64 {
    assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.sin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        let x2 = x*x;
        1.0 - x2/6.0 + x2*x2*0.075
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
        VectorT::from_lonlat(self)
    }
}

impl<S> LonLat<S> for LonLatT<S>
where
    S: BaseFloat,
{
    #[inline]
    fn lon(&self) -> Angle<S> {
        self.0
    }

    #[inline]
    fn lat(&self) -> Angle<S> {
        self.1
    }

    #[inline]
    fn lonlat(&self) -> LonLatT<S> {
        *self
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        *lonlat
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

 use al_api::coo_system::CooSystem;
 use al_api::coo_system::CooBaseFloat;
/// This is conversion method returning a transformation
/// matrix when the system requested by the user is not
/// icrs j2000.
/// The core projections are always performed in icrs j2000
/// so one must call these methods to convert them to icrs before.
#[inline]
pub fn apply_coo_system<'a, S>(c1: &CooSystem, c2: &CooSystem, v1: &Vector4<S>) -> Vector4<S>
where
    S: BaseFloat + CooBaseFloat,
{
    let c1_2_c2_mat = c1.to::<S>(c2);
    c1_2_c2_mat * (*v1)
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
    assert!((-1.0 / std::f32::consts::E..0.0).contains(&x));
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

mod tests {
    #[allow(unused_macros)]
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    use crate::{ArcDeg, LonLatT};
    use al_api::coo_system::CooSystem;
    use crate::math::LonLat;

    #[test]
    fn j2000_to_gal() {
        let lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let gal_lonlat = super::apply_coo_system(&CooSystem::ICRSJ2000, &CooSystem::GAL, &lonlat.vector()).lonlat();

        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 96.33723581, 1e-3);
        assert_delta!(gal_lat_deg, -60.18845577, 1e-3);
    }

    #[test]
    fn gal_to_j2000() {
        let lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let j2000_lonlat = super::apply_coo_system(&CooSystem::GAL, &CooSystem::ICRSJ2000, &lonlat.vector()).lonlat();
        let j2000_lon_deg = j2000_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let j2000_lat_deg = j2000_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(j2000_lon_deg, 266.40506655, 1e-3);
        assert_delta!(j2000_lat_deg, -28.93616241, 1e-3);
    }

    #[test]
    fn j2000_gal_roundtrip() {
        let gal_lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());

        let icrsj2000_pos = super::apply_coo_system(
            &CooSystem::GAL, 
            &CooSystem::ICRSJ2000,
            &gal_lonlat.vector()
        );

        let gal_lonlat = super::apply_coo_system(
            &CooSystem::ICRSJ2000, 
            &CooSystem::GAL,
            &icrsj2000_pos
        );

        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 0.0, 1e-3);
        assert_delta!(gal_lat_deg, 0.0, 1e-3);
    }
}
