use crate::buffer::Texture;
use crate::healpix_cell::HEALPixCell;
pub struct TextureToDraw<'a> {
    pub starting_texture: &'a Texture,
    pub ending_texture: &'a Texture,
}

impl<'a> TextureToDraw<'a> {
    fn new(starting_texture: &'a Texture, ending_texture: &'a Texture) -> TextureToDraw<'a> {
        TextureToDraw {
            starting_texture,
            ending_texture
        }
    }
}

use std::collections::{HashMap, HashSet};
pub struct TexturesToDraw<'a>(HashMap<HEALPixCell, TextureToDraw<'a>>);

impl<'a> TexturesToDraw<'a> {
    fn new(cap: usize) -> TexturesToDraw<'a> {
        let states = HashMap::with_capacity(cap);

        TexturesToDraw(states)
    }
}

impl<'a> core::ops::Deref for TexturesToDraw<'a> {
    type Target = HashMap<HEALPixCell, TextureToDraw<'a>>;

    fn deref (self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for TexturesToDraw<'a> {
    fn deref_mut (self: &'_  mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

use crate::healpix_cell::SphereSubdivided;
pub trait RecomputeRasterizer {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        cells_to_draw: &HEALPixCells,
        // The survey from which we get the textures to plot
        // Usually it is the most refined survey
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a>;

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8;
}

pub struct Move;
pub struct Zoom;
pub struct UnZoom;

impl RecomputeRasterizer for Move  {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let mut textures = TexturesToDraw::new(cells_to_draw.len());

        for cell in cells_to_draw.iter() {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }
    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        sphere_sub.get_num_subdivide::<P>(cell)
    }
}

impl RecomputeRasterizer for Zoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let mut textures = TexturesToDraw::new(cells_to_draw.len());

        for cell in cells_to_draw.iter() {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        sphere_sub.get_num_subdivide::<P>(cell)
    }
}

impl RecomputeRasterizer for UnZoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let mut textures = TexturesToDraw::new(cells_to_draw.len());

        for cell in cells_to_draw.iter() {
            let parent_cell = cell.parent();

            if survey.contains(&parent_cell) {
                let starting_cell = if survey.contains(&cell) {
                    *cell
                } else {
                    survey.get_nearest_parent(&parent_cell)
                };
                let starting_cell_in_tex = survey.get(&starting_cell).unwrap();
                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let starting_cell = if survey.contains(&cell) {
                    *cell
                } else {
                    survey.get_nearest_parent(&parent_cell)
                };

                let ending_cell = starting_cell;

                let starting_cell_in_tex = survey.get(&starting_cell).unwrap();
                let ending_cell_in_tex = survey.get(&ending_cell).unwrap();

                textures.insert(*cell, TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        let num_subdivision = sphere_sub.get_num_subdivide::<P>(cell);
        if num_subdivision <= 1 {
            0
        } else {
            num_subdivision - 1
        }
    }
}

use crate::camera::CameraViewPort;
use crate::WebGl2Context;

use crate::renderable::projection::Projection;

use crate::buffer::ImageSurveyTextures;
use crate::renderable::RayTracer;
use crate::renderable::Rasterizer;
use crate::shaders::Colormap;

trait Draw {
    fn draw<P: Projection>(&mut self, raster: &Rasterizer, raytracer: &RayTracer, shaders: &mut ShaderManager, camera: &CameraViewPort);
}

#[derive(Clone, Copy)]
pub struct GrayscaleParameter {
    h: TransferFunction,
    min_value: f32,
    max_value: f32,

    scale: f32,
    offset: f32,
    blank: f32,
}

use crate::shader::{Shader, ShaderBound};
impl SendUniforms for GrayscaleParameter {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.h)
            .attach_uniform("min_value", &self.min_value)
            .attach_uniform("max_value", &self.max_value)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);

        shader
    }
}

/// List of the different type of surveys
#[derive(Clone, Copy)]
pub enum Color {
    Colored,
    Grayscale2Colormap {
        colormap: Colormap,
        param: GrayscaleParameter,
    },
    Grayscale2Color {
        // A color associated to the component
        color: [f32; 3],
        k: f32, // factor controlling the amount of this HiPS
        param: GrayscaleParameter,
    }
}

