use crate::math::TWICE_PI;
use crate::Abort;
use cgmath::{BaseFloat, Matrix3, Rad, Vector3};

pub trait LonLat<S: BaseFloat> {
    fn lon(&self) -> Angle<S>;
    fn lat(&self) -> Angle<S>;
    fn lonlat(&self) -> LonLatT<S>;
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self;
}
use crate::math::angle::Angle;
use serde::Deserialize;
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
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
    pub fn new(mut lon: Angle<S>, lat: Angle<S>) -> LonLatT<S> {
        if lon.to_radians() < S::zero() {
            lon = lon + S::from(TWICE_PI).unwrap_abort();
        }

        LonLatT(lon, lat)
    }

    #[inline]
    pub fn lon(&self) -> Angle<S> {
        self.0
    }

    #[inline]
    pub fn lat(&self) -> Angle<S> {
        self.1
    }

    pub fn vector<T: LonLat<S>>(&self) -> T {
        T::from_lonlat(self)
    }
}

use crate::math::angle::ToAngle;
impl From<wcs::LonLat> for LonLatT<f64> {
    fn from(lonlat: wcs::LonLat) -> Self {
        Self(lonlat.lon().to_angle(), lonlat.lat().to_angle())
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

        LonLatT::new(Angle::new(lon), Angle::new(lat))
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        let theta = lonlat.lon();
        let delta = lonlat.lat();

        let (dc, ds) = (delta.cos(), delta.sin());
        let (tc, ts) = (theta.cos(), theta.sin());

        Vector3::<S>::new(dc * ts, ds, dc * tc)
    }
}

#[inline]
pub fn ang_between_lonlat<S: BaseFloat>(lonlat1: LonLatT<S>, lonlat2: LonLatT<S>) -> Angle<S> {
    let abs_diff_lon = (lonlat1.lon() - lonlat2.lon()).abs();
    (lonlat1.lat().sin() * lonlat2.lat().sin()
        + lonlat1.lat().cos() * lonlat2.lat().cos() * abs_diff_lon.cos())
    .acos().to_angle()
}

#[inline]
pub fn xyz_to_radec<S: BaseFloat>(v: &Vector3<S>) -> (Angle<S>, Angle<S>) {
    let lon = (v.x.atan2(v.z)).to_angle();
    let lat = (v.y.atan2((v.x * v.x + v.z * v.z).sqrt())).to_angle();

    (lon, lat)
}

#[inline]
pub fn radec_to_xyz<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Vector3<S> {
    let (ds, dc) = delta.to_radians().sin_cos();
    let (ts, tc) = theta.to_radians().sin_cos();

    Vector3::<S>::new(dc * ts, ds, dc * tc)
}

#[inline]
pub fn radec_to_basis<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Matrix3<S> {
    Matrix3::<S>::new(
        // e_r
        delta.cos() * theta.sin(),
        delta.sin(),
        delta.cos() * theta.cos(),
        // e_delta
        delta.sin() * theta.sin(),
        -delta.cos(),
        delta.sin() * theta.cos(),
        // e_theta
        theta.cos(),
        S::zero(),
        -theta.sin(),
    )
}

use crate::CameraViewPort;
use crate::ProjectionType;

use super::projection::coo_space::XYScreen;
use super::projection::coo_space::XYNDC;
#[inline]
pub fn proj(
    lonlat: &LonLatT<f64>,
    projection: &ProjectionType,
    camera: &CameraViewPort,
) -> Option<XYNDC<f64>> {
    let xyz = lonlat.vector();
    projection.model_to_normalized_device_space(&xyz, camera)
}

#[inline]
pub fn unproj(
    ndc_xy: &XYNDC<f64>,
    projection: &ProjectionType,
    camera: &CameraViewPort,
) -> Option<LonLatT<f64>> {
    projection
        .normalized_device_to_model_space(&ndc_xy, camera)
        .map(|model_pos| model_pos.lonlat())
}

#[inline]
pub fn proj_to_screen(
    lonlat: &LonLatT<f64>,
    projection: &ProjectionType,
    camera: &CameraViewPort,
) -> Option<XYScreen<f64>> {
    let xyz = lonlat.vector();
    projection.model_to_screen_space(&xyz, camera)
}

#[inline]
pub fn unproj_from_screen(
    xy: &XYScreen<f64>,
    projection: &ProjectionType,
    camera: &CameraViewPort,
) -> Option<LonLatT<f64>> {
    projection
        .screen_to_model_space(&xy, camera)
        .map(|model_pos| model_pos.lonlat())
}
