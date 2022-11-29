use crate::{coosys, healpix::cell::HEALPixCell};
use std::collections::HashMap;


use crate::math::angle::Angle;
use crate::Projection;
use crate::math::projection::*;
use cgmath::Vector2;

pub fn vertices(cell: &HEALPixCell, camera: &CameraViewPort, projection: ProjectionType) -> Result<[Vector2<f64>; 4], &'static str> {
    let project_vertex = |(lon, lat): (f64, f64)| -> Result<Vector2<f64>, &'static str> {
        let vertex = crate::math::lonlat::radec_to_xyzw(Angle(lon), Angle(lat));
        projection.view_to_screen_space(&vertex, camera).ok_or("Cannot project")
    };
    
    let vertices = cell.vertices();
    let reversed_longitude = camera.get_longitude_reversed();

    let invalid_tri = |tri_ccw: bool, reversed_longitude: bool| -> bool {
        (!reversed_longitude && !tri_ccw) || (reversed_longitude && tri_ccw)
    };

    let c0 = project_vertex(vertices[0])?;
    let c1 = project_vertex(vertices[1])?;
    let c2 = project_vertex(vertices[2])?;
    let c3 = project_vertex(vertices[3])?;

    let first_tri_ccw = crate::math::vector::ccw_tri(&c0, &c1, &c2);
    let second_tri_ccw = crate::math::vector::ccw_tri(&c2, &c3, &c0);
    //let third_tri_ccw = crate::math::vector::ccw_tri(&c2, &c3, &c0);
    //let fourth_tri_ccw = crate::math::vector::ccw_tri(&c3, &c0, &c1);

    let invalid_cell = invalid_tri(first_tri_ccw, reversed_longitude) || invalid_tri(second_tri_ccw, reversed_longitude);

    if invalid_cell {
        Err("Cell out of the view")
    } else {
        Ok([c0, c1, c2, c3])
    }
}

use al_api::cell::HEALPixCellProjeted;

pub fn project(cell: HEALPixCellProjeted, camera: &CameraViewPort, projection: ProjectionType) -> Option<HEALPixCellProjeted> {
    match projection {
        ProjectionType::HEALPix(_) => {
            let tri_idx_in_collignon_zone = |x: f64, y: f64| -> u8 {
                let zoom_factor = camera.get_clip_zoom_factor() as f32;
                let x = (((x as f32) / camera.get_width()) - 0.5) * zoom_factor;
                let y = (((y as f32) / camera.get_height()) - 0.5) * zoom_factor;
    
                let x_zone = ((x + 0.5) * 4.0).floor() as u8;
                x_zone + 4 * ((y > 0.0) as u8)
            };
    
            let is_in_collignon = |_x: f64, y: f64| -> bool {
                let y = (((y as f32) / camera.get_height()) - 0.5) * (camera.get_clip_zoom_factor() as f32);
                !(-0.25..=0.25).contains(&y)
            };
    
            if is_in_collignon(cell.vx[0], cell.vy[0]) && is_in_collignon(cell.vx[1], cell.vy[1]) && is_in_collignon(cell.vx[2], cell.vy[2]) && is_in_collignon(cell.vx[3], cell.vy[3]) {
                let all_vertices_in_same_collignon_region = tri_idx_in_collignon_zone(cell.vx[0], cell.vy[0]) == tri_idx_in_collignon_zone(cell.vx[1], cell.vy[1]) && (tri_idx_in_collignon_zone(cell.vx[0], cell.vy[0]) == tri_idx_in_collignon_zone(cell.vx[2], cell.vy[2])) && (tri_idx_in_collignon_zone(cell.vx[0], cell.vy[0]) == tri_idx_in_collignon_zone(cell.vx[3], cell.vy[3]));
                if !all_vertices_in_same_collignon_region {
                    None
                } else {
                    Some(cell)
                }
            } else {
                Some(cell)
            }
        },
        _ => Some(cell)
    }
}

use healpix::coverage::HEALPixCoverage;
pub fn compute_view_coverage(camera: &CameraViewPort, depth: u8, dst_frame: &CooSystem) -> HEALPixCoverage {
    if depth <= 1 {
        HEALPixCoverage::allsky(depth)
    } else {
        if let Some(vertices) = camera.get_vertices() {
            // The vertices coming from the camera are in a specific coo sys
            // but cdshealpix accepts them to be given in ICRSJ2000 coo sys
            let camera_frame = camera.get_system();
            let vertices = vertices
                .iter()
                .map(|v| coosys::apply_coo_system(camera_frame, dst_frame, v))
                .collect::<Vec<_>>();

            // Check if the polygon is too small with respect to the angular size
            // of a cell at depth order
            let fov_bbox = camera.get_bounding_box();
            let d_lon = fov_bbox.get_lon_size();
            let d_lat = fov_bbox.get_lat_size();

            let size_hpx_cell = crate::healpix::utils::MEAN_HPX_CELL_RES[depth as usize];
            if d_lon < size_hpx_cell && d_lat < size_hpx_cell {
                // Polygon is small and this may result in a moc having only a few cells
                // One can build the moc from a list of cells
                // This particular case avoids falling into a panic in cdshealpix
                // See https://github.com/cds-astro/cds-moc-rust/issues/3

                let hpx_idxs_iter = vertices
                    .iter()
                    .map(|v| {
                        let (lon, lat) = crate::math::lonlat::xyzw_to_radec(&v);
                        cdshealpix::nested::hash(depth, lon.0, lat.0)
                    });

                HEALPixCoverage::from_hpx_cells(depth, hpx_idxs_iter, Some(vertices.len()))
            } else {
                // The polygon is not too small for the depth asked
                let inside_vertex = camera.get_center();
                let inside_vertex = coosys::apply_coo_system(camera_frame, dst_frame, inside_vertex);

                // Prefer to query from_polygon with depth >= 2
                HEALPixCoverage::new(
                    depth,
                    &vertices[..],
                    &inside_vertex.truncate(),
                )
            }
        } else {
            HEALPixCoverage::allsky(depth)
        }
    }
}

use crate::healpix;

// Contains the cells being in the FOV for a specific
pub struct HEALPixCellsInView {
    // The set of cells being in the current view for a
    // specific image survey
    pub depth: u8,
    prev_depth: u8,
    view_unchanged: bool,
    frame: CooSystem,

    // flags associating true to cells that
    // are new in the fov
    cells: HashMap<HEALPixCell, bool>,
    // A flag telling whether there has been
    // new cells added from the last frame
    is_new_cells_added: bool,

    coverage: HEALPixCoverage,
}

impl Default for HEALPixCellsInView {
    fn default() -> Self {
        Self::new()
    }
}

use al_api::coo_system::CooSystem;
use crate::camera::CameraViewPort;
impl HEALPixCellsInView {
    pub fn new() -> Self {
        let cells = HashMap::new();
        let coverage = HEALPixCoverage::allsky(0);

        let view_unchanged = false;
        let frame = CooSystem::ICRSJ2000;
        Self {
            cells,
            prev_depth: 0,
            depth: 0,
            is_new_cells_added: false,
            view_unchanged,
            frame,
            coverage,
        }
    }

    pub fn reset_frame(&mut self) {
        self.is_new_cells_added = false;
        self.view_unchanged = false;
        self.prev_depth = self.get_depth();
    }

    // This method is called whenever the user does an action
    // that moves the camera.
    // Everytime the user moves or zoom, the views must be updated
    // The new cells obtained are used for sending new requests
    pub fn refresh(&mut self, tile_depth: u8, hips_frame: CooSystem, camera: &CameraViewPort) {
        self.depth = tile_depth;
        self.frame = hips_frame;

        // Get the cells of that depth in the current field of view
        let coverage = compute_view_coverage(camera, tile_depth, &self.frame);
        let new_cells = coverage.flatten_to_fixed_depth_cells()
            .map(|idx| {
                let cell = HEALPixCell(tile_depth, idx);
                let new = !self.cells.contains_key(&cell);
                self.is_new_cells_added |= new;

                (cell, new)
            })
            .collect::<HashMap<_, _>>();
        self.coverage = coverage;

        // If no new cells have been added
        self.view_unchanged = !self.is_new_cells_added && new_cells.len() == self.cells.len();
        self.cells = new_cells;
    }

    // Accessors
    #[inline]
    pub fn get_cells(&self) -> impl Iterator<Item = &HEALPixCell> {
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
    pub fn get_frame(&self) -> &CooSystem {
        &self.frame
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
    pub fn get_coverage(&self) -> &HEALPixCoverage {
        &self.coverage
    }

    #[inline]
    pub fn is_there_new_cells_added(&self) -> bool {
        //self.new_cells.is_there_new_cells_added()
        self.is_new_cells_added
    }

    #[inline]
    pub fn has_view_changed(&self) -> bool {
        //self.new_cells.is_there_new_cells_added()
        !self.view_unchanged
    }
}
