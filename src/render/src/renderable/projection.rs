// Screen space: pixels space between
// * x_px in [0, width-1]
// * y_px in [0, height-1]

// Homogeneous space
// * x_h in [-1, 1]
// * y_h in [-1, 1]

// World space
use crate::camera::CameraViewPort;



pub fn screen_to_ndc_space(pos_screen_space: &Vector2<f32>, camera: &CameraViewPort) -> Vector2<f32> {
    // Screen space in pixels to homogeneous screen space (values between [-1, 1])
    let window_size = camera.get_screen_size();
    // Change of origin
    let origin = pos_screen_space - window_size/2_f32;

    // Scale to fit in [-1, 1]
    let pos_normalized_device = Vector2::new(2_f32 * (origin.x/window_size.x), -2_f32 * (origin.y/window_size.y));
    pos_normalized_device
}
pub fn ndc_to_screen_space(pos_normalized_device: &Vector2<f32>, camera: &CameraViewPort) -> Vector2<f32> {
    let window_size = camera.get_screen_size();

    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5_f32 + 0.5_f32) * window_size.x,
        (0.5_f32 - pos_normalized_device.y * 0.5_f32) * window_size.y,
    );

    pos_screen_space
}
pub fn clip_to_screen_space(pos_clip_space: &Vector2<f32>, camera: &CameraViewPort) -> Vector2<f32> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();
    
    let pos_normalized_device = Vector2::new(
        pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
        pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
    );

    let window_size = camera.get_screen_size();
    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5_f32 + 0.5_f32) * window_size.x,
        (0.5_f32 - pos_normalized_device.y * 0.5_f32) * window_size.y,
    );

    pos_screen_space
}

pub fn screen_to_clip_space(pos_screen_space: &Vector2<f32>, camera: &CameraViewPort) -> Vector2<f32> {
    let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);

    ndc_to_clip_space(&pos_normalized_device, camera)
}

pub fn ndc_to_clip_space(pos_normalized_device: &Vector2<f32>, camera: &CameraViewPort) -> Vector2<f32> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    let pos_clip_space = Vector2::new(
        pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
        pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
    );

    pos_clip_space
}

use cgmath::Vector4;
use cgmath::InnerSpace;

use crate::renderable::{
 catalog::CatalogShaderProjection,
 grid::GridShaderProjection,
};
use crate::shader::GetShader;
pub trait Projection: GetShader + CatalogShaderProjection + GridShaderProjection + std::marker::Sized {
    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    /// 
    /// # Arguments
    /// 
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    fn screen_to_world_space(pos_screen_space: &Vector2<f32>, camera: &CameraViewPort) -> Option<Vector4<f32>> {
        let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);

        let ndc_to_clip = camera.get_ndc_to_clip();
        let clip_zoom_factor = camera.get_clip_zoom_factor();

        let pos_clip_space = Vector2::new(
            pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
            pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
        );
        let pos_world_space = Self::clip_to_world_space(&pos_clip_space);
        if let Some(pos_world_space) = pos_world_space {
            let pos_world_space = pos_world_space.normalize();

            Some(pos_world_space)
        } else {
            None
        }
    }

    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    /// 
    /// # Arguments
    /// 
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    fn screen_to_model_space(pos_screen_space: &Vector2<f32>, camera: &CameraViewPort) -> Option<Vector4<f32>> {
        let pos_world_space = Self::screen_to_world_space(pos_screen_space, camera);

        if let Some(pos_world_space) = pos_world_space {
            let r = camera.get_rotation();
            let pos_model_space = r.rotate(&pos_world_space);

            Some(pos_model_space)
        } else {
            None
        }
    }

    /// Perform a clip to the world space deprojection
    /// 
    /// # Arguments
    /// 
    /// * ``pos_clip_space`` - The position in the clipping space (orthonorlized space)
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<Vector4<f32>>;

    fn clip_to_model_space(pos_clip_space: &Vector2<f32>, camera: &CameraViewPort) -> Option<Vector4<f32>> {
        let pos_world_space = Self::clip_to_world_space(pos_clip_space);

        if let Some(pos_world_space) = pos_world_space {
            let r = camera.get_rotation();
            let pos_model_space = r.rotate(&pos_world_space);

            Some(pos_model_space)
        } else {
            None
        }
    }

    /// World to screen space projection

    /// World to screen space transformation
    /// 
    /// # Arguments
    /// 
    /// * `x` - X mouse position in homogenous screen space (between [-1, 1])
    /// * `y` - Y mouse position in homogenous screen space (between [-1, 1])
    fn world_to_normalized_device_space(pos_model_space: &Vector4<f32>, camera: &CameraViewPort) -> Vector2<f32> {
        let pos_clip_space = Self::world_to_clip_space(pos_model_space);

        let ndc_to_clip = camera.get_ndc_to_clip();
        let clip_zoom_factor = camera.get_clip_zoom_factor();

        let pos_normalized_device = Vector2::new(
            pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
            pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
        );
        pos_normalized_device
    }

    fn world_to_screen_space(pos_world_space: &Vector4<f32>, camera: &CameraViewPort) -> Vector2<f32> {
        let pos_normalized_device = Self::world_to_normalized_device_space(pos_world_space, camera);
        self::ndc_to_screen_space(&pos_normalized_device, camera)
    }
    /// World to the clipping space deprojection
    /// 
    /// # Arguments
    /// 
    /// * ``pos_world_space`` - The position in the world space
    fn world_to_clip_space(pos_world_space: &Vector4<f32>) -> Vector2<f32>;

    // Aperture angle at the start of the application (full view)
    // - 180 degrees for the 3D projections (i.e. ortho)
    // - 360 degrees for the 2D projections (i.e. mollweide, arc, aitoff...)
    fn aperture_start() -> Angle<f32>;

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool;

    fn is_front_of_camera(pos_world_space: &Vector4<f32>) -> bool;

    fn compute_ndc_to_clip_factor(width: f32, height: f32) -> Vector2<f32>;
}

pub struct Aitoff;
pub struct Mollweide;
pub struct Orthographic;
pub struct AzimutalEquidistant;
pub struct Mercator;

use cgmath::Vector2;

use crate::renderable::ArcDeg;

impl Projection for Aitoff {
    fn compute_ndc_to_clip_factor(width: f32, height: f32) -> Vector2<f32> {
        Vector2::new(1_f32, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool {
        // Semi-major axis length
        let a = 1_f32;
        // Semi-minor axis length
        let b = 0.5_f32;

        let a2 = a * a;
        let b2 = b * b;
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 * b2 + py2 * a2) <= a2 * b2
    }

    /// View to world space transformation
    /// 
    /// This returns a normalized vector along its first 3 dimensions.
    /// Its fourth component is set to 1.
    /// 
    /// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
    /// 
    /// # Arguments
    /// 
    /// * `x` - in normalized device coordinates between [-1; 1]
    /// * `y` - in normalized device coordinates between [-1; 1]
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<cgmath::Vector4<f32>> {
        if Self::is_included_inside_projection(&pos_clip_space) {
            let u = pos_clip_space.x * std::f32::consts::PI * 0.5_f32;
            let v = pos_clip_space.y * std::f32::consts::PI;
            //da uv a lat/lon
            let c = (v*v + u*u).sqrt();

            let (phi, mut theta) = if c != 0_f32 {
                let phi = (v * c.sin() / c).asin();
                let theta = (u * c.sin()).atan2(c * c.cos());
                (phi, theta)
            } else {
                let phi = v.asin();
                let theta = u.atan();
                (phi, theta)
            };
            theta *= 2_f32;

            let pos_world_space = cgmath::Vector4::new(
                -theta.sin() * phi.cos(),
                phi.sin(),
                theta.cos() * phi.cos(),
                1_f32
            );

            Some(pos_world_space)
        } else {
            None
        }
    }

    /// World to screen space transformation
    /// X is between [-1, 1]
    /// Y is between [-0.5, 0.5]
    /// 
    /// # Arguments
    /// 
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(pos_world_space: &Vector4<f32>) -> Vector2<f32> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        //return vec3(X / PI, aspect * Y / PI, 0.f);
        let xyz = pos_world_space.truncate();
        let (theta, delta) = math::xyz_to_radec(&xyz);

        let theta_by_two = theta / 2_f32;

        let alpha = (delta.0.cos() * theta_by_two.0.cos()).acos();
        let inv_sinc_alpha = if alpha < 1e-3 {
            1_f32
        } else {
            alpha / alpha.sin()
        };

        // The minus is an astronomical convention.
        // longitudes are increasing from right to left
        let x = -2_f32 * inv_sinc_alpha * delta.0.cos() * theta_by_two.0.sin();
        let y = inv_sinc_alpha * delta.0.sin();

        Vector2::new(x / std::f32::consts::PI, y / std::f32::consts::PI)
    }

    fn aperture_start() -> Angle<f32> {
        ArcDeg(360_f32).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f32>) -> bool {
        // 2D projections always faces the camera
        true
    }
}


use crate::math;
impl Projection for Mollweide {
    fn compute_ndc_to_clip_factor(width: f32, height: f32) -> Vector2<f32> {
        Vector2::new(1_f32, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool {
        // Semi-major axis length
        let a = 1_f32;
        // Semi-minor axis length
        let b = 0.5_f32;

        let a2 = a * a;
        let b2 = b * b;
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 * b2 + py2 * a2) <= a2 * b2
    }

    /// View to world space transformation
    /// 
    /// This returns a normalized vector along its first 3 dimensions.
    /// Its fourth component is set to 1.
    /// 
    /// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
    /// 
    /// # Arguments
    /// 
    /// * `x` - in normalized device coordinates between [-1; 1]
    /// * `y` - in normalized device coordinates between [-1; 1]
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<cgmath::Vector4<f32>> {
        if Self::is_included_inside_projection(&pos_clip_space) {
            let y2 = pos_clip_space.y * pos_clip_space.y;
            let k = (1_f32 - 4_f32 * y2).sqrt();

            let theta = std::f32::consts::PI * pos_clip_space.x / k;
            let delta = ((2_f32 * (2_f32 * pos_clip_space.y).asin() + 4_f32 * pos_clip_space.y * k) / std::f32::consts::PI).asin();

            // The minus is an astronomical convention.
            // longitudes are increasing from right to left
            let pos_world_space = cgmath::Vector4::new(
                -theta.sin() * delta.cos(),
                delta.sin(),
                theta.cos() * delta.cos(),
                1_f32
            );

            Some(pos_world_space)
        } else {
            None
        }
    }

    /// World to screen space transformation
    /// X is between [-1, 1]
    /// Y is between [-0.5, 0.5]
    /// 
    /// # Arguments
    /// 
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(pos_world_space: &Vector4<f32>) -> Vector2<f32> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        let epsilon = 1e-3;
        let max_iter = 10;

        let xyz = pos_world_space.truncate();
        let (lon, lat) = math::xyz_to_radec(&xyz);
 
        let cst = std::f32::consts::PI * lat.0.sin();

        let mut theta = lat.0;
        let mut f = theta + theta.sin() - cst;

        let mut k = 0;
        while f.abs() > epsilon && k < max_iter {
            theta -= f / (1_f32 + theta.cos());
            f = theta + theta.sin() - cst;

            k += 1;
        }

        theta /= 2_f32;

        // The minus is an astronomical convention.
        // longitudes are increasing from right to left
        let x = -(lon.0 / std::f32::consts::PI) * theta.cos();
        let y = 0.5_f32 * theta.sin();

        Vector2::new(x, y)
    }

    fn aperture_start() -> Angle<f32> {
        ArcDeg(360_f32).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f32>) -> bool {
        // 2D projections always faces the camera
        true
    }
}

use crate::renderable::Angle;
impl Projection for Orthographic {
    fn compute_ndc_to_clip_factor(width: f32, height: f32) -> Vector2<f32> {
        Vector2::new(1_f32, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) <= 1_f32
    }

    /// View to world space transformation
    /// 
    /// This returns a normalized vector along its first 3 dimensions.
    /// Its fourth component is set to 1.
    /// 
    /// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
    /// 
    /// # Arguments
    /// 
    /// * `x` - in normalized device coordinates between [-1; 1]
    /// * `y` - in normalized device coordinates between [-1; 1]
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<cgmath::Vector4<f32>> {
        let xw_2 = 1_f32 - pos_clip_space.x*pos_clip_space.x - pos_clip_space.y*pos_clip_space.y;
        if xw_2 > 0_f32 {
            let pos_world_space = cgmath::Vector4::new(-pos_clip_space.x, pos_clip_space.y, xw_2.sqrt(), 1_f32);

            Some(pos_world_space)
        } else {
            // Out of the sphere
            None
        }
    }

    /// World to screen space transformation
    /// 
    /// # Arguments
    /// 
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(pos_world_space: &cgmath::Vector4<f32>) -> Vector2<f32> {
        Vector2::new(-pos_world_space.x, pos_world_space.y)
    }

    fn aperture_start() -> Angle<f32> {
        ArcDeg(180_f32).into()
    }

    fn is_front_of_camera(pos_world_space: &Vector4<f32>) -> bool {
        pos_world_space.z > 0_f32
    }
}

impl Projection for AzimutalEquidistant {
    fn compute_ndc_to_clip_factor(width: f32, height: f32) -> Vector2<f32> {
        Vector2::new(1_f32, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) <= 1_f32
    }

    /// View to world space transformation
    /// 
    /// This returns a normalized vector along its first 3 dimensions.
    /// Its fourth component is set to 1.
    /// 
    /// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
    /// 
    /// # Arguments
    /// 
    /// * `x` - in normalized device coordinates between [-1; 1]
    /// * `y` - in normalized device coordinates between [-1; 1]
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<cgmath::Vector4<f32>> {
        let xw_2 = 1_f32 - pos_clip_space.x*pos_clip_space.x - pos_clip_space.y*pos_clip_space.y;
        if xw_2 > 0_f32 {
            let (x, y) = (2_f32 * pos_clip_space.x, 2_f32 * pos_clip_space.y);

            let rho2 = x*x + y*y;
            let rho = rho2.sqrt();

            let c = 2_f32 * (0.5_f32 * rho).asin();

            let mut delta = 0_f32;
            let mut theta = 0_f32;
            if c >= 1e-4 {
                delta = (y * c.sin() / rho).asin() * std::f32::consts::PI;
                theta = -(x * c.sin()).atan2(rho * c.cos()) * std::f32::consts::PI;
            }
            let pos_world_space = math::radec_to_xyzw(Angle(theta), Angle(delta));
            Some(pos_world_space)
        } else {
            // Out of the sphere
            None
        }
    }

    /// World to screen space transformation
    /// 
    /// # Arguments
    /// 
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(pos_world_space: &Vector4<f32>) -> Vector2<f32> {
        let (theta, delta) = math::xyzw_to_radec(&pos_world_space);
        let c = delta.cos() * theta.cos();

        let k = c / c.sin();

        let x = -k* delta.cos() * theta.sin();
        let y = k*delta.sin();

        Vector2::new(x.0 / std::f32::consts::PI, y.0 / std::f32::consts::PI)
    }

    fn aperture_start() -> Angle<f32> {
        ArcDeg(180_f32).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f32>) -> bool {
        // 2D projections always faces the camera
        true
    }
}

impl Projection for Mercator {
    fn compute_ndc_to_clip_factor(_width: f32, _height: f32) -> Vector2<f32> {
        Vector2::new(1_f32, 0.5_f32)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f32>) -> bool {
        let px = pos_clip_space.x;
        let py = pos_clip_space.y;

        px >= -1_f32 && px <= 1_f32 && py >= -1_f32 && py <= 1_f32
    }

    /// View to world space transformation
    /// 
    /// This returns a normalized vector along its first 3 dimensions.
    /// Its fourth component is set to 1.
    /// 
    /// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
    /// 
    /// # Arguments
    /// 
    /// * `x` - in normalized device coordinates between [-1; 1]
    /// * `y` - in normalized device coordinates between [-1; 1]
    fn clip_to_world_space(pos_clip_space: &Vector2<f32>) -> Option<cgmath::Vector4<f32>> {
        /*let xw_2 = 1_f32 - pos_clip_space.x*pos_clip_space.x - pos_clip_space.y*pos_clip_space.y;
        if xw_2 > 0_f32 {
            let (x, y) = (2_f32 * pos_clip_space.x, 2_f32 * pos_clip_space.y);

            let rho2 = (x*x + y*y);
            let rho = rho2.sqrt();

            let c = 2_f32 * (0.5_f32 * rho).asin();

            let mut delta = 0_f32;
            let mut theta = 0_f32;
            //if c >= 1e-4 {
            delta = (y * c.sin() / rho).asin();
            theta = -(x * c.sin()).atan2(rho * c.cos());
            //}
            let pos_world_space = math::radec_to_xyzw(Rad(theta), Rad(delta));
            Some(pos_world_space)
        } else {
            // Out of the sphere
            None
        }*/
        let theta = -pos_clip_space.x * std::f32::consts::PI;
        let delta = (pos_clip_space.y.sinh()).atan() * std::f32::consts::PI;

        let pos_world_space = math::radec_to_xyzw(Angle(theta), Angle(delta));
        Some(pos_world_space)
    }

    /// World to screen space transformation
    /// 
    /// # Arguments
    /// 
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(pos_world_space: &Vector4<f32>) -> Vector2<f32> {
        let (theta, delta) = math::xyzw_to_radec(&pos_world_space);

        Vector2::new(
            -theta.0 / std::f32::consts::PI,
            (((std::f32::consts::PI / 4_f32) + (delta.0 / 2_f32)).tan()).ln() / std::f32::consts::PI
        )
    }

    fn aperture_start() -> Angle<f32> {
        ArcDeg(360_f32).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f32>) -> bool {
        // 2D projections always faces the camera
        true
    }
}