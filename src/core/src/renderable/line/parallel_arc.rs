use crate::math::angle::Angle;
use crate::math::projection::coo_space::XYZWModel;
use cgmath::Vector2;
use crate::ProjectionType;
use crate::CameraViewPort;
use cgmath::Zero;
use cgmath::InnerSpace;
use crate::math::angle::ToAngle;
use crate::math::lonlat::LonLat;
use crate::coo_space::XYNDC;
use crate::math::{TWICE_PI, PI};
use crate::ArcDeg;
use crate::LonLatT;
const MAX_ANGLE_BEFORE_SUBDIVISION: Angle<f64> = Angle(0.10);
const MAX_ITERATION: usize = 4;

// Requirement:
// * Parallel latitude between [-0.5*pi; 0.5*pi]
// * First longitude between [0; 2\pi[
// * Second lon length between [0; 2\pi[
// * (lon1 - lon2).abs() < PI
pub fn project(lat: f64, mut lon1: f64, mut lon2: f64, camera: &CameraViewPort, projection: &ProjectionType) -> Vec<XYNDC> {
    let mut vertices = vec![];

    let lon_len = crate::math::sph_geom::distance_from_two_lon(lon1, lon2);
    let mut lon2 = lon1 + lon_len;

    // Can only cross the 0 meridian but not 0 and 180 ones
    if lon2 > TWICE_PI {
        // it crosses the zero meridian
        lon2 -= TWICE_PI;
        // lon1 is > PI because the lon len is <= PI
        lon1 -= TWICE_PI;
    }

    // We know (lon1, lat) can be projected as it is a requirement of that method
    let v1 = crate::math::lonlat::proj(&LonLatT::new(lon1.to_angle(), lat.to_angle()), projection, camera);
    let v2 = crate::math::lonlat::proj(&LonLatT::new(lon2.to_angle(), lat.to_angle()), projection, camera);

    match (v1, v2) {
        (Some(v1), Some(v2)) => {
            subdivide_multi(&mut vertices, lat, lon1, lon2, camera, projection);
        },
        (None, Some(v2)) => {
            let (lon1, lon2) = sub_valid_domain(lat, lon2, lon1, projection, camera);
            subdivide_multi(&mut vertices, lat, lon1, lon2, camera, projection);
        },
        (Some(v1), None) => {
            let (lon1, lon2) = sub_valid_domain(lat, lon1, lon2, projection, camera);
            subdivide_multi(&mut vertices, lat, lon1, lon2, camera, projection);
        },
        (None, None) => {}
    }

    vertices
}

// Precondition:
// * angular distance between valid_lon and invalid_lon is < PI
// * valid_lon and invalid_lon are well defined, i.e. they can be between [-PI; PI] or [0, 2PI] depending
//   whether they cross or not the zero meridian
fn sub_valid_domain(lat: f64, mut valid_lon: f64, mut invalid_lon: f64, projection: &ProjectionType, camera: &CameraViewPort) -> (f64, f64) {
    let d_alpha = camera.get_aperture().to_radians() * 0.02;

    let mut l_valid = valid_lon;
    let mut l_invalid = invalid_lon;
    while (l_valid - l_invalid).abs() > d_alpha {
        let lm = (l_valid + l_invalid)*0.5;
        // check whether is it defined or not
        let mid_lonlat = LonLatT::new(lm.to_angle(), lat.to_angle());
        if let Some(_) = crate::math::lonlat::proj(&mid_lonlat, projection, camera) {
            l_valid = lm;
        } else {
            l_invalid = lm;
        }
    }

    // l2 is invalid while l1 is valid
    if valid_lon > invalid_lon {
        (l_valid, valid_lon)
    } else {
        (valid_lon, l_valid)
    }
}

fn subdivide_multi(
    vertices: &mut Vec<XYNDC>,
    lat: f64,

    lon_s: f64,
    lon_e: f64, 

    camera: &CameraViewPort,
    projection: &ProjectionType,
) {
    let num_vertices = 5;
    let dlon = (lon_e - lon_s) / (num_vertices as f64);
    for i in 0..num_vertices {
        let lon1 = lon_s + (i as f64) * dlon;
        let lon2 = lon1 + dlon;

        subdivide(vertices, lat, lon1, lon2, camera, projection, 0);
    }
}


fn subdivide(
    vertices: &mut Vec<XYNDC>,
    lat: f64,

    lon1: f64,
    lon2: f64, 

    camera: &CameraViewPort,
    projection: &ProjectionType,
    iter: usize,
) -> bool {
    let p1 = crate::math::lonlat::proj(&LonLatT::new(lon1.to_angle(), lat.to_angle()), projection, camera);
    let p2 = crate::math::lonlat::proj(&LonLatT::new(lon2.to_angle(), lat.to_angle()), projection, camera);

    if iter < MAX_ITERATION {
        // Project them. We are always facing the camera
        let lon0 = (lon1 + lon2)*0.5;
        let pm = crate::math::lonlat::proj(&LonLatT::new(lon0.to_angle(), lat.to_angle()), projection, camera);
    
        match (p1, pm, p2) {
            (Some(p1), Some(pm), Some(p2)) => {
                let ab = pm - p1;
                let bc = p2 - pm;
                let ab_l = ab.magnitude2();
                let bc_l = bc.magnitude2();
        
                let ab = ab.normalize();
                let bc = bc.normalize();
                let theta = crate::math::vector::angle2(&ab, &bc);
                let vectors_nearly_colinear = theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION;
        
                if vectors_nearly_colinear {
                    // Check if ab and bc are colinear
                    if crate::math::vector::det(&ab, &bc).abs() < 1e-2 {
                        vertices.push(p1);
                        vertices.push(p2);
                    } else {
                        // not colinear
                        vertices.push(p1);
                        vertices.push(pm);
        
                        vertices.push(pm);
                        vertices.push(p2);
                    }
                } else if ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
                    if ab_l < bc_l {
                        vertices.push(p1);
                        vertices.push(pm);
                    } else {
                        vertices.push(pm);
                        vertices.push(p2);
                    }
                } else {
                    // Subdivide a->b and b->c
                    if !subdivide(
                        vertices,
                        lat,
                        lon1,
                        lon0,
                        camera,
                        projection,
                        iter + 1
                    ) {
                        vertices.push(p1);
                        vertices.push(pm);
                    }
        
                    if !subdivide(
                        vertices,
                        lat,
                        lon0,
                        lon2,
                        camera,
                        projection,
                        iter + 1
                    ) {
                        vertices.push(pm);
                        vertices.push(p2);
                    }
                }

                true
            },
            _ => false
        }
    } else {
        false
    }
}