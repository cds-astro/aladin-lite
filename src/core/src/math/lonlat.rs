use cgmath::{BaseFloat, Rad, Vector3, Vector4, Matrix3};

pub trait LonLat<S: BaseFloat> {
    fn lon(&self) -> Angle<S>;
    fn lat(&self) -> Angle<S>;
    fn lonlat(&self) -> LonLatT<S>;
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self;
}

use crate::math::angle::Angle;
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
pub fn ang_between_lonlat<S: BaseFloat>(lonlat1: LonLatT<S>, lonlat2: LonLatT<S>) -> Angle<S> {
    let abs_diff_lon = (lonlat1.lon() - lonlat2.lon()).abs();
    Angle(
        (lonlat1.lat().sin() * lonlat2.lat().sin()
            + lonlat1.lat().cos() * lonlat2.lat().cos() * abs_diff_lon.cos())
        .acos(),
    )
}

#[inline]
pub fn xyz_to_radec<S: BaseFloat>(v: &Vector3<S>) -> (Angle<S>, Angle<S>) {
    let lon = Angle(v.x.atan2(v.z));
    let lat = Angle(v.y.atan2((v.x * v.x + v.z * v.z).sqrt()));

    (lon, lat)
}

#[inline]
pub fn xyzw_to_radec<S: BaseFloat>(v: &Vector4<S>) -> (Angle<S>, Angle<S>) {
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
        -theta.sin()
    )
}
