use cgmath::{Vector2, Vector3, Vector4};
use cgmath::Matrix4;
use cgmath::SquareMatrix;


use std::collections::HashSet;
use crate::sphere_geometry::GreatCirclesInFieldOfView;

struct HEALPixCells {
    pub depth: u8,
    pub cells: HashSet<HEALPixCell>,
}
use std::collections::hash_set::Iter;
struct HEALPixCellsIter(Iter<HEALPixCell>);

impl Iterator for HEALPixCellsIter {
    type Item = &HEALPixCell;
    
    fn next(&mut self) -> Option<&HEALPixCell> {
        self.0.next()
    }
}

impl HEALPixCells {
    fn contains(&self, cell: &HEALPixCell) -> bool {
        self.contains(cell)
    }

    fn allsky(depth: u8) -> HEALPixCells {
        let npix = 12 << ((depth as usize) << 1);

        let mut cells = (0_u64..(npix as u64))
            .map(|pix| HEALPixCell(depth, ipix))
            .collect::<HashSet<_>>();

        HEALPixCells {
            depth,
            cells
        }
    }

    fn iter(&self) ->  HEALPixCellsIter {
        HEALPixCellsIter(self.cells.iter())
    }
}

struct NewHEALPixCells {
    depth: u8,
    // flags associating true to cells that
    // are new in the fov
    flags: HashMap<HEALPixCell, bool>,
    // A flag telling whether there has been
    // new cells added from the last frame
    new_cells_added: bool,
};

impl NewHEALPixCells {
    fn new(cells: &HEALPixCells) -> NewHEALPixCells {
        let depth = cells.depth;
        let flags = cells.iter()
            .cloned()
            .map(|cell| {
                (cell, true)
            })
            .collect::<HashMap<_, _>>();

        let new_cells_added = true;
        NewHEALPixCells {
            depth,
            flags
            new_cells_added
        }
    }

    fn insert_new_cells(self, cells: &HEALPixCells) -> NewHEALPixCells {
        let mut new_cells_added = false;
        let new_depth = cells.depth;
        let flags = if new_cells != self.depth {
            new_cells_added = true;
            // Change of depth => all cells are new
            cells.iter()
                .cloned()
                .map(|cell| {
                    (cell, true)
                })
                .collect::<HashMap<_, _>>()
        } else {
            cells.iter()
                .cloned()
                .map(|cell| {
                    let new = if let Some(found) = self.flags.get(cell) {
                        // It is found
                        found
                    } else {
                        // It is not found in the previous hash map, so it is a new cell
                        true
                    };
                    
                    new_cells_added |= new;
                    
                    (cell, new)
                })
                .collect::<HashMap<_, _>>()
        };

        NewHEALPixCells {
            depth: new_depth,
            flags,
            new_cells_added
        }
    }

    #[inline]
    fn is_there_new_cells_added(&self) -> bool {
        self.new_cells_added
    }
}

fn get_current_cells_from_fov(survey: &ImageSurvey, viewport: &FieldOfViewVertices) -> HEALPixCells {
    // Compute the depth corresponding to the angular resolution of a pixel
    // along the width of the screen
    let depth = viewport.get_depth_from_survey(survey);

    let cells = if let Some(vertices) = viewport.get() {
        polygon_coverage(depth, vertices)
    } else {
        crate::healpix_cell::allsky(depth)
    };

    HEALPixCells {
        depth,
        cells
    }
}

fn polygon_coverage(
    vertices: &[Vector4<f32>],
    depth: u8,
    inside: &Vector4<f32>
) -> HEALPixCells {
    let coverage = cdshealpix::HEALPixCoverage::new(depth, vertices, &inside);

    coverage.flat_iter()
        .map(|idx| {
            HEALPixCell(depth, idx)
        })
        .collect()
}

// Contains the cells being in the FOV for a specific
// image survey
// This keep traces of the new cells to download for an image survey
struct ViewOnImageSurvey {
    idx_survey: usize,
    // The set of cells being in the current view for a
    // specific image survey
    cells: HEALPixCells,
    new_cells: NewHEALPixCells,
}

impl ViewOnImageSurvey {
    fn create(survey: &ImageSurvey, viewport: &Viewport) -> ViewOnImageSurvey {
        let idx_survey = survey.get_idx();

        let cells = get_current_cells_from_fov(survey, viewport);
        let new_cells = NewHEALPixCells::new(&cells);

        ViewOnImageSurvey {
            idx_survey,
            cells,
            new_cells,
        }
    }

    // This method is called whenever the user does an action
    // that moves the viewport
    fn update(&mut self, surveys: &[ImageSurvey], viewport: &Viewport) {
        let cells = {
            let survey = surveys[self.idx_survey];
            get_current_cells_from_fov(survey, viewport)
        };

        self.new_cells.insert_new_cells(&cells);
        self.cells = cells;
    }

}

type NormalizedDeviceCoord = Vector2<f32>;
type WorldCoord = Vector4<f32>;
type ModelCoord = Vector4<f32>;

fn ndc_to_world<P: Projection>(
    ndc_coo: &[NormalizedDeviceCoord],
    viewport: &Viewport
) -> Option<Vec<WorldCoord>> {
    // Deproject the FOV from ndc to the world space
    let mut world_coo = Vec::with_capacity(ndc_coo.len());
    let mut out_of_fov = false;

    for n in ndc_coo {
        let c = crate::projection::ndc_to_clip_space(n, viewport);

        let w = P::ndc_to_world(&c);
        if let Some(w) = w {
            world_coo.push(w);
        } else {
            // out of fov
            return None;
        }
    }

    Some(world_coo)
}
fn world_to_model(&mut self, world_coo: &[WorldCoord], viewport: &Viewport) -> Vec<ModelCoord> {
    let mut model_coo = Vec::with_capacity(world_coo.len());
    
    let r = viewport.get_rotation();

    for w in &world_coo {
        let m = r.rotate(w);
        model_coo.push(m);
    }

    model_coo
}

const NUM_VERTICES_WIDTH: usize = 5;
const NUM_VERTICES_HEIGHT: usize = 5;
const NUM_VERTICES: usize = 4 + 2*NUM_VERTICES_WIDTH + 2*NUM_VERTICES_HEIGHT;
// Viewport vertices in model space
// i.e. after the unprojection + rotation of the sphere
struct FieldOfViewVertices {
    ndc_coo: Vec<NormalizedDeviceCoord>,
    world_coo: Option<Vec<WorldCoord>>,
    model_coo: Option<Vec<ModelCoord>>,
}

impl FieldOfViewVertices {
    fn new<P: Projection>(viewport: &Viewport) -> Self {
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

        let world_coo = ndc_to_world(&self.ndc_coo, viewport);
        let model_coo = if Some(world_coo) = world_coo {
            Some(world_to_model(world_coo, viewport))
        } else {
            None
        };

        FieldOfViewVertices {
            ndc_coo,
            world_coo,
            model_coo
        }
    }

    pub fn zoom(&mut self, viewport: &ViewPort) {
        self.world_coo = ndc_to_world(&self.ndc_coo, viewport);
        if Some(world_coo) = self.world_coo {
            self.model_coo = world_to_model(world_coo, viewport);
        }
    }

    pub fn move(&mut self, viewport: &ViewPort) {
        if Some(world_coo) = self.world_coo {
            self.model_coo = world_to_model(world_coo, viewport);
        }
    }
    
    pub fn get_vertices(&self) ->  Option<&[ModelCoord]> {
        self.model_coo.as_ref()
    }
}

pub struct FieldOfView {
    vertices: FieldOfViewVertices,

    // A polygon reprensenting the current field of view
    // (containing the vertices lon, lat in model space)
    //great_circles: GreatCirclesInFieldOfView,

    //aperture_angle: Angle<f32>, // fov can be None if the camera is out of the projection
    //r: Matrix4<f32>, // Rotation matrix of the FOV (i.e. same as the HiPS sphere model matrix)

    views: Vec<ViewOnImageSurvey>,

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
