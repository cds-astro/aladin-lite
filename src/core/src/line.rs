pub const MAX_ANGLE_BEFORE_SUBDIVISION: Angle<f64> = Angle(5.0 * std::f64::consts::PI / 180.0);
const MIN_LENGTH_ANGLE: f64 = 0.50;

use math::lonlat::{LonLat};
use math::projection::{ndc_to_screen_space, Projection};

pub fn project_along_longitudes_and_latitudes<P: Projection>(
    v1: &Vector3<f64>,
    v2: &Vector3<f64>,
    camera: &CameraViewPort,
    _reversed_longitude: bool,
) -> Vec<Vector2<f64>> {
    let mid = (v1 + v2).normalize().lonlat();
    let start = v1.lonlat();
    let end = v2.lonlat();

    let mut s_vert = vec![];

    subdivide_along_longitude_and_latitudes::<P>(
        &mut s_vert,
        [
            &Vector2::new(start.0 .0, start.1 .0),
            &Vector2::new(mid.0 .0, mid.1 .0),
            &Vector2::new(end.0 .0, end.1 .0),
        ],
        camera,
    );

    for ndc_vert in s_vert.iter_mut() {
        *ndc_vert = ndc_to_screen_space(&ndc_vert, camera);
    }

    s_vert
}
pub fn project_along_great_circles<P: Projection>(
    v1: &Vector3<f64>,
    v2: &Vector3<f64>,
    camera: &CameraViewPort,
) -> Vec<Vector2<f64>> {
    let mid = (v1 + v2).normalize();

    let mut s_vert = vec![];
    subdivide_along_great_circles::<P>(&mut s_vert, &[*v1, mid, *v2], camera);

    for ndc_vert in s_vert.iter_mut() {
        *ndc_vert = ndc_to_screen_space(ndc_vert, camera);
    }

    s_vert
}

use crate::ArcDeg;
const MAX_LENGTH_LINE_SEGMENT_SQUARED: f64 = 2.5e-2;
use crate::math::{self, angle::Angle};
use crate::CameraViewPort;
use cgmath::InnerSpace;
use cgmath::{Vector2, Vector3};