impl Color {
    pub fn get_raster_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raster_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                P::get_raster_shader_gray2colormap(gl, shaders)
            },
            Color::Grayscale2Color { .. } => {
                P::get_raster_shader_gray2color(gl, shaders)
            },
        }
    }

    pub fn get_raytracer_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raytracer_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                P::get_raytracer_shader_gray2colormap(gl, shaders)
            },
            Color::Grayscale2Color { .. } => {
                P::get_raytracer_shader_gray2color(gl, shaders)
            },
        }
    }
}

use crate::shader::SendUniforms;
impl SendUniforms for Color {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        match self {
            Color::Colored => (),
            Color::Grayscale2Colormap { colormap, param } => {
                shader
                    .attach_uniforms_from(colormap)
                    .attach_uniforms_from(param);
            },
            Color::Grayscale2Color { color, k, param } => {
                shader
                    .attach_uniforms_from(param)
                    .attach_uniform("C", color)
                    .attach_uniform("K", k);
            }
        }

        shader
    }
}

// Compute the size of the VBO in bytes
// We do want to draw maximum 768 tiles
const MAX_NUM_CELLS_TO_DRAW: usize = 768;
// Each cell has 4 vertices
pub const MAX_NUM_VERTICES_TO_DRAW: usize = MAX_NUM_CELLS_TO_DRAW * 4;
// There is 12 floats per vertices (lonlat, pos, uv_start, uv_end, time_start) = 2 + 3 + 3 + 3 + 1 = 12
const MAX_NUM_FLOATS_TO_DRAW: usize = MAX_NUM_VERTICES_TO_DRAW * 12;
const MAX_NUM_INDICES_TO_DRAW: usize = MAX_NUM_CELLS_TO_DRAW * 6;

#[derive(Clone, Copy)]
enum ImageSurveyType {
    Simple,
    Component
}

#[repr(C)]
struct Position {
    lon: Angle<f32>,
    lat: Angle<f32>,

    pos: Vector3<f32>,
}

impl Position {
    fn new(lonlat: &LonLatT<f32>) -> Position {
        let pos = lonlat.vector();
        let lon = lonlat.lon();
        let lat = lonlat.lat();
        Position {
            lon,
            lat,

            pos,
        }
    }

    fn add_to_positions(&self, positions: &mut Vec<f32>) {
        positions.push(self.lon.0);
        positions.push(self.lat.0);

        positions.push(self.pos.x);
        positions.push(self.pos.y);
        positions.push(self.pos.z);
    }
}

use cgmath::Vector3;
#[repr(C)]
struct Vertex {
    lon: Angle<f32>,
    lat: Angle<f32>,

    pos: Vector3<f32>,

    uv_0: Vector3<f32>,
    uv_1: Vector3<f32>,

    time_received: f32,
}

use math::LonLatT;
impl Vertex {
    #[inline]
    fn _size_of_float() -> usize {
        std::mem::size_of::<Self>() / std::mem::size_of::<f32>()
    }

    fn new(
        lonlat: &LonLatT<f32>,
        uv_0: Vector3<f32>,
        uv_1: Vector3<f32>,
        time_received: f32
    ) -> Vertex {
        let pos = lonlat.vector();
        let lon = lonlat.lon();
        let lat = lonlat.lat();
        Vertex {
            lon,
            lat,

            pos,

            uv_0,
            uv_1,

            time_received,
        }
    }

    fn add_to_vertices(&self, vertices: &mut Vec<f32>) {
        //assert!(off + 12 <= 30000);
        vertices.push(self.lon.0);
        vertices.push(self.lat.0);

        vertices.push(self.pos.x);
        vertices.push(self.pos.y);
        vertices.push(self.pos.z);

        vertices.push(self.uv_0.x);
        vertices.push(self.uv_0.y);
        vertices.push(self.uv_0.z);

        vertices.push(self.uv_1.x);
        vertices.push(self.uv_1.y);
        vertices.push(self.uv_1.z);

        vertices.push(self.time_received);
    }
}
// One tile contains 2 triangles of 3 vertices each
//#[repr(C)]
//struct TileVertices([Vertex; 6]);

use crate::math;
use std::mem;

use crate::renderable::uv::{TileUVW, TileCorner};
use crate::time::Time;

pub type LonLatVec = Vec<f32>;
pub type PositionVec = Vec<f32>;
pub type UVStartVec = Vec<f32>;
pub type UVEndVec = Vec<f32>;
pub type StartAnimTimeVec = Vec<f32>;

