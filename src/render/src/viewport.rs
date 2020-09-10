#[derive(PartialEq, Clone, Copy)]
pub enum LastAction {
    Zooming = 1,
    Unzooming = 2,
    Moving = 3,
    Starting = 4,
}

use crate::field_of_view::FieldOfView;
use cgmath::Vector2;
pub struct ViewPort {
    fov: FieldOfView,
    
    // Tag the last action done by the user
    pub user_action: LastAction,

    sr: SphericalRotation<f32>,
    model_mat: cgmath::Matrix4::<f32>,
    inverted_model_mat: cgmath::Matrix4<f32>,

    viewport_updated: bool,
}

use crate::WebGl2Context;

use crate::{
    renderable::{HiPSSphere,
        catalog,
        projection::Projection,
        Angle,
    },
    buffer::HiPSConfig,
    rotation::SphericalRotation,
    sphere_geometry::GreatCirclesInFieldOfView,
    healpix_cell::HEALPixCell
};
use std::collections::{HashSet, HashMap};
use cgmath::{Matrix3, Matrix4, Vector4, SquareMatrix};

impl ViewPort {
    pub fn new<P: Projection>(gl: &WebGl2Context, config: &HiPSConfig) -> ViewPort {
        let user_action = LastAction::Starting;

        let fov = FieldOfView::new::<P>(gl, P::aperture_start(), config);

        let model_mat = Matrix4::identity();
        let inverted_model_mat = model_mat;

        let viewport_updated = false;

        let sr = SphericalRotation::zero();

        let viewport = ViewPort {
            fov,

            user_action,

            sr,
            model_mat,
            inverted_model_mat,
            viewport_updated,
        };

        viewport
    }

    // Tell the viewport the HiPS have changed
    pub fn set_image_survey<P: Projection>(&mut self, config: &HiPSConfig) {
        self.fov.set_image_survey::<P>(config);

        self.user_action = LastAction::Starting;
    }

    pub fn reset_zoom_level<P: Projection>(&mut self, config: &HiPSConfig) {
        // Update the aperture of the Field Of View
        let aperture: Angle<f32> = P::aperture_start();
        self.fov.set_aperture::<P>(aperture, config);
    }

    pub fn get_aperture(&self) -> Angle<f32> {
        self.fov.get_aperture()
    }

    pub fn set_aperture<P: Projection>(&mut self, aperture: Angle<f32>, config: &HiPSConfig) {
        // Checking if we are zooming or unzooming
        // This is used internaly for the raytracer to compute
        // blending between tiles and their parents (or children)
        if self.get_aperture() > aperture {
            self.user_action = LastAction::Zooming;
        } else {
            self.user_action = LastAction::Unzooming;
        }

        let aperture = if aperture <= P::aperture_start() {
            aperture
        } else {
            // The start aperture of the new projection is < to the current aperture
            // We reset the wheel idx too
            P::aperture_start()
        };
        // Recompute the depth and field of view
        self.fov.set_aperture::<P>(aperture, config);

        self.viewport_updated = true;
    }

    // Called when the projection changes
    pub fn reset<P: Projection>(&mut self, config: &HiPSConfig) {
        let current_aperture = self.fov.get_aperture();
        self.set_aperture::<P>(current_aperture, config);

        let size = self.get_window_size();
        self.fov.resize_window::<P>(size.x, size.y, config);
    }

    pub fn resize_window<P: Projection>(&mut self,
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
    }

    pub fn depth(&self) -> u8 {
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
    pub fn cells(&self) -> &HashSet<HEALPixCell> {
        &self.fov.healpix_cells()
    }
    pub fn new_healpix_cells(&self) -> &HashMap<HEALPixCell, bool> {
        &self.fov.new_healpix_cells()
    }
    pub fn get_cells_in_fov<P: Projection>(&self, depth: u8) -> HashSet<HEALPixCell> {
        self.fov.get_cells_in_fov::<P>(depth)
    }

    pub fn last_user_action(&self) -> LastAction {
        self.user_action
    }

    pub fn is_viewport_updated(&self) -> bool {
        self.viewport_updated
    }

    pub fn update<P: Projection>(&mut self, manager: &mut catalog::Manager, config: &HiPSConfig) {
        // Each time the viewport is updated
        // we update the manager
        if self.viewport_updated {
            manager.update::<P>(&self, config);

        }
        self.viewport_updated = false;
    }

    pub fn get_ndc_to_clip(&self) -> &Vector2<f32> {
        self.fov.get_ndc_to_clip()
    }

    pub fn get_clip_zoom_factor(&self) -> f32 {
        self.fov.get_clip_zoom_factor()
    }

    // Viewport model matrices
    pub fn get_window_size(&self) -> Vector2<f32> {
        let (width, height) = self.fov.get_size_screen();
        Vector2::new(width, height)
    }

    pub fn compute_center_model_pos<P: Projection>(&self) -> Vector4<f32> {
        P::clip_to_model_space(
            &Vector2::new(0_f32, 0_f32),
            self
        ).unwrap()
    }

    // Check whether border of the screen are inside
    // the projection
    pub fn screen_inside_of_projection<P: Projection>(&self) -> bool {
        // Projection are symmetric, we can check for only one vertex
        // of the screen
        let corner_tl_ndc = Vector2::new(-1_f32, 1_f32);
        let corner_tl_clip = crate::renderable::projection::ndc_to_clip_space(&corner_tl_ndc, self);

        P::clip_to_world_space(&corner_tl_clip).is_some()
    }

    pub fn apply_rotation<P: Projection>(&mut self, axis: &cgmath::Vector3<f32>, angle: Angle<f32>, config: &HiPSConfig) {
        let dq = SphericalRotation::from_axis_angle(axis, angle);
        self.sr = dq * self.sr;

        self.compute_model_mat::<P>(config);
    }

    pub fn set_rotation<P: Projection>(&mut self, rot: &SphericalRotation<f32>, config: &HiPSConfig) {
        self.sr = *rot;

        self.compute_model_mat::<P>(config);
    }

    fn compute_model_mat<P: Projection>(&mut self, config: &HiPSConfig) {
        self.model_mat = (&self.sr).into();
        self.inverted_model_mat = self.model_mat.invert().unwrap();

        // Translate the field of view in consequence
        self.fov.set_rotation_mat::<P>(&self.model_mat, config);
        self.user_action = LastAction::Moving;

        self.viewport_updated = true;
    }

    pub fn get_rotation(&self) -> &SphericalRotation<f32> {
        &self.sr
    }

    pub fn get_model_mat(&self) -> &cgmath::Matrix4<f32> {
        &self.model_mat 
    }
    pub fn get_quat(&self) -> cgmath::Quaternion<f32> {
        // Extract a 3x3 matrix from the model 4x4 matrix
        let v: [[f32; 4]; 4] = self.model_mat.into();

        let mat3 = Matrix3::new(
            v[0][0], v[0][1], v[0][2],
            v[1][0], v[1][1], v[1][2],
            v[2][0], v[2][1], v[2][2]
        );

        mat3.into()
    }

    pub fn get_inverted_model_mat(&self) -> &cgmath::Matrix4<f32> {
        &self.inverted_model_mat
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
    pub fn get_great_circles_inside(&self) -> &GreatCirclesInFieldOfView {
        self.fov.get_great_circles_intersecting()
    }
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;

impl HasUniforms for ViewPort {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("ndc_to_clip", self.fov.get_ndc_to_clip()) // Send ndc to clip
            .attach_uniform("clip_zoom_factor", &self.fov.get_clip_zoom_factor()) // Send clip zoom factor
            .attach_uniform("user_action", &(self.user_action as i32)) // Send last zoom action
            .attach_uniform("window_size", &self.get_window_size()) // Window size
            .attach_uniform("fov", &self.get_aperture());

        shader
    }
}