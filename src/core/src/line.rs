use crate::math::angle::Angle;
use cgmath::Vector2;
use crate::ProjectionType;
use crate::CameraViewPort;
use cgmath::Zero;
use cgmath::InnerSpace;
pub fn project_along_longitudes_and_latitudes(
    mut start_lon: f64,
    mut start_lat: f64,
    mut end_lon: f64,
    mut end_lat: f64, 
    camera: &CameraViewPort,
    projection: ProjectionType
) -> Vec<Vector2<f64>> {
    if start_lat >= end_lat {
        std::mem::swap(&mut start_lat, &mut end_lat);
    }

    if start_lon >= end_lon {
        std::mem::swap(&mut start_lon, &mut end_lon);
    }

    let num_point_max = if camera.is_allsky() {
        12
    } else {
        let one_deg: Angle<f64> = ArcDeg(40.0).into();
        if camera.get_aperture() < one_deg && !camera.contains_pole() {
            2
        } else {
            6
        }
    };

    let delta_lon = (end_lon - start_lon) / ((num_point_max - 1) as f64);
    let delta_lat = (end_lat - start_lat) / ((num_point_max - 1) as f64);

    let mut s_vert: Vec<Vector2<f64>> = vec![];

    let mut start = true;
    let mut cur = (0.0, 0.0, Vector2::zero());
    let mut prev = (0.0, 0.0, Vector2::zero());
    for i in 0..num_point_max {
        let (lon, lat) = (start_lon + (i as f64) * delta_lon, start_lat + (i as f64) * delta_lat);

        if let Some(p) = crate::math::lonlat::proj(lon, lat, projection, camera) {            
            if start {
                prev = (lon, lat, p);
                start = false;
            } else {
                cur = (lon, lat, p);
                subdivide_along_longitude_and_latitudes(&mut s_vert, prev, cur, camera, projection, 0);

                prev = cur;
            }
        } else if !start {
            start = true;
        }
    }

    s_vert
}
use crate::ArcDeg;
use crate::LonLatT;
const MAX_ANGLE_BEFORE_SUBDIVISION: Angle<f64> = Angle(0.10943951023); // 12 degrees
const MAX_ITERATION: usize = 3;
pub fn subdivide_along_longitude_and_latitudes(
    vertices: &mut Vec<Vector2<f64>>,
    (lon_s, lat_s, p_s): (f64, f64, Vector2<f64>),
    (lon_e, lat_e, p_e): (f64, f64, Vector2<f64>),
    camera: &CameraViewPort,
    projection: ProjectionType,
    iter: usize,
) {
    if iter > MAX_ITERATION {
        vertices.push(p_s);
        vertices.push(p_e);
        return;
    }

    // Project them. We are always facing the camera
    let lon_m = (lon_s + lon_e)*0.5;
    let lat_m = (lat_s + lat_e)*0.5;

    if let Some(p_m) = crate::math::lonlat::proj(lon_m, lat_m, projection, camera) {
        let ab = p_m - p_s;
        let bc = p_e - p_m;
        let ab_l = ab.magnitude2();
        let bc_l = bc.magnitude2();

        if ab_l < 1e-5 || bc_l < 1e-5 {
            return;
        }

        let ab = ab.normalize();
        let bc = bc.normalize();
        let theta = crate::math::vector::angle2(&ab, &bc);
        let vectors_nearly_colinear = theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION;

        if vectors_nearly_colinear {
            // Check if ab and bc are colinear
            if crate::math::vector::det(&ab, &bc).abs() < 1e-2 {
                vertices.push(p_s);
                vertices.push(p_e);
            } else {
                // not colinear
                vertices.push(p_s);
                vertices.push(p_m);

                vertices.push(p_m);
                vertices.push(p_e);
            }
        } else if ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
            if ab_l < bc_l {
                vertices.push(p_s);
                vertices.push(p_m);
            } else {
                vertices.push(p_m);
                vertices.push(p_e);
            }
        } else {
            // Subdivide a->b and b->c
            subdivide_along_longitude_and_latitudes(
                vertices,
                (lon_s, lat_s, p_s),
                (lon_m, lat_m, p_m),
                camera,
                projection,
                iter + 1
            );

            subdivide_along_longitude_and_latitudes(
                vertices,
                (lon_m, lat_m, p_m),
                (lon_e, lat_e, p_e),
                camera,
                projection,
                iter + 1
            );
        }
    }
}