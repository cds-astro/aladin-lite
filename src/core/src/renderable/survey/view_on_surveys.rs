use crate::healpix_cell::HEALPixCell;
use std::collections::{HashMap};

// Compute a depth from a number of pixels on screen
pub fn depth_from_pixels_on_screen(camera: &CameraViewPort, num_pixels: i32) -> u8 {
    let width = camera.get_screen_size().x;
    let aperture = camera.get_aperture().0 as f32;

    let angle_per_pixel = aperture / width;

    let two_power_two_times_depth_pixel =
        std::f32::consts::PI / (3.0 * angle_per_pixel * angle_per_pixel);
    let depth_pixel = (two_power_two_times_depth_pixel.log2() / 2.0).round() as u32;

    //let survey_max_depth = conf.get_max_depth();
    // The depth of the texture
    // A texture of 512x512 pixels will have a depth of 9
    let depth_offset_texture = crate::math::log_2_unchecked(num_pixels);
    // The depth of the texture corresponds to the depth of a pixel
    // minus the offset depth of the texture
    if depth_offset_texture > depth_pixel {
        0_u8
    } else {
        (depth_pixel - depth_offset_texture) as u8
    }
}
use al_api::coo_system::CooSystem;
use crate::cdshealpix;
pub fn get_cells_in_camera(depth: u8, camera: &CameraViewPort) -> Vec<HEALPixCell> {
    if let Some(vertices) = camera.get_vertices() {
        // The vertices coming from the camera are in a specific coo sys
        // but cdshealpix accepts them to be given in ICRSJ2000 coo sys
        let view_system = camera.get_system();
        let icrsj2000_fov_vertices_pos = vertices.iter()
            .map(|v| {
                crate::math::apply_coo_system(view_system, &CooSystem::ICRSJ2000, v)
            })
            .collect::<Vec<_>>();

        let vs_inside_pos = camera.get_center();
        let icrsj2000_inside_pos = crate::math::apply_coo_system(view_system, &CooSystem::ICRSJ2000, &vs_inside_pos);
        // Prefer to query from_polygon with depth >= 2
        let coverage = cdshealpix::from_polygon(depth, &icrsj2000_fov_vertices_pos[..], &icrsj2000_inside_pos.truncate());

        coverage
            .flat_iter()
            .map(|idx| HEALPixCell(depth, idx))
            .collect()
    } else {
        HEALPixCell::allsky(depth).collect()
    }
}

// Contains the cells being in the FOV for a specific
pub struct HEALPixCellsInView {
    // The set of cells being in the current view for a
    // specific image survey
    pub depth: u8,
    prev_depth: u8,
    look_for_parents: bool,

    // flags associating true to cells that
    // are new in the fov
    cells: HashMap<HEALPixCell, bool>,
    // A flag telling whether there has been
    // new cells added from the last frame
    is_new_cells_added: bool,
}

use crate::camera::{CameraViewPort, UserAction};
impl HEALPixCellsInView {
    pub fn new(_survey_tex_size: i32, _max_depth: u8, _camera: &CameraViewPort) -> Self {
        let cells = HashMap::new();
        Self {
            cells,
            prev_depth: 0,
            depth: 0,
            look_for_parents: false,
            is_new_cells_added: false,
        }
    }

    pub fn reset_frame(&mut self) {
        self.is_new_cells_added = false;
        self.prev_depth = self.get_depth();
    }

    // This method is called whenever the user does an action
    // that moves the camera.
    // Everytime the user moves or zoom, the views must be updated
    // The new cells obtained are used for sending new requests
    pub fn refresh_cells(&mut self, texture_size: i32, max_depth: u8, camera: &CameraViewPort) {
        // Compute that depth
        let new_depth = depth_from_pixels_on_screen(camera, texture_size);

        self.depth = new_depth.min(max_depth);
        // Get the cells of that depth in the current field of view
        let cells = get_cells_in_camera(self.depth, camera);
        // Update cells in the fov
        self.update_cells_in_fov(&cells, camera);
    }

    fn update_cells_in_fov(&mut self, cells_in_fov: &[HEALPixCell], camera: &CameraViewPort) {
        let new_cells = cells_in_fov
            .iter()
            .map(|cell| {
                let new = !self.cells.contains_key(cell);
                self.is_new_cells_added |= new;

                (*cell, new)
            })
            .collect::<HashMap<_, _>>();
        self.cells = new_cells;

        if camera.get_last_user_action() == UserAction::Unzooming {
            self.look_for_parents = self.has_depth_decreased();
        } else {
            self.look_for_parents = false;
        }
    }

    // Accessors
    #[inline]
    pub fn get_cells(&self) -> impl Iterator<Item=&HEALPixCell> {
        self.cells.keys()
    }

    #[inline]
    pub fn num_of_cells(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    #[inline]
    pub fn is_new(&self, cell: &HEALPixCell) -> bool {
        if let Some(&is_cell_new) = self.cells.get(cell) {
            is_cell_new
        } else {
            false
        }
    }

    #[inline]
    pub fn is_there_new_cells_added(&self) -> bool {
        //self.new_cells.is_there_new_cells_added()
        self.is_new_cells_added
    }

    #[inline]
    pub fn has_depth_decreased_while_unzooming(&self, camera: &CameraViewPort) -> bool {
        assert!(camera.get_last_user_action() == UserAction::Unzooming);
        self.look_for_parents
    }

    #[inline]
    fn has_depth_decreased(&self) -> bool {
        let depth = self.get_depth();
        depth < self.prev_depth
    }
}
