use std::collections::{HashSet, HashMap};
use crate::healpix_cell::HEALPixCell;

use std::collections::hash_set::Iter;

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

impl HEALPixCells {
    fn new() -> Self {
        HEALPixCells {
            depth: 0,
            cells: HashSet::new()
        }
    }

    fn contains(&self, cell: &HEALPixCell) -> bool {
        self.contains(cell)
    }

    pub fn allsky(depth: u8) -> HEALPixCells {
        let npix = 12 << ((depth as usize) << 1);

        let mut cells = (0_u64..(npix as u64))
            .map(|pix| HEALPixCell(depth, pix))
            .collect::<HashSet<_>>();

        HEALPixCells {
            depth,
            cells
        }
    }

    pub fn degrade(self, depth: u8) -> Self {
        // Degrade to a more precise depth is
        // not possible
        if depth >= self.depth {
            self
        } else {
            let delta_depth = self.depth - depth;
            let two_times_delta_depth = 2*delta_depth;
            let cells = self.cells.into_iter()
                .map(|HEALPixCell(_, idx)| {
                    HEALPixCell(depth, idx >> two_times_delta_depth)
                })
                .collect::<HashSet<_>>();

            HEALPixCells {
                depth,
                cells
            }
        }
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

        let flags = cells.iter()
            .cloned()
            .map(|cell| {
                is_new_cells_added = true;
                (cell, true)
            })
            .collect::<HashMap<_, _>>();

        NewHEALPixCells {
            depth,
            flags,
            is_new_cells_added
        }
    }

    fn insert_new_cells(self, cells: &HEALPixCells) -> NewHEALPixCells {
        let mut is_new_cells_added = false;
        let new_depth = cells.depth;
        let flags = cells.iter()
            .cloned()
            .map(|cell| {
                let new = !self.flags.contains_key(&cell);
                is_new_cells_added |= new;

                (cell, new)
            })
            .collect::<HashMap<_, _>>();

        NewHEALPixCells {
            depth: new_depth,
            flags,
            is_new_cells_added
        }
    }

    #[inline]
    fn is_there_new_cells_added(&self) -> bool {
        self.is_new_cells_added
    }

    #[inline]
    fn iter<'a>(&'a self) -> NewHEALPixCellsIter<'a> {
        let iter = self.flags.iter()
            .filter_map(|(cell, new)| {
                if *new {
                    Some(*cell)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>()
            .iter();

        NewHEALPixCellsIter(iter)
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
fn compute_depth_for_survey(camera: &CameraViewPort, survey: &ImageSurvey) -> u8 {
    let width = camera.get_screen_size().x;
    let aperture = camera.get_aperture().0;

    let angle_per_pixel = aperture / width;

    let depth_pixel = (
        (
            std::f32::consts::PI / (3.0 * angle_per_pixel * angle_per_pixel)
        ).log2() / 2.0
    ).round() as i8;

    //let depth_texture = {
        let conf = survey.get_textures().config();
        // The texture size in pixels
        let texture_size = conf.get_texture_size();
        //let survey_max_depth = conf.get_max_depth();
        // The depth of the texture
        // A texture of 512x512 pixels will have a depth of 9
        let depth_offset_texture = log_2(texture_size);
        // The depth of the texture corresponds to the depth of a pixel
        // minus the offset depth of the texture
        let mut depth_texture = depth_pixel - depth_offset_texture;
        if depth_texture < 0 {
            depth_texture = 0;
        }

        //std::cmp::min(survey_max_depth, depth_texture)
    //};
   
    depth_texture as u8
}

pub fn get_cells_in_camera(depth: u8, camera: &CameraViewPort) -> HEALPixCells {
    let cells = if let Some(vertices) = camera.get_vertices() {
        let inside = camera.get_center();
        polygon_coverage(vertices, depth, inside)
    } else {
        crate::healpix_cell::allsky(depth)
    };

    HEALPixCells {
        depth,
        cells
    }
}

use cgmath::Vector4;
use crate::cdshealpix;
fn polygon_coverage(
    vertices: &[Vector4<f32>],
    depth: u8,
    inside: &Vector3<f32>
) -> HEALPixCells {
    let coverage = cdshealpix::HEALPixCoverage::new(depth, vertices, &inside);

    let cells = coverage.flat_iter()
        .map(|idx| {
            HEALPixCell(depth, idx)
        })
        .collect();
    
    HEALPixCells {
        cells,
        depth
    }
}

// Contains the cells being in the FOV for a specific
pub struct HEALPixCellsInView {
    // The set of cells being in the current view for a
    // specific image survey
    cells: HEALPixCells,
    new_cells: NewHEALPixCells,
    prev_depth: u8,
}

use crate::camera::CameraViewPort;
use super::image_survey::ImageSurvey;
impl HEALPixCellsInView {
    pub fn new() -> Self {
        let cells = HEALPixCells::new();
        let new_cells = NewHEALPixCells::new(&cells);
        let prev_depth = 0;

        HEALPixCellsInView {
            cells,
            new_cells,
            prev_depth,
        }
    }

    // This method is called whenever the user does an action
    // that moves the camera.
    // Everytime the user moves or zoom, the views must be updated
    // The new cells obtained are used for sending new requests
    pub fn update(&mut self, survey: &ImageSurvey, camera: &CameraViewPort) {
        self.prev_depth = self.cells.get_depth();

        // Compute that depth
        let depth = compute_depth_for_survey(camera, survey);
        // Get the cells of that depth in the current field of view
        self.cells = get_cells_in_camera(depth, camera);
        self.new_cells.insert_new_cells(&self.cells);
    }

    // Accessors
    #[inline]
    pub fn get_new_cells<'a>(&'a self) -> NewHEALPixCellsIter<'a> {
        self.new_cells.iter()
    }

    #[inline]
    pub fn get_cells_iter<'a>(&'a self) -> HEALPixCellsIter<'a> {
        self.cells.iter()
    }

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
        self.new_cells.is_there_new_cells_added()
    }

    #[inline]
    pub fn has_depth_decreased(&self) -> bool {
        let depth = self.cells.get_depth();
        depth < self.prev_depth
    }
}