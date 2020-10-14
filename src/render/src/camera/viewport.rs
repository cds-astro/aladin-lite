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


use super::fov_vertices::{
    FieldOfViewVertices,
    ModelCoord
};
use cgmath::{Vector2, Vector3, Matrix4};

const J2000_TO_GALACTIC: Matrix4<f32> = Matrix4::new(
    -0.8676661489811610,
    -0.0548755604024359,
    0.4941094279435681,
    0.0,

    -0.1980763734646737,
    -0.873437090247923,
    -0.4448296299195045,
    0.0,

    0.4559837762325372,
    -0.4838350155267381,
    0.7469822444763707,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);
pub struct CameraViewPort {
    // The field of view angle
    aperture: Angle<f32>,
    center: Vector4<f32>,
    // The rotation of the camera
    w2m_rot: Rotation<f32>,
    w2m: Matrix4<f32>,
    m2w: Matrix4<f32>,
    // The width over height ratio
    aspect: f32,
    // The width of the screen in pixels
    width: f32,
    // The height of the screen in pixels
    height: f32,

    // Internal variable used for projection purposes
    ndc_to_clip: Vector2<f32>,
    clip_zoom_factor: f32,
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

    // A reference to the WebGL2 context
    gl: WebGl2Context,
}

use crate::WebGl2Context;

use crate::{
    renderable::{
        projection::Projection,
        Angle,
    },
    rotation::Rotation,
    sphere_geometry::GreatCirclesInFieldOfView,
};
use std::collections::{HashSet, HashMap};
use cgmath::{Matrix3, Vector4, SquareMatrix};
use wasm_bindgen::JsCast;
fn set_canvas_size(gl: &WebGl2Context, width: u32, height: u32) {
    let canvas = gl.canvas().unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    canvas.set_width(width);
    canvas.set_height(height);
    gl.viewport(0, 0, width as i32, height as i32);
    gl.scissor(0, 0, width as i32, height as i32);
}

use crate::math;
use crate::renderable::angle::ArcDeg;

impl CameraViewPort {
    pub fn new<P: Projection>(gl: &WebGl2Context) -> CameraViewPort {
        let last_user_action = UserAction::Starting;

        //let fov = FieldOfView::new::<P>(gl, P::aperture_start(), config);
        let aperture = P::aperture_start();

        let w2m = Matrix4::identity();
        let m2w = w2m;
        let center = Vector4::new(0.0, 0.0, 1.0, 1.0);

        let moved = false;

        let w2m_rot = Rotation::zero();

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
        let ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);
        let clip_zoom_factor = 1_f32;

        let longitude_reversed = true;
        let vertices = FieldOfViewVertices::new::<P>(&center, &ndc_to_clip, clip_zoom_factor, &w2m, longitude_reversed);
        let gl = gl.clone();

        let mut camera = CameraViewPort {
            // The field of view angle
            aperture,
            center,
            // The rotation of the camera
            w2m_rot,
            w2m,
            m2w,
            // The width over height ratio
            aspect,
            // The width of the screen in pixels
            width,
            // The height of the screen in pixels
            height,

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

            // A reference to the WebGL2 context
            gl,
        };

        camera
    }

    pub fn set_screen_size<P: Projection>(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.aspect = width / height;

        set_canvas_size(&self.gl, width as u32, height as u32);

        // Compute the new clip zoom factor
        self.ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);

        self.moved = true;
        self.last_user_action = UserAction::Starting;

        self.vertices.set_fov::<P>(&self.ndc_to_clip, self.clip_zoom_factor, &self.w2m, self.longitude_reversed);
    }

    pub fn set_aperture<P: Projection>(&mut self, aperture: Angle<f32>) {
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

        /*self.aperture = if aperture <= P::aperture_start() {
            aperture
        } else {
            // The start aperture of the new projection is < to the current aperture
            // We reset the wheel idx too
            P::aperture_start()
        };*/
        self.aperture = aperture;

        // Compute the new clip zoom factor
        let lon = aperture.abs() / 2_f32;

        // Vertex in the WCS of the FOV
        let v0 = math::radec_to_xyzw(lon, Angle(0_f32));

        // Project this vertex into the screen
        if let Some(p0) = P::world_to_clip_space(&v0, self.longitude_reversed) {
            self.clip_zoom_factor = p0.x.abs().min(1.0);
        } else {
            self.clip_zoom_factor = self.aperture.0 / P::aperture_start().0;
        }

        self.moved = true;

        self.vertices.set_fov::<P>(&self.ndc_to_clip, self.clip_zoom_factor, &self.w2m, self.longitude_reversed);
    }

    pub fn rotate<P: Projection>(&mut self, axis: &cgmath::Vector3<f32>, angle: Angle<f32>) {
        //let j2000_to_gal: Rotation<f32> = (&J2000_TO_GALACTIC).into();

        // Rotate the axis:
        //let axis = (J2000_TO_GALACTIC.invert().unwrap() * Vector4::new(axis.x, axis.y, axis.z, 1.0)).truncate().normalize();
        let drot = Rotation::from_axis_angle(&(axis), angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_rotation<P: Projection>(&mut self, rot: &Rotation<f32>) {
        self.w2m_rot = *rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_projection<P: Projection>(&mut self) {
        // Recompute the ndc_to_clip
        self.set_screen_size::<P>(self.width, self.height);
        // Recompute clip zoom factor
        self.set_aperture::<P>(self.get_aperture());

        //self.last_user_action = UserAction::Starting;

        //self.vertices.set_projection::<P>(&self.ndc_to_clip, self.clip_zoom_factor, &self.w2m, self.longitude_reversed);
    }
    pub fn set_longitude_reversed(&mut self, reversed: bool) {
        self.longitude_reversed = reversed;
    }

    // Accessors
    pub fn get_rotation(&self) -> &Rotation<f32> {
        &self.w2m_rot
    }

    pub fn get_w2m(&self) -> &cgmath::Matrix4<f32> {
        &self.w2m
    }

    pub fn get_m2w(&self) -> &cgmath::Matrix4<f32> {
        &self.m2w
    }


    pub fn get_ndc_to_clip(&self) -> &Vector2<f32> {
        &self.ndc_to_clip
    }

    pub fn get_clip_zoom_factor(&self) -> f32 {
        self.clip_zoom_factor
    }

    pub fn get_vertices(&self) -> Option<&Vec<ModelCoord>> {
        self.vertices.get_vertices()
    }

    /*pub fn get_radius(&self) -> Option<&Angle<f32>> {
        self.vertices.get_radius()
    }
    */
    pub fn get_screen_size(&self) -> Vector2<f32> {
        Vector2::new(self.width, self.height)
    }

    pub fn get_last_user_action(&self) -> UserAction {
        self.last_user_action
    }

    pub fn has_camera_moved(&self) -> bool {
        self.moved
    }

    // Reset moving flag
    pub fn reset(&mut self) {
        self.moved = false;
    }

    /*pub fn center_model_pos<P: Projection>(&self) -> Vector4<f32> {
        P::clip_to_model_space(
            &Vector2::new(0_f32, 0_f32),
            self
        ).unwrap()
    }*/

    pub fn get_aperture(&self) -> Angle<f32> {
        self.aperture
    }

    pub fn get_center(&self) -> &Vector4<f32> {
        &self.center
    }
    pub fn is_reversed_longitude(&self) -> bool {
        self.longitude_reversed
    }

    // Useful methods for the grid purpose
    /*pub fn intersect_meridian<LonT: Into<Rad<f32>>>(&self, lon: LonT) -> bool {
        self.fov.intersect_meridian(lon)
    }

    pub fn intersect_parallel<LatT: Into<Rad<f32>>>(&self, lat: LatT) -> bool {
        self.fov.intersect_parallel(lat)
    }*/

    // Get the grid containing the meridians and parallels
    // that are inside the grid
    // TODO: move FieldOfViewType out of the FieldOfView, make it intern to the grid
    // The only thing to do is to recompute the grid whenever the field of view changes
    /*pub fn get_great_circles_inside(&self) -> &GreatCirclesInFieldOfView {
        self.fov.get_great_circles_intersecting()
    }*/
}
use cgmath::Matrix;
impl CameraViewPort {
    // private methods
    fn update_rot_matrices<P: Projection>(&mut self) {
        self.w2m = (&self.w2m_rot).into();
        //self.w2m = self.w2m * J2000_TO_GALACTIC;
        self.m2w = self.w2m.transpose();

        self.last_user_action = UserAction::Moving;

        self.moved = true;

        self.vertices.set_rotation(&self.w2m);
        self.update_center::<P>();
    }

    fn update_center<P: Projection>(&mut self) {
        // update the center position
        self.center = P::clip_to_model_space(
            &Vector2::new(0_f32, 0_f32),
            self
        ).unwrap();
    }
}
use cgmath::InnerSpace;
use crate::shader::SendUniforms;
use crate::shader::ShaderBound;

impl SendUniforms for CameraViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniforms_from(&self.last_user_action)
            .attach_uniform("model", &self.w2m)
            .attach_uniform("inv_model", &self.m2w)
            .attach_uniform("ndc_to_clip", &self.ndc_to_clip) // Send ndc to clip
            .attach_uniform("clip_zoom_factor", &self.clip_zoom_factor) // Send clip zoom factor
            .attach_uniform("inversed_longitude", &(self.longitude_reversed as i32)) 
            .attach_uniform("window_size", &self.get_screen_size()) // Window size
            .attach_uniform("fov", &self.aperture);

        shader
    }
}