#[derive(PartialEq, Clone, Copy)]
pub enum UserAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

use super::fov::FieldOfViewVertices;
use crate::math::{
    projection::coo_space::XYZWModel,
    spherical::BoundingBox,
    projection::domain::sdf::ProjDef
};
use cgmath::{Matrix4, Vector2};

pub struct CameraViewPort {
    // The field of view angle
    aperture: Angle<f64>,
    center: Vector4<f64>,
    // The rotation of the camera
    rotation_center_angle: Angle<f64>,
    w2m_rot: Rotation<f64>,
    final_rot: Rotation<f64>,

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
    tile_depth: u8,

    // Internal variable used for projection purposes
    ndc_to_clip: Vector2<f64>,
    clip_zoom_factor: f64,
    // The vertices in model space of the camera
    // This is useful for computing views according
    // to different image surveys
    vertices: FieldOfViewVertices,

    // A flag telling whether the camera has been moved during the frame
    moved: bool,

    // Tag the last action done by the user
    last_user_action: UserAction,

    // longitude reversed flag
    is_allsky: bool,

    // Time when the camera has moved
    time_last_move: Time,

    // A reference to the WebGL2 context
    gl: WebGlContext,
    system: CooSystem,
    reversed_longitude: bool,
}
use al_api::coo_system::CooSystem;
use al_core::WebGlContext;

use crate::{
    coosys,
    math::{angle::Angle, projection::Projection, rotation::Rotation, spherical::FieldOfViewType},
};

use crate::LonLatT;
use cgmath::{SquareMatrix, Vector4};
use wasm_bindgen::JsCast;

const MAX_DPI_LIMIT: f32 = 3.0;
use crate::Abort;
use crate::math;
use crate::time::Time;
use crate::ArcDeg;
impl CameraViewPort {
    pub fn new(gl: &WebGlContext, system: CooSystem, projection: &ProjectionType) -> CameraViewPort {
        let last_user_action = UserAction::Starting;

        let aperture = Angle(projection.aperture_start());

        let w2m = Matrix4::identity();
        let m2w = w2m;
        let center = Vector4::new(0.0, 0.0, 1.0, 1.0);

        let moved = false;

        let w2m_rot = Rotation::zero();
        let final_rot = Rotation::zero();

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

        //let dpi = 1.0;
        //gl.scissor(0, 0, width as i32, height as i32);

        let aspect = height / width;
        let ndc_to_clip = Vector2::new(1.0, (height as f64) / (width as f64));
        let clip_zoom_factor = 1.0;

        let vertices = FieldOfViewVertices::new(&ndc_to_clip, clip_zoom_factor, &w2m, &center, projection);
        let gl = gl.clone();

        let is_allsky = true;
        let time_last_move = Time::now();
        let rotation_center_angle = Angle(0.0);
        let reversed_longitude = false;

        let tile_depth = 0;

        let camera = CameraViewPort {
            // The field of view angle
            aperture,
            center,
            // The rotation of the cameraq
            w2m_rot,
            w2m,
            m2w,

            dpi,
            final_rot,
            rotation_center_angle,
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
            // The vertices in model space of the camera
            // This is useful for computing views according
            // to different image surveys
            vertices,
            // A flag telling whether the camera has been moved during the frame
            moved,

            // Tag the last action done by the user
            last_user_action,
            // Time when the camera has moved
            // for the last time
            time_last_move,

            tile_depth,

            // A reference to the WebGL2 context
            gl,
            // coo system
            system,
            // a flag telling if the viewport has a reversed longitude axis
            reversed_longitude,
        };
        camera.set_canvas_size();

        camera
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
        self.gl.scissor((tl_s.x as i32).max(0), (tl_s.y as i32).max(0), w as i32, h as i32);
    }

    fn set_canvas_size(&self) {
        let canvas = self.gl
            .canvas()
            .unwrap_abort()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_abort();

        canvas.set_width(self.width as u32);
        canvas.set_height(self.height as u32);
        // Once the canvas size is changed, we have to set the viewport as well
        self.gl.viewport(0, 0, self.width as i32, self.height as i32);
    }

    pub fn contains_pole(&self) -> bool {
        self.vertices.contains_pole()
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32, projection: &ProjectionType) {
        let canvas = self
            .gl
            .canvas()
            .unwrap_abort()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_abort();

        self.width = (width as f32) * self.dpi;
        self.height = (height as f32) * self.dpi;

        canvas
            .style()
            .set_property("width", &format!("{}px", width))
            .unwrap_abort();
        canvas
            .style()
            .set_property("height", &format!("{}px", height))
            .unwrap_abort();

        self.aspect = width / height;

        // Compute the new clip zoom factor
        self.compute_ndc_to_clip_factor(projection);

        //self.moved = true;
        //self.last_user_action = UserAction::Starting;

        self.vertices.set_fov(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            &self.center,
            projection,
        );
        let proj_area = projection.get_area();
        self.is_allsky = !proj_area.is_in(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));
        // Update the size of the canvas
        self.set_canvas_size();
        // Once it is done, recompute the scissor
        self.recompute_scissor();
    }

    pub fn compute_ndc_to_clip_factor(&mut self, proj: &ProjectionType) {
        self.ndc_to_clip = if self.height < self.width {
            Vector2::new(
                1.0,
                (self.height as f64) / (self.width as f64)
            )
        } else {
            Vector2::new(
                (self.width as f64) / (self.height as f64),
                1.0,
            )
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

        let can_unzoom_more = match proj {
            ProjectionType::Tan(_) | ProjectionType::Mer(_) | ProjectionType::Air(_) | ProjectionType::Stg(_) | ProjectionType::Car(_) | ProjectionType::Cea(_) | ProjectionType::Cyp(_) | ProjectionType::Hpx(_) => false,
            _ => true
        };

        let aperture_start: Angle<f64> = ArcDeg(proj.aperture_start()).into();

        self.aperture = if aperture <= aperture_start {
            // Compute the new clip zoom factor
            let lon = aperture.abs();

            // Vertex in the WCS of the FOV
            let v0 = math::lonlat::radec_to_xyzw(-lon / 2.0, Angle(0.0));
            let v1 = math::lonlat::radec_to_xyzw(lon / 2.0, Angle(0.0));

            self.clip_zoom_factor = if let Some(p0) = proj.world_to_clip_space(&v0) {
                if let Some(p1) = proj.world_to_clip_space(&v1) {
                    (0.5*(p1.x - p0.x).abs()).min(1.0)
                } else {
                    (aperture / aperture_start).0
                }
            } else {
                (aperture / aperture_start).0
            };

            aperture
        } else if can_unzoom_more {
            self.clip_zoom_factor = aperture.0 / aperture_start.0;
            aperture
        } else {
            self.clip_zoom_factor = 1.0;
            aperture_start
        };

        // Project this vertex into the screen
        self.moved = true;

        self.vertices.set_fov(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            &self.center,
            proj
        );
        let proj_area = proj.get_area();
        self.is_allsky = !proj_area.is_in(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));

        self.compute_tile_depth();

        // recompute the scissor with the new aperture
        self.recompute_scissor();
    }

    fn compute_tile_depth(&mut self) {
        // Compute a depth from a number of pixels on screen
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
        self.tile_depth = if DEPTH_OFFSET_TEXTURE > depth_pixel {
            0_u8
        } else {
            (depth_pixel - DEPTH_OFFSET_TEXTURE) as u8
        };
    }

    pub fn get_tile_depth(&self) -> u8 {
        self.tile_depth
    }

    pub fn rotate(&mut self, axis: &cgmath::Vector3<f64>, angle: Angle<f64>, projection: &ProjectionType) {
        // Rotate the axis:
        let drot = Rotation::from_axis_angle(axis, angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices(projection);
    }

    pub fn set_center(&mut self, lonlat: &LonLatT<f64>, system: &CooSystem, projection: &ProjectionType) {
        let icrs_pos: Vector4<_> = lonlat.vector();

        let view_pos = coosys::apply_coo_system(
            system,
            self.get_system(),
            &icrs_pos,
        );
        let rot = Rotation::from_sky_position(&view_pos);

        // Apply the rotation to the camera to go
        // to the next lonlat
        self.set_rotation(&rot, projection);
    }

    fn set_rotation(&mut self, rot: &Rotation<f64>, projection: &ProjectionType) {
        self.w2m_rot = *rot;

        self.update_rot_matrices(projection);
    }

    pub fn get_field_of_view(&self) -> &FieldOfViewType {
        self.vertices._type()
    }

    /*pub fn get_coverage(&mut self, hips_frame: &CooSystem) -> &HEALPixCoverage {
        self.vertices.get_coverage(&self.system, hips_frame, &self.center)
    }*/

    pub fn set_coo_system(&mut self, new_system: CooSystem, projection: &ProjectionType) {
        // Compute the center position according to the new coordinate frame system
        let new_center = coosys::apply_coo_system(&self.system, &new_system, &self.center);
        // Create a rotation object from that position
        let new_rotation = Rotation::from_sky_position(&new_center);
        // Apply it to the center of the view
        self.set_rotation(&new_rotation, projection);

        // Record the new system
        self.system = new_system;
    }

    pub fn set_longitude_reversed(&mut self, reversed_longitude: bool, projection: &ProjectionType) {
        if self.reversed_longitude != reversed_longitude {
            self.rotation_center_angle = -self.rotation_center_angle;
            self.update_rot_matrices(projection);
        }
        self.reversed_longitude = reversed_longitude;

        // The camera is reversed => it has moved
        self.moved = true;
    }

    pub fn get_longitude_reversed(&self) -> bool {
        self.reversed_longitude
    }

    // Accessors
    pub fn get_rotation(&self) -> &Rotation<f64> {
        &self.w2m_rot
    }

    // This rotation is the final rotation, i.e. a composite of
    // two rotations:
    // - The current rotation of the sphere
    // - The rotation around the center axis of a specific angle
    pub fn get_final_rotation(&self) -> &Rotation<f64> {
        &self.final_rot
    }

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

    pub fn get_vertices(&self) -> Option<&Vec<XYZWModel>> {
        self.vertices.get_vertices()
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

    // Reset moving flag
    pub fn reset(&mut self) {
        self.moved = false;
    }

    pub fn get_aperture(&self) -> Angle<f64> {
        self.aperture
    }

    pub fn get_center(&self) -> &Vector4<f64> {
        &self.center
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        self.vertices.get_bounding_box()
    }

    pub fn is_allsky(&self) -> bool {
        self.is_allsky
    }

    pub fn get_time_of_last_move(&self) -> Time {
        self.time_last_move
    }

    pub fn get_system(&self) -> &CooSystem {
        &self.system
    }

    pub fn set_rotation_around_center(&mut self, theta: Angle<f64>, projection: &ProjectionType) {
        self.rotation_center_angle = theta;
        self.update_rot_matrices(projection);
    }

    pub fn get_rotation_around_center(&self) -> &Angle<f64> {
        &self.rotation_center_angle
    }
}
use cgmath::Matrix;
use crate::ProjectionType;
//use crate::coo_conversion::CooBaseFloat;
impl CameraViewPort {
    // private methods
    fn update_rot_matrices(&mut self, projection: &ProjectionType) {
        self.w2m = (&(self.w2m_rot)).into();
        self.m2w = self.w2m.transpose();

        // Update the center with the new rotation
        self.update_center(projection);

        // Rotate the fov vertices
        self.vertices
            .set_rotation(&self.w2m, &self.center);

        self.time_last_move = Time::now();
        self.last_user_action = UserAction::Moving;
        self.moved = true;
    }

    fn update_center(&mut self, projection: &ProjectionType) {
        // update the center position
        let center_world_space = projection.clip_to_world_space(&Vector2::new(0.0, 0.0)).unwrap_abort();
        // Change from galactic to icrs if necessary

        // Change to model space
        self.center = self.w2m * center_world_space;

        let axis = &self.center.truncate();
        let center_rot = Rotation::from_axis_angle(axis, self.rotation_center_angle);

        // Re-update the model matrix to take into account the rotation
        // by theta around the center axis
        self.final_rot = center_rot * self.w2m_rot;
        self.w2m = (&self.final_rot).into();
        self.m2w = self.w2m.transpose();
    }
}

use al_core::shader::{SendUniforms, ShaderBound};
impl SendUniforms for CameraViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            //.attach_uniforms_from(&self.last_user_action)
            //.attach_uniform("to_icrs", &self.system.to_icrs_j2000::<f32>())
            //.attach_uniform("to_galactic", &self.system.to_gal::<f32>())
            //.attach_uniform("model", &self.w2m)
            //.attach_uniform("inv_model", &self.m2w)
            .attach_uniform("ndc_to_clip", &self.ndc_to_clip) // Send ndc to clip
            .attach_uniform("czf", &self.clip_zoom_factor) // Send clip zoom factor
            .attach_uniform("window_size", &self.get_screen_size()) // Window size
            .attach_uniform("fov", &self.aperture);

        shader
    }
}