pub type IdxVerticesVec = Vec<u16>;

// This method only computes the vertex positions
// of a HEALPix cell and append them
// to lonlats and positions vectors
fn add_positions_grid<P: Projection, E: RecomputeRasterizer>(
    lonlats: &mut LonLatVec,
    positions: &mut PositionVec,
    idx_positions: &mut IdxVerticesVec,
    cell: &HEALPixCell,
    sphere_sub: &SphereSubdivided,
) {
    let num_subdivision = E::num_subdivision::<P>(cell, sphere_sub);

    let n_segments_by_side: u16 = 1_u16 << num_subdivision;
    let lonlat = cdshealpix::grid_lonlat::<f32>(cell, n_segments_by_side);

    let n_vertices_per_segment = n_segments_by_side + 1;

    let off_idx_vertices = (positions.len()/3) as u16;
    for i in 0..n_vertices_per_segment {
        for j in 0..n_vertices_per_segment {
            let id_vertex_0 = (j + i * n_vertices_per_segment) as usize;

            let (lon, lat) = (lonlat[id_vertex_0].lon().0, lonlat[id_vertex_0].lat().0);
            let position: Vector3<f32> = lonlat[id_vertex_0].vector();

            lonlats.push(lon);
            lonlats.push(lat);

            positions.push(position.x);
            positions.push(position.y);
            positions.push(position.z);
        }
    }

    for i in 0..n_segments_by_side {
        for j in 0..n_segments_by_side {
            let idx_0 = (j + i * n_vertices_per_segment) as u16;
            let idx_1 = (j + 1 + i * n_vertices_per_segment) as u16;
            let idx_2 = (j + (i + 1) * n_vertices_per_segment) as u16;
            let idx_3 = (j + 1 + (i + 1) * n_vertices_per_segment) as u16;

            idx_positions.push(off_idx_vertices + idx_0);
            idx_positions.push(off_idx_vertices + idx_1);
            idx_positions.push(off_idx_vertices + idx_2);

            idx_positions.push(off_idx_vertices + idx_1);
            idx_positions.push(off_idx_vertices + idx_3);
            idx_positions.push(off_idx_vertices + idx_2);
        }
    }
}

// This method computes positions and UVs of a healpix cells
use crate::cdshealpix;
fn add_uv_grid<P: Projection, E: RecomputeRasterizer>(
    uv_start: &mut UVStartVec,
    uv_end: &mut UVEndVec,
    start_time: &mut StartAnimTimeVec,

    cell: &HEALPixCell,
    sphere_sub: &SphereSubdivided,

    uv_0: &TileUVW,
    uv_1: &TileUVW,
    alpha: f32
) {
    let num_subdivision = E::num_subdivision::<P>(cell, sphere_sub);
    let n_segments_by_side: u16 = 1_u16 << num_subdivision;

    let n_vertices_per_segment = n_segments_by_side + 1;

    for i in 0..n_vertices_per_segment {
        for j in 0..n_vertices_per_segment {

            let hj0 = (j as f32) / (n_segments_by_side as f32);
            let hi0 = (i as f32) / (n_segments_by_side as f32);

            let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
            let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;

            let uv_s_vertex_0 = Vector3::new(
                uv_0[TileCorner::BottomLeft].x + hj0 * d01s,
                uv_0[TileCorner::BottomLeft].y + hi0 * d02s,
                uv_0[TileCorner::BottomLeft].z
            );

            let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
            let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;
            let uv_e_vertex_0 = Vector3::new(
                uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                uv_1[TileCorner::BottomLeft].z
            );

            uv_start.push(uv_s_vertex_0.x);
            uv_start.push(uv_s_vertex_0.y);
            uv_start.push(uv_s_vertex_0.z);
    
            uv_end.push(uv_e_vertex_0.x);
            uv_end.push(uv_e_vertex_0.y);
            uv_end.push(uv_e_vertex_0.z);

            start_time.push(alpha);
        }
    }
}

use web_sys::WebGlBuffer;
pub struct ImageSurvey {
    id: String,
    color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: HEALPixCellsInView,
    // Flag telling if the user is unzooming and
    // the depth has decreased by at least 1
    // This is used to draw the vbo vertices when users are unzooming
    cells_depth_increased: bool,

    num_idx: usize,

