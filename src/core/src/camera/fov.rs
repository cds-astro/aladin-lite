use cgmath::{Vector2, Vector4, Matrix4};

use crate::math::spherical::FieldOfViewType;

pub type NormalizedDeviceCoord = Vector2<f64>;
pub type WorldCoord = Vector4<f64>;
pub type ModelCoord = Vector4<f64>;

fn ndc_to_world<P: Projection>(
    ndc_coo: &[NormalizedDeviceCoord],
    ndc_to_clip: &Vector2<f64>,
    clip_zoom_factor: f64,
) -> Option<Vec<WorldCoord>> {
    // Deproject the FOV from ndc to the world space
    let mut world_coo = Vec::with_capacity(ndc_coo.len());

    for n in ndc_coo {
        let c = Vector2::new(
            n.x * ndc_to_clip.x * clip_zoom_factor,
            n.y * ndc_to_clip.y * clip_zoom_factor,
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
fn world_to_model(world_coo: &[WorldCoord], w2m: &Matrix4<f64>) -> Vec<ModelCoord> {
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

const NUM_VERTICES_WIDTH: usize = 10;
const NUM_VERTICES_HEIGHT: usize = 10;
const NUM_VERTICES: usize = 4 + 2 * NUM_VERTICES_WIDTH + 2 * NUM_VERTICES_HEIGHT;
// This struct belongs to the CameraViewPort
pub struct FieldOfViewVertices {
    ndc_coo: Vec<NormalizedDeviceCoord>,
    world_coo: Option<Vec<WorldCoord>>,
    model_coo: Option<Vec<ModelCoord>>,

    // Meridians and parallels contained
    // in the field of view
    great_circles: FieldOfViewType,
    //moc: [Option<HEALPixCoverage>; al_api::coo_system::NUM_COOSYSTEM],
    //depth: u8,
}
/*
fn create_coverage(vertices: &[Vector4<f64>], inside: &Vector3<f64>, camera_frame: &CooSystem, hips_frame: &CooSystem) -> HEALPixCoverage {
    let mut depth = 0;
    let mut coverage = HEALPixCoverage::new(depth, vertices, inside);

    let vertices = vertices
        .iter()
        .map(|v| crate::coosys::apply_coo_system(camera_frame, &hips_frame, v))
        .collect::<Vec<_>>();

    let inside = crate::coosys::apply_coo_system(camera_frame, &hips_frame, &inside);
    // Prefer to query from_polygon with depth >= 2
    let mut coverage = crate::healpix::coverage::HEALPixCoverage::new(
        self.depth,
        &vertices[..],
        &inside.truncate(),
    );

    while coverage.n_depth_max_cells() < 7 && depth < cdshealpix::DEPTH_MAX {
        depth += 1;
        coverage = HEALPixCoverage::new(depth, &vertices, &inside.truncate());
    }

    coverage
}*/

impl FieldOfViewVertices {
    pub fn new<P: Projection>(
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        mat: &Matrix4<f64>,
        center: &Vector4<f64>,
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

        let world_coo = ndc_to_world::<P>(&ndc_coo, ndc_to_clip, clip_zoom_factor);
        let model_coo = world_coo
            .as_ref()
            .map(|world_coo| world_to_model(world_coo, mat));


        /*let (great_circles, coverage) = if let Some(vertices) = &model_coo {
            (FieldOfViewType::new_polygon(vertices, &center), create_view_moc(vertices, &center.truncate()))
        } else {
            (FieldOfViewType::Allsky, HEALPixCoverage::allsky())
        };*/
        let great_circles = if let Some(vertices) = &model_coo {
            FieldOfViewType::new_polygon(vertices, &center)
        } else {
            FieldOfViewType::Allsky
        };
        //let depth = coverage.depth_max();
        //let mut moc = [None, None];
        //moc[*system as usize] = Some(coverage);

        FieldOfViewVertices {
            ndc_coo,
            world_coo,
            model_coo,
            great_circles,
            //moc,
            //depth
        }
    }

    pub fn set_fov<P: Projection>(
        &mut self,
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        w2m: &Matrix4<f64>,
        center: &Vector4<f64>,
    ) {
        self.world_coo = ndc_to_world::<P>(&self.ndc_coo, ndc_to_clip, clip_zoom_factor);
        self.set_rotation::<P>(w2m, center);
    }

    pub fn set_rotation<P: Projection>(
        &mut self,
        w2m: &Matrix4<f64>,
        center: &Vector4<f64>,
    ) {
        if let Some(world_coo) = &self.world_coo {
            self.model_coo = Some(world_to_model(world_coo, w2m));
        } else {
            self.model_coo = None;
        }

        self.set_great_circles::<P>(center);
    }

    fn set_great_circles<P: Projection>(&mut self, center: &Vector4<f64>) {
        if let Some(vertices) = &self.model_coo {
            self.great_circles = FieldOfViewType::new_polygon(&vertices, center);
            /*if self.great_circles.contains_both_poles() {
                self.great_circles = FieldOfViewType::Allsky;
            }*/
        } else {
            self.great_circles = FieldOfViewType::Allsky;
        }
    }

    /*pub fn get_depth(&self) -> u8 {
        self.depth
    }*/

    pub fn get_vertices(&self) -> Option<&Vec<ModelCoord>> {
        self.model_coo.as_ref()
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        self.great_circles.get_bounding_box()
    }

    /*pub fn get_coverage(&mut self, camera_frame: &CooSystem, hips_frame: &CooSystem, inside: &Vector4<f64>) -> &HEALPixCoverage {
        if self.moc[*hips_frame as usize].is_none() {
            let coverage = if let Some(vertices) = &self.model_coo {
                let vertices = vertices
                    .iter()
                    .map(|v| crate::coosys::apply_coo_system(camera_frame, &hips_frame, v))
                    .collect::<Vec<_>>();

                let inside = crate::coosys::apply_coo_system(camera_frame, &hips_frame, &inside);
                // Prefer to query from_polygon with depth >= 2
                crate::healpix::coverage::HEALPixCoverage::new(
                    self.depth,
                    &vertices[..],
                    &inside.truncate(),
                )
            } else {
                HEALPixCoverage::allsky()
            };

            self.moc[*hips_frame as usize] = Some(coverage);
        }

        self.moc[*hips_frame as usize].as_ref().unwrap()
    }*/

    pub fn _type(&self) -> &FieldOfViewType {
        &self.great_circles
    }
}
use crate::math::{projection::Projection, spherical::BoundingBox};
use std::iter;
