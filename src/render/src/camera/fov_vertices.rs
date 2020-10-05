use cgmath::{Vector2, Vector3, Vector4};
use cgmath::Matrix4;
use cgmath::SquareMatrix;


use std::collections::HashSet;
use crate::sphere_geometry::GreatCirclesInFieldOfView;

pub type NormalizedDeviceCoord = Vector2<f32>;
pub type WorldCoord = Vector4<f32>;
pub type ModelCoord = Vector4<f32>;

fn ndc_to_world<P: Projection>(
    ndc_coo: &[NormalizedDeviceCoord],
    ndc_to_clip: &Vector2<f32>,
    clip_zoom_factor: f32
) -> Option<Vec<WorldCoord>> {
    // Deproject the FOV from ndc to the world space
    let mut world_coo = Vec::with_capacity(ndc_coo.len());
    let mut out_of_fov = false;

    for n in ndc_coo {
        let c = Vector2::new(
            n.x * ndc_to_clip.x * clip_zoom_factor,
            n.y * ndc_to_clip.y * clip_zoom_factor
        );
        let w = P::clip_to_world_space(&c);
        if let Some(w) = w {
            world_coo.push(w);
        } else {
            // out of fov
            return None;
        }
    }

    Some(world_coo)
}
fn world_to_model(world_coo: &[WorldCoord], r: &Rotation<f32>) -> Vec<ModelCoord> {
    let mut model_coo = Vec::with_capacity(world_coo.len());

    for w in world_coo.iter() {
        let m = r.rotate(w);
        model_coo.push(m);
    }

    model_coo
}
use crate::renderable::angle::Angle;
const NUM_VERTICES_WIDTH: usize = 5;
const NUM_VERTICES_HEIGHT: usize = 5;
const NUM_VERTICES: usize = 4 + 2*NUM_VERTICES_WIDTH + 2*NUM_VERTICES_HEIGHT;
// This struct belongs to the CameraViewPort
pub struct FieldOfViewVertices {
    ndc_coo: Vec<NormalizedDeviceCoord>,
    world_coo: Option<Vec<WorldCoord>>,
    model_coo: Option<Vec<ModelCoord>>,
    radius: Option<Angle<f32>>,
}

use crate::Rotation;
impl FieldOfViewVertices {
    pub fn new<P: Projection>(center: &Vector4<f32>, ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &Rotation<f32>) -> Self {
        let mut x_ndc = itertools_num::linspace::<f32>(-1., 1., NUM_VERTICES_WIDTH + 2)
            .collect::<Vec<_>>();

        x_ndc.extend(iter::repeat(1_f32).take(NUM_VERTICES_HEIGHT));
        x_ndc.extend(itertools_num::linspace::<f32>(1., -1., NUM_VERTICES_WIDTH + 2));
        x_ndc.extend(iter::repeat(-1_f32).take(NUM_VERTICES_HEIGHT));

        let mut y_ndc = iter::repeat(-1_f32).take(NUM_VERTICES_WIDTH + 1)
            .collect::<Vec<_>>();

        y_ndc.extend(itertools_num::linspace::<f32>(-1., 1., NUM_VERTICES_HEIGHT + 2));
        y_ndc.extend(iter::repeat(1_f32).take(NUM_VERTICES_WIDTH));
        y_ndc.extend(itertools_num::linspace::<f32>(1., -1., NUM_VERTICES_HEIGHT + 2));
        y_ndc.pop();

        let mut ndc_coo = Vec::with_capacity(NUM_VERTICES);
        for idx_vertex in 0..NUM_VERTICES {
            ndc_coo.push(Vector2::new(
                x_ndc[idx_vertex],
                y_ndc[idx_vertex],
            ));
        }

        let world_coo = ndc_to_world::<P>(&ndc_coo, ndc_to_clip, clip_zoom_factor);
        let model_coo = if let Some(world_coo) = &world_coo {
            Some(world_to_model(world_coo, r))
        } else {
            None
        };

        let radius = None;

        let mut fov = FieldOfViewVertices {
            ndc_coo,
            world_coo,
            model_coo,
            radius
        };

        //fov.compute_radius(center);

        fov
    }

    // Recompute the camera fov vertices when the projection is changing
    pub fn set_projection<P: Projection>(&mut self, center: &Vector4<f32>, ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &Rotation<f32>) {
        self.world_coo = ndc_to_world::<P>(&self.ndc_coo, ndc_to_clip, clip_zoom_factor);
        self.model_coo = if let Some(world_coo) = &self.world_coo {
            Some(world_to_model(world_coo, r))
        } else {
            None
        };

        //self.compute_radius(center);
    }

    pub fn set_fov<P: Projection>(&mut self, center: &Vector4<f32>, ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &Rotation<f32>) {
        self.world_coo = ndc_to_world::<P>(&self.ndc_coo, ndc_to_clip, clip_zoom_factor);
        if let Some(world_coo) = &self.world_coo {
            self.model_coo = Some(world_to_model(world_coo, r));
        } else {
            self.model_coo = None;
        }

        //self.compute_radius(center);
    }

    pub fn set_rotation(&mut self, center: &Vector4<f32>, r: &Rotation<f32>) {
        if let Some(world_coo) = &self.world_coo {
            self.model_coo = Some(world_to_model(world_coo, r));
        } else {
            self.model_coo = None;
        }

        //self.compute_radius(center);
    }
    
    pub fn get_vertices(&self) -> Option<&Vec<ModelCoord>> {
        self.model_coo.as_ref()
    }

    /*pub fn get_radius(&self) -> Option<&Angle<f32>> {
        self.radius.as_ref()
    }*/
}

impl FieldOfViewVertices {
    fn compute_radius(&mut self, center: &Vector4<f32>) {
        self.radius = if let Some(model_coo) = &self.model_coo {
            crate::log("compute radius");
            Some(math::ang_between_vect(&center.truncate(), &model_coo[0].truncate()))
        } else {
            None
        };
    }
}

use std::iter;
use crate::math;

use crate::healpix_cell::HEALPixCell;

use wasm_bindgen::JsCast;
use crate::renderable::projection::Projection;
use crate::WebGl2Context;

use std::collections::HashMap;
use crate::healpix_cell;
use crate::cdshealpix;
