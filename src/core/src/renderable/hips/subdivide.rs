use crate::camera::CameraViewPort;
use crate::math::angle::Angle;
use crate::math::projection::ProjectionType;
use crate::math::vector::dist2;
use crate::HEALPixCell;

const M: f64 = 280.0 * 280.0;
const N: f64 = 150.0 * 150.0;
const RAP: f64 = 0.7;

fn is_too_large(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> bool {
    let vertices = cell
        .vertices()
        .iter()
        .filter_map(|(lon, lat)| {
            let vertex = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            projection.icrs_celestial_to_screen_space(&vertex, camera)
        })
        .collect::<Vec<_>>();

    if vertices.len() < 4 {
        false
    } else {
        let d1 = dist2(vertices[0].as_ref(), &vertices[2].as_ref());
        let d2 = dist2(vertices[1].as_ref(), &vertices[3].as_ref());
        if d1 > M || d2 > M {
            true
        } else if d1 < N && d2 < N {
            false
        } else {
            let rap = if d2 > d1 { d1 / d2 } else { d2 / d1 };

            rap < RAP
        }
    }
}

pub fn num_hpxcell_subdivision(
    cell: &HEALPixCell,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> u8 {
    let d = cell.depth();
    // Subdivide all cells at least one time.
    // TODO: use a single subdivision number computed from the current cells inside the view
    // i.e. subdivide all cells in the view with the cell that has to be the most subdivided
    let mut num_sub = 1;
    if d < 2 {
        num_sub = 2 - d;
    }

    // Largest deformation cell among the cells of a specific depth
    let largest_center_to_vertex_dist =
        healpix::largest_center_to_vertex_distance(d, 0.0, healpix::TRANSITION_LATITUDE);
    let smallest_center_to_vertex_dist =
        healpix::largest_center_to_vertex_distance(d, 0.0, healpix::LAT_OF_SQUARE_CELL);

    let (lon, lat) = cell.center();
    let center_to_vertex_dist = healpix::largest_center_to_vertex_distance(d, lon, lat);

    let skewed_factor = (center_to_vertex_dist - smallest_center_to_vertex_dist)
        / (largest_center_to_vertex_dist - smallest_center_to_vertex_dist);

    if skewed_factor > 0.25 || is_too_large(cell, camera, projection) || cell.is_on_pole() {
        num_sub += 1;
    }

    num_sub
}
