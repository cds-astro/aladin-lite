use super::MOC;
use crate::{camera::CameraViewPort, HEALPixCoverage};
use al_api::moc::MOC as Cfg;

pub struct MOCHierarchy {
    full_res_depth: u8,
    // MOC at different resolution
    mocs: Vec<MOC>,
}
use al_core::WebGlContext;
impl MOCHierarchy {
    pub fn from_full_res_moc(gl: WebGlContext, full_res_moc: HEALPixCoverage, cfg: &Cfg) -> Self {
        let full_res_depth = full_res_moc.depth();

        let mut mocs: Vec<_> = (0..full_res_depth)
            .map(|d| MOC::new(gl.clone(), HEALPixCoverage(full_res_moc.degraded(d)), cfg))
            .collect();

        mocs.push(MOC::new(gl, full_res_moc, cfg));

        Self {
            mocs,
            full_res_depth,
        }
    }

    pub fn select_moc_from_view(&mut self, camera: &mut CameraViewPort) -> &mut MOC {
        /*const MAX_NUM_CELLS_TO_DRAW: usize = 1500;

        let mut d = self.full_res_depth as usize;

        while d > 5 {
            self.mocs[d].cell_indices_in_view(camera);

            let num_cells = self.mocs[d].num_cells_in_view(camera);
            if num_cells < MAX_NUM_CELLS_TO_DRAW {
                break;
            }

            d = d - 1;
        }

        self.mocs[d].cell_indices_in_view(camera);
        &mut self.mocs[d]*/

        let w_screen_px = camera.get_width() as f64;
        let smallest_cell_size_px = 8.0;
        let mut d = self.full_res_depth as usize;

        let hpx_cell_size_rad =
            (smallest_cell_size_px / w_screen_px) * camera.get_aperture().to_radians();

        while d > 0 {
            //self.mocs[d].cell_indices_in_view(camera);

            if crate::healpix::utils::MEAN_HPX_CELL_RES[d] > hpx_cell_size_rad {
                break;
            }

            d = d - 1;
        }

        &mut self.mocs[d]
    }

    pub fn get_full_moc(&self) -> &HEALPixCoverage {
        &self.mocs.last().unwrap().moc
    }

    pub fn get_full_res_depth(&self) -> u8 {
        self.full_res_depth
    }
}
