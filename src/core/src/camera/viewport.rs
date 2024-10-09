#[derive(PartialEq, Clone, Copy)]
pub enum UserAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

// Longitude reversed identity matrix
const ID_R: &Matrix4<f64> = &Matrix4::new(
    -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

use super::{fov::FieldOfView, view_hpx_cells::ViewHpxCells};
use crate::healpix::cell::HEALPixCell;
use crate::healpix::coverage::HEALPixCoverage;
use crate::math::angle::ToAngle;
use crate::math::{projection::coo_space::XYZWModel, projection::domain::sdf::ProjDef};

use cgmath::{Matrix4, Vector2};
pub struct CameraViewPort {
    // The field of view angle
    aperture: Angle<f64>,
    // The rotation of the camera
    center: Vector4<f64>,
    w2m_rot: Rotation<f64>,

    w2m: Matrix4<f64>,
    m2w: Matrix4<f64>,
    // The width over height ratio
    aspect: f32,
    // The width of the screen in pixels
    width: f32,
    // The height of the screen in pixels
    height: f32,
    // dpi. Equals to 1.0 normally but HDI screens
    // can have greater values. For macbook pro retina screen, this
    // should be equal to 2
    dpi: f32,

    // HEALPix depth of 512 large tiles
    texture_depth: u8,

    // Internal variable used for projection purposes
    ndc_to_clip: Vector2<f64>,
    clip_zoom_factor: f64,
    // The vertices in model space of the camera
    // This is useful for computing views according
    // to different image surveys
    fov: FieldOfView,
    // A data structure storing HEALPix cells contained in the fov
    // for different frame and depth
    view_hpx_cells: ViewHpxCells,

    // A flag telling whether the camera has been moved during the frame
    moved: bool,
    // A flag telling whether the camera has zoomed during the frame
    zoomed: bool,

    // Tag the last action done by the user
    last_user_action: UserAction,

    // longitude reversed flag
    is_allsky: bool,

    // Time when the camera has moved
    time_last_move: Time,

    // A reference to the WebGL2 context
    gl: WebGlContext,
    coo_sys: CooSystem,
    reversed_longitude: bool,
}
use al_api::coo_system::CooSystem;
use al_core::WebGlContext;

use crate::{
    coosys,
    math::{angle::Angle, projection::Projection, rotation::Rotation},
};

use crate::LonLatT;
use cgmath::{SquareMatrix, Vector4};
use wasm_bindgen::JsCast;

const MAX_DPI_LIMIT: f32 = 2.0;
use crate::math;
use crate::time::Time;
use crate::Abort;
use crate::ArcDeg;
impl CameraViewPort {
    pub fn new(
        gl: &WebGlContext,
        coo_sys: CooSystem,
        projection: &ProjectionType,
    ) -> CameraViewPort {
        let last_user_action = UserAction::Starting;

        let aperture = Angle(projection.aperture_start());

        let w2m = Matrix4::identity();
        let m2w = w2m;
        let center = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let moved = false;
        let zoomed = false;

        let w2m_rot = Rotation::zero();

        // Get the initial size of the window
        let window = web_sys::window().unwrap_abort();
        let width = window.inner_width().unwrap_abort().as_f64().unwrap_abort() as f32;
        let height = window.inner_height().unwrap_abort().as_f64().unwrap_abort() as f32;
        // Clamp it to 3 at maximum, this for limiting the number of pixels drawn
        let dpi = if window.device_pixel_ratio() as f32 > MAX_DPI_LIMIT {
            MAX_DPI_LIMIT
        } else {
            window.device_pixel_ratio() as f32
        };

        let width = width * dpi;
        let height = height * dpi;

        let aspect = height / width;
        let ndc_to_clip = Vector2::new(1.0, (height as f64) / (width as f64));
        let clip_zoom_factor = 1.0;

        let fov = FieldOfView::new(&ndc_to_clip, clip_zoom_factor, &w2m, projection);
        let gl = gl.clone();

        let is_allsky = true;
        let time_last_move = Time::now();
        let reversed_longitude = false;

        let texture_depth = 0;

        let view_hpx_cells = ViewHpxCells::new();
        CameraViewPort {
            // The field of view angle
            aperture,
            center,
            // The rotation of the cameraq
            w2m_rot,
            w2m,
            m2w,

            dpi,
            // The width over height ratio
            aspect,
            // The width of the screen in pixels
            width,
            // The height of the screen in pixels
            height,
            is_allsky,

            // Internal variable used for projection purposes
            ndc_to_clip,
            clip_zoom_factor,
            // The field of view
            fov,
            view_hpx_cells,
            // A flag telling whether the camera has been moved during the frame
            moved,
            // A flag telling if the camera has zoomed during the frame
            zoomed,

            // Tag the last action done by the user
            last_user_action,
            // Time when the camera has moved
            // for the last time
            time_last_move,

            texture_depth,

            // A reference to the WebGL2 context
            gl,
            // coo system
            coo_sys,
            // a flag telling if the viewport has a reversed longitude axis
            reversed_longitude,
        }
    }

    pub fn register_view_frame(&mut self, frame: CooSystem, proj: &ProjectionType) {
        self.view_hpx_cells.register_frame(
            self.texture_depth,
            &self.fov,
            &self.center,
            self.coo_sys,
            proj,
            frame,
        );
    }

    pub fn unregister_view_frame(&mut self, frame: CooSystem, proj: &ProjectionType) {
        self.view_hpx_cells.unregister_frame(
            self.texture_depth,
            &self.fov,
            &self.center,
            self.coo_sys,
            proj,
            frame,
        );
    }

    /*pub fn has_new_hpx_cells(&mut self) -> bool {
        self.view_hpx_cells.has_changed()
    }*/

    pub fn get_cov(&self, frame: CooSystem) -> &HEALPixCoverage {
        self.view_hpx_cells.get_cov(frame)
    }

    pub fn get_hpx_cells(&self, depth: u8, frame: CooSystem) -> Vec<HEALPixCell> {
        self.view_hpx_cells.get_cells(depth, frame)
    }

    pub fn is_raytracing(&self, proj: &ProjectionType) -> bool {
        // Check whether the tile depth is 0 for square projection
        // definition domains i.e. Mercator
        if self.is_allsky() {
            return true;
        }

        // check the projection
        match proj {
            ProjectionType::Tan(_) => self.aperture >= 100.0_f64.to_radians().to_angle(),
            ProjectionType::Mer(_) => self.aperture >= 120.0_f64.to_radians().to_angle(),
            ProjectionType::Stg(_) => self.aperture >= 200.0_f64.to_radians().to_angle(),
            ProjectionType::Sin(_) => false,
            ProjectionType::Ait(_) => self.aperture >= 100.0_f64.to_radians().to_angle(),
            ProjectionType::Mol(_) => self.aperture >= 100.0_f64.to_radians().to_angle(),
            ProjectionType::Zea(_) => self.aperture >= 140.0_f64.to_radians().to_angle(),
        }
    }

    fn recompute_scissor(&self) {
        // Clear all the screen before updating the scissor
        //self.gl.scissor(0, 0, self.width as i32, self.height as i32);
        //self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // Width and Height of the clipping space
        const WC: f64 = 2.0;
        const HC: f64 = 2.0;

        let tl_c = Vector2::new(-WC * 0.5, HC * 0.5);
        let tr_c = Vector2::new(WC * 0.5, HC * 0.5);
        let br_c = Vector2::new(WC * 0.5, -HC * 0.5);
        let mut tl_s = crate::math::projection::clip_to_screen_space(&tl_c, self);
        let mut tr_s = crate::math::projection::clip_to_screen_space(&tr_c, self);
        let mut br_s = crate::math::projection::clip_to_screen_space(&br_c, self);

        tl_s.x *= self.dpi as f64;
        tl_s.y *= self.dpi as f64;

        tr_s.x *= self.dpi as f64;
        tr_s.y *= self.dpi as f64;

        br_s.x *= self.dpi as f64;
        br_s.y *= self.dpi as f64;

        let w = (tr_s.x - tl_s.x).min(self.width as f64);
        let h = (br_s.y - tr_s.y).min(self.height as f64);

        // Specify a scissor here
        self.gl.scissor(
            (tl_s.x as i32).max(0),
            (tl_s.y as i32).max(0),
            w as i32,
            h as i32,
        );
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32, projection: &ProjectionType) {
        self.width = (width as f32) * self.dpi;
        self.height = (height as f32) * self.dpi;

        self.aspect = width / height;
        // Compute the new clip zoom factor
        self.compute_ndc_to_clip_factor(projection);

        self.fov.set_aperture(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            projection,
        );

        let proj_area = projection.get_area();
        self.is_allsky = !proj_area.is_in(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));

        // Update the size of the canvas
        let canvas = self
            .gl
            .canvas()
            .unwrap_abort()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_abort();

        canvas.set_width(self.width as u32);
        canvas.set_height(self.height as u32);
        // Once the canvas size is changed, we have to set the viewport as well
        self.gl
            .viewport(0, 0, self.width as i32, self.height as i32);
        // Once it is done, recompute the scissor
        self.recompute_scissor();
    }

    pub fn compute_ndc_to_clip_factor(&mut self, proj: &ProjectionType) {
        self.ndc_to_clip = if self.height < self.width {
            Vector2::new(1.0, (self.height as f64) / (self.width as f64))
        } else {
            Vector2::new((self.width as f64) / (self.height as f64), 1.0)
        };

        let bounds_size_ratio = proj.bounds_size_ratio();
        self.ndc_to_clip.y *= bounds_size_ratio;
    }

    pub fn set_projection(&mut self, proj: &ProjectionType) {
        // Compute the new clip zoom factor
        self.compute_ndc_to_clip_factor(proj);
        self.set_aperture(self.aperture, proj);
    }

    pub fn set_aperture(&mut self, aperture: Angle<f64>, proj: &ProjectionType) {
        // Checking if we are zooming or unzooming
        // This is used internaly for the raytracer to compute
        // blending between tiles and their parents (or children)
        self.last_user_action = if self.get_aperture() > aperture {
            UserAction::Zooming
        } else if self.get_aperture() < aperture {
            UserAction::Unzooming
        } else {
            self.last_user_action
        };

        let _can_unzoom_more = match proj {
            ProjectionType::Tan(_)
            | ProjectionType::Mer(_)
            //| ProjectionType::Air(_)
            | ProjectionType::Stg(_) => false,
            //| ProjectionType::Car(_)
            //| ProjectionType::Cea(_)
            //| ProjectionType::Cyp(_)
            //| ProjectionType::Hpx(_) => false,
            _ => true,
        };

        let aperture_start: Angle<f64> = ArcDeg(proj.aperture_start()).into();

        self.aperture = aperture.min(aperture_start);
        // Compute the new clip zoom factor
        let a = aperture.abs();

        let v0 = math::lonlat::radec_to_xyzw(-a / 2.0, Angle(0.0));
        let v1 = math::lonlat::radec_to_xyzw(a / 2.0, Angle(0.0));

        // Vertex in the WCS of the FOV
        self.clip_zoom_factor = if self.width < self.height {
            if let (Some(p0), Some(p1)) =
                (proj.world_to_clip_space(&v0), proj.world_to_clip_space(&v1))
            {
                (0.5 * (p1.x - p0.x).abs()).min(1.0)
            } else {
                1.0
            }
        } else {
            if let (Some(p0), Some(p1)) =
                (proj.world_to_clip_space(&v0), proj.world_to_clip_space(&v1))
            {
                (0.5 * (p1.x - p0.x).abs()).min(1.0)
            } else {
                1.0
            }
        };

        //console_log(&format!("clip factor {:?}", self.aperture));

        // Project this vertex into the screen
        self.moved = true;
        self.zoomed = true;
        self.time_last_move = Time::now();

        self.fov
            .set_aperture(&self.ndc_to_clip, self.clip_zoom_factor, &self.w2m, proj);

        let proj_area = proj.get_area();
        self.is_allsky = !proj_area.is_in(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));

        self.compute_texture_depth();

        // recompute the scissor with the new aperture
        self.recompute_scissor();

        // compute the hpx cells
        self.view_hpx_cells.update(
            self.texture_depth,
            &self.fov,
            &self.center,
            self.get_coo_system(),
            proj,
        );
    }

    fn compute_texture_depth(&mut self) {
        /*// Compute a depth from a number of pixels on screen
        let width = self.width;
        let aperture = self.aperture.0 as f32;

        let angle_per_pixel = aperture / width;

        let two_power_two_times_depth_pixel =
            std::f32::consts::PI / (3.0 * angle_per_pixel * angle_per_pixel);
        let depth_pixel = (two_power_two_times_depth_pixel.log2() / 2.0).floor() as u32;

        //let survey_max_depth = conf.get_max_depth();
        // The depth of the texture
        // A texture of 512x512 pixels will have a depth of 9
        const DEPTH_OFFSET_TEXTURE: u32 = 9;
        // The depth of the texture corresponds to the depth of a pixel
        // minus the offset depth of the texture
        self.texture_depth = if DEPTH_OFFSET_TEXTURE > depth_pixel {
            0_u8
        } else {
            (depth_pixel - DEPTH_OFFSET_TEXTURE) as u8
        };*/

        let w_screen_px = self.width as f64;
        let smallest_cell_size_px = self.dpi as f64;
        let mut depth_pixel = 29 as usize;

        let hpx_cell_size_rad =
            (smallest_cell_size_px / w_screen_px) * self.get_aperture().to_radians();

        while depth_pixel > 0 {
            if crate::healpix::utils::MEAN_HPX_CELL_RES[depth_pixel] > hpx_cell_size_rad {
                break;
            }

            depth_pixel = depth_pixel - 1;
        }
        depth_pixel += 1;
        const DEPTH_OFFSET_TEXTURE: usize = 9;
        self.texture_depth = if DEPTH_OFFSET_TEXTURE > depth_pixel {
            0_u8
        } else {
            (depth_pixel - DEPTH_OFFSET_TEXTURE) as u8
        };
    }

    pub fn get_texture_depth(&self) -> u8 {
        self.texture_depth
    }

    pub fn apply_rotation(
        &mut self,
        axis: &cgmath::Vector3<f64>,
        angle: Angle<f64>,
        proj: &ProjectionType,
    ) {
        // Rotate the axis:
        let drot = Rotation::from_axis_angle(axis, angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices(proj);
    }

    /// center lonlat must be given in icrs frame
    pub fn set_center(&mut self, lonlat: &LonLatT<f64>, proj: &ProjectionType) {
        let icrs_pos: Vector4<_> = lonlat.vector();

        let view_pos = CooSystem::ICRS.to(self.get_coo_system()) * icrs_pos;
        let rot_to_center = Rotation::from_sky_position(&view_pos);

        let phi = self.get_center_pos_angle();
        let third_euler_rot = Rotation::from_axis_angle(&view_pos.truncate(), phi);

        let rot = third_euler_rot * rot_to_center;

        // Apply the rotation to the camera to go
        // to the next lonlat
        self.set_rotation(&rot, proj);
    }

    pub fn set_center_pos_angle(&mut self, phi: Angle<f64>, proj: &ProjectionType) {
        let rot_to_center = Rotation::from_sky_position(&self.center);
        let third_euler_rot = Rotation::from_axis_angle(&self.center.truncate(), phi);

        let total_rot = third_euler_rot * rot_to_center;
        self.set_rotation(&total_rot, proj);
    }

    fn set_rotation(&mut self, rot: &Rotation<f64>, proj: &ProjectionType) {
        self.w2m_rot = *rot;

        self.update_rot_matrices(proj);
    }

    pub fn get_field_of_view(&self) -> &FieldOfView {
        &self.fov
    }

    pub fn set_coo_system(&mut self, new_coo_sys: CooSystem, proj: &ProjectionType) {
        // Compute the center position according to the new coordinate frame system
        let new_center = coosys::apply_coo_system(self.coo_sys, new_coo_sys, &self.center);
        // Create a rotation object from that position
        let new_rotation = Rotation::from_sky_position(&new_center);
        // Apply it to the center of the view
        self.set_rotation(&new_rotation, proj);

        // unregister the coo sys
        //self.view_hpx_cells.unregister_frame(self.coo_sys);
        // register the new one
        //self.view_hpx_cells.register_frame(new_coo_sys);
        // recompute the coverage if necessary
        self.view_hpx_cells.update(
            self.texture_depth,
            &self.fov,
            &self.center,
            new_coo_sys,
            proj,
        );

        // Record the new system
        self.coo_sys = new_coo_sys;
    }

    pub fn set_longitude_reversed(&mut self, reversed_longitude: bool, proj: &ProjectionType) {
        if self.reversed_longitude != reversed_longitude {
            self.reversed_longitude = reversed_longitude;

            self.update_rot_matrices(proj);
        }
    }

    pub fn get_longitude_reversed(&self) -> bool {
        self.reversed_longitude
    }

    // Accessors
    pub fn get_w2m(&self) -> &cgmath::Matrix4<f64> {
        &self.w2m
    }

    pub fn get_m2w(&self) -> &cgmath::Matrix4<f64> {
        &self.m2w
    }

    pub fn get_aspect(&self) -> f32 {
        self.aspect
    }

    pub fn get_ndc_to_clip(&self) -> &Vector2<f64> {
        &self.ndc_to_clip
    }

    pub fn get_clip_zoom_factor(&self) -> f64 {
        self.clip_zoom_factor
    }

    pub fn get_vertices(&self) -> Option<&Vec<XYZWModel<f64>>> {
        self.fov.get_vertices()
    }

    pub fn get_screen_size(&self) -> Vector2<f32> {
        Vector2::new(self.width, self.height)
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn get_last_user_action(&self) -> UserAction {
        self.last_user_action
    }

    pub fn get_dpi(&self) -> f32 {
        self.dpi
    }

    pub fn has_moved(&self) -> bool {
        self.moved
    }

    pub fn has_zoomed(&self) -> bool {
        self.zoomed
    }

    // Reset moving flag
    pub fn reset(&mut self) {
        self.moved = false;
        self.zoomed = false;
    }

    #[inline]
    pub fn get_aperture(&self) -> Angle<f64> {
        self.aperture
    }

    #[inline]
    pub fn get_center(&self) -> &Vector4<f64> {
        &self.center
    }

    #[inline]
    pub fn is_allsky(&self) -> bool {
        self.is_allsky
    }

    pub fn get_time_of_last_move(&self) -> Time {
        self.time_last_move
    }

    pub fn get_coo_system(&self) -> CooSystem {
        self.coo_sys
    }

    pub fn get_center_pos_angle(&self) -> Angle<f64> {
        (self.w2m.x.y).atan2(self.w2m.y.y).to_angle()
    }
}
use crate::ProjectionType;
use cgmath::Matrix;
//use crate::coo_conversion::CooBaseFloat;
impl CameraViewPort {
    // private methods
    fn update_rot_matrices(&mut self, proj: &ProjectionType) {
        self.w2m = (&(self.w2m_rot)).into();

        if self.reversed_longitude {
            self.w2m = self.w2m * ID_R;
        }
        self.m2w = self.w2m.transpose();

        self.center = self.w2m.z;

        // Rotate the fov vertices
        self.fov.set_rotation(&self.w2m);

        self.time_last_move = Time::now();
        self.last_user_action = UserAction::Moving;
        self.moved = true;

        // compute the hpx cells
        self.view_hpx_cells.update(
            self.texture_depth,
            &self.fov,
            &self.center,
            self.get_coo_system(),
            proj,
        );
    }
}

use al_core::shader::{SendUniforms, ShaderBound};
impl SendUniforms for CameraViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniform("ndc_to_clip", &self.ndc_to_clip) // Send ndc to clip
            .attach_uniform("czf", &self.clip_zoom_factor); // Send clip zoom factor

        shader
    }
}
