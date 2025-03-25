use crate::camera::XYZWModel;
use crate::healpix::cell::HEALPixCell;

use crate::math::projection::*;

use crate::HEALPixCoverage;

use moclib::moc::{range::op::degrade::degrade, RangeMOCIterator};

pub(super) struct ViewHpxCells {
    hpx_cells: [HpxCells; NUM_COOSYSTEM],
    reg_frames: [u8; NUM_COOSYSTEM],
}

impl ViewHpxCells {
    pub(super) fn new() -> Self {
        let reg_frames = [0; NUM_COOSYSTEM];
        let hpx_cells = [
            HpxCells::new(CooSystem::ICRS),
            HpxCells::new(CooSystem::GAL),
        ];

        Self {
            hpx_cells,
            reg_frames,
        }
    }

    pub(super) fn register_frame(
        &mut self,
        camera_depth: u8,
        fov: &FieldOfView,
        center: &XYZWModel<f64>,
        camera_frame: CooSystem,
        proj: &ProjectionType,
        // survey frame
        frame: CooSystem,
    ) {
        self.reg_frames[frame as usize] += 1;

        if self.reg_frames[frame as usize] == 1 {
            // a new frame has been added
            self.update(camera_depth, fov, center, camera_frame, proj);
        }
    }

    pub(super) fn unregister_frame(
        &mut self,
        camera_depth: u8,
        fov: &FieldOfView,
        center: &XYZWModel<f64>,
        camera_frame: CooSystem,
        proj: &ProjectionType,
        // survey frame
        frame: CooSystem,
    ) {
        if self.reg_frames[frame as usize] > 0 {
            self.reg_frames[frame as usize] -= 1;
        }

        if self.reg_frames[frame as usize] == 0 {
            // a frame has been deleted
            self.update(camera_depth, fov, center, camera_frame, proj);
        }
    }

    pub(super) fn update(
        &mut self,
        camera_depth: u8,
        fov: &FieldOfView,
        center: &XYZWModel<f64>,
        camera_frame: CooSystem,
        proj: &ProjectionType,
    ) {
        for (frame, num_req) in self.reg_frames.iter().enumerate() {
            // if there are surveys/camera requesting the coverage
            if *num_req > 0 {
                self.hpx_cells[frame].update(camera_depth, fov, center, camera_frame, proj);
            }
        }
    }

    pub(super) fn get_cells(&self, depth: u8, frame: CooSystem) -> Vec<HEALPixCell> {
        self.hpx_cells[frame as usize].get_cells(depth)
    }

    pub(super) fn get_cov(&self, frame: CooSystem) -> &HEALPixCoverage {
        self.hpx_cells[frame as usize].get_cov()
    }

    /*pub(super) fn has_changed(&mut self) -> bool {
        let mut c = false;
        for (frame, num_req) in self.reg_frames.iter().enumerate() {
            // if there are surveys/camera requesting the coverage
            if *num_req > 0 {
                c |= self.hpx_cells[frame].has_view_changed();
            }
        }

        c
    }*/
}

// Contains the cells being in the FOV for a specific
pub struct HpxCells {
    frame: CooSystem,
    // the set of cells all depth
    //cells: Vec<HEALPixCell>,
    // An index vector referring to the indices of each depth cells
    //idx_rng: [Option<Range<usize>>; MAX_HPX_DEPTH as usize + 1],
    // Coverage created in the frame
    cov: HEALPixCoverage,
    // boolean refering to if the cells in the view has changed
    //new_cells: bool,
}

impl Default for HpxCells {
    fn default() -> Self {
        Self::new(CooSystem::ICRS)
    }
}

use al_api::coo_system::{CooSystem, NUM_COOSYSTEM};
use moclib::moc::RangeMOCIntoIterator;

use super::FieldOfView;
impl HpxCells {
    pub fn new(frame: CooSystem) -> Self {
        //let cells = Vec::new();
        let cov = HEALPixCoverage::empty(29);

        //let idx_rng = Default::default();

        Self {
            //cells,

            //idx_rng,
            cov,
            frame,
            //new_cells: true,
        }
    }

    // This method is called whenever the user does an action
    // that moves the camera.
    // Everytime the user moves or zoom, the views must be updated
    // The new cells obtained are used for sending new requests
    fn update(
        &mut self,
        camera_depth: u8,
        fov: &FieldOfView,
        center: &XYZWModel<f64>,
        camera_frame: CooSystem,
        proj: &ProjectionType,
    ) {
        // Compute the new coverage for that frame
        self.cov =
            super::build_fov_coverage(camera_depth, fov, center, camera_frame, self.frame, proj);

        // Clear the old cells
        /*let r = self.idx_rng[camera_depth as usize]
            .as_ref()
            .unwrap_or(&(0..0));
        let old_cells = &self.cells[r.clone()];

        self.idx_rng = Default::default();

        let mut new_cells = false;
        // Compute the cells at the tile_depth
        let cells = self
            .cov
            .flatten_to_fixed_depth_cells()
            .enumerate()
            .map(|(j, idx)| {
                let c = HEALPixCell(camera_depth, idx);

                if j >= old_cells.len() || old_cells[j] != c {
                    new_cells = true;
                }

                c
            })
            .collect::<Vec<_>>();

        if cells.len() != old_cells.len() {
            new_cells = true;
        }

        self.cells = cells;
        let num_cur = self.cells.len();
        self.idx_rng[camera_depth as usize] = Some(0..num_cur);

        if new_cells {
            self.new_cells = true;
        }*/
    }

    // Accessors
    // depth MUST be < to camera tile depth
    pub fn get_cells(&self, depth: u8) -> Vec<HEALPixCell> {
        let cov_depth = self.cov.depth_max();

        if depth == cov_depth {
            self.cov
                .flatten_to_fixed_depth_cells()
                .map(move |idx| HEALPixCell(depth, idx))
                .collect()
        } else if depth > self.cov.depth_max() {
            let cov_d = self.cov.depth_max();
            let dd = depth - cov_d;
            // compute the cells from the coverage

            self.cov
                .flatten_to_fixed_depth_cells()
                .flat_map(move |idx| {
                    // idx is at depth_max
                    HEALPixCell(cov_d, idx).get_children_cells(dd)
                })
                .collect()
        } else {
            // compute the cells from the coverage
            degrade((&self.cov.0).into_range_moc_iter(), depth)
                .flatten_to_fixed_depth_cells()
                .map(move |idx| HEALPixCell(depth, idx))
                .collect()
        }
    }

    /*
    #[inline(always)]
        pub fn num_of_cells(&self, depth: u8) -> usize {
            if let Some(rng) = &self.idx_rng[depth as usize] {
                rng.end - rng.start
            } else {
                0
            }
        }
     */

    /*#[inline]
    pub fn get_depth(&self) -> u8 {
        self.depth
    }*/

    /*#[inline]
    pub fn get_frame(&self) -> &CooSystem {
        &self.frame
    }*/

    /*#[inline]
    pub fn is_new(&self, cell: &HEALPixCell) -> bool {
        if let Some(&is_cell_new) = self.cells.get(cell) {
            is_cell_new
        } else {
            false
        }
    }*/

    #[inline(always)]
    pub fn get_cov(&self) -> &HEALPixCoverage {
        &self.cov
    }

    /*#[inline]
    pub fn is_there_new_cells_added(&self) -> bool {
        //self.new_cells.is_there_new_cells_added()
        self.is_new_cells_added
    }*/

    /*#[inline]
    pub fn has_view_changed(&mut self) -> bool {
        let new_cells = self.new_cells;
        self.new_cells = false;
        new_cells
    }*/
}
