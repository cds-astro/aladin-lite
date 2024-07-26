pub mod viewport;
use crate::math::lonlat::LonLat;
use crate::math::projection::coo_space::XYZWModel;
pub use viewport::CameraViewPort;

pub mod fov;
pub use fov::FieldOfView;
pub mod view_hpx_cells;

use crate::CooSystem;
use crate::HEALPixCoverage;
use crate::ProjectionType;

pub fn build_fov_coverage(
    depth: u8,
    fov: &FieldOfView,
    camera_center: &XYZWModel<f64>,
    camera_frame: CooSystem,
    frame: CooSystem,
    proj: &ProjectionType,
) -> HEALPixCoverage {
    if let Some(vertices) = fov.get_vertices() {
        // The vertices coming from the camera are in a specific coo sys
        // but cdshealpix accepts them to be given in ICRS coo sys
        let vertices_iter = vertices
            .iter()
            .map(|v| crate::coosys::apply_coo_system(camera_frame, frame, v));

        // Check if the polygon is too small with respect to the angular size
        // of a cell at depth order
        let fov_bbox = fov.get_bounding_box();
        let d_lon = fov_bbox.get_lon_size();
        let d_lat = fov_bbox.get_lat_size();

        let size_hpx_cell = crate::healpix::utils::MEAN_HPX_CELL_RES[depth as usize];
        if d_lon < size_hpx_cell && d_lat < size_hpx_cell {
            // Polygon is small and this may result in a moc having only a few cells
            // One can build the moc from a list of cells
            // This particular case avoids falling into a panic in cdshealpix
            // See https://github.com/cds-astro/cds-moc-rust/issues/3

            let hpx_idxs_iter = vertices_iter.map(|v| {
                let (lon, lat) = crate::math::lonlat::xyzw_to_radec(&v);
                ::healpix::nested::hash(depth, lon.0, lat.0)
            });

            HEALPixCoverage::from_fixed_hpx_cells(depth, hpx_idxs_iter, Some(vertices.len()))
        } else {
            // The polygon is not too small for the depth asked
            let inside_vertex = crate::coosys::apply_coo_system(camera_frame, frame, camera_center);

            // Prefer to query from_polygon with depth >= 2
            let moc = HEALPixCoverage::from_3d_coos(depth, vertices_iter, &inside_vertex);

            moc
        }
    } else {
        let center_xyzw = crate::coosys::apply_coo_system(camera_frame, frame, camera_center);

        let biggest_fov_rad = proj.aperture_start().to_radians();
        let lonlat = center_xyzw.lonlat();
        HEALPixCoverage::from_cone(&lonlat, biggest_fov_rad * 0.5, depth)
    }
}