pub fn subdivide_along_longitude_and_latitudes<P: Projection>(
    vertices: &mut Vec<Vector2<f64>>,
    mp: [&Vector2<f64>; 3],
    camera: &CameraViewPort,
) {
    // Project them. We are always facing the camera
    let aa = math::lonlat::radec_to_xyz(Angle(mp[0].x), Angle(mp[0].y));
    let bb = math::lonlat::radec_to_xyz(Angle(mp[1].x), Angle(mp[1].y));
    let cc = math::lonlat::radec_to_xyz(Angle(mp[2].x), Angle(mp[2].y));

    let a = P::model_to_ndc_space(&aa.extend(1.0), camera);
    let b = P::model_to_ndc_space(&bb.extend(1.0), camera);
    let c = P::model_to_ndc_space(&cc.extend(1.0), camera);

    match (a, b, c) {
        (None, None, None) => (),
        (Some(a), Some(b), Some(c)) => {
            // Compute the angle between a->b and b->c
            let ab = b - a;
            let bc = c - b;
            let ab_l = ab.magnitude2();
            let bc_l = bc.magnitude2();

            if ab_l < 1e-6 || bc_l < 1e-6 {
                return;
            }

            let ab = ab.normalize();
            let bc = bc.normalize();
            let theta = math::vector::angle2(&ab, &bc);

            let vectors_nearly_colinear = theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION;
            let ndc_length_enough_small = ab_l < MAX_LENGTH_LINE_SEGMENT_SQUARED
                && bc_l < MAX_LENGTH_LINE_SEGMENT_SQUARED
                || camera.get_aperture() < ArcDeg(10.0);
            let is_vertices_near = math::vector::angle3(&aa, &bb) < ArcDeg(1.0)
                && math::vector::angle3(&bb, &cc) < ArcDeg(1.0);

            if vectors_nearly_colinear && ndc_length_enough_small {
                // Check if ab and bc are colinear
                let colinear = (ab.x * bc.y - bc.x * ab.y).abs() < 1e-2;
                if colinear {
                    vertices.push(a);
                    vertices.push(c);
                } else {
                    // not colinear
                    vertices.push(a);
                    vertices.push(b);

                    vertices.push(b);
                    vertices.push(c);
                }
            } else if is_vertices_near && ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
                if ab_l < bc_l {
                    vertices.push(a);
                    vertices.push(b);
                } else {
                    vertices.push(b);
                    vertices.push(c);
                }
            } else {
                // Subdivide a->b and b->c
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[0], &((mp[0] + mp[1]) * 0.5), &mp[1]],
                    camera,
                );

                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[1], &((mp[1] + mp[2]) * 0.5), &mp[2]],
                    camera,
                );
            }
        }
        (Some(_), Some(_), None) => {
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[0], &((mp[0] + mp[1]) * 0.5), &mp[1]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[1] + mp[2]) * 0.5;
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[1], &((mp[1] + e) * 0.5), &e],
                camera,
            );

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&e, &((mp[2] + e) * 0.5), &mp[2]],
                    camera,
                );
            }
        }
        (None, Some(_), Some(_)) => {
            // relay the subdivision to the second half
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[1], &((mp[1] + mp[2]) * 0.5), &mp[2]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[0] + mp[1]) * 0.5;
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&e, &((mp[1] + e) * 0.5), &mp[1]],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[0], &((mp[0] + e) * 0.5), &e],
                    camera,
                );
            }
        }
        (Some(_), None, Some(_)) => {
            let e = (mp[0] + mp[1]) * 0.5;
            // relay the subdivision to the second half
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[0], &((mp[0] + e) * 0.5), &e],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&e, &((mp[1] + e) * 0.5), &mp[1]],
                    camera,
                );
            }

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[1] + mp[2]) * 0.5;
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&e, &((mp[2] + e) * 0.5), &mp[2]],
                camera,
            );

            let half_angle_length_sq = (mp[2] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[1], &((mp[1] + e) * 0.5), &e],
                    camera,
                );
            }
        }
        (None, Some(_), None) => {
            let e1 = (mp[0] + mp[1]) * 0.5;
            let e2 = (mp[1] + mp[2]) * 0.5;
            // relay the subdivision to the second half
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&e1, &((e1 + mp[1]) * 0.5), &mp[1]],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[0], &((e1 + mp[0]) * 0.5), &e1],
                    camera,
                );
            }

            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[1], &((e2 + mp[1]) * 0.5), &e2],
                camera,
            );

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&e2, &((e2 + mp[2]) * 0.5), &mp[2]],
                    camera,
                );
            }
            //}
        }
        (Some(_), None, None) => {
            let e1 = (mp[0] + mp[1]) * 0.5;
            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&mp[0], &((e1 + mp[0]) * 0.5), &e1],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&e1, &((e1 + mp[1]) * 0.5), &mp[1]],
                    camera,
                );
            }
        }
        (None, None, Some(_)) => {
            let e2 = (mp[1] + mp[2]) * 0.5;

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_longitude_and_latitudes::<P>(
                    vertices,
                    [&mp[1], &((e2 + mp[1]) * 0.5), &e2],
                    camera,
                );
            }

            subdivide_along_longitude_and_latitudes::<P>(
                vertices,
                [&e2, &((e2 + mp[2]) * 0.5), &mp[2]],
                camera,
            );
        }
    }
}