    sphere_sub: SphereSubdivided,
    vbo: WebGlBuffer,
    ebo: WebGlBuffer,

    gl: WebGl2Context,

    _type: ImageSurveyType,
}
use crate::utils;
use crate::camera::UserAction;
use super::view_on_surveys::HEALPixCells;
use web_sys::WebGl2RenderingContext;
impl ImageSurvey {
    fn new(gl: &WebGl2Context,
        config: HiPSConfig,
        color: Color,
        exec: Rc<RefCell<TaskExecutor>>,
        _type: ImageSurveyType
    ) -> Self {
        let id = config.root_url.clone();

        let textures = ImageSurveyTextures::new(gl, config, exec);
        let view = HEALPixCellsInView::new();
        let cells_to_draw = Cow::Borrowed(&view.get_cells());

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            (MAX_NUM_FLOATS_TO_DRAW * std::mem::size_of::<f32>()) as i32,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );
        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            (MAX_NUM_FLOATS_TO_DRAW * std::mem::size_of::<u16>()) as i32,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        let num_idx = 0;
        let sphere_sub = SphereSubdivided::new();
        let gl = gl.clone();
        let cells_depth_increased = false;
        ImageSurvey {
            id,
            color,
            // The image survey texture buffer
            textures,
            // Keep track of the cells in the FOV
            view,
            cells_depth_increased,
        
            num_idx,
        
            sphere_sub,
            vbo,
            ebo,
        
            gl,

            _type,
        }
    }

    pub fn from<T: HiPS>(hips: T, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<Self, JsValue> {
        hips.create(gl, exec)
    }

    pub fn set_color(&mut self, color: &Color) {
        self.color = *color;
    }

    pub fn set_positions<P: Projection>(&mut self, last_user_action: UserAction) {
        match last_user_action {
            UserAction::Unzooming => {
                self.update_positions::<P, UnZoom>();
            },
            UserAction::Zooming => {
                self.update_positions::<P, Zoom>();
            },
            UserAction::Moving => {
                self.update_positions::<P, Move>();
            },
            UserAction::Starting => {
                self.update_positions::<P, Move>();
            }
        }
    }

    fn update_positions<P: Projection, T: RecomputeRasterizer>(&mut self) {
        let cells_to_draw = self.view.get_cells();

        let mut lonlats = vec![];
        let mut positions = vec![];
        let mut idx_vertices = vec![];

        for cell in cells_to_draw.iter() {
            add_positions_grid::<P, T>(
                &mut lonlats,
                &mut positions,
                &mut idx_vertices,
                &cell,
                &self.sphere_sub,
            );
        }

        let mut coo = lonlats;
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 2 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);
        coo.extend(positions);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 5 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);

        let buf_positions = unsafe { js_sys::Float32Array::view(&coo) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0 as i32,
            &buf_positions
        );

        self.num_idx = idx_vertices.len();
        let buf_idx = unsafe { js_sys::Uint16Array::view(&idx_vertices) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0 as i32,
            &buf_idx
        );
    }

    pub fn set_UVs<P: Projection>(&mut self, last_user_action: UserAction) {
        let cells_to_draw = self.view.get_cells();

        match last_user_action {
            UserAction::Unzooming => {
                let textures = UnZoom::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, UnZoom>(&textures);
            },
            UserAction::Zooming => {
                let textures = Zoom::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Zoom>(&textures);
            },
            UserAction::Moving => {
                let textures = Move::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Move>(&textures);
            },
            UserAction::Starting => {
                let textures = Move::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Move>(&textures);
            }
        }
    }

    fn update_UVs<'a, P: Projection, T: RecomputeRasterizer>(&'a self, textures: &TexturesToDraw<'a>) {
        let mut uv_start = vec![];
        let mut uv_end = vec![];
        let mut start_times = vec![];

        let survey_config = self.textures.config();

        for (cell, state) in textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture, survey_config);
            let uv_1 = TileUVW::new(cell, &state.ending_texture, survey_config);
            let start_time = state.ending_texture.start_time();

            add_uv_grid::<P, T>(
                &mut uv_start,
                &mut uv_end,
                &mut start_times,
                &cell,
                &self.sphere_sub,

                &uv_0, &uv_1,
                start_time.as_millis(),
            );
        }

        let mut uv = uv_start;
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 3 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(uv_end);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 6 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(start_times);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 7 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        let buf_uvs = unsafe { js_sys::Float32Array::view(&uv) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            (MAX_NUM_VERTICES_TO_DRAW * 5 * std::mem::size_of::<f32>()) as i32,
            &buf_uvs
        );
    }

    fn refresh_view(&mut self, camera: &CameraViewPort) {
        let texture_size = self.textures.config().get_texture_size();
        self.view.refresh_cells(texture_size, camera);
    }

    #[inline]
    pub fn get_textures(&self) -> &ImageSurveyTextures {
        &self.textures
    }

    pub fn get_textures_mut(&mut self) -> &mut ImageSurveyTextures {
        &mut self.textures
    }

    #[inline]
    pub fn get_view(&self) -> &HEALPixCellsInView {
        &self.view
    }

    #[inline]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    #[inline]
    fn get_type(&self) -> ImageSurveyType {
        self._type
    }

    #[inline]
    fn get_color(&self) -> &Color {
        &self.color
    }
}

