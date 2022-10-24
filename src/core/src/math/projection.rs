// Screen space: pixels space between
// * x_px in [0, width-1]
// * y_px in [0, height-1]

// Homogeneous space
// * x_h in [-1, 1]
// * y_h in [-1, 1]

// World space
use crate::camera::CameraViewPort;
//use crate::num_traits::FloatConst;
use crate::math::PI;

pub fn screen_to_ndc_space(
    pos_screen_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    // Screen space in pixels to homogeneous screen space (values between [-1, 1])
    let window_size = camera.get_screen_size();
    let window_size = Vector2::new(window_size.x as f64, window_size.y as f64);
    // Change of origin
    let dpi = camera.get_dpi() as f64;
    let origin = pos_screen_space * dpi - window_size / 2.0;

    // Scale to fit in [-1, 1]

    Vector2::new(
        2.0 * (origin.x / window_size.x),
        -2.0 * (origin.y / window_size.y),
    )
}

pub fn ndc_to_screen_space(
    pos_normalized_device: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let window_size = camera.get_screen_size();
    let dpi = camera.get_dpi() as f64;

    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5 + 0.5) * (window_size.x as f64),
        (0.5 - pos_normalized_device.y * 0.5) * (window_size.y as f64),
    );

    pos_screen_space / dpi
}

pub fn clip_to_ndc_space(pos_clip_space: &Vector2<f64>, camera: &CameraViewPort) -> Vector2<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    Vector2::new(
        pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
        pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
    )
}

pub fn clip_to_screen_space(
    pos_clip_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let pos_normalized_device = clip_to_ndc_space(pos_clip_space, camera);
    ndc_to_screen_space(&pos_normalized_device, camera)
}

pub fn screen_to_clip_space(
    pos_screen_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);
    ndc_to_clip_space(&pos_normalized_device, camera)
}

pub fn ndc_to_clip_space(
    pos_normalized_device: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    Vector2::new(
        pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
        pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
    )
}

use al_api::coo_system::CooSystem;
use cgmath::InnerSpace;

#[derive(Clone, Copy)]
#[enum_dispatch]
pub enum ProjectionType {
    Orthographic,
    Gnomonic,
    Aitoff,
    Mercator,
    HEALPix,
    Mollweide,
    AzimuthalEquidistant
}

use cgmath::Vector4;

