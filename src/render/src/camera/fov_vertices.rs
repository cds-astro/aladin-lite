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
fn world_to_model(&mut self, world_coo: &[WorldCoord], r: &SphericalRotation<f32>) -> Vec<ModelCoord> {
    let mut model_coo = Vec::with_capacity(world_coo.len());

    for w in &world_coo {
        let m = r.rotate(w);
        model_coo.push(m);
    }

    model_coo
}

const NUM_VERTICES_WIDTH: usize = 5;
const NUM_VERTICES_HEIGHT: usize = 5;
const NUM_VERTICES: usize = 4 + 2*NUM_VERTICES_WIDTH + 2*NUM_VERTICES_HEIGHT;
// This struct belongs to the CameraViewPort
pub struct FieldOfViewVertices {
    ndc_coo: Vec<NormalizedDeviceCoord>,
    world_coo: Option<Vec<WorldCoord>>,
    model_coo: Option<Vec<ModelCoord>>,
}

use crate::SphericalRotation;
impl FieldOfViewVertices {
    pub fn new<P: Projection>(ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &SphericalRotation<f32>) -> Self {
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
        let model_coo = if Some(world_coo) = world_coo {
            Some(world_to_model(world_coo, r))
        } else {
            None
        };

        FieldOfViewVertices {
            ndc_coo,
            world_coo,
            model_coo
        }
    }

    // Recompute the camera fov vertices when the projection is changing
    pub fn set_projection<P: Projection>(&mut self, ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &SphericalRotation<f32>) {
        self.world_coo = ndc_to_world::<P>(&self.ndc_coo, ndc_to_clip, clip_zoom_factor);
        self.model_coo = if let Some(world_coo) = self.world_coo {
            Some(world_to_model(world_coo, r))
        } else {
            None
        };
    }

    pub fn set_fov(&mut self, ndc_to_clip: &Vector2<f32>, clip_zoom_factor: f32, r: &SphericalRotation<f32>) {
        self.world_coo = ndc_to_world(&self.ndc_coo, ndc_to_clip, clip_zoom_factor);
        if let Some(world_coo) = self.world_coo {
            self.model_coo = world_to_model(world_coo, r);
        }
    }

    pub fn set_rotation(&mut self, r: &SphericalRotation<f32>) {
        if let Some(world_coo) = self.world_coo {
            self.model_coo = world_to_model(world_coo, r);
        }
    }
    
