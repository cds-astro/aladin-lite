// Screen space: pixels space between
// * x_px in [0, width-1]
// * y_px in [0, height-1]

// Homogeneous space
// * x_h in [-1, 1]
// * y_h in [-1, 1]

// World space
use crate::camera::CameraViewPort;

use num_traits::FloatConst;
trait MyFloat: cgmath::BaseFloat + FloatConst {}
impl MyFloat for f32 {}
impl MyFloat for f64 {}

#[allow(dead_code)]
pub fn screen_to_ndc_space(
    pos_screen_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    // Screen space in pixels to homogeneous screen space (values between [-1, 1])
    let window_size = camera.get_screen_size();
    let window_size = Vector2::new(window_size.x as f64, window_size.y as f64);
    // Change of origin
    let origin = pos_screen_space - window_size / 2.0;

    // Scale to fit in [-1, 1]
    let pos_normalized_device = Vector2::new(
        2.0 * (origin.x / window_size.x),
        -2.0 * (origin.y / window_size.y),
    );
    pos_normalized_device
}

#[allow(dead_code)]
pub fn ndc_to_screen_space(
    pos_normalized_device: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let window_size = camera.get_screen_size();

    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5 + 0.5) * (window_size.x as f64),
        (0.5 - pos_normalized_device.y * 0.5) * (window_size.y as f64),
    );

    pos_screen_space
}

#[allow(dead_code)]
pub fn clip_to_ndc_space(pos_clip_space: &Vector2<f64>, camera: &CameraViewPort) -> Vector2<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    Vector2::new(
        pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
        pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
    )
}

#[allow(dead_code)]
pub fn clip_to_screen_space(
    pos_clip_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let pos_normalized_device = clip_to_ndc_space(pos_clip_space, camera);

    let window_size = camera.get_screen_size();
    let pos_screen_space = Vector2::new(
        (pos_normalized_device.x * 0.5 + 0.5) * (window_size.x as f64),
        (0.5 - pos_normalized_device.y * 0.5) * (window_size.y as f64),
    );

    pos_screen_space
}

#[allow(dead_code)]
pub fn screen_to_clip_space(
    pos_screen_space: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);

    ndc_to_clip_space(&pos_normalized_device, camera)
}

#[allow(dead_code)]
pub fn ndc_to_clip_space(
    pos_normalized_device: &Vector2<f64>,
    camera: &CameraViewPort,
) -> Vector2<f64> {
    let ndc_to_clip = camera.get_ndc_to_clip();
    let clip_zoom_factor = camera.get_clip_zoom_factor();

    let pos_clip_space = Vector2::new(
        pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
        pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
    );

    pos_clip_space
}

use crate::renderable::{catalog::CatalogShaderProjection, grid::GridShaderProjection};
use crate::shader::GetShader;
use cgmath::InnerSpace;
use cgmath::Vector4;
pub trait Projection:
    GetShader + CatalogShaderProjection + GridShaderProjection + std::marker::Sized
{
    /// Screen to model space deprojection

    /// Perform a screen to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_screen_space`` - The position in the screen pixel space (top-left of the screen being the origin
    /// * ``camera`` - The camera object
    fn screen_to_world_space(
        pos_screen_space: &Vector2<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector4<f64>> {
        let pos_normalized_device = screen_to_ndc_space(pos_screen_space, camera);

        let ndc_to_clip = camera.get_ndc_to_clip();
        let clip_zoom_factor = camera.get_clip_zoom_factor();

        let pos_clip_space = Vector2::new(
            pos_normalized_device.x * ndc_to_clip.x * clip_zoom_factor,
            pos_normalized_device.y * ndc_to_clip.y * clip_zoom_factor,
        );
        let pos_world_space =
            Self::clip_to_world_space(&pos_clip_space, camera.is_reversed_longitude());
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
    fn screen_to_model_space(
        pos_screen_space: &Vector2<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector4<f64>> {
        let pos_world_space = Self::screen_to_world_space(pos_screen_space, camera);

        if let Some(pos_world_space) = pos_world_space {
            let r = camera.get_final_rotation();
            let pos_model_space = r.rotate(&pos_world_space);

            Some(pos_model_space)
        } else {
            None
        }
    }

    fn model_to_screen_space(
        pos_model_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        Self::world_to_screen_space(&pos_world_space, camera)
    }

    fn model_to_ndc_space(
        pos_model_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        let m2w = camera.get_m2w();
        let pos_world_space = m2w * pos_model_space;
        //pos_world_space.x = -pos_world_space.x;
        Self::world_to_normalized_device_space(&pos_world_space, camera)
    }

    fn clip_to_model_space(
        pos_clip_space: &Vector2<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector4<f64>> {
        let pos_world_space =
            Self::clip_to_world_space(pos_clip_space, camera.is_reversed_longitude());

        if let Some(pos_world_space) = pos_world_space {
            let w2m = camera.get_w2m();
            let pos_model_space = w2m * pos_world_space;

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
    fn world_to_normalized_device_space(
        pos_world_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        if let Some(pos_clip_space) =
            Self::world_to_clip_space(pos_world_space, camera.is_reversed_longitude())
        {
            let ndc_to_clip = camera.get_ndc_to_clip();
            let clip_zoom_factor = camera.get_clip_zoom_factor();

            let pos_normalized_device = Vector2::new(
                pos_clip_space.x / (ndc_to_clip.x * clip_zoom_factor),
                pos_clip_space.y / (ndc_to_clip.y * clip_zoom_factor),
            );
            Some(pos_normalized_device)
        } else {
            None
        }
    }

    fn world_to_screen_space(
        pos_world_space: &Vector4<f64>,
        camera: &CameraViewPort,
    ) -> Option<Vector2<f64>> {
        if let Some(pos_normalized_device) =
            Self::world_to_normalized_device_space(pos_world_space, camera)
        {
            Some(self::ndc_to_screen_space(&pos_normalized_device, camera))
        } else {
            None
        }
    }

    /// Perform a clip to the world space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_clip_space`` - The position in the clipping space (orthonorlized space)
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector4<f64>>;
    /// World to the clipping space deprojection
    ///
    /// # Arguments
    ///
    /// * ``pos_world_space`` - The position in the world space
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>>;

    // Aperture angle at the start of the application (full view)
    // - 180 degrees for the 3D projections (i.e. ortho)
    // - 360 degrees for the 2D projections (i.e. mollweide, arc, aitoff...)
    fn aperture_start() -> Angle<f64>;

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool;

    fn is_front_of_camera(pos_world_space: &Vector4<f64>) -> bool;

    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64>;

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)>;
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)>;

    const ALLOW_UNZOOM_MORE: bool;

    const RASTER_THRESHOLD_ANGLE: Angle<f64>;
}

pub struct Aitoff;
pub struct Mollweide;
pub struct Orthographic;
pub struct AzimuthalEquidistant;
pub struct Gnomonic;
pub struct Mercator;

use cgmath::Vector2;

use crate::renderable::ArcDeg;

impl Projection for Aitoff {
    const ALLOW_UNZOOM_MORE: bool = true;

    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1.0, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
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

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 0.5 {
            None
        } else {
            let x = (1.0 - 4.0 * y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1.0 {
            None
        } else {
            let y = (1.0 - x * x).sqrt() * 0.5;
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        if Self::is_included_inside_projection(&pos_clip_space) {
            let u = pos_clip_space.x * f64::PI() * 0.5;
            let v = pos_clip_space.y * f64::PI();
            //da uv a lat/lon
            let c = (v * v + u * u).sqrt();

            let (phi, mut theta) = if c != 0.0 {
                let phi = (v * c.sin() / c).asin();
                let theta = (u * c.sin()).atan2(c * c.cos());
                (phi, theta)
            } else {
                let phi = v.asin();
                let theta = u.atan();
                (phi, theta)
            };
            theta *= 2.0;

            let mut pos_world_space = cgmath::Vector4::new(
                theta.sin() * phi.cos(),
                phi.sin(),
                theta.cos() * phi.cos(),
                1.0,
            );

            if longitude_reversed {
                pos_world_space.x = -pos_world_space.x;
            }
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
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        //return vec3(X / PI, aspect * Y / PI, 0.f);

        //let pos_world_space = pos_world_space;

        let xyz = pos_world_space.truncate();
        let (mut theta, delta) = math::xyz_to_radec(&xyz);
        if longitude_reversed {
            theta.0 = -theta.0;
        }

        let theta_by_two = theta / 2.0;

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

        Some(Vector2::new(
            x / std::f64::consts::PI,
            y / std::f64::consts::PI,
        ))
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(360.0).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f64>) -> bool {
        // 2D projections always faces the camera
        true
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((150.0 / 180.0) * std::f64::consts::PI);
}

use crate::math;
impl Projection for Mollweide {
    const ALLOW_UNZOOM_MORE: bool = true;

    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
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

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 0.5_f64 {
            None
        } else {
            let x = (1.0 - 4.0 * y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        if Self::is_included_inside_projection(&pos_clip_space) {
            let y2 = pos_clip_space.y * pos_clip_space.y;
            let k = (1.0 - 4.0 * y2).sqrt();

            let theta = f64::PI() * pos_clip_space.x / k;
            let delta = ((2.0 * (2.0 * pos_clip_space.y).asin() + 4.0 * pos_clip_space.y * k)
                / f64::PI())
            .asin();

            // The minus is an astronomical convention.
            // longitudes are increasing from right to left
            let mut pos_world_space = cgmath::Vector4::new(
                theta.sin() * delta.cos(),
                delta.sin(),
                theta.cos() * delta.cos(),
                1.0,
            );

            if longitude_reversed {
                pos_world_space.x = -pos_world_space.x;
            }

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
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        // X in [-1, 1]
        // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
        let epsilon = 1e-12;
        let max_iter = 10;

        let xyz = pos_world_space.truncate();
        let (mut lon, lat) = math::xyz_to_radec(&xyz);
        if longitude_reversed {
            lon = -lon;
        }

        let cst = std::f64::consts::PI * lat.sin();

        let mut theta = lat.0;
        let mut f = theta + theta.sin() - cst;

        let mut k = 0;
        while f.abs() > epsilon && k < max_iter {
            theta -= f / (1.0 + theta.cos());
            f = theta + theta.sin() - cst;

            k += 1;
        }

        theta = theta / 2.0;

        // The minus is an astronomical convention.
        // longitudes are increasing from right to left
        let x = (lon.0 / std::f64::consts::PI) * theta.cos();
        let y = 0.5 * theta.sin();

        Some(Vector2::new(x, y))
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(360_f64).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f64>) -> bool {
        // 2D projections always faces the camera
        true
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((180.0 / 180.0) * std::f64::consts::PI);
}

use crate::renderable::Angle;
impl Projection for Orthographic {
    const ALLOW_UNZOOM_MORE: bool = true;

    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) < 1_f64
    }

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0_f64 {
            None
        } else {
            let x = (1.0 - y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        let xw_2 = 1.0 - pos_clip_space.x * pos_clip_space.x - pos_clip_space.y * pos_clip_space.y;
        if xw_2 > 0.0 {
            let mut pos_world_space =
                cgmath::Vector4::new(pos_clip_space.x, pos_clip_space.y, xw_2.sqrt(), 1_f64);

            if longitude_reversed {
                pos_world_space.x = -pos_world_space.x;
            }

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
    fn world_to_clip_space(
        pos_world_space: &cgmath::Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        if pos_world_space.z < 0.0_f64 {
            None
        } else if longitude_reversed {
            Some(Vector2::new(-pos_world_space.x, pos_world_space.y))
        } else {
            Some(Vector2::new(pos_world_space.x, pos_world_space.y))
        }
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(180_f64).into()
    }

    fn is_front_of_camera(pos_world_space: &Vector4<f64>) -> bool {
        pos_world_space.z > 0_f64
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((110.0 / 180.0) * std::f64::consts::PI);
}

impl Projection for AzimuthalEquidistant {
    const ALLOW_UNZOOM_MORE: bool = true;

    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1.0, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) < 1.0
    }

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            let x = (1.0 - y * y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        // r <= pi
        let x = pos_clip_space.x * f64::PI();
        let y = pos_clip_space.y * f64::PI();
        let mut r = (x * x + y * y).sqrt();
        if r > f64::PI() {
            None
        } else {
            let z = r.cos();
            r = math::sinc_positive(r);

            let pos_world_space = if longitude_reversed {
                Vector4::new(-x * r, y * r, z, 1.0)
            } else {
                Vector4::new(-x * r, y * r, z, 1.0)
            };

            Some(pos_world_space)
        }
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        if pos_world_space.z > -1.0 {
            // Distance in the Euclidean plane (xy)
            // Angular distance is acos(x), but for small separation, asin(r)
            // is more accurate.
            let mut r = (pos_world_space.x * pos_world_space.x
                + pos_world_space.y * pos_world_space.y)
                .sqrt();
            if pos_world_space.z > 0.0 {
                // Angular distance < PI/2, angular distance = asin(r)
                r = math::asinc_positive::<f64>(r);
            } else {
                // Angular distance > PI/2, angular distance = acos(x)
                r = pos_world_space.z.acos() / r;
            }
            let x = if longitude_reversed {
                -pos_world_space.x * r
            } else {
                pos_world_space.x * r
            };
            let y = pos_world_space.y * r;

            Some(Vector2::new(
                x / std::f64::consts::PI,
                y / std::f64::consts::PI,
            ))
        } else {
            Some(Vector2::new(1.0, 0.0))
        }
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(360.0).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f64>) -> bool {
        // 2D projections always faces the camera
        true
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((160.0 / 180.0) * std::f64::consts::PI);
}

impl Projection for Gnomonic {
    const ALLOW_UNZOOM_MORE: bool = false;

    /*fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
        let px2 = pos_clip_space.x * pos_clip_space.x;
        let py2 = pos_clip_space.y * pos_clip_space.y;

        (px2 + py2) < 1_f64
    }

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0_f64 {
            None
        } else {
            let x = (1.0 - y*y).sqrt();
            Some((-x + 1e-3, x - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
        if x.abs() > 1_f64 {
            None
        } else {
            let y = (1.0 - x*x).sqrt();
            Some((-y + 1e-3, y - 1e-3))
        }
    }*/
    fn compute_ndc_to_clip_factor(width: f64, height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, height / width)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
        let px = pos_clip_space.x;
        let py = pos_clip_space.y;

        px > -1.0 && px < 1.0 && py > -1.0 && py < 1.0
    }

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        //if pos_clip_space.x * pos_clip_space.x + pos_clip_space.y * pos_clip_space.y >= 1.0 {
        //    None
        //} else {
        let x_2d = pos_clip_space.x * f64::PI();
        let y_2d = pos_clip_space.y * f64::PI();
        let r = x_2d * x_2d + y_2d * y_2d;

        let z = (1.0 + r).sqrt();
        let pos_world_space = if longitude_reversed {
            Vector4::new(-z * x_2d, z * y_2d, z, 1.0)
        } else {
            Vector4::new(z * x_2d, z * y_2d, z, 1.0)
        };

        Some(pos_world_space)
        //}
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        if pos_world_space.z <= 1e-2 {
            // Back hemisphere (z < 0) + diverges near z=0
            None
        } else {
            let pos_clip_space = if longitude_reversed {
                Vector2::new(
                    (-pos_world_space.x / pos_world_space.z) / std::f64::consts::PI,
                    (pos_world_space.y / pos_world_space.z) / std::f64::consts::PI,
                )
            } else {
                Vector2::new(
                    (pos_world_space.x / pos_world_space.z) / std::f64::consts::PI,
                    (pos_world_space.y / pos_world_space.z) / std::f64::consts::PI,
                )
            };
            Some(pos_clip_space)
        }
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(180.0).into()
    }

    fn is_front_of_camera(pos_world_space: &Vector4<f64>) -> bool {
        // 2D projections always faces the camera
        pos_world_space.z >= 1e-2
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((90.0 / 180.0) * std::f64::consts::PI);
}

impl Projection for Mercator {
    const ALLOW_UNZOOM_MORE: bool = false;

    fn compute_ndc_to_clip_factor(_width: f64, _height: f64) -> Vector2<f64> {
        Vector2::new(1_f64, 0.5_f64)
    }

    fn is_included_inside_projection(pos_clip_space: &Vector2<f64>) -> bool {
        let px = pos_clip_space.x;
        let py = pos_clip_space.y;

        px > -1.0 && px < 1.0 && py > -1.0 && py < 1.0
    }

    fn solve_along_abscissa(y: f64) -> Option<(f64, f64)> {
        if y.abs() > 1.0 {
            None
        } else {
            Some((-1.0 + 1e-3, 1.0 - 1e-3))
        }
    }
    fn solve_along_ordinate(x: f64) -> Option<(f64, f64)> {
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
    fn clip_to_world_space(
        pos_clip_space: &Vector2<f64>,
        longitude_reversed: bool,
    ) -> Option<cgmath::Vector4<f64>> {
        /*let xw_2 = 1_f64 - pos_clip_space.x*pos_clip_space.x - pos_clip_space.y*pos_clip_space.y;
        if xw_2 > 0_f64 {
            let (x, y) = (2_f64 * pos_clip_space.x, 2_f64 * pos_clip_space.y);

            let rho2 = (x*x + y*y);
            let rho = rho2.sqrt();

            let c = 2_f64 * (0.5_f64 * rho).asin();

            let mut delta = 0_f64;
            let mut theta = 0_f64;
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
        let theta = pos_clip_space.x * f64::PI();
        let delta = (pos_clip_space.y.sinh()).atan() * f64::PI();

        let mut pos_world_space = math::radec_to_xyzw(Angle(theta), Angle(delta));
        if longitude_reversed {
            pos_world_space.x = -pos_world_space.x;
        }

        Some(pos_world_space)
    }

    /// World to screen space transformation
    ///
    /// # Arguments
    ///
    /// * `pos_world_space` - Position in the world space. Must be a normalized vector
    fn world_to_clip_space(
        pos_world_space: &Vector4<f64>,
        longitude_reversed: bool,
    ) -> Option<Vector2<f64>> {
        let (mut theta, delta) = math::xyzw_to_radec(&pos_world_space);

        if longitude_reversed {
            theta.0 = -theta.0;
        }

        Some(Vector2::new(
            theta.0 / std::f64::consts::PI,
            ((delta.0 / std::f64::consts::PI).tan()).asinh() as f64,
        ))
    }

    fn aperture_start() -> Angle<f64> {
        ArcDeg(360_f64).into()
    }

    fn is_front_of_camera(_pos_world_space: &Vector4<f64>) -> bool {
        // 2D projections always faces the camera
        true
    }

    const RASTER_THRESHOLD_ANGLE: Angle<f64> = Angle((180.0 / 180.0) * std::f64::consts::PI);
}