use std::borrow::Cow;
impl Draw for ImageSurvey {
    fn draw<P: Projection>(&mut self, raster: &Rasterizer, raytracer: &RayTracer, shaders: &mut ShaderManager, camera: &CameraViewPort) {
        if camera.get_aperture() > 150.0 {
            // Raytracer
            let shader = self.color.get_raytracer_shader::<P>(&self.gl, shaders).bind(&self.gl);

            let cells_to_draw = self.view.get_cells();
            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(&self.color)
                .attach_uniform("model", camera.get_w2m())
                .attach_uniform("current_depth", &(cells_to_draw.get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time());

            // The raytracer vao is bound at the lib.rs level
            raytracer.draw();
            return;
        }

        // The rasterizer has a buffer containing:
        // - The vertices of the HEALPix cells for the most refined survey
        // - The starting and ending uv for the blending animation
        // - The time for each HEALPix cell at which the animation begins
        //
        // Each of these data can be changed at different circumstances:
        // - The vertices are changed if:
        //     * new cells are added/removed (because new cells are added)
        //       to the previous frame.
        // - The UVs are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU 
        // - The starting blending animation times are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU

        let last_user_action = camera.last_user_action();
        // Get the cells to draw
        /*let cells_to_draw = if last_user_action == UserAction::Unzooming {
            if self.view.has_depth_decreased() || self.cells_depth_increased {
                self.cells_depth_increased = true;
                let new_depth = self.view.get_depth();

                Cow::Owned(&super::view_on_surveys::get_cells_in_camera(new_depth + 1, &camera))
            } else {
                Cow::Borrowed(&self.view.get_cells())
            }
        } else {
            // no more unzooming
            self.cells_depth_increased = false;
            Cow::Borrowed(&self.view.get_cells())
        };*/

        let new_cells_added = self.view.is_there_new_cells_added();
        let recompute_vertex_positions = new_cells_added;
        if recompute_vertex_positions {
            self.set_positions::<P>(last_user_action);
        }

        let recompute_uvs = new_cells_added | self.textures.is_there_available_tiles();
        if recompute_uvs {
            self.set_UVs::<P>(last_user_action);
        }

        let shader = self.color.get_raster_shader::<P>(&self.gl, shaders).bind(&self.gl);
        shader
            .attach_uniforms_from(camera)
            .attach_uniforms_from(&self.textures)
            .attach_uniforms_from(&self.color)
            .attach_uniform("model", camera.get_w2m())
            .attach_uniform("current_depth", &(self.view.get_cells().get_depth() as i32))
            .attach_uniform("current_time", &utils::get_current_time());

        // The raster vao is bound at the lib.rs level
        raster.draw(self.num_idx as i32);        
    }
}

use wasm_bindgen::JsValue;
pub trait HiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue>;
}

use std::rc::Rc;
use std::cell::RefCell;
impl HiPS for SimpleHiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue> {
        let SimpleHiPS { properties, colormap, transfer } = self;

        let config = HiPSConfig::new(gl, &properties)?;

