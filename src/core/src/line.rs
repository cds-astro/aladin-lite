use crate::renderable::grid::MAX_ANGLE_BEFORE_SUBDIVISION;
use crate::renderable::projection::Projection;
pub fn project<P: Projection>(
    v1: &Vector3<f64>,
    v2: &Vector3<f64>,
    camera: &CameraViewPort,
) -> Vec<Vector2<f64>> {
    // v belongs to the great circle defined by v1 and v2
    let mut v = (v1 + v2) * 0.5;
    v = v.normalize();

    let mut s_vert = vec![];

    subdivide::<P>(&mut s_vert, [v1, &v, v2], camera);

    s_vert
}
use crate::math;
use crate::CameraViewPort;
use cgmath::InnerSpace;
use cgmath::{Vector2, Vector3};
fn subdivide<P: Projection>(
    vertices: &mut Vec<Vector2<f64>>,
    mp: [&Vector3<f64>; 3],
    camera: &CameraViewPort,
) {
    // Project them. We are always facing the camera
    let a = P::model_to_screen_space(&mp[0].extend(1.0), camera);
    let b = P::model_to_screen_space(&mp[1].extend(1.0), camera);
    let c = P::model_to_screen_space(&mp[2].extend(1.0), camera);
    match (a, b, c) {
        (None, None, None) => {}
        (Some(a), Some(b), Some(c)) => {
            // Compute the angle between a->b and b->c
            let ab = b - a;
            let bc = c - b;
            let ab_l = ab.magnitude2();
            let bc_l = bc.magnitude2();

            let ab = ab.normalize();
            let bc = bc.normalize();
            let theta = math::angle(&ab, &bc);

            if theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION {
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
            } else if ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
                if ab_l == ab_l.min(bc_l) {
                    vertices.push(a);
                    vertices.push(b);
                } else {
                    vertices.push(b);
                    vertices.push(c);
                }
            } else {
                // Subdivide a->b and b->c
                subdivide::<P>(
                    vertices,
                    [mp[0], &((mp[0] + mp[1]) * 0.5).normalize(), mp[1]],
                    camera,
                );

                subdivide::<P>(
                    vertices,
                    [mp[1], &((mp[1] + mp[2]) * 0.5).normalize(), mp[2]],
                    camera,
                );
            }
        }
        (Some(_), Some(_), None) => {
            // relay the subdivision to the first half
            subdivide::<P>(
                vertices,
                [mp[0], &((mp[0] + mp[1]) * 0.5).normalize(), mp[1]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[1] + mp[2]) * 0.5;
            subdivide::<P>(
                vertices,
                [mp[1], &((mp[1] + e) * 0.5).normalize(), &e],
                camera,
            );
            //vertices.push(a);
            //vertices.push(b);
        }
        (None, Some(_), Some(_)) => {
            // relay the subdivision to the second half
            subdivide::<P>(
                vertices,
                [mp[1], &((mp[1] + mp[2]) * 0.5).normalize(), mp[2]],
                camera,
            );

            // and try subdividing a little further
            // hoping that the projection is defined for e
            let e = (mp[0] + mp[1]) * 0.5;
            subdivide::<P>(
                vertices,
                [&e, &((mp[1] + e) * 0.5).normalize(), &mp[1]],
                camera,
            );
            //vertices.push(b);
            //vertices.push(c);
        }
        _ => (),
    }
}