pub fn subdivide_along_great_circles<P: Projection>(
    vertices: &mut Vec<Vector2<f64>>,
    mp: &[Vector3<f64>; 3],
    camera: &CameraViewPort,
) {
    // Project them. We are always facing the camera
    let mp = &[mp[0].normalize(), mp[1].normalize(), mp[2].normalize()];

    let a = P::model_to_ndc_space(&mp[0].extend(1.0), camera);
    let b = P::model_to_ndc_space(&mp[1].extend(1.0), camera);
    let c = P::model_to_ndc_space(&mp[2].extend(1.0), camera);

    match (a, b, c) {
        (None, None, None) => (),
        (Some(a), Some(b), Some(c)) => {
            // Compute the angle between a->b and b->c
            let ab = b - a;
            let bc = c - b;
            let ab_l = ab.magnitude2();
            let bc_l = bc.magnitude2();

            let ab = ab.normalize();
            let bc = bc.normalize();
            let theta = math::vector::angle2(&ab, &bc);

            let vectors_nearly_colinear = theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION;
            let ndc_length_enough_small = ab_l < MAX_LENGTH_LINE_SEGMENT_SQUARED
                && bc_l < MAX_LENGTH_LINE_SEGMENT_SQUARED
                || camera.get_aperture() < ArcDeg(10.0);
            let ndc_length_too_small = ab_l < 2e-6 || bc_l < 2e-6;

            if (vectors_nearly_colinear && ndc_length_enough_small) || ndc_length_too_small {
                // Check if ab and bc are colinear
                let colinear = (ab.x * bc.y - bc.x * ab.y).abs() < 1e-2;
                if colinear {
                    vertices.push(a);
                    vertices.push(c);
                } else {
                    vertices.push(a);
                    vertices.push(b);

                    vertices.push(b);
                    vertices.push(c);
                }
            } else if (ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1)
                && (ab_l.min(bc_l) < MAX_LENGTH_LINE_SEGMENT_SQUARED
                    || camera.get_aperture() < ArcDeg(10.0))
            {
                if ab_l < bc_l {
                    vertices.push(a);
                    vertices.push(b);
                } else {
                    vertices.push(b);
                    vertices.push(c);
                }
            } else {
                // Subdivide a->b and b->c
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[0], (mp[0] + mp[1]) * 0.5, mp[1]],
                    camera,
                );

                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[1], ((mp[1] + mp[2]) * 0.5), mp[2]],
                    camera,
                );
            }
        }
        (Some(_), Some(_), None) => {
            subdivide_along_great_circles::<P>(
                vertices,
                &[mp[0], ((mp[0] + mp[1]) * 0.5), mp[1]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[1] + mp[2]) * 0.5;
            subdivide_along_great_circles::<P>(vertices, &[mp[1], ((mp[1] + e) * 0.5), e], camera);

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[e, ((mp[2] + e) * 0.5), mp[2]],
                    camera,
                );
            }
        }
        (None, Some(_), Some(_)) => {
            // relay the subdivision to the second half
            subdivide_along_great_circles::<P>(
                vertices,
                &[mp[1], ((mp[1] + mp[2]) * 0.5), mp[2]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[0] + mp[1]) * 0.5;
            subdivide_along_great_circles::<P>(vertices, &[e, ((mp[1] + e) * 0.5), mp[1]], camera);

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[0], ((mp[0] + e) * 0.5), e],
                    camera,
                );
            }
        }
        (Some(_), None, Some(_)) => {
            let e = (mp[0] + mp[1]) * 0.5;
            // relay the subdivision to the second half
            subdivide_along_great_circles::<P>(vertices, &[mp[0], ((mp[0] + e) * 0.5), e], camera);

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[e, ((mp[1] + e) * 0.5), mp[1]],
                    camera,
                );
            }

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[1] + mp[2]) * 0.5;
            subdivide_along_great_circles::<P>(vertices, &[e, ((mp[2] + e) * 0.5), mp[2]], camera);

            let half_angle_length_sq = (mp[2] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[1], ((mp[1] + e) * 0.5), e],
                    camera,
                );
            }

            //}
        }
        (None, Some(_), None) => {
            let e1 = (mp[0] + mp[1]) * 0.5;
            let e2 = (mp[1] + mp[2]) * 0.5;
            // relay the subdivision to the second half
            subdivide_along_great_circles::<P>(
                vertices,
                &[e1, ((e1 + mp[1]) * 0.5), mp[1]],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[0], ((e1 + mp[0]) * 0.5), e1],
                    camera,
                );
            }

            subdivide_along_great_circles::<P>(
                vertices,
                &[mp[1], ((e2 + mp[1]) * 0.5), e2],
                camera,
            );

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[e2, ((e2 + mp[2]) * 0.5), mp[2]],
                    camera,
                );
            }
        }
        (Some(_), None, None) => {
            let e1 = (mp[0] + mp[1]) * 0.5;
            subdivide_along_great_circles::<P>(
                vertices,
                &[mp[0], ((e1 + mp[0]) * 0.5), e1],
                camera,
            );

            let half_angle_length_sq = (mp[0] - mp[1]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[e1, ((e1 + mp[1]) * 0.5), mp[1]],
                    camera,
                );
            }
        }
        (None, None, Some(_)) => {
            let e2 = (mp[1] + mp[2]) * 0.5;

            let half_angle_length_sq = (mp[1] - mp[2]).magnitude2();
            if half_angle_length_sq > MIN_LENGTH_ANGLE {
                subdivide_along_great_circles::<P>(
                    vertices,
                    &[mp[1], ((e2 + mp[1]) * 0.5), e2],
                    camera,
                );
            }

            subdivide_along_great_circles::<P>(
                vertices,
                &[e2, ((e2 + mp[2]) * 0.5), mp[2]],
                camera,
            );
        }
    }
}