        if properties.isColor {
            Ok(ImageSurvey::new(gl, config, Color::Colored, exec, ImageSurveyType::Simple))
        } else {
            Ok(ImageSurvey::new(
                gl,
                config,
                Color::Grayscale2Colormap {
                    colormap: colormap.into(),
                    param: GrayscaleParameter {
                        h: transfer.into(),
                        min_value: properties.minCutout,
                        max_value: properties.maxCutout,
                        
                        // These parameters are not in the properties
                        // They will be retrieved by looking inside a tile
                        scale: 1.0,
                        offset: 0.0,
                        blank: 0.0,
                    }
                },
                exec,
                ImageSurveyType::Simple
            ))
        }
    }
}
use crate::{SimpleHiPS, ComponentHiPS};
impl HiPS for ComponentHiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue> {
        let ComponentHiPS { properties, color, transfer, k } = self;

        let config = HiPSConfig::new(gl, &properties)?;

        if properties.isColor {
            Err(format!("{} tiles does not contain grayscale data!", config.root_url).into())
        } else {
            Ok(ImageSurvey::new(
                gl,
                config,
                Color::Grayscale2Color {
                    color,
                    k,
                    param: GrayscaleParameter {
                        h: transfer.into(),
                        min_value: properties.minCutout,
                        max_value: properties.maxCutout,
                        
                        // These Parameters are not in the properties
                        // They will be retrieved by looking inside a tile
                        scale: 1.0,
                        offset: 0.0,
                        blank: 0.0,
                    }
                },
                exec,
                ImageSurveyType::Component
            ))
        }
    }
}

enum ImageSurveyIdx {
    Composite(Vec<String>),
    Simple(String),
    None,
}

use crate::renderable::view_on_surveys::HEALPixCellsInView;
pub struct ImageSurveys {
    surveys: HashMap<String, ImageSurvey>,

    primary: ImageSurveyIdx,
    overlay: ImageSurveyIdx,

    rasterizer: Rasterizer,
    raytracer: RayTracer,

    gl: WebGl2Context
}
use crate::buffer::Tiles;
use crate::buffer::{TileArrayBufferImage, TileHTMLImage};
use crate::buffer::{TileResolved, ResolvedTiles, RetrievedImageType};
impl ImageSurveys {
    pub fn new<P: Projection>(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager) -> Self {
        let surveys = HashMap::new();

        let primary = ImageSurveyIdx::None;
        let overlay = ImageSurveyIdx::None;

        // Two mode of render, each storing a specific VBO
        // - The rasterizer draws the HEALPix cells being in the current view
        // This mode of rendering is used for small FoVs
        let rasterizer = Rasterizer::new(&gl, shaders);
        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new::<P>(&gl, &camera, shaders);

        let gl = gl.clone();
        ImageSurveys {
            surveys,

            primary,
            overlay,

            rasterizer,
            raytracer,

            gl
        }
    }