    pub fn get_vertices(&self) ->  Option<&[ModelCoord]> {
        self.model_coo.as_ref()
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
/*
use crate::renderable::Angle;
impl Views {
    pub fn new<P: Projection>(gl: &WebGl2Context, aperture_angle: Angle<f32>, config: &HiPSConfig) -> FieldOfView {
        //let great_circles = GreatCirclesInFieldOfView::new_allsky();

        let cells = HEALPixCells::allsky(0);

        

        let r = Matrix4::identity();

        let gl = gl.clone();
        let mut fov = FieldOfView {
            pos_ndc_space,
            pos_world_space,
            pos_model_space,
            //great_circles,

            aperture_angle,
            r,
            
            ndc_to_clip,
            clip_zoom_factor,

            current_depth,

            aspect,

            width,
            height,

            canvas,
            gl,
        };

        fov.set_aperture::<P>(aperture_angle, config);
        fov
    }

    pub fn set_image_survey<P: Projection>(&mut self, config: &HiPSConfig) {
        // Recompute the cells in the fov because the max depth of the new HiPS
        // can have changed
        self.compute_healpix_cells::<P>(config);
    }

    pub fn get_aperture(&self) -> Angle<f32> {
        self.aperture_angle
    }

    fn deproj_field_of_view<P: Projection>(&mut self, config: &HiPSConfig) {
        // Deproject the FOV from ndc to the world space
        let mut vertices_world_space = [Vector4::new(0_f32, 0_f32, 0_f32, 0_f32); NUM_VERTICES];
        let mut out_of_fov = false;

        let num_vertices = vertices_world_space.len();
        for idx_vertex in 0..num_vertices {
            let pos_ndc_space = &self.pos_ndc_space[idx_vertex];

            let pos_clip_space = Vector2::new(
                pos_ndc_space.x * self.ndc_to_clip.x * self.clip_zoom_factor,
                pos_ndc_space.y * self.ndc_to_clip.y * self.clip_zoom_factor,
            );

            let pos_world_space = P::clip_to_world_space(&pos_clip_space);
            if let Some(pos_world_space) = pos_world_space {
                vertices_world_space[idx_vertex] = pos_world_space;
            } else {
                out_of_fov = true;
                break;
            }
        }
        if out_of_fov {
            self.pos_world_space = None;
        } else {
            self.pos_world_space = Some(vertices_world_space);
        }

        // Rotate the FOV
        self.rotate::<P>(config);
    }

    pub fn set_rotation_mat<P: Projection>(&mut self, r: &Matrix4<f32>, config: &HiPSConfig) {
        self.r = *r;

        self.rotate::<P>(config);
    }

    fn rotate<P: Projection>(&mut self, config: &HiPSConfig) {
        if let Some(pos_world_space) = self.pos_world_space {
            let mut pos_model_space = [Vector4::new(0_f32, 0_f32, 0_f32, 0_f32); NUM_VERTICES];
            for idx_vertex in 0..NUM_VERTICES {
                pos_model_space[idx_vertex] = self.r * pos_world_space[idx_vertex];
            }

            self.pos_model_space = Some(pos_model_space);
            // The model vertex positions have changed
            // We compute the new polygon
            //self.great_circles = GreatCirclesInFieldOfView::new_polygon(pos_model_space.to_vec(), self.aspect);
        } else {
            // Allsky
            self.pos_model_space = None;
            //self.great_circles = GreatCirclesInFieldOfView::new_allsky();
        }

        self.compute_healpix_cells::<P>(config);
    }
    
    

    /*pub fn intersect_meridian<LonT: Into<Rad<f32>>>(&self, lon: LonT) -> bool {
        self.field_of_view.intersect_meridian(lon)
    }

    pub fn intersect_parallel<LatT: Into<Rad<f32>>>(&self, lat: LatT) -> bool {
        self.field_of_view.intersect_parallel(lat)
    }*/

    // Get the grid containing the meridians and parallels
    // that are inside the grid
    // TODO: move FieldOfViewType out of the FieldOfView, make it intern to the grid
    // The only thing to do is to recompute the grid whenever the field of view changes
    /*pub fn get_great_circles_intersecting(&self) -> &GreatCirclesInFieldOfView {
        &self.great_circles
    }*/

    // Returns the current set of HEALPix cells contained in the field of view
    pub fn healpix_cells(&self) -> &HashSet<HEALPixCell> {
        //console::log_1(&format!("healpix cells {:?}", self.cells.len()).into());
        &self.cells
    }

    // Returns the current set of HEALPix cells contained in the field of view,
    // each associated with a flag telling whether the cell is new or not.
    pub fn new_healpix_cells(&self) -> &HashMap<HEALPixCell, bool> {
        &self.new_cells
    }

    pub fn current_depth(&self) -> u8 {
        self.current_depth
    }

    pub fn get_ndc_to_clip(&self) -> &Vector2<f32> {
        &self.ndc_to_clip
    }

    pub fn get_clip_zoom_factor(&self) -> f32 {
        self.clip_zoom_factor
    }

    pub fn get_width_screen(&self) -> f32 {
        self.width
    }

    pub fn get_size_screen(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn compute_center_model_pos<P: Projection>(&self) -> Vector3<f32> {
        let center_model_pos = self.r * P::clip_to_world_space(&Vector2::new(0_f32, 0_f32)).unwrap();
        center_model_pos.truncate()
    }
}
*/