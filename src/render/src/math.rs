use cgmath::{InnerSpace, BaseFloat};
use cgmath::Rad;
use cgmath::{Vector4, Vector3};
#[inline]
pub fn ang_between_vect<S: BaseFloat>(x: &Vector3<S>, y: &cgmath::Vector3<S>) -> Angle<S> {
    let rad = Rad(x.cross(*y).magnitude().atan2(x.dot(*y)));
    Angle::new(rad)
}

#[inline]
pub fn _ang_between_lonlat<S: BaseFloat>(lon1: Angle<S>, lat1: Angle<S>, lon2: Angle<S>, lat2: Angle<S>) -> Angle<S> {
    let abs_diff_lon = (lon1 - lon2).abs();
    (lat1.sin()*lat2.sin() + lat1.cos()*lat2.cos()*abs_diff_lon.cos()).acos()
}
/*
#[inline]
pub fn course_between_lonlat<S: BaseFloat>(lon1: Angle<S>, lat1: Angle<S>, lon2: Angle<S>, lat2: Angle<S>) -> Angle<S> {
    // Check if the starting vertex is on a pole
    let eps = S::from(1e-5).unwrap();
    let tc1 = if lat1.cos() < eps {
        if lat1 > S::zero() {
            // starting vertex is located on the N pole
            S::from(std::f32::consts::PI).unwrap()
        } else {
            // starting vertex is located on the S pole
            S::from(2_f32 * std::f32::consts::PI).unwrap()
        }
    } else {
        // d stores the great circle distance between the two vertices
        let d = ang_between_lonlat(lon1, lat1, lon2, lat2);

        let a = ((lat2.sin()-lat1.sin()*d.cos())/(d.sin()*lat1.cos())).acos();
        if (lon2 - lon1).sin() < S::zero() {
            a
        } else {
            S::from(2_f32 * std::f32::consts::PI).unwrap() - a 
        }
    };

    tc1*S::from(180_f32 / std::f32::consts::PI).unwrap()
}
*/
pub trait LonLat<S: BaseFloat> {
    fn lon(&self) -> Angle<S>;
    fn lat(&self) -> Angle<S>;
    fn lonlat(&self) -> LonLatT<S>;
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self;
}

use crate::renderable::angle::Angle;
#[derive(Clone, Copy)]
pub struct LonLatT<S: BaseFloat>(Angle<S>, Angle<S>);

impl<S> LonLatT<S>
where S: BaseFloat {
    pub fn new(lon: Angle<S>, lat: Angle<S>) -> LonLatT<S> {
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

    pub fn vector<VectorT: LonLat<S>>(&self) -> VectorT {
        VectorT::from_lonlat(&self)
    }
}

impl<S> LonLat<S> for Vector3<S>
where S: BaseFloat {
    #[inline]
    fn lon(&self) -> Angle<S> {
        let rad = Rad(self.x.atan2(self.z));
        Angle::new(rad)
    }

    #[inline]
    fn lat(&self) -> Angle<S> {
        let rad = Rad(self.y.atan2((self.x*self.x + self.z*self.z).sqrt()));
        Angle::new(rad)
    }

    #[inline]
    fn lonlat(&self) -> LonLatT<S> {
        let lon = Rad(self.x.atan2(self.z));
        let lat = Rad(self.y.atan2((self.x*self.x + self.z*self.z).sqrt()));

        LonLatT(Angle::new(lon), Angle::new(lat))
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        let theta = lonlat.lon();
        let delta = lonlat.lat();

        Vector3::<S>::new(
            (delta.cos() * theta.sin()).0,
            delta.sin().0,
            (delta.cos() * theta.cos()).0,
        )
    }
}

impl<S> LonLat<S> for Vector4<S>
where S: BaseFloat {
    #[inline]
    fn lon(&self) -> Angle<S> {
        let rad = Rad(self.x.atan2(self.z));
        Angle::new(rad)
    }
    #[inline]
    fn lat(&self) -> Angle<S> {
        let rad = Rad(self.y.atan2(
            (self.x*self.x + self.z*self.z).sqrt()
        ));
        Angle::new(rad)
    }
    #[inline]
    fn lonlat(&self) -> LonLatT<S> {
        let lon = Rad(self.x.atan2(self.z));
        let lat = Rad(self.y.atan2(
            (self.x*self.x + self.z*self.z).sqrt()
        ));

        LonLatT(Angle::new(lon), Angle::new(lat))
    }

    #[inline]
    fn from_lonlat(lonlat: &LonLatT<S>) -> Self {
        let theta = lonlat.lon();
        let delta = lonlat.lat();
        Vector4::<S>::new(
            (delta.cos() * theta.sin()).0,
            delta.sin().0,
            (delta.cos() * theta.cos()).0,
            S::one()
        )
    }
}


#[inline]
pub fn xyz_to_radec<S: BaseFloat>(v: &cgmath::Vector3<S>) -> (Angle<S>, Angle<S>) {
    let lon = Angle(v.x.atan2(v.z));
    let lat = Angle(v.y.atan2((v.x*v.x + v.z*v.z).sqrt()));

    (lon, lat)
}

#[inline]
pub fn xyzw_to_radec<S: BaseFloat>(v: &cgmath::Vector4<S>) -> (Angle<S>, Angle<S>) {
    let lon = Angle(v.x.atan2(v.z));
    let lat = Angle(v.y.atan2((v.x*v.x + v.z*v.z).sqrt()));

    (lon, lat)
}

#[inline]
pub fn radec_to_xyzw<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Vector4<S> {
    Vector4::<S>::new(
        (delta.cos() * theta.sin()).0,
        delta.sin().0,
        (delta.cos() * theta.cos()).0,
        S::one()
    )
}

#[inline]
pub fn radec_to_xyz<S: BaseFloat>(theta: Angle<S>, delta: Angle<S>) -> Vector3<S> {
    Vector3::<S>::new(
        (delta.cos() * theta.sin()).0,
        delta.sin().0,
        (delta.cos() * theta.cos()).0,
    )
}

/*pub fn ang_per_pixel_to_depth(x: f32) -> u8 {
    let depth_pixel = (((4_f32 * std::f32::consts::PI) / (12_f32 * x * x)).log2() / 2_f32).floor() as i32;

    let mut depth = depth_pixel - 9;
    if depth < 0 {
        depth = 0;
    }
    depth as u8
}*/

// Used for selecting the current depth for a given FOV
// We need to select a depth so that we do not see any pixels
// This takes into account the screen resolution and can impact
// the number of healpix cells to load. Bigger resolution will need
// more cells which can overfit the buffer!
use crate::buffer::HiPSConfig;
pub fn fov_to_depth(fov: Angle<f32>, width: f32, config: &HiPSConfig) -> u8 {
    let pixel_ang = fov / width;

    let depth_pixel = ((((4_f32 * std::f32::consts::PI) / (12_f32 * pixel_ang.0 * pixel_ang.0)).log2() / 2_f32) + 0.5_f32).floor() as i8;

    // The texture size in pixels
    let texture_size = config.get_texture_size();
    // The depth of the texture
    // A texture of 512x512 pixels will have a depth of 9
    let depth_offset_texture = log_2(texture_size);
    // The depth of the texture corresponds to the depth of a pixel
    // minus the offset depth of the texture
    let mut depth_texture = depth_pixel - depth_offset_texture;
    if depth_texture < 0 {
        depth_texture = 0;
    }
    depth_texture as u8
}
pub fn fov_to_depth_precise(fov: Angle<f32>, width: f32, config: &HiPSConfig) -> f32 {
    let pixel_ang = fov / width;

    let depth_pixel = (((4_f32 * std::f32::consts::PI) / (12_f32 * pixel_ang.0 * pixel_ang.0)).log2() / 2_f32);

    // The texture size in pixels
    let texture_size = config.get_texture_size();
    // The depth of the texture
    // A texture of 512x512 pixels will have a depth of 9
    let depth_offset_texture = log_2(texture_size) as f32;
    // The depth of the texture corresponds to the depth of a pixel
    // minus the offset depth of the texture
    let mut depth_texture = depth_pixel - depth_offset_texture;
    if depth_texture < 0.0 {
        depth_texture = 0.0;
    }
    depth_texture
}
/*
pub fn depth_to_fov(depth: u8) -> Rad<f32> {
    let sphere_area = 4_f32 * std::f32::consts::PI;
    let num_hpx_cells = 12_f32 * 4_f32.powf(depth as f32);
    let hpx_cell_ang = Rad((sphere_area / num_hpx_cells).sqrt());

    hpx_cell_ang
}
*/
/*use cgmath::Vector2;
pub fn is_inside_ellipse(screen_pos: &Vector2<f32>, a: f32, b: f32) -> bool {
    let a2 = a * a;
    let b2 = b * b;
    let px2 = screen_pos.x * screen_pos.x;
    let py2 = screen_pos.y * screen_pos.y;

    return (px2 * b2 + py2 * a2) <= a2 * b2;
}
*/
#[inline]
const fn num_bits<T>() -> usize { std::mem::size_of::<T>() * 8 }

#[inline]
pub fn log_2(x: i32) -> i8 {
    assert!(x > 0);
    (num_bits::<i32>() as u32 - x.leading_zeros() - 1) as i8
}
