#[derive(PartialEq, Clone, Copy)]
pub enum UserAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

impl SendUniforms for UserAction {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("user_action", &(*self as i32));

        shader
    }
}
use super::fov_vertices::{FieldOfViewVertices, ModelCoord};
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
    longitude_reversed: bool,
    is_allsky: bool,

    // Time when the camera has moved
    time_last_move: Time,

    // A reference to the WebGL2 context
    gl: WebGl2Context,
    system: CooSystem,
}
use crate::coo_conversion::CooSystem;
use crate::WebGl2Context;

use crate::{
    renderable::{projection::Projection, Angle},
    rotation::Rotation,
    sphere_geometry::FieldOfViewType,
};

use cgmath::{SquareMatrix, Vector4};
use wasm_bindgen::JsCast;
fn set_canvas_size(gl: &WebGl2Context, width: u32, height: u32) {
    let canvas = gl
        .canvas()
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    canvas.set_width(width);
    canvas.set_height(height);
    gl.viewport(0, 0, width as i32, height as i32);
    gl.scissor(0, 0, width as i32, height as i32);
}

use crate::math;

use crate::sphere_geometry::BoundingBox;
use crate::time::Time;
impl CameraViewPort {
    pub fn new<P: Projection>(gl: &WebGl2Context, system: CooSystem) -> CameraViewPort {
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
        let width = web_sys::window()
            .unwrap()
            .inner_width()
            .unwrap()
            .as_f64()
            .unwrap() as f32;
        let height = web_sys::window()
            .unwrap()
            .inner_height()
            .unwrap()
            .as_f64()
            .unwrap() as f32;
        set_canvas_size(&gl, width as u32, height as u32);

        let aspect = width / height;
        let ndc_to_clip = P::compute_ndc_to_clip_factor(width as f64, height as f64);
        let clip_zoom_factor = 1.0;

        let longitude_reversed = true;
        let vertices =
            FieldOfViewVertices::new::<P>(&ndc_to_clip, clip_zoom_factor, &w2m, longitude_reversed);
        let gl = gl.clone();

        let is_allsky = true;
        let time_last_move = Time::now();
        let rotation_center_angle = Angle(0.0);

        let camera = CameraViewPort {
            // The field of view angle
            aperture,
            center,
            // The rotation of the camera
            w2m_rot,
            w2m,
            m2w,

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
            // longitude reversed flag
            longitude_reversed,
            // Time when the camera has moved
            // for the last time
            time_last_move,

            // A reference to the WebGL2 context
            gl,
            // coo system
            system,
        };

        camera
    }

    pub fn set_screen_size<P: Projection>(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.aspect = width / height;

        set_canvas_size(&self.gl, width as u32, height as u32);

        // Compute the new clip zoom factor
        self.ndc_to_clip = P::compute_ndc_to_clip_factor(width as f64, height as f64);

        self.moved = true;
        self.last_user_action = UserAction::Starting;

        self.vertices.set_fov::<P>(
            &self.ndc_to_clip,
            self.clip_zoom_factor,
            &self.w2m,
            self.aperture,
            self.longitude_reversed,
            &self.system,
        );
        self.is_allsky = !P::is_included_inside_projection(
            &crate::renderable::projection::ndc_to_clip_space(&Vector2::new(-1.0, -1.0), self),
        );
    }

    /*pub fn set_clip_zoom_factor<P: Projection>(&mut self, aperture) {

    }*/

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
            let v0 = math::radec_to_xyzw(lon, Angle(0.0));
            if let Some(p0) = P::world_to_clip_space(&v0, self.longitude_reversed) {
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
            self.aperture,
            self.longitude_reversed,
            &self.system,
        );
        self.is_allsky = !P::is_included_inside_projection(
            &crate::renderable::projection::ndc_to_clip_space(&Vector2::new(-1.0, -1.0), self),
        );
    }

    pub fn rotate<P: Projection>(&mut self, axis: &cgmath::Vector3<f64>, angle: Angle<f64>) {
        // Rotate the axis:
        let drot = Rotation::from_axis_angle(&(axis), angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_rotation<P: Projection>(&mut self, rot: &Rotation<f64>) {
        self.w2m_rot = *rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_projection<P: Projection>(&mut self) {
        // Recompute the ndc_to_clip
        self.set_screen_size::<P>(self.width, self.height);
        // Recompute clip zoom factor
        self.set_aperture::<P>(self.get_aperture());
    }
    pub fn set_longitude_reversed(&mut self, reversed: bool) {
        self.longitude_reversed = reversed;
    }

    pub fn get_field_of_view(&self) -> &FieldOfViewType {
        self.vertices._type()
    }

    pub fn set_coo_system<P: Projection>(&mut self, system: CooSystem) {
        self.system = system;
        self.vertices
            .set_rotation::<P>(&self.w2m, self.aperture, &self.system);
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
    pub fn is_reversed_longitude(&self) -> bool {
        self.longitude_reversed
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
impl CameraViewPort {
    // private methods
    fn update_rot_matrices<P: Projection>(&mut self) {
        self.w2m = (&(self.w2m_rot)).into();
        self.m2w = self.w2m.transpose();

        // Update the center with the new rotation
        self.update_center::<P>();

        // Rotate the fov vertices
        self.vertices
            .set_rotation::<P>(&self.w2m, self.aperture, &self.system);

        self.time_last_move = Time::now();
        self.last_user_action = UserAction::Moving;
        self.moved = true;
    }

    fn update_center<P: Projection>(&mut self) {
        // update the center position
        let center_world_space =
            P::clip_to_world_space(&Vector2::new(0.0, 0.0), self.is_reversed_longitude()).unwrap();
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

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
impl SendUniforms for CameraViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniforms_from(&self.last_user_action)
            .attach_uniform("to_icrs", &self.system.to_icrs_j2000::<f32>())
            .attach_uniform("model", &self.w2m)
            .attach_uniform("inv_model", &self.m2w)
            .attach_uniform("ndc_to_clip", &self.ndc_to_clip) // Send ndc to clip
            .attach_uniform("czf", &self.clip_zoom_factor) // Send clip zoom factor
            .attach_uniform("inversed_longitude", &(self.longitude_reversed as i32))
            .attach_uniform("window_size", &self.get_screen_size()) // Window size
            .attach_uniform("fov", &self.aperture);

        shader
    }
}
