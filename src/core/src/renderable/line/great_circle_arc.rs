use crate::{math::{lonlat::LonLatT, projection::{ProjectionType, coo_space::XYNDC}}, camera::CameraViewPort};
use crate::math::angle::Angle;
use cgmath::Vector2;
use cgmath::Zero;
use cgmath::InnerSpace;
use crate::math::angle::ToAngle;
use al_core::{log, info, inforec};
use crate::coo_space::XYZWModel;
use crate::math::{TWICE_PI, PI};

use crate::ArcDeg;
const MAX_ANGLE_BEFORE_SUBDIVISION: Angle<f64> = Angle(0.10943951023); // 12 degrees
const MAX_ITERATION: usize = 3;

pub fn project(lonlat1: &LonLatT<f64>, lonlat2: &LonLatT<f64>, camera: &CameraViewPort, projection: &ProjectionType) -> Vec<XYNDC> {
    // First longitude between [0; 2*pi[
    let mut lon1 = lonlat1.lon().to_radians();
    // Parallel latitude between [-0.5*pi; 0.5*pi]
    let lat1 = lonlat1.lat().to_radians();
    // Second longitude between [0; 2*pi[
    let mut lon2 = lonlat2.lon().to_radians();
    // Parallel latitude between [-0.5*pi; 0.5*pi]
    let lat2 = lonlat2.lat().to_radians();
    let is_intersecting_zero_meridian = lon1 > lon2;

    if is_intersecting_zero_meridian {
        // Make the longitudes lie between [-PI; PI];
        if lon1 > PI {
            lon1 -= TWICE_PI;
        }

        if lon2 > PI {
            lon2 -= TWICE_PI;
        }
    }

    let mut ndc_vertices: Vec<XYNDC> = vec![];

    let start_world_vertex = LonLatT::new(lon1.to_angle(), lat1.to_angle()).vector();
    let end_world_vertex = LonLatT::new(lon2.to_angle(), lat2.to_angle()).vector();

    let ndc_v1 = projection.model_to_normalized_device_space(&start_world_vertex, camera);
    let ndc_v2 = projection.model_to_normalized_device_space(&end_world_vertex, camera);

    if let (Some(start_ndc_vertex), Some(end_ndc_vertex)) = (ndc_v1, ndc_v2) {
        subdivide(
            &mut ndc_vertices,
            start_world_vertex,
            start_ndc_vertex,
            end_world_vertex,
            end_ndc_vertex,
            camera,
            projection,
            0
        );
    }

    ndc_vertices
}

fn subdivide(
    ndc_vertices: &mut Vec<XYNDC>,
    start_world_vertex: XYZWModel,
    start_ndc_vertex: XYNDC,

    end_world_vertex: XYZWModel,
    end_ndc_vertex: XYNDC,

    camera: &CameraViewPort,
    projection: &ProjectionType,

    iter: usize,
) {
    if iter > MAX_ITERATION {
        ndc_vertices.push(start_ndc_vertex);
        ndc_vertices.push(end_ndc_vertex);
        return;
    }

    // Project them. We are always facing the camera
    let mid_world_vertex = (start_world_vertex + end_world_vertex).normalize();

    if let Some(mid_ndc_vertex) = projection.model_to_normalized_device_space(&mid_world_vertex, camera) {
        let ab = mid_ndc_vertex - start_ndc_vertex;
        let bc = end_ndc_vertex - mid_ndc_vertex;

        let ab_l = ab.magnitude2();
        let bc_l = bc.magnitude2();

        let ab = ab.normalize();
        let bc = bc.normalize();
        let theta = crate::math::vector::angle2(&ab, &bc);

        // nearly colinear vectors
        if theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION {
            if crate::math::vector::det(&ab, &bc).abs() < 1e-2 {
                ndc_vertices.push(start_ndc_vertex);
                ndc_vertices.push(end_ndc_vertex);
            } else {
                // not colinear
                ndc_vertices.push(start_ndc_vertex);
                ndc_vertices.push(mid_ndc_vertex);

                ndc_vertices.push(mid_ndc_vertex);
                ndc_vertices.push(end_ndc_vertex);
            }
        } else if ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
            if ab_l < bc_l {
                ndc_vertices.push(start_ndc_vertex);
                ndc_vertices.push(mid_ndc_vertex);
            } else {
                ndc_vertices.push(mid_ndc_vertex);
                ndc_vertices.push(end_ndc_vertex);
            }
        } else {
            // Subdivide a->b and b->c
            subdivide(
                ndc_vertices,
                start_world_vertex,
                start_ndc_vertex,
                mid_world_vertex,
                mid_ndc_vertex,
                camera,
                projection,
                iter + 1
            );

            subdivide(
                ndc_vertices,
                mid_world_vertex,
                mid_ndc_vertex,
                end_world_vertex,
                end_ndc_vertex,
                camera,
                projection,
                iter + 1
            );
        }
    }
}