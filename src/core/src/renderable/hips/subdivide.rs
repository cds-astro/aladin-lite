use crate::camera::CameraViewPort;
use crate::math::projection::ProjectionType;
use crate::math::vector::dist2;
use crate::HEALPixCell;
use crate::math::angle::ToAngle;

const M: f64 = 220.0 * 220.0;

fn is_too_large(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> bool {
    let vertices = cell
        .vertices()
        .iter()
        .filter_map(|(lon, lat)| {
            let vertex = crate::math::lonlat::radec_to_xyz(lon.to_angle(), lat.to_angle());
            projection.icrs_celestial_to_screen_space(&vertex, camera)
        })
        .collect::<Vec<_>>();

    if vertices.len() < 4 {
        false
    } else {
        let d1 = dist2(vertices[0].as_ref(), &vertices[2].as_ref());
        let d2 = dist2(vertices[1].as_ref(), &vertices[3].as_ref());

        d1 > M || d2 > M
    }
}

pub(crate) fn num_hpx_subdivision(
    cell: &HEALPixCell,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> u8 {
    let d0 = cell.depth();

    let num_sub = num_hpx_subdivision_rec(cell, d0, camera, projection);

    // Ensure that the HEALPix cells wireframe is at least composed of order 3 cells
    let sub_cells_depth = d0 + num_sub;

    if sub_cells_depth < 3 {
        num_sub + (3 - sub_cells_depth)
    } else {
        num_sub
    }
}

fn num_hpx_subdivision_rec(
    cell: &HEALPixCell,
    d0: u8,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> u8 {
    if (cell.depth() - d0) < MAX_HPX_CELL_SUBDIVISION
        && cell.depth() < 29
        && is_too_large(cell, camera, projection)
    {
        let sub_cells = cell.subdivide();

        1 + [
            num_hpx_subdivision_rec(&sub_cells[0], d0, camera, projection),
            num_hpx_subdivision_rec(&sub_cells[1], d0, camera, projection),
            num_hpx_subdivision_rec(&sub_cells[2], d0, camera, projection),
            num_hpx_subdivision_rec(&sub_cells[3], d0, camera, projection),
        ]
        .iter()
        .max()
        .unwrap()
    } else {
        0
    }
}

const MAX_HPX_CELL_SUBDIVISION: u8 = 4;
/// Subdivide a HEALPix cell that is directly mapped to a tile image in many sub cells
/// taking into account the distortion occuring when projecting its vertices into the screen pixel space
pub(crate) fn subdivide_hpx_cell(
    cell: &HEALPixCell,
    num_subdivisions: u8,
    camera: &CameraViewPort,
) -> Box<[HEALPixCell]> {
    cell.get_children_cells(num_subdivisions)
        .map(|child_cell| {
            /*let cell_depth = child_cell.depth();

            // Largest deformation cell among the cells of a specific depth
            let largest_center_to_vertex_dist = healpix::largest_center_to_vertex_distance(
                cell_depth,
                0.0,
                healpix::TRANSITION_LATITUDE,
            );
            let smallest_center_to_vertex_dist = healpix::largest_center_to_vertex_distance(
                cell_depth,
                0.0,
                healpix::LAT_OF_SQUARE_CELL,
            );

            let (lon, lat) = child_cell.center();
            let center_to_vertex_dist =
                healpix::largest_center_to_vertex_distance(cell_depth, lon, lat);

            let skewed_factor = (center_to_vertex_dist - smallest_center_to_vertex_dist)
                / (largest_center_to_vertex_dist - smallest_center_to_vertex_dist);*/

            let lat = child_cell.center().1;


            let c0 = child_cell.ancestor(child_cell.depth()).idx();

            let c3 = child_cell.ancestor(child_cell.depth() - 3);
            let (x3i, y3i) = c3.get_offset_in_texture_cell(c3.depth());
            let nside3i = c3.nside() as u32;
            let c3_s = (x3i, y3i) == (0, nside3i - 1) || (x3i, y3i) == (nside3i - 1, 0);

            let (x, y) = child_cell.get_offset_in_texture_cell(child_cell.depth());
            let nside = child_cell.nside() as u32;

            let collignon_equatorial_frontier = ((0..=3).contains(&c0) || (8..=11).contains(&c0)) && ((x + y) as i32 - (nside as i32) + 1).abs() <= 1;

            // A HEALPix cell would be distorted if
            let hpx_num_sub = if camera.get_aperture() >= 15.0_f64.to_radians() &&
                    // cells at the frontier between collignon and equatorial HEALpix zones
                    ((0..=3).contains(&c0) || (8..=11).contains(&c0)) &&
                    // neighbors of cells having only 7 neighbors (the most distorted ones)
                    c3_s
                {
                    // more specific cells needing a subdivision
                    2
                } else if collignon_equatorial_frontier && c3_s {
                    // on the collignon/equatorial fence and part of a 3 order cell having only 7 neighbors
                    1
                } else if child_cell.is_on_pole() {
                    // it lies on a pole
                    1
                } else if child_cell.is_on_base_cell_edges() && lat.abs() >= healpix::TRANSITION_LATITUDE {
                    // it lies on a frontier between base cells and at a high absolute latitude
                    1
                } else {
                    0
                };

            // Subdivide one more time if the HEALPix cell is distorted
            child_cell.get_children_cells(hpx_num_sub)
        })
        .flatten()
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
