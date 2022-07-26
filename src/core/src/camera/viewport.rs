#[derive(PartialEq, Clone, Copy)]
pub enum UserAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

use super::fov::{FieldOfViewVertices, ModelCoord};
use crate::math::spherical::BoundingBox;
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

use crate::math;
use crate::time::Time;
impl CameraViewPort {
    pub fn new<P: Projection>(gl: &WebGlContext, system: CooSystem) -> CameraViewPort {
        let last_user_action = UserAction::Starting;

        //let fov = FieldOfView::new::<P>(gl, P::aperture_start(), config);
        let aperture = P::aperture_start();

        let w2m = Matrix4::identity();
        let m2w = w2m;
        let center = Vector4::new(0.0, 0.0, 1.0, 1.0);

        let moved = false;

        let w2m_rot = Rotation::zero();
        let final_rot = Rotation::zero();

        // Get the initial size of the window
        let window = web_sys::window().unwrap();
        let width = window.inner_width().unwrap().as_f64().unwrap() as f32;
        let height = window.inner_height().unwrap().as_f64().unwrap() as f32;
        let dpi = window.device_pixel_ratio() as f32;
        /*if width < height {
            dpi = 1.0;
        }*/

        let width = width * dpi;
        let height = height * dpi;

        //let dpi = 1.0;
        //gl.scissor(0, 0, width as i32, height as i32);

        let aspect = height / width;
        let ndc_to_clip = P::compute_ndc_to_clip_factor(width as f64, height as f64);
        let clip_zoom_factor = 1.0;

        let vertices = FieldOfViewVertices::new::<P>(&ndc_to_clip, clip_zoom_factor, &w2m, &center);
        let gl = gl.clone();

        let is_allsky = true;
        let time_last_move = Time::now();
        let rotation_center_angle = Angle(0.0);
        let reversed_longitude = false;

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

            // A reference to the WebGL2 context
            gl,
            // coo system
            system,
            // a flag telling if the viewport has a reversed longitude axis
            reversed_longitude,
        };
        camera.set_canvas_size::<P>();

        camera
    }

    fn set_canvas_size<P: Projection>(&self) {
        self.gl.viewport(0, 0, self.width as i32, self.height as i32);
        self.gl.scissor(0, 0, self.width as i32, self.height as i32);

        let canvas = self.gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        canvas.set_width(self.width as u32);
        canvas.set_height(self.height as u32);

        self.gl.clear_color(0.08, 0.08, 0.08, 1.0);
        self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.update_scissor::<P>();
    }

    fn update_scissor<P: Projection>(&self) {
        let (wc, hc) = P::clip_size();

        let tl_c = Vector2::new(-wc * 0.5, hc * 0.5);
        let tr_c = Vector2::new(wc * 0.5, hc * 0.5);
        let br_c = Vector2::new(wc * 0.5, -hc * 0.5);
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
        self.gl.scissor((tl_s.x as i32).max(0), (tl_s.y as i32).max(0), w as i32, h as i32);
    }

    pub fn set_screen_size<P: Projection>(&mut self, width: f32, height: f32) {
        let canvas = self
            .gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas
            .style()
            .set_property("width", &format!("{}px", width))
            .unwrap();
        canvas
            .style()
            .set_property("height", &format!("{}px", height))
            .unwrap();

        self.width = (width as f32) * self.dpi;
        self.height = (height as f32) * self.dpi;

        self.aspect = width / height;

        // Compute the new clip zoom factor
        self.ndc_to_clip = P::compute_ndc_to_clip_factor(self.width as f64, self.height as f64);

        self.moved = true;
        self.last_user_action = UserAction::Starting;

        self.vertices.set_fov::<P>(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            &self.center,
        );
        self.is_allsky = !P::is_included_inside_projection(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));

        self.set_canvas_size::<P>();
    }

    pub fn set_aperture<P: Projection>(&mut self, aperture: Angle<f64>) {
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

        self.aperture = if aperture <= P::aperture_start() {
            // Compute the new clip zoom factor
            let lon = aperture.abs() / 2.0;

            // Vertex in the WCS of the FOV
            let v0 = math::lonlat::radec_to_xyzw(lon, Angle(0.0));
            if let Some(p0) = P::world_to_clip_space(&v0) {
                self.clip_zoom_factor = p0.x.abs().min(1.0);
            } else {
                // Gnomonic unzoomed case!
                self.clip_zoom_factor = aperture.0 / P::aperture_start().0;
            }
            aperture
        } else {
            if !P::ALLOW_UNZOOM_MORE {
                // Some projections have a limit of unzooming
                // like Mercator
                self.set_aperture::<P>(P::aperture_start());
                return;
            }

            self.clip_zoom_factor = aperture.0 / P::aperture_start().0;

            aperture
        };
        // Project this vertex into the screen

        self.moved = true;

        self.vertices.set_fov::<P>(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            &self.center,
        );
        self.is_allsky = !P::is_included_inside_projection(&math::projection::ndc_to_clip_space(
            &Vector2::new(-1.0, -1.0),
            self,
        ));

        self.gl.clear_color(0.08, 0.08, 0.08, 1.0);
        self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.update_scissor::<P>();
    }

    /*pub fn depth(&self) -> u8 {
        self.vertices.get_depth()
    }*/

    pub fn rotate<P: Projection>(&mut self, axis: &cgmath::Vector3<f64>, angle: Angle<f64>) {
        // Rotate the axis:
        let drot = Rotation::from_axis_angle(axis, angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_center<P: Projection>(&mut self, lonlat: &LonLatT<f64>, system: &CooSystem) {
        let icrsj2000_pos: Vector4<_> = lonlat.vector();

        let view_pos = coosys::apply_coo_system(
            system,
            self.get_system(),
            &icrsj2000_pos,
        );
        let rot = Rotation::from_sky_position(&view_pos);

        // Apply the rotation to the camera to go
        // to the next lonlat
        self.set_rotation::<P>(&rot);
    }

    fn set_rotation<P: Projection>(&mut self, rot: &Rotation<f64>) {
        self.w2m_rot = *rot;

        self.update_rot_matrices::<P>();
    }

    pub fn get_field_of_view(&self) -> &FieldOfViewType {
        self.vertices._type()
    }

    /*pub fn get_coverage(&mut self, hips_frame: &CooSystem) -> &HEALPixCoverage {
        self.vertices.get_coverage(&self.system, hips_frame, &self.center)
    }*/

    pub fn set_coo_system<P: Projection>(&mut self, new_system: CooSystem) {
        // Compute the center position according to the new coordinate frame system
        let new_center = coosys::apply_coo_system(&self.system, &new_system, &self.center);
        // Create a rotation object from that position
        let new_rotation = Rotation::from_sky_position(&new_center);
        // Apply it to the center of the view
        self.set_rotation::<P>(&new_rotation);

        // Record the new system
        self.system = new_system;
    }

    pub fn set_longitude_reversed(&mut self, reversed_longitude: bool) {
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

    pub fn get_vertices(&self) -> Option<&Vec<ModelCoord>> {
        self.vertices.get_vertices()
    }

    pub fn get_screen_size(&self) -> Vector2<f32> {
        Vector2::new(self.width, self.height)
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

    pub fn set_rotation_around_center<P: Projection>(&mut self, theta: Angle<f64>) {
        self.rotation_center_angle = theta;
        self.update_rot_matrices::<P>();
    }

    pub fn get_rotation_around_center(&self) -> &Angle<f64> {
        &self.rotation_center_angle
    }
}
use cgmath::Matrix;
//use crate::coo_conversion::CooBaseFloat;
impl CameraViewPort {
    // private methods
    fn update_rot_matrices<P: Projection>(&mut self) {
        self.w2m = (&(self.w2m_rot)).into();
        self.m2w = self.w2m.transpose();

        // Update the center with the new rotation
        self.update_center::<P>();

        // Rotate the fov vertices
        self.vertices
            .set_rotation::<P>(&self.w2m, &self.center);

        self.time_last_move = Time::now();
        self.last_user_action = UserAction::Moving;
        self.moved = true;
    }

    fn update_center<P: Projection>(&mut self) {
        // update the center position
        let center_world_space = P::clip_to_world_space(&Vector2::new(0.0, 0.0)).unwrap();
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
