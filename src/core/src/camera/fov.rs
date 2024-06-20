use cgmath::{Matrix4, Vector2};

use crate::math::projection::coo_space::{XYZWModel, XYZWWorld, XYNDC};

use crate::math::sph_geom::region::{Intersection, PoleContained, Region};
use crate::math::{projection::Projection, sph_geom::bbox::BoundingBox};
use crate::LonLatT;

use crate::ProjectionType;
use std::iter;

fn ndc_to_world(
    ndc_coo: &[XYNDC<f64>],
    ndc_to_clip: &Vector2<f64>,
    clip_zoom_factor: f64,
    projection: &ProjectionType,
) -> Option<Vec<XYZWWorld<f64>>> {
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
fn world_to_model(world_coo: &[XYZWWorld<f64>], w2m: &Matrix4<f64>) -> Vec<XYZWModel<f64>> {
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

const NUM_VERTICES_WIDTH: usize = 3;
const NUM_VERTICES_HEIGHT: usize = 3;
const NUM_VERTICES: usize = 4 + 2 * NUM_VERTICES_WIDTH + 2 * NUM_VERTICES_HEIGHT;
// This struct belongs to the CameraViewPort
pub struct FieldOfView {
    // Vertices
    ndc_vertices: Vec<XYNDC<f64>>,
    world_vertices: Option<Vec<XYZWWorld<f64>>>,
    model_vertices: Option<Vec<XYZWModel<f64>>>,

    reg: Region,
}

impl FieldOfView {
    pub fn new(
        // ndc to clip parameters
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        // rotation
        rotation_mat: &Matrix4<f64>,
        // projection
        projection: &ProjectionType,
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

        let mut ndc_vertices = Vec::with_capacity(NUM_VERTICES);
        for idx_vertex in 0..NUM_VERTICES {
            ndc_vertices.push(Vector2::new(x_ndc[idx_vertex], y_ndc[idx_vertex]));
        }

        let world_vertices = ndc_to_world(&ndc_vertices, ndc_to_clip, clip_zoom_factor, projection);
        let model_vertices = world_vertices
            .as_ref()
            .map(|world_vertex| world_to_model(world_vertex, rotation_mat));

        let reg = if let Some(vertices) = &model_vertices {
            Region::from_vertices(vertices, &rotation_mat.z)
        } else {
            Region::AllSky
        };

        // Allsky case
        FieldOfView {
            ndc_vertices,
            world_vertices,
            model_vertices,

            reg,
        }
    }

    // Update the vertices
    pub fn set_aperture(
        &mut self,
        ndc_to_clip: &Vector2<f64>,
        clip_zoom_factor: f64,
        rotate_mat: &Matrix4<f64>,
        projection: &ProjectionType,
    ) {
        self.world_vertices = ndc_to_world(
            &self.ndc_vertices,
            ndc_to_clip,
            clip_zoom_factor,
            projection,
        );
        self.set_rotation(rotate_mat);
    }

    pub fn set_rotation(&mut self, rotate_mat: &Matrix4<f64>) {
        if let Some(world_vertices) = &self.world_vertices {
            self.model_vertices = Some(world_to_model(world_vertices, rotate_mat));
        } else {
            self.model_vertices = None;
        }

        if let Some(vertices) = &self.model_vertices {
            self.reg = Region::from_vertices(vertices, &rotate_mat.z);
        } else {
            self.reg = Region::AllSky;
        }
    }

    // Interface over the region object
    pub fn contains(&self, lonlat: &LonLatT<f64>) -> bool {
        self.reg.contains(lonlat)
    }

    pub fn intersects_parallel(&self, lat: f64) -> Intersection {
        self.reg.intersects_parallel(lat)
    }

    pub fn intersects_meridian(&self, lon: f64) -> Intersection {
        self.reg.intersects_meridian(lon)
    }

    /*pub fn intersects_great_circle(&self, n: &Vector3<f64>) -> Intersection {
        self.reg.intersects_great_circle(n)
    }*/

    pub fn intersects_great_circle_arc(
        &self,
        lonlat1: &LonLatT<f64>,
        lonlat2: &LonLatT<f64>,
    ) -> Intersection {
        self.reg.intersects_great_circle_arc(lonlat1, lonlat2)
    }

    // Accessors
    pub fn get_bounding_box(&self) -> &BoundingBox {
        match &self.reg {
            Region::AllSky => &crate::math::sph_geom::bbox::ALLSKY_BBOX,
            Region::Polygon { bbox, .. } => bbox,
        }
    }

    pub fn get_vertices(&self) -> Option<&Vec<XYZWModel<f64>>> {
        self.model_vertices.as_ref()
    }

    pub fn is_intersecting_zero_meridian(&self) -> bool {
        match &self.reg {
            Region::AllSky => true,
            Region::Polygon {
                is_intersecting_zero_meridian,
                ..
            } => *is_intersecting_zero_meridian,
        }
    }

    pub fn is_allsky(&self) -> bool {
        matches!(self.reg, Region::AllSky)
    }

    pub fn contains_pole(&self) -> bool {
        match &self.reg {
            Region::AllSky => true,
            Region::Polygon { poles, .. } => *poles != PoleContained::None,
        }
    }

    pub fn contains_north_pole(&self) -> bool {
        match &self.reg {
            Region::AllSky => true,
            Region::Polygon { poles, .. } => {
                *poles == PoleContained::North || *poles == PoleContained::Both
            }
        }
    }

    pub fn contains_south_pole(&self) -> bool {
        match &self.reg {
            Region::AllSky => true,
            Region::Polygon { poles, .. } => {
                *poles == PoleContained::South || *poles == PoleContained::Both
            }
        }
    }

    pub fn contains_both_poles(&self) -> bool {
        match &self.reg {
            Region::AllSky => true,
            Region::Polygon { poles, .. } => *poles == PoleContained::Both,
        }
    }
}
