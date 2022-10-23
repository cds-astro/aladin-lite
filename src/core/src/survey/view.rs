use crate::{coosys, healpix::cell::HEALPixCell, math};
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

// Compute a depth from a number of pixels on screen
pub fn depth_from_pixels_on_screen(camera: &CameraViewPort, num_pixels: i32) -> u8 {
    let width = camera.get_screen_size().x;
    let aperture = camera.get_aperture().0 as f32;

    let angle_per_pixel = aperture / width;

    let two_power_two_times_depth_pixel =
        std::f32::consts::PI / (3.0 * angle_per_pixel * angle_per_pixel);
    let depth_pixel = (two_power_two_times_depth_pixel.log2() / 2.0).floor() as u32;

    //let survey_max_depth = conf.get_max_depth();
    // The depth of the texture
    // A texture of 512x512 pixels will have a depth of 9
    let depth_offset_texture = math::utils::log_2_unchecked(num_pixels);
    // The depth of the texture corresponds to the depth of a pixel
    // minus the offset depth of the texture
    if depth_offset_texture > depth_pixel {
        0_u8
    } else {
        (depth_pixel - depth_offset_texture) as u8
    }

    /*let mut depth = 0;
    let mut d1 = std::f64::MAX;
    let mut d2 = std::f64::MAX;

    while d1 > 512.0*512.0 && d2 > 512.0*512.0 {

        let (lon, lat) = crate::math::lonlat::xyzw_to_radec(camera.get_center());
        let lonlat = math::lonlat::LonLatT(lon, lat);

        let (ipix, _, _) = crate::healpix::utils::hash_with_dxdy(depth, &lonlat);
        let vertices = project_vertices::<P>(&HEALPixCell(depth, ipix), camera);

        d1 = crate::math::vector::dist2(&vertices[0], &vertices[2]);
        d2 = crate::math::vector::dist2(&vertices[1], &vertices[3]);

        depth += 1;
    }
    al_core::info!(depth);
    if depth > 0 {
        depth - 1
    } else {
        0
    }*/
}

use healpix::coverage::HEALPixCoverage;
pub fn compute_view_coverage(camera: &CameraViewPort, depth: u8, dst_frame: &CooSystem) -> HEALPixCoverage {
    if depth <= 1 {
        HEALPixCoverage::allsky(depth)
    } else if let Some(vertices) = camera.get_vertices() {
        // The vertices coming from the camera are in a specific coo sys
        // but cdshealpix accepts them to be given in ICRSJ2000 coo sys
        let camera_frame = camera.get_system();
        let vertices = vertices
            .iter()
            .map(|v| coosys::apply_coo_system(camera_frame, dst_frame, v))
            .collect::<Vec<_>>();

        let inside_vertex = camera.get_center();
        let inside_vertex = coosys::apply_coo_system(camera_frame, dst_frame, inside_vertex);

        // Prefer to query from_polygon with depth >= 2
        HEALPixCoverage::new(
            depth,
            &vertices[..],
            &inside_vertex.truncate(),
        )
    } else {
        HEALPixCoverage::allsky(depth)
    }
}

use crate::healpix;
use al_api::coo_system::CooSystem;
pub fn get_tile_cells_in_camera(
    depth_tile: u8,
    camera: &CameraViewPort,
    hips_frame: &CooSystem,
) -> (HEALPixCoverage, Vec<HEALPixCell>) {
    let moc = compute_view_coverage(camera, depth_tile, hips_frame);
    let cells = moc.flatten_to_fixed_depth_cells()
        .map(|idx| {
            HEALPixCell(depth_tile, idx)
        })
        .collect();
    
    (moc, cells)
}

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
    pub fn refresh(&mut self, new_depth: u8, hips_frame: CooSystem, camera: &CameraViewPort) {
        self.depth = new_depth;
        self.frame = hips_frame;

        // Get the cells of that depth in the current field of view
        let (coverage, tile_cells) = get_tile_cells_in_camera(self.depth, camera, &self.frame);
        self.coverage = coverage;

        // Update cells in the fov
        self.update_cells_in_fov(&tile_cells);
    }

    fn update_cells_in_fov(&mut self, cells_in_fov: &[HEALPixCell]) {
        let new_cells = cells_in_fov
            .iter()
            .map(|cell| {
                let new = !self.cells.contains_key(cell);
                self.is_new_cells_added |= new;

                (*cell, new)
            })
            .collect::<HashMap<_, _>>();

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