#[enum_dispatch(ProjectionType)]
pub trait Projection
{
    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    fn screen_to_world_space(
        &self,
        pos_screen_space: &Vector2<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector4<f64>> {
        // Change the screen position according to the dpi
        //let dpi = camera.get_dpi();
        let pos_screen_space = *pos_screen_space;
        let pos_normalized_device = screen_to_ndc_space(&pos_screen_space, camera);

        let ndc_to_clip = camera.get_ndc_to_clip();
        let clip_zoom_factor = camera.get_clip_zoom_factor();

        let pos_clip_space = Vector2::new(
            pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
            pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
        );
        self.clip_to_world_space(&pos_clip_space)
            .map(|mut pos_world_space| {
                if camera.get_longitude_reversed() {
                    pos_world_space.x = -pos_world_space.x;
                }

                pos_world_space.normalize()
            })
    }

    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    fn screen_to_model_space(
        &self,
        pos_screen_space: &Vector2<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector4<f64>> {
        let pos_world_space = self.screen_to_world_space(pos_screen_space, camera);

        if let Some(pos_world_space) = pos_world_space {
            let r = camera.get_final_rotation();
            let pos_model_space = r.rotate(&pos_world_space);
            Some(pos_model_space)
        } else {
            None
        }
    }

    fn model_to_screen_space(
        &self,
        pos_model_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        self.world_to_screen_space(&pos_world_space, camera)
    }

    fn view_to_screen_space(
        &self,
        pos_model_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        self.view_to_normalized_device_space(pos_model_space, camera)
            .map(|ndc_pos| {
                crate::ndc_to_screen_space(&ndc_pos, camera)
            })
    }

    fn view_to_normalized_device_space(
        &self,
        pos_view_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        let view_coosys = camera.get_system();
        let c = CooSystem::ICRSJ2000.to::<f64>(view_coosys);

        let m2w = camera.get_m2w();
        let pos_world_space = m2w * c * pos_view_space;
        self.world_to_normalized_device_space(&pos_world_space, camera)
    }

    fn view_to_normalized_device_space_unchecked(
        &self,
        pos_view_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Vector2<f64> {
        let view_coosys = camera.get_system();
        let c = CooSystem::ICRSJ2000.to::<f64>(view_coosys);

        let m2w = camera.get_m2w();
        let pos_world_space = m2w * c * pos_view_space;
        self.world_to_normalized_device_space_unchecked(&pos_world_space, camera)
    }

    fn model_to_normalized_device_space(
        &self,
        pos_model_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        //pos_world_space.x = -pos_world_space.x;
        self.world_to_normalized_device_space(&pos_world_space, camera)
    }

    /// World to screen space projection

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `x` - X mouse position in homogenous screen space (between [-1, 1])
    /// * `y` - Y mouse position in homogenous screen space (between [-1, 1])
    fn world_to_normalized_device_space(
        &self,
        pos_world_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        self.world_to_clip_space(pos_world_space)
            .map(|mut pos_clip_space| {
                if camera.get_longitude_reversed() {
                    pos_clip_space.x = -pos_clip_space.x;
                }
                let ndc_to_clip = camera.get_ndc_to_clip();
                let clip_zoom_factor = camera.get_clip_zoom_factor();

                Vector2::new(
                    pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
                    pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
                )
            })
    }

    fn world_to_normalized_device_space_unchecked(
        &self,
        pos_world_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Vector2<f64> {
        let mut pos_clip_space = self.world_to_clip_space_unchecked(pos_world_space);
        if camera.get_longitude_reversed() {
            pos_clip_space.x = -pos_clip_space.x;
        }
        let ndc_to_clip = camera.get_ndc_to_clip();
        let clip_zoom_factor = camera.get_clip_zoom_factor();

        Vector2::new(
            pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
            pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
        )
    }

    fn world_to_screen_space(
        &self,
        pos_world_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        self.world_to_normalized_device_space(pos_world_space, camera)
            .map(|pos_normalized_device| ndc_to_screen_space(&pos_normalized_device, camera))
    }

    /// Perform a clip to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_clip_space`` - The position in the clipping space (orthonorlized space)
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<Vector4<f64>>;
    /// World to the clipping space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_world_space`` - The position in the world space
    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>>;
    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64>;

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool;

    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64>;

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)>;
    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)>;
    fn clip_size(&self) -> (f64, f64);

    //const ALLOW_UNZOOM_MORE: bool;
    // Aperture angle at the start of the application (full view)
    // - 180 degrees for the 3D projections (i.e. ortho)
    // - 360 degrees for the 2D projections (i.e. mollweide, arc, aitoff...)
    //const APERTURE_START: f64;
}

impl ProjectionType {
    pub fn aperture_start(&self) -> f64 {
        match self {
            ProjectionType::Orthographic(_) | ProjectionType::Gnomonic(_) => 180.0,
            _ => 360.0
        }
    }
}

#[derive(Clone, Copy)]
pub struct Aitoff;
#[derive(Clone, Copy)]
pub struct Mollweide;
#[derive(Clone, Copy)]
pub struct Orthographic;
#[derive(Clone, Copy)]
pub struct AzimuthalEquidistant;
#[derive(Clone, Copy)]
pub struct Gnomonic;
#[derive(Clone, Copy)]
pub struct Mercator;
#[derive(Clone, Copy)]
pub struct HEALPix;

use cgmath::Vector2;

impl Projection for Aitoff {
    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1.0, height / width)
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        // Semi-major axis length
        let a = 1.0;
        // Semi-minor axis length
        let b = 0.5;

        let a2 = a * a;
        let b2 = b * b;
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 * b2 + py2 * a2) < a2 * b2
    }

    fn clip_size(&self) -> (f64, f64) {
        (2.0, 1.0)
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 0.5 {
            None
        } else {
            let x = (1.0 - 4.0 * y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }

    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1.0 {
            None
        } else {
            let y = (1.0 - x * x).sqrt() * 0.5;
            Some((-y + 1e-3, y - 1e-3))
        }
    }

    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        let x2d = -pos_clip_space.x * TWO_SQRT_TWO;
        let y2d = pos_clip_space.y * TWO_SQRT_TWO;
        let mut r = 0.125 * x2d * x2d + 0.5 * y2d * y2d; //  = 1 - cos(b) cos(l/2)
        if r > 1.0  {
          if r < 1.0 + 1e-15 { // Accept approximations in the projection
            r = 1.0;
          } else {
            return None;
          }
        }

        let mut x = 1.0 - r; // cos(b) cos(l/2)
        let mut w = (1.0 - 0.5 * r).sqrt(); // sqrt(HALF * (1 + x)) ;  //  = Z = sqrt[ (1 + cos(b) cos(l/2)) / 2]
        let mut y = 0.5 * x2d * w; // cos(b) sin(l/2)
        let z = y2d * w ; // z
        // Convert from Cartesian (l/2, b) to Cartesian (l, b) 
        r = (x * x + y * y).sqrt();  // cos(b)
        if r > 0.0 {
            w = x;
            x = (w * w - y * y) / r; // cos(b) cos(l)
            y = 2.0 * w * y / r; // cos(b) sin(l)
        }

        Some(Vector4::new(y, z, x, 1.0))
    }

    /// World to screen space transformation
    /// X is between [-1, 1]
    /// Y is between [-0.5, 0.5]
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        Some(self.world_to_clip_space_unchecked(pos_world_space))
    }

    /// World to screen space transformation
    /// X is between [-1, 1]
    /// Y is between [-0.5, 0.5]
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    /*fn world_to_clip_space_unchecked(pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        //return vec3(X / PI, aspect * Y / PI, 0.f);

        //let pos_world_space = pos_world_space;

        let xyz = pos_world_space.truncate();
        let (theta, delta) = math::lonlat::xyz_to_radec(&xyz);

        let theta_by_two = -theta / 2.0;

        let alpha = (delta.0.cos() * theta_by_two.0.cos()).acos();
        let inv_sinc_alpha = if alpha < 1e-3 {
            1.0
        } else {
            alpha / alpha.sin()
        };

        // The minus is an astronomical convention.
        // longitudes are increasing from right to left
        let x = 2.0 * inv_sinc_alpha * delta.0.cos() * theta_by_two.0.sin();
        let y = inv_sinc_alpha * delta.0.sin();

        Vector2::new(
            x / std::f64::consts::PI,
            y / std::f64::consts::PI,
        )
    }*/

    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        let x = pos_world_space.z;
        let y = pos_world_space.x;
        let z = pos_world_space.y;

        let r = (x * x + y * y).sqrt();
        let mut w = (0.5 * r * (r + x)).sqrt(); // = cos(b) cos(l/2)
        w = (0.5 * (1.0 + w)).sqrt();            // = 1 / gamma
        let y2d = z / w;
        w = (2.0 * r * (r - x)).sqrt() / w;       // = 2 * gamma * cos(b) sin(l/2)
        let x2d = if y < 0.0 { -w } else { w };

        Vector2::new(-x2d / TWO_SQRT_TWO, y2d / TWO_SQRT_TWO)
    }
    //const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((170.0 / 180.0) * std::f64::consts::PI);
}