    pub fn set_projection<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders);
    }

    pub fn draw<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        let raytracing = camera.get_aperture() > 150.0;
        // Bind the good VAO
        if raytracing {
            self.raytracer.bind();
        } else {
            self.rasterizer.bind();
        }

        match &self.primary {
            ImageSurveyIdx::Simple(idx) => {
                let mut survey = self.surveys.get_mut(idx).unwrap();
                survey.draw::<P>(&self.rasterizer, &self.raytracer, shaders, camera);
            },
            ImageSurveyIdx::Composite(indices) => {
                // Add additive blending here
                for idx in indices {
                    let mut survey = self.surveys.get_mut(idx).unwrap();
                    survey.draw::<P>(&self.rasterizer, &self.raytracer, shaders, camera);
                }
            },
            _ => unreachable!()
        }

        // Overlay
        match &self.overlay {
            ImageSurveyIdx::Simple(idx) => {
                let mut survey = self.surveys.get_mut(idx).unwrap();
                survey.draw::<P>(&self.rasterizer, &self.raytracer, shaders, camera);
            },
            ImageSurveyIdx::Composite(indices) => {
                // Add additive blending here
                for idx in indices {
                    let mut survey = self.surveys.get_mut(idx).unwrap();
                    survey.draw::<P>(&self.rasterizer, &self.raytracer, shaders, camera);
                }
            },
            // If no HiPS are overlaying we do nothing
            _ => ()
        }
    }

    /*pub fn remove_survey(&mut self, id: &str) {
        match &mut self.primary {
            ImageSurveyIdx::Simple(curr_id) => {
                if id == curr_id {
                    self.surveys.remove(curr_id);
                    self.primary = ImageSurveyIdx::None;
                }
            },
            ImageSurveyIdx::Composite(curr_indices) => {
                let mut idx_to_remove = -1;
                for (idx, curr_id) in curr_indices.iter().enumerate() {
                    if id == curr_id {
                        self.surveys.remove(curr_id);

                        idx_to_remove = idx as i32;
                        break;
                    }
                }

                if idx_to_remove >= 0 {
                    curr_indices.remove(idx_to_remove as usize);
                    if curr_indices.is_empty() {
                        self.primary = ImageSurveyIdx::None;
                    }
                }
            },
        }
    }*/

    pub fn set_simple_hips(&mut self, survey: ImageSurvey) {
        let id = survey.get_id().to_string();
        let _type = survey.get_type();
        
        match (&mut self.primary, _type) {
            (ImageSurveyIdx::Simple(curr_id), ImageSurveyType::Simple) => {
                if &id == curr_id {
                    // The same survey is already selected.
                    // We update it with the new color and end up here
                    let mut s = self.surveys.get_mut(curr_id).unwrap();
                    s.set_color(survey.get_color());
                } else {
                    // There is one other survey. We remove it
                    // from the container and add the new one
                    self.surveys.remove(curr_id);
                    self.surveys.insert(id.clone(), survey);

                    self.primary = ImageSurveyIdx::Simple(id.to_string());
                }
            },
            (ImageSurveyIdx::Simple(curr_id), ImageSurveyType::Component) => {
                // A simple HiPS was in place, we replace it by a composite HiPS
                self.surveys.remove(curr_id);
                self.surveys.insert(id.clone(), survey);

                self.primary = ImageSurveyIdx::Composite(vec![id]);
            },
            (ImageSurveyIdx::Composite(curr_indices), ImageSurveyType::Simple) => {
                // A composite HiPS was in place, we replace it by a simple HiPS
                for idx in curr_indices {
                    // We remove all the component surveys that are bound
                    // to the composite HiPS
                    self.surveys.remove(idx);
                }

                self.surveys.insert(id.clone(), survey);

                self.primary = ImageSurveyIdx::Simple(id);
            },
            (ImageSurveyIdx::Composite(curr_indices), ImageSurveyType::Component) => {
                // A composite HiPS was in place, we replace it by a simple HiPS
                for idx in curr_indices.iter() {
                    // If it is already found in the components
                    if &id == idx {
                        let mut s = self.surveys.get_mut(idx).unwrap();
                        s.set_color(survey.get_color());
                        return;
                    }
                }

                self.surveys.insert(id.to_string(), survey);
                curr_indices.push(id.to_string());
            },
            (ImageSurveyIdx::None, ImageSurveyType::Simple) => {
                self.surveys.insert(id.clone(), survey);
                self.primary = ImageSurveyIdx::Simple(id.to_string());
            },
            (ImageSurveyIdx::None, ImageSurveyType::Component) => {
                self.surveys.insert(id.clone(), survey);
                self.primary = ImageSurveyIdx::Composite(vec![id]);
            }
        }
    }

    pub fn get_view(&self) -> Option<&HEALPixCellsInView> {
        if self.surveys.is_empty() {
            None
        } else {
            match &self.primary {
                ImageSurveyIdx::Simple(idx) => {
                    Some(self.surveys.get(idx).unwrap().get_view())
                },
                ImageSurveyIdx::Composite(indices) => {
                    let idx = indices.first().unwrap();
                    Some(self.surveys.get(idx).unwrap().get_view())
                },
                ImageSurveyIdx::None => {
                    None
                }
            }
        }
    }

    pub fn refresh_views(&mut self, camera: &CameraViewPort) {
        for survey in self.surveys.values_mut() {
            survey.refresh_view(camera);
        }
    }

    // Update the surveys by telling which tiles are available
    pub fn set_available_tiles(&mut self, available_tiles: &Tiles) {
        for tile in available_tiles {
            let textures = &mut self.surveys.get_mut(&tile.root_url)
                .unwrap()
                .get_textures_mut();
            textures.register_available_tile(tile);
        }
    }

    // Update the surveys by adding to the surveys the tiles
    // that have been resolved
    pub fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles) {
        for (tile, result) in resolved_tiles.into_iter() {
            let textures = self.surveys.get_mut(&tile.root_url)
                .unwrap()
                .get_textures_mut();

            match result {
                TileResolved::Missing { time_req } => {
                    let default_image = textures.config().get_black_tile();
                    textures.push::<Rc<TileArrayBufferImage>>(tile, default_image, time_req);
                },
                TileResolved::Found { image, time_req } => {
                    match image {
                        RetrievedImageType::FITSImage { image, metadata } => {
                            textures.push::<TileArrayBufferImage>(tile, image, time_req);
                        },
                        RetrievedImageType::CompressedImage { image } => {
                            textures.push::<TileHTMLImage>(tile, image, time_req);
                        }
                    }
                }
            }
        }
    }

    // Accessors
    pub fn get(&self, root_url: &str) -> Option<&ImageSurvey> {
        self.surveys.get(root_url)
    }

    fn len(&self) -> usize {
        self.surveys.len()
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, String, ImageSurvey> {
        self.surveys.iter()
    }
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, String, ImageSurvey> {
        self.surveys.iter_mut()
    }
}
use std::collections::hash_map::{Iter, IterMut};
use crate::{
    renderable::{Angle, ArcDeg},
    buffer::HiPSConfig,
    shader::ShaderManager,
    time::DeltaTime,
    async_task::TaskExecutor,
};

use crate::TransferFunction;

// This is specific to the rasterizer method of rendering
/*impl HEALPixSphere {
    pub fn new(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager) -> Self {

        crate::log(&format!("raytracer"));
        HEALPixSphere {
            buffer,
            surveys,

            gl,
        }
    }

    pub fn set_image_survey<P: Projection>(&mut self, hips_definition: HiPSDefinition, camera: &mut CameraViewPort, task_executor: &mut TaskExecutor) -> Result<(), JsValue> {        
        self.config.set_HiPS_definition(hips_definition)?;
        // Tell the camera the config has changed
        camera.set_image_survey::<P>(&self.config);

        // Clear the buffer
        self.buffer.reset(&self.gl, &self.config, camera, task_executor);

        Ok(())
    }*/
    
    /*pub fn ask_for_tiles<P: Projection>(&mut self, cells: &HashMap<HEALPixCell, bool>) {
        // Ask for the real tiles being in the camera
        self.buffer.ask_for_tiles(cells, &self.config);
    }*/

    /*pub fn request(&mut self, available_tiles: &Tiles, task_executor: &mut TaskExecutor) {
        //survey.register_tiles_sent_to_gpu(copied_tiles);
        self.buffer.get_resolved_tiles(available_tiles);
    }

    pub fn set_projection<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        self.update::<P>(camera);
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders);
    }

    pub fn update<P: Projection>(&mut self, available_tiles: &Tiles, camera: &CameraViewPort, exec: &mut TaskExecutor) -> IsNextFrameRendered {


        if self.survey.is_ready() {
            // Update the scene if:
            // - The camera changed
            // - There are remaining tiles to write to the GPU
            // - The tiles blending in GPU must be done (500ms + the write time)
            let update =  |
                (Time::now() < self.time_last_tile_written + DeltaTime::from_millis(500_f32));

            if !update {
                false
            } else {
                let aperture = camera.get_aperture();
                let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();
                if aperture <= limit_aperture {
                    // Rasterizer mode
                    self.raster.update::<P>(&mut self.buffer, camera, &self.config);
                }

                true
            }   
        } else {
            // Do not render the scene while the buffer is not ready
            true
        }
    }

    pub fn draw<P: Projection>(
        &mut self,
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
    ) {
        let aperture = camera.get_aperture();
        let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();

        if aperture <= limit_aperture {
            // Rasterization
            let shader = Rasterizer::get_shader::<P>(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(camera)
                .attach_uniforms_from(&self.survey)
                //.attach_uniforms_from(&self.config)
                //.attach_uniforms_from(&self.buffer)
                .attach_uniform("inv_model", camera.get_inverted_model_mat())
                .attach_uniform("current_time", &utils::get_current_time());

            self.raster.draw::<P>(gl, &shader_bound);
        } else {
            // Ray-tracing
            let shader = RayTracer::get_shader(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(camera)
                .attach_uniforms_from(&self.survey)
                //.attach_uniforms_from(&self.config)
                //.attach_uniforms_from(&self.buffer)
                .attach_uniform("model", camera.get_model_mat())
                .attach_uniform("current_depth", &(camera.depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time());

            self.raytracer.draw(gl, &shader_bound);
        }   
    }

    #[inline]
    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }
}*/