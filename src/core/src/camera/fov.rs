use cgmath::{Vector2, Vector4, Matrix4};

use crate::math::projection::coo_space::{XYNDC, XYZWWorld, XYZWModel};
use crate::math::spherical::FieldOfViewType;

fn ndc_to_world(
    ndc_coo: &[XYNDC],
    ndc_to_clip: &Vector2<f64>,
    clip_zoom_factor: f64,
    projection: &ProjectionType
) -> Option<Vec<XYZWWorld>> {
    // Deproject the FOV from ndc to the world space
    let mut world_coo = Vec::with_capacity(ndc_coo.len());

    for n in ndc_coo {
        let c = Vector2::new(
            n.x * ndc_to_clip.x * clip_zoom_factor,
            n.y * ndc_to_clip.y * clip_zoom_factor,
        );
        let w = projection.clip_to_world_space(&c);
        if let Some(w) = w {
            world_coo.push(w);
        } else {
            // out of fov
            return None;
        }
    }

    Some(world_coo)
}
fn world_to_model(world_coo: &[XYZWWorld], w2m: &Matrix4<f64>) -> Vec<XYZWModel> {
    let mut model_coo = Vec::with_capacity(world_coo.len());

    for w in world_coo.iter() {
        model_coo.push(w2m * w);
    }

    model_coo
}

fn linspace(a: f64, b: f64, num: usize) -> Vec<f64> {
    let step = (b - a) / ((num - 1) as f64);
    let mut res = vec![];
    for i in 0..num {
        res.push(a + (i as f64) * step);
    }

    res
}

const NUM_VERTICES_WIDTH: usize = 4;
const NUM_VERTICES_HEIGHT: usize = 4;
const NUM_VERTICES: usize = 4 + 2 * NUM_VERTICES_WIDTH + 2 * NUM_VERTICES_HEIGHT;
// This struct belongs to the CameraViewPort
pub struct FieldOfViewVertices {
    ndc_coo: Vec<XYNDC>,
    world_coo: Option<Vec<XYZWWorld>>,
    model_coo: Option<Vec<XYZWModel>>,

    // Meridians and parallels contained
    // in the field of view
    great_circles: FieldOfViewType,
    //moc: [Option<HEALPixCoverage>; al_api::coo_system::NUM_COOSYSTEM],
    //depth: u8,
}

use crate::ProjectionType;
impl FieldOfViewVertices {
    pub fn new(
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        mat: &Matrix4<f64>,
        center: &Vector4<f64>,
        projection: &ProjectionType
    ) -> Self {
        let mut x_ndc = linspace(-1., 1., NUM_VERTICES_WIDTH + 2);

        x_ndc.extend(iter::repeat(1.0).take(NUM_VERTICES_HEIGHT));
        x_ndc.extend(linspace(1., -1., NUM_VERTICES_WIDTH + 2));
        x_ndc.extend(iter::repeat(-1.0).take(NUM_VERTICES_HEIGHT));

        let mut y_ndc = iter::repeat(-1.0)
            .take(NUM_VERTICES_WIDTH + 1)
            .collect::<Vec<_>>();

        y_ndc.extend(linspace(-1., 1., NUM_VERTICES_HEIGHT + 2));
        y_ndc.extend(iter::repeat(1.0).take(NUM_VERTICES_WIDTH));
        y_ndc.extend(linspace(1., -1., NUM_VERTICES_HEIGHT + 2));
        y_ndc.pop();

        let mut ndc_coo = Vec::with_capacity(NUM_VERTICES);
        for idx_vertex in 0..NUM_VERTICES {
            ndc_coo.push(Vector2::new(x_ndc[idx_vertex], y_ndc[idx_vertex]));
        }

        let world_coo = ndc_to_world(&ndc_coo, ndc_to_clip, clip_zoom_factor, projection);
        let model_coo = world_coo
            .as_ref()
            .map(|world_coo| world_to_model(world_coo, mat));

        let great_circles = if let Some(vertices) = &model_coo {
            FieldOfViewType::new_polygon(vertices, center)
        } else {
            FieldOfViewType::Allsky
        };

        FieldOfViewVertices {
            ndc_coo,
            world_coo,
            model_coo,
            great_circles,
        }
    }

    pub fn set_fov(
        &mut self,
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        w2m: &Matrix4<f64>,
        center: &Vector4<f64>,
        projection: &ProjectionType
    ) {
        self.world_coo = ndc_to_world(&self.ndc_coo, ndc_to_clip, clip_zoom_factor, projection);
        self.set_rotation(w2m, center);
    }

    pub fn set_rotation(
        &mut self,
        w2m: &Matrix4<f64>,
        center: &Vector4<f64>,
    ) {
        if let Some(world_coo) = &self.world_coo {
            self.model_coo = Some(world_to_model(world_coo, w2m));
        } else {
            self.model_coo = None;
        }

        self.set_great_circles(center);
    }

    fn set_great_circles(&mut self, center: &Vector4<f64>) {
        if let Some(vertices) = &self.model_coo {
            self.great_circles = FieldOfViewType::new_polygon(vertices, center);
        } else {
            self.great_circles = FieldOfViewType::Allsky;
        }
    }

    /*pub fn get_depth(&self) -> u8 {
        self.depth
    }*/

    pub fn get_vertices(&self) -> Option<&Vec<XYZWModel>> {
        self.model_coo.as_ref()
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        self.great_circles.get_bounding_box()
    }

    pub fn contains_pole(&self) -> bool {
        self.great_circles.contains_pole()
    }

    pub fn _type(&self) -> &FieldOfViewType {
        &self.great_circles
    }
}
use crate::math::{projection::Projection, spherical::BoundingBox};
use std::iter;
