#[derive(PartialEq, Clone, Copy)]
pub enum UserAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

use crate::field_of_view::FieldOfView;
use cgmath::{Vector2, Matrix4};
pub struct CameraViewPort {
    // The field of view angle
    aperture: Angle<f32>,
    // The rotation of the camera
    w2m_rot: SphericalRotation<f32>,
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

    // A flag telling whether the camera has been moved during the frame
    moved: bool,

    // Tag the last action done by the user
    last_user_action: UserAction,

    // A reference to the WebGL2 context
    gl: WebGl2Context,
}

use crate::WebGl2Context;

use crate::{
    renderable::{
        projection::Projection,
        Angle,
    },
    rotation::SphericalRotation,
    sphere_geometry::GreatCirclesInFieldOfView,
};
use std::collections::{HashSet, HashMap};
use cgmath::{Matrix3, Matrix4, Vector4, SquareMatrix};

fn set_canvas_size(gl: &WebGl2Context, width: u32, height: u32) {
    let canvas = gl.canvas().unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();

    canvas.set_width(width);
    canvas.set_height(height);
    gl.viewport(0, 0, width as i32, height as i32);
    gl.scissor(0, 0, width as i32, height as i32);
}

impl CameraViewPort {
    pub fn new<P: Projection>(gl: &WebGl2Context) -> CameraViewPort {
        let last_user_action = UserAction::Starting;

        //let fov = FieldOfView::new::<P>(gl, P::aperture_start(), config);
        let aperture = P::aperture_start();

        let w2m = Matrix4::identity();
        let m2w = w2m;

        let changed = false;

        let w2m_rot = SphericalRotation::zero();

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
        set_canvas_size(width as u32, height as u32);

        let aspect = width / height;
        let ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);
        let clip_zoom_factor = 0_f32;

        let gl = gl.clone();
        CameraViewPort {
            // The field of view angle
            aperture,
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

            // A flag telling whether the camera has been moved during the frame
            changed,

            // Tag the last action done by the user
            last_user_action,

            // A reference to the WebGL2 context
            gl,
        }
    }
    /*pub fn resize_window<P: Projection>(&mut self,
        width: f32,
        height: f32,
        sphere: &mut HiPSSphere,
        manager: &mut catalog::Manager,
    ) {
        self.fov.resize_window::<P>(width, height, sphere.config());

        // Launch the new tile requests
        sphere.ask_for_tiles::<P>(&self.new_healpix_cells());
        manager.set_kernel_size(&self);

        self.viewport_updated = true;
    }*/

    /*pub fn depth(&self) -> u8 {
        self.fov.current_depth()
    }
    pub fn depth_precise(&self, config: &HiPSConfig) -> f32 {
        let max_depth = config.max_depth() as f32;
        
        let depth = max_depth.min(
            crate::math::fov_to_depth_precise(
                self.fov.get_aperture(),
                self.fov.get_width_screen(),
                config
            )
        );

        depth
    }
*/
    /*pub fn update<P: Projection>(&mut self, manager: &mut catalog::Manager, config: &HiPSConfig) {
        // Each time the viewport is updated
        // we update the manager
        if self.moved {
            manager.update::<P>(&self, config);

        }
        self.moved = false;
    }*/
/*
    // Tell the viewport the HiPS have changed
    pub fn set_image_survey<P: Projection>(&mut self, config: &HiPSConfig) {
        self.fov.set_image_survey::<P>(config);

        self.user_action = LastAction::Starting;
    }
*/
/*    pub fn reset_zoom_level<P: Projection>(&mut self, config: &HiPSConfig) {
        // Update the aperture of the Field Of View
        let aperture: Angle<f32> = P::aperture_start();
        self.fov.set_aperture::<P>(aperture, config);
    }
*/

    pub fn set_screen_size<P: Projection>(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.aspect = width / height;

        set_canvas_size(width as u32, height as u32);

        // Compute the new clip zoom factor
        self.ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);

        self.changed = true;
    }

    pub fn set_aperture<P: Projection>(&mut self, aperture: Angle<f32>) {
        // Checking if we are zooming or unzooming
        // This is used internaly for the raytracer to compute
        // blending between tiles and their parents (or children)
        self.last_user_action = if self.get_aperture() > aperture {
            LastAction::Zooming
        } else {
            LastAction::Unzooming
        };

        self.aperture = if aperture <= P::aperture_start() {
            aperture
        } else {
            // The start aperture of the new projection is < to the current aperture
            // We reset the wheel idx too
            P::aperture_start()
        };

        // Compute the new clip zoom factor
        let lon = aperture.abs() / 2_f32;

        // Vertex in the WCS of the FOV
        let v0 = math::radec_to_xyzw(lon, Angle(0_f32));

        // Project this vertex into the screen
        let p0 = P::world_to_clip_space(&v0);
        self.clip_zoom_factor = p0.x.abs();

        self.changed = true;
    }

    pub fn rotate<P: Projection>(&mut self, axis: &cgmath::Vector3<f32>, angle: Angle<f32>) {
        let drot = SphericalRotation::from_axis_angle(axis, angle);
        self.w2m_rot = drot * self.w2m_rot;

        self.update_rot_matrices::<P>();
    }

    pub fn set_rotation<P: Projection>(&mut self, rot: &SphericalRotation<f32>) {
        self.w2m_rot = *rot;

        self.update_rot_matrices::<P>();
    }

    // Accessors
    pub fn get_rotation(&self) -> &SphericalRotation<f32> {
        &self.sr
    }

    pub fn get_w2m(&self) -> &cgmath::Matrix4<f32> {
        &self.model_mat
    }

    pub fn get_m2w(&self) -> &cgmath::Matrix4<f32> {
        &self.inverted_model_mat
    }

    pub fn get_ndc_to_clip(&self) -> &Vector2<f32> {
        self.ndc_to_clip
    }

    pub fn get_clip_zoom_factor(&self) -> f32 {
        self.clip_zoom_factor
    }

    pub fn get_screen_size(&self) -> Vector2<f32> {
        Vector2::new(self.width, self.height)
    }

    pub fn last_user_action(&self) -> UserAction {
        self.user_action
    }

    pub fn has_camera_moved(&mut self) -> bool {
        let res = self.moved;
        self.moved = false;

        res
    }

    pub fn center_model_pos<P: Projection>(&self) -> Vector4<f32> {
        P::clip_to_model_space(
            &Vector2::new(0_f32, 0_f32),
            self
        ).unwrap()
    }

    pub fn get_aperture(&self) -> Angle<f32> {
        self.aperture
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

impl CameraViewPort {
    // private methods
    fn update_rot_matrices<P: Projection>(&mut self) {
        self.w2m = (&self.w2m_rot).into();
        self.m2w = self.w2m.invert().unwrap();

        self.last_user_action = UserAction::Moving;

        self.changed = true;
    }
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;

impl HasUniforms for CameraViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("ndc_to_clip", self.ndc_to_clip) // Send ndc to clip
            .attach_uniform("clip_zoom_factor", &self.clip_zoom_factor) // Send clip zoom factor
            .attach_uniform("user_action", &(self.user_action as i32)) // Send last zoom action
            .attach_uniform("window_size", &self.get_screen_size()) // Window size
            .attach_uniform("fov", &self.aperture);

        shader
    }
}