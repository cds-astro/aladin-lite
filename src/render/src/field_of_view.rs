use cgmath::{Vector2, Vector3, Vector4};
use cgmath::Matrix4;
use cgmath::SquareMatrix;

const NUM_VERTICES_WIDTH: usize = 5;
const NUM_VERTICES_HEIGHT: usize = 5;
const NUM_VERTICES: usize = 4 + 2*NUM_VERTICES_WIDTH + 2*NUM_VERTICES_HEIGHT;


use std::collections::HashSet;
use crate::buffer::HiPSConfig;
use crate::sphere_geometry::GreatCirclesInFieldOfView;
pub struct FieldOfView {
    pos_ndc_space: [Vector2<f32>; NUM_VERTICES],
    pos_world_space: Option<[Vector4<f32>; NUM_VERTICES]>,
    pos_model_space: Option<[Vector4<f32>; NUM_VERTICES]>,
    // A polygon reprensenting the current field of view
    // (containing the vertices lon, lat in model space)
    great_circles: GreatCirclesInFieldOfView,

    aperture_angle: Angle<f32>, // fov can be None if the camera is out of the projection
    r: Matrix4<f32>, // Rotation matrix of the FOV (i.e. same as the HiPS sphere model matrix)

    ndc_to_clip: Vector2<f32>,
    clip_zoom_factor: f32,

    // The set of cells being in the current field of view
    cells: HashSet<HEALPixCell>,
    // A map describing the cells in the current field of view
    // A boolean is associated with the cells telling if the
    // cell is new (meaning it was not in the previous field of view).
    // ``cells`` is always equal to its keys!
    new_cells: HashMap<HEALPixCell, bool>,
    is_there_new_cells: bool,
    // The current depth of the cells in the field of view
    current_depth: u8,

    // The width over height ratio
    aspect: f32,

    // The width of the screen in pixels
    width: f32,
    // The height of the screen in pixels
    height: f32,

    // Canvas HtmlElement
    canvas: web_sys::HtmlCanvasElement,

    // WebGL2 context
    gl: WebGl2Context,
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
impl FieldOfView {
    pub fn new<P: Projection>(gl: &WebGl2Context, aperture_angle: Angle<f32>, config: &HiPSConfig) -> FieldOfView {
        let mut x_ndc_space = itertools_num::linspace::<f32>(-1., 1., NUM_VERTICES_WIDTH + 2)
            .collect::<Vec<_>>();

        x_ndc_space.extend(iter::repeat(1_f32).take(NUM_VERTICES_HEIGHT));
        x_ndc_space.extend(itertools_num::linspace::<f32>(1., -1., NUM_VERTICES_WIDTH + 2));
        x_ndc_space.extend(iter::repeat(-1_f32).take(NUM_VERTICES_HEIGHT));

        let mut y_ndc_space = iter::repeat(-1_f32).take(NUM_VERTICES_WIDTH + 1)
            .collect::<Vec<_>>();

        y_ndc_space.extend(itertools_num::linspace::<f32>(-1., 1., NUM_VERTICES_HEIGHT + 2));
        y_ndc_space.extend(iter::repeat(1_f32).take(NUM_VERTICES_WIDTH));
        y_ndc_space.extend(itertools_num::linspace::<f32>(1., -1., NUM_VERTICES_HEIGHT + 2));
        y_ndc_space.pop();

        let mut pos_ndc_space = [Vector2::new(0_f32, 0_f32); NUM_VERTICES];
        for idx_vertex in 0..NUM_VERTICES {
            pos_ndc_space[idx_vertex] = Vector2::new(
                x_ndc_space[idx_vertex],
                y_ndc_space[idx_vertex],
            );
        }
        
        let pos_world_space = None;
        let pos_model_space = None;
        let great_circles = GreatCirclesInFieldOfView::new_allsky();

        let cells = healpix_cell::allsky(0);
        let new_cells = cells.iter()
            .cloned()
            .map(|cell| (cell, true))
            .collect();
        let is_there_new_cells = true;

        let width = web_sys::window()
            .unwrap()
            .inner_width()
            .unwrap()
            .as_f64()
            .unwrap() as f32;
        let height = web_sys::window()
            .unwrap()
            .inner_height()
            .unwrap()
            .as_f64()
            .unwrap() as f32;

        let aspect = width / height;
        let ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);
        let clip_zoom_factor = 0_f32;
        let current_depth = 0;

        // Canvas definition
        let canvas = gl.canvas().unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        gl.viewport(0, 0, width as i32, height as i32);
        gl.scissor(0, 0, width as i32, height as i32);

        let r = Matrix4::identity();

        let gl = gl.clone();
        let mut fov = FieldOfView {
            pos_ndc_space,
            pos_world_space,
            pos_model_space,
            great_circles,

            aperture_angle,
            r,
            
            ndc_to_clip,
            clip_zoom_factor,

            cells,
            new_cells,
            is_there_new_cells,
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

    pub fn resize_window<P: Projection>(&mut self, width: f32, height: f32, config: &HiPSConfig) {
        self.width = width;
        self.height = height;

        self.aspect = width / height;

        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.gl.scissor(0, 0, width as i32, height as i32);

        // Compute the new clip zoom factor
        self.ndc_to_clip = P::compute_ndc_to_clip_factor(width, height);

        self.deproj_field_of_view::<P>(config);
    }

    fn compute_clip_zoom_factor<P: Projection>(fov: Angle<f32>) -> f32 {
        let lon = fov.abs() / 2_f32;

        // Vertex in the WCS of the FOV
        let v0 = math::radec_to_xyzw(lon, Angle(0_f32));

        // Project this vertex into the screen
        let p0 = P::world_to_clip_space(&v0);
        p0.x.abs()
    }

    pub fn set_aperture<P: Projection>(&mut self, angle: Angle<f32>, config: &HiPSConfig) {
        self.aperture_angle = angle;
        // Compute the new clip zoom factor
        self.clip_zoom_factor = Self::compute_clip_zoom_factor::<P>(self.aperture_angle);
        
        self.deproj_field_of_view::<P>(config);
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
            self.great_circles = GreatCirclesInFieldOfView::new_polygon(pos_model_space.to_vec(), self.aspect);
        } else {
            // Allsky
            self.pos_model_space = None;
            self.great_circles = GreatCirclesInFieldOfView::new_allsky();
        }

        self.compute_healpix_cells::<P>(config);
    }
    
    fn compute_healpix_cells<P: Projection>(&mut self, config: &HiPSConfig) {
        // Compute the depth corresponding to the angular resolution of a pixel
        // along the width of the screen
        let max_depth = config
            // Max depth of the current HiPS tiles
            .max_depth();
        
        let depth = std::cmp::min(
            math::fov_to_depth(self.aperture_angle, self.width, &config),
            max_depth,
        );
        //console::log_1(&format!("max depth {:?}", max_depth).into());

        let cells = self.get_cells_in_fov::<P>(depth);

        // Look for the newly added cells in the field of view
        // by doing the difference of the new cells set with the previous one
        let new_cells = cells.difference(&self.cells).collect::<HashSet<_>>();
        self.is_there_new_cells = !new_cells.is_empty();
        self.new_cells = cells.iter().cloned()
            .map(|cell| {
                (cell, new_cells.contains(&cell))
            })
            .collect::<HashMap<_, _>>();
        self.cells = cells;
        self.current_depth = depth;
    }

    pub fn get_cells_in_fov<P: Projection>(&self, depth: u8) -> HashSet<HEALPixCell> {
        if let Some(pos_model_space) = self.pos_model_space {
            self.polygon_coverage::<P>(depth, &pos_model_space)
        } else {
            crate::healpix_cell::allsky(depth)
        }
    }

    fn polygon_coverage<P: Projection>(&self, depth: u8, vertices: &[Vector4<f32>; NUM_VERTICES]) -> HashSet<HEALPixCell> {
        let inside = self.compute_center_model_pos::<P>();
        let moc = cdshealpix::HEALPixCoverage::new(depth, vertices as &[Vector4<f32>], &inside);

        let cells: HashSet<HEALPixCell> = moc.flat_iter()
            .map(|idx| {
                HEALPixCell(depth, idx)
            })
            .collect();
        cells
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
    pub fn get_great_circles_intersecting(&self) -> &GreatCirclesInFieldOfView {
        &self.great_circles
    }

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