use crate::math;
impl Projection for Mollweide {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 1.0)
    }

    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, height / width)
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        // Semi-major axis length
        let a = 1_f64;
        // Semi-minor axis length
        let b = 0.5_f64;

        let a2 = a * a;
        let b2 = b * b;
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 * b2 + py2 * a2) < a2 * b2
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 0.5_f64 {
            None
        } else {
            let x = (1.0 - 4.0 * y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1_f64 {
            None
        } else {
            let y = (1.0 - x * x).sqrt() * 0.5_f64;
            Some((-y + 1e-3, y - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        if self.is_included_inside_projection(pos_clip_space) {
            let y2 = pos_clip_space.y * pos_clip_space.y;
            let k = (1.0 - 4.0 * y2).sqrt();

            let theta = -PI * pos_clip_space.x / k;
            let delta =
                ((2.0 * (2.0 * pos_clip_space.y).asin() + 4.0 * pos_clip_space.y * k) / PI).asin();

            // The minus is an astronomical convention.
            // longitudes are increasing from right to left
            let pos_world_space = math::lonlat::radec_to_xyzw(Angle(theta), Angle(delta));

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
    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        let epsilon = 1e-12;
        let max_iter = 10;

        let xyz = pos_world_space.truncate();
        let (lon, lat) = math::lonlat::xyz_to_radec(&xyz);

        let cst = std::f64::consts::PI * lat.sin();

        let mut theta = lat.0;
        let mut f = theta + theta.sin() - cst;

        let mut k = 0;
        while f.abs() > epsilon && k < max_iter {
            theta -= f / (1.0 + theta.cos());
            f = theta + theta.sin() - cst;

            k += 1;
        }

        theta /= 2.0;

        // The minus is an astronomical convention.
        // longitudes are increasing from right to left
        let x = (-lon.0 / std::f64::consts::PI) * theta.cos();
        let y = 0.5 * theta.sin();

        Vector2::new(x, y)
    }

    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        Some(self.world_to_clip_space_unchecked(pos_world_space))
    }
}

use crate::math::angle::Angle;

use super::TWO_SQRT_TWO;
impl Projection for Orthographic {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 2.0)
    }

    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64> {
        //Vector2::new(1_f64, height / width)
        //Vector2::new(width / height, 1.0)
        if width > height {
            Vector2::new(1_f64, height / width)
        } else {
            Vector2::new(width / height, 1.0)
        }
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) < 1_f64
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0_f64 {
            None
        } else {
            let x = (1.0 - y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1_f64 {
            None
        } else {
            let y = (1.0 - x * x).sqrt();
            Some((-y + 1e-3, y - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        let xw_2 = 1.0 - pos_clip_space.x * pos_clip_space.x - pos_clip_space.y * pos_clip_space.y;
        if xw_2 > 0.0 {
            let pos_world_space =
                Vector4::new(-pos_clip_space.x, pos_clip_space.y, xw_2.sqrt(), 1_f64);
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
    fn world_to_clip_space(&self, pos_world_space: &cgmath::Vector4<f64>) -> Option<Vector2<f64>> {
        if pos_world_space.z < 0.0_f64 {
            None
        } else {
            Some(self.world_to_clip_space_unchecked(pos_world_space))
        }
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space_unchecked(&self, pos_world_space: &cgmath::Vector4<f64>) -> Vector2<f64> {
        Vector2::new(-pos_world_space.x, pos_world_space.y)
    }
}

impl Projection for AzimuthalEquidistant {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 2.0)
    }

    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64> {
        //Vector2::new(width / height, 1.0)
        if width > height {
            Vector2::new(1_f64, height / width)
        } else {
            Vector2::new(width / height, 1.0)
        }
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) < 1.0
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            let x = (1.0 - y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1.0 {
            None
        } else {
            let y = (1.0 - x * x).sqrt();
            Some((-y + 1e-3, y - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        // r <= pi
        let x = pos_clip_space.x * PI;
        let y = pos_clip_space.y * PI;
        let r = (x * x + y * y).sqrt();
        if r > PI {
            None
        } else {
            let z = r.cos();
            let r = math::utils::sinc_positive(r);

            let pos_world_space = Vector4::new(-x * r, y * r, z, 1.0);
            Some(pos_world_space)
        }
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        //if pos_world_space.z > -1.0 {
            // Distance in the Euclidean plane (xy)
            // Angular distance is acos(x), but for small separation, asin(r)
            // is more accurate.
            let mut r = (pos_world_space.x * pos_world_space.x
                + pos_world_space.y * pos_world_space.y)
                .sqrt();
            if pos_world_space.z > 0.0 {
                // Angular distance < PI/2, angular distance = asin(r)
                r = math::utils::asinc_positive(r);
            } else {
                // Angular distance > PI/2, angular distance = acos(x)
                r = pos_world_space.z.acos() / r;
            }

            Vector2::new(
                -pos_world_space.x * r / std::f64::consts::PI,
                pos_world_space.y * r / std::f64::consts::PI,
            )
        //} else {
        //    Some(Vector2::new(1.0, 0.0))
        //}
    }

    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        Some(self.world_to_clip_space_unchecked(pos_world_space))
    }
}

impl Projection for Gnomonic {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 2.0)
    }

    fn compute_ndc_to_clip_factor(&self, width: f64, height: f64) -> Vector2<f64> {
        if width > height {
            Vector2::new(1_f64, height / width)
        } else {
            Vector2::new(width / height, 1.0)
        }
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        let px = pos_clip_space.x;
        let py = pos_clip_space.y;

        px > -1.0 && px < 1.0 && py > -1.0 && py < 1.0
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
    }

    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1_f64 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        let x_2d = pos_clip_space.x;
        let y_2d = pos_clip_space.y;
        let r = x_2d * x_2d + y_2d * y_2d;

        let z = 1.0 / (1.0 + r).sqrt();
        let pos_world_space = Vector4::new(-z * x_2d, z * y_2d, z, 1.0);

        Some(pos_world_space)
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        if pos_world_space.z <= 1e-2 {
            // Back hemisphere (z < 0) + diverges near z=0
            None
        } else {
            Some(self.world_to_clip_space_unchecked(pos_world_space))
        }
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        let z = pos_world_space.z.abs();
        Vector2::new(
            -pos_world_space.x / z,
            pos_world_space.y / z,
        )
    }
}

impl Projection for Mercator {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 2.0)
    }

    fn compute_ndc_to_clip_factor(&self, _width: f64, _height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, 0.5f64)
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        let px = pos_clip_space.x;
        let py = pos_clip_space.y;

        px > -1.0 && px < 1.0 && py > -1.0 && py < 1.0
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
    }
    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        let theta = -pos_clip_space.x * PI;
        let delta = (pos_clip_space.y.sinh()).atan() * PI;

        let pos_world_space = math::lonlat::radec_to_xyzw(Angle(theta), Angle(delta));
        Some(pos_world_space)
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        Some(self.world_to_clip_space_unchecked(pos_world_space))
    }

    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        let (theta, delta) = math::lonlat::xyzw_to_radec(pos_world_space);

        Vector2::new(
            -theta.0 / std::f64::consts::PI,
            ((delta.0 / std::f64::consts::PI).tan()).asinh() as f64,
        )
    }
}

impl Projection for HEALPix {
    fn clip_size(&self) -> (f64, f64) {
        (2.0, 2.0)
    }

    fn compute_ndc_to_clip_factor(&self, _width: f64, _height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, 1_f64)
    }

    fn is_included_inside_projection(&self, pos_clip_space: &Vector2<f64>) -> bool {
        let px = pos_clip_space.x * 4.0; // [-4; 4]
        let py = pos_clip_space.y * 2.0; // [-2; 2]

        if py.abs() < 1.0 {
            return true;
        }

        let px = px.rem_euclid(2.0); // [0; 2]
        if px < 1.0 {
            py.abs() <= px + 1.0 + 1e-2
        } else {
            py.abs() <= 3.0 - px
        }
    }

    fn solve_along_abscissa(&self, y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
    }

    fn solve_along_ordinate(&self, x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
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
    fn clip_to_world_space(&self, pos_clip_space: &Vector2<f64>) -> Option<cgmath::Vector4<f64>> {
        if self.is_included_inside_projection(pos_clip_space) {
            let x = -pos_clip_space.x * 4.0;
            let y = pos_clip_space.y * 2.0;

            let (lon, lat) = cdshealpix::unproj(x, y);
            Some(math::lonlat::radec_to_xyzw(Angle(lon), Angle(lat)))
        } else {
            None
        }
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(&self, pos_world_space: &Vector4<f64>) -> Option<Vector2<f64>> {
        Some(self.world_to_clip_space_unchecked(pos_world_space))
    }

    fn world_to_clip_space_unchecked(&self, pos_world_space: &Vector4<f64>) -> Vector2<f64> {
        let (lon, lat) = math::lonlat::xyzw_to_radec(pos_world_space);

        let (x, y) = cdshealpix::proj(lon.0, lat.0);
        let (x, y) = (-x * 0.25, y * 0.5);

        //assert_debug!(x >= -1.0 && x <= 1.0);
        Vector2::new(x, y)
    }
}

mod tests {
    use crate::Abort;
    #[test]
    fn generate_maps() {
        use super::*;
        use cgmath::Vector2;
        use image_decoder::{Rgb, RgbImage};
        fn generate_projection_map(filename: &str, projection: ProjectionType) {
            let (w, h) = (1024.0, 1024.0);
            let mut img = RgbImage::new(w as u32, h as u32);
            for x in 0..(w as u32) {
                for y in 0..(h as u32) {
                    let xy = Vector2::new(x, y);
                    let clip_xy = Vector2::new(
                        2.0 * ((xy.x as f64) / (w as f64)) - 1.0,
                        2.0 * ((xy.y as f64) / (h as f64)) - 1.0,
                    );
                    let rgb = if let Some(pos) = projection.clip_to_world_space(&clip_xy) {
                        let pos = pos.truncate().normalize();
                        Rgb([
                            ((pos.x * 0.5 + 0.5) * 256.0) as u8,
                            ((pos.y * 0.5 + 0.5) * 256.0) as u8,
                            ((pos.z * 0.5 + 0.5) * 256.0) as u8,
                        ])
                    } else {
                        Rgb([255, 255, 255])
                    };

                    img.put_pixel(x as u32, y as u32, rgb);
                }
            }
            img.save(filename).unwrap_abort();
        }

        generate_projection_map("./../img/aitoff.png", ProjectionType::Aitoff(Aitoff));
        generate_projection_map("./../img/tan.png", ProjectionType::Gnomonic(Gnomonic));
        generate_projection_map("./../img/arc.png", ProjectionType::AzimuthalEquidistant(AzimuthalEquidistant));
        generate_projection_map("./../img/mollweide.png", ProjectionType::Mollweide(Mollweide));
        generate_projection_map("./../img/mercator.png", ProjectionType::Mercator(Mercator));
        generate_projection_map("./../img/sinus.png", ProjectionType::Orthographic(Orthographic));
        generate_projection_map("./../img/healpix.png", ProjectionType::HEALPix(HEALPix));
    }
}
