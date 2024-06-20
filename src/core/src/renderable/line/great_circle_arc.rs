use crate::CameraViewPort;
use crate::ProjectionType;
use cgmath::Vector3;

use crate::math::angle::ToAngle;
use cgmath::InnerSpace;

use crate::coo_space::XYZModel;
use crate::coo_space::XYNDC;

use crate::LonLatT;
const MAX_ITERATION: usize = 5;

// Requirement:
// * Latitudes between [-0.5*pi; 0.5*pi]
// * Longitudes between [0; 2\pi[
// * (lon1 - lon2).abs() < PI so that is can only either cross the preimary meridian or opposite primary meridian
//   (the latest is handled because of the longitudes intervals)
pub fn project(
    lon1: f64,
    lat1: f64,
    lon2: f64,
    lat2: f64,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> Vec<XYNDC<f64>> {
    let mut vertices = vec![];

    let lonlat1 = LonLatT::new(lon1.to_angle(), lat1.to_angle());
    let lonlat2 = LonLatT::new(lon2.to_angle(), lat2.to_angle());

    let v1: Vector3<_> = lonlat1.vector();
    let v2: Vector3<_> = lonlat2.vector();

    let p1 = projection.model_to_normalized_device_space(&v1.extend(1.0), camera);
    let p2 = projection.model_to_normalized_device_space(&v2.extend(1.0), camera);

    match (p1, p2) {
        (Some(_), Some(_)) => {
            project_line(&mut vertices, &v1, &v2, camera, projection, 0);
        }
        (None, Some(_)) => {
            let (v1, v2) = sub_valid_domain(v2, v1, projection, camera);
            project_line(&mut vertices, &v1, &v2, camera, projection, 0);
        }
        (Some(_), None) => {
            let (v1, v2) = sub_valid_domain(v1, v2, projection, camera);
            project_line(&mut vertices, &v1, &v2, camera, projection, 0);
        }
        (None, None) => {}
    }

    vertices
}

// Precondition:
// * angular distance between valid_lon and invalid_lon is < PI
// * valid_lon and invalid_lon are well defined, i.e. they can be between [-PI; PI] or [0, 2PI] depending
//   whether they cross or not the zero meridian
fn sub_valid_domain(
    valid_v: XYZModel<f64>,
    invalid_v: XYZModel<f64>,
    projection: &ProjectionType,
    camera: &CameraViewPort,
) -> (XYZModel<f64>, XYZModel<f64>) {
    let d_alpha = camera.get_aperture().to_radians() * 0.02;

    let mut vv = valid_v;
    let mut vi = invalid_v;
    while crate::math::vector::angle3(&vv, &vi).to_radians() > d_alpha {
        let vm = (vv + vi).normalize();
        // check whether is it defined or not
        if let Some(_) = projection.model_to_normalized_device_space(&vm.extend(1.0), camera) {
            vv = vm;
        } else {
            vi = vm;
        }
    }

    // Return the valid interval found by dichotomy
    (vv, valid_v)
}

fn project_line(
    vertices: &mut Vec<XYNDC<f64>>,
    v1: &XYZModel<f64>,
    v2: &XYZModel<f64>,
    camera: &CameraViewPort,
    projection: &ProjectionType,
    iter: usize,
) -> bool {
    let p1 = projection.model_to_normalized_device_space(&v1.extend(1.0), camera);
    let p2 = projection.model_to_normalized_device_space(&v2.extend(1.0), camera);

    if iter < MAX_ITERATION {
        // Project them. We are always facing the camera
        let vm = (v1 + v2).normalize();
        let pm = projection.model_to_normalized_device_space(&vm.extend(1.0), camera);

        match (p1, pm, p2) {
            (Some(p1), Some(pm), Some(p2)) => {
                let d12 = crate::math::vector::angle3(v1, v2).to_radians();

                // Subdivide when until it is > 30 degrees
                if d12 > 30.0_f64.to_radians() {
                    subdivide(vertices, v1, v2, &vm, p1, p2, pm, camera, projection, iter);
                } else {
                    // enough to stop the recursion
                    let ab = pm - p1;
                    let bc = p2 - pm;

                    let ab_u = ab.normalize();
                    let bc_u = bc.normalize();

                    let dot_abbc = crate::math::vector::dot(&ab_u, &bc_u);
                    let theta_abbc = dot_abbc.acos();

                    if theta_abbc.abs() < 5.0_f64.to_radians() {
                        let det_abbc = crate::math::vector::det(&ab_u, &bc_u);

                        if det_abbc.abs() < 1e-2 {
                            vertices.push(p1);
                            vertices.push(p2);
                        } else {
                            // not colinear but enough to stop
                            vertices.push(p1);
                            vertices.push(pm);

                            vertices.push(pm);
                            vertices.push(p2);
                        }
                    } else {
                        let ab_l = ab.magnitude2();
                        let bc_l = bc.magnitude2();

                        let r = (ab_l - bc_l).abs() / (ab_l + bc_l);

                        if r > 0.8 {
                            if ab_l < bc_l {
                                vertices.push(p1);
                                vertices.push(pm);
                            } else {
                                vertices.push(pm);
                                vertices.push(p2);
                            }
                        } else {
                            // Subdivide a->b and b->c
                            subdivide(vertices, v1, v2, &vm, p1, p2, pm, camera, projection, iter);
                        }
                    }
                }

                true
            }
            _ => false,
        }
    } else {
        false
    }
}

fn subdivide(
    vertices: &mut Vec<XYNDC<f64>>,
    v1: &XYZModel<f64>,
    v2: &XYZModel<f64>,
    vm: &XYZModel<f64>,
    p1: XYNDC<f64>,
    p2: XYNDC<f64>,
    pm: XYNDC<f64>,
    camera: &CameraViewPort,
    projection: &ProjectionType,
    iter: usize,
) {
    // Subdivide a->b and b->c
    if !project_line(vertices, v1, vm, camera, projection, iter + 1) {
        vertices.push(p1);
        vertices.push(pm);
    }

    if !project_line(vertices, vm, v2, camera, projection, iter + 1) {
        vertices.push(pm);
        vertices.push(p2);
    }
}
