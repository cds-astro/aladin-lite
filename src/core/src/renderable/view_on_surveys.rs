use crate::healpix_cell::HEALPixCell;
use std::collections::{HashMap, HashSet};

use std::collections::hash_set::Iter;

#[derive(Debug, Clone, std::cmp::PartialEq)]
pub struct HEALPixCells {
    pub depth: u8,
    pub cells: HashSet<HEALPixCell>,
}
pub struct HEALPixCellsIter<'a>(Iter<'a, HEALPixCell>);

impl<'a> Iterator for HEALPixCellsIter<'a> {
    type Item = &'a HEALPixCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[allow(dead_code)]
impl HEALPixCells {
    fn new() -> Self {
        HEALPixCells {
            depth: 0,
            cells: HashSet::new(),
        }
    }

    pub fn allsky(depth: u8) -> HEALPixCells {
        let npix = 12 << ((depth as usize) << 1);

        let cells = (0_u64..(npix as u64))
            .map(|pix| HEALPixCell(depth, pix))
            .collect::<HashSet<_>>();

        HEALPixCells { depth, cells }
    }

    pub fn degrade(self, depth: u8) -> Self {
        // Degrade to a more precise depth is
        // not possible
        if depth >= self.depth {
            self
        } else {
            let delta_depth = self.depth - depth;
            let two_times_delta_depth = 2 * delta_depth;
            let cells = self
                .cells
                .into_iter()
                .map(|HEALPixCell(_, idx)| HEALPixCell(depth, idx >> two_times_delta_depth))
                .collect::<HashSet<_>>();

            HEALPixCells { depth, cells }
        }
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> HashSet<&'a HEALPixCell> {
        self.cells.intersection(&other.cells).collect()
    }
    pub fn difference<'a>(&'a self, other: &'a Self) -> HashSet<&'a HEALPixCell> {
        self.cells.difference(&other.cells).collect()
    }

    pub fn iter(&self) -> HEALPixCellsIter {
        HEALPixCellsIter(self.cells.iter())
    }

    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }
}

pub struct NewHEALPixCells {
    depth: u8,
    // flags associating true to cells that
    // are new in the fov
    flags: HashMap<HEALPixCell, bool>,
    // A flag telling whether there has been
    // new cells added from the last frame
    is_new_cells_added: bool,
}

impl NewHEALPixCells {
    fn new(cells: &HEALPixCells) -> NewHEALPixCells {
        let depth = cells.depth;
        let mut is_new_cells_added = false;

        let flags = cells
            .iter()
            .cloned()
            .map(|cell| {
                is_new_cells_added = true;
                (cell, true)
            })
            .collect::<HashMap<_, _>>();

        NewHEALPixCells {
            depth,
            flags,
            is_new_cells_added,
        }
    }

    fn insert_new_cells(&mut self, cells: &HEALPixCells) {
        let mut is_new_cells_added = false;
        self.depth = cells.depth;
        self.flags = cells
            .iter()
            .cloned()
            .map(|cell| {
                let new = !self.flags.contains_key(&cell);
                is_new_cells_added |= new;

                (cell, new)
            })
            .collect::<HashMap<_, _>>();

        self.is_new_cells_added = is_new_cells_added;
    }

    #[inline]
    pub fn is_new(&self, cell: &HEALPixCell) -> bool {
        if let Some(is_cell_new) = self.flags.get(cell) {
            *is_cell_new
        } else {
            false
        }
    }
}

struct NewHEALPixCellsIter<'a>(Iter<'a, HEALPixCell>);

impl<'a> Iterator for NewHEALPixCellsIter<'a> {
    type Item = &'a HEALPixCell;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
use crate::math::log_2;
// Compute a depth from a number of pixels on screen
pub fn depth_from_pixels_on_screen(camera: &CameraViewPort, num_pixels: i32) -> f32 {
    let width = camera.get_screen_size().x;
    let aperture = camera.get_aperture().0 as f32;

    let angle_per_pixel = aperture / width;

    let depth_pixel =
        (std::f32::consts::PI / (3.0 * angle_per_pixel * angle_per_pixel)).log2() / 2.0;

    //let survey_max_depth = conf.get_max_depth();
    // The depth of the texture
    // A texture of 512x512 pixels will have a depth of 9
    let depth_offset_texture = log_2(num_pixels);
    // The depth of the texture corresponds to the depth of a pixel
    // minus the offset depth of the texture
    let mut depth_texture = depth_pixel - (depth_offset_texture as f32);
    if depth_texture < 0.0 {
        depth_texture = 0.0;
    }

    depth_texture
}

use cgmath::Vector3;
pub fn get_cells_in_camera(depth: u8, camera: &CameraViewPort) -> HEALPixCells {
    if let Some(vertices) = camera.get_vertices() {
        let inside = camera.get_center().truncate();
        polygon_coverage(vertices, depth, &inside)
    } else {
        HEALPixCells::allsky(depth)
    }
}

use crate::cdshealpix;
use cgmath::Vector4;
fn polygon_coverage(vertices: &[Vector4<f64>], depth: u8, inside: &Vector3<f64>) -> HEALPixCells {
    let coverage = cdshealpix::from_polygon(depth, vertices, &inside);

    let cells = coverage
        .flat_iter()
        .map(|idx| HEALPixCell(depth, idx))
        .collect();

    HEALPixCells { cells, depth }
}

// Contains the cells being in the FOV for a specific
pub struct HEALPixCellsInView {
    // The set of cells being in the current view for a
    // specific image survey
    cells: HEALPixCells,
    new_cells: NewHEALPixCells,
    prev_depth: u8,
    look_for_parents: bool,
    is_new_cells: bool,
}

use crate::camera::{CameraViewPort, UserAction};

impl HEALPixCellsInView {
    pub fn new(survey_tex_size: i32, max_depth: u8, camera: &CameraViewPort) -> Self {
        let cells = HEALPixCells::new();
        let new_cells = NewHEALPixCells::new(&cells);
        let prev_depth = 0;
        let look_for_parents = false;
        let is_new_cells = false;

        let mut view = HEALPixCellsInView {
            cells,
            new_cells,
            prev_depth,
            look_for_parents,
            is_new_cells,
        };

        view.refresh_cells(survey_tex_size, max_depth, camera);
        view
    }

    // This method is called whenever the user does an action
    // that moves the camera.
    // Everytime the user moves or zoom, the views must be updated
    // The new cells obtained are used for sending new requests
    pub fn refresh_cells(&mut self, survey_tex_size: i32, max_depth: u8, camera: &CameraViewPort) {
        self.prev_depth = self.cells.get_depth();

        // Compute that depth
        let num_pixels = survey_tex_size;
        let mut depth = depth_from_pixels_on_screen(camera, num_pixels).round() as u8;
        if depth > max_depth {
            depth = max_depth;
        }
        // Get the cells of that depth in the current field of view
        let cells = get_cells_in_camera(depth, camera);
        //self.is_new_cells = (cells.intersection(&self.cells).len() == cells.len());
        //self.is_new_cells = !cells.difference(&self.cells).is_empty();
        self.is_new_cells = (cells.len() != self.cells.len()) || (cells != self.cells);
        self.cells = cells;
        self.new_cells.insert_new_cells(&self.cells);

        if camera.get_last_user_action() == UserAction::Unzooming {
            if self.has_depth_decreased() {
                self.look_for_parents = true;
            }
        } else {
            self.look_for_parents = false;
        }
    }

    // Accessors
    #[inline]
    pub fn get_cells(&self) -> &HEALPixCells {
        &self.cells
    }

    #[inline]
    pub fn get_depth(&self) -> u8 {
        self.cells.get_depth()
    }

    #[inline]
    pub fn is_new(&self, cell: &HEALPixCell) -> bool {
        self.new_cells.is_new(cell)
    }

    #[inline]
    pub fn is_there_new_cells_added(&self) -> bool {
        //self.new_cells.is_there_new_cells_added()
        self.is_new_cells
    }

    #[inline]
    pub fn has_depth_decreased_while_unzooming(&self, camera: &CameraViewPort) -> bool {
        assert!(camera.get_last_user_action() == UserAction::Unzooming);
        self.look_for_parents
    }

    #[inline]
    fn has_depth_decreased(&self) -> bool {
        let depth = self.cells.get_depth();
        depth < self.prev_depth
    }
}
