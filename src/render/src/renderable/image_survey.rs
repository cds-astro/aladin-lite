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
        camera: &CameraViewPort,
        view: &HEALPixCellsInView,
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
    fn get_textures_from_survey<'a>(camera: &CameraViewPort, view: &HEALPixCellsInView, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
        //crate::log(&format!("cells to draw: {:?}", cells_to_draw));
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
    fn get_textures_from_survey<'a>(camera: &CameraViewPort, view: &HEALPixCellsInView, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
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
    fn get_textures_from_survey<'a>(camera: &CameraViewPort, view: &HEALPixCellsInView, survey: &'a ImageSurveyTextures) -> TexturesToDraw<'a> {
        let mut depth = view.get_depth();
        let max_depth = survey.config().get_max_depth();

        // We do not draw the parent cells if the depth has not decreased by at least one
        let cells_to_draw = if depth < max_depth && view.has_depth_decreased_while_unzooming(camera) {
            Cow::Owned(crate::renderable::view_on_surveys::get_cells_in_camera(depth + 1, camera))
            //Cow::Borrowed(view.get_cells())
        } else {
            Cow::Borrowed(view.get_cells())
        };

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
    fn draw<P: Projection>(&mut self, raytracer: &RayTracer, shaders: &mut ShaderManager, camera: &CameraViewPort, color: &Color, opacity: f32);
}

#[derive(Clone, Debug)]
pub struct GrayscaleParameter {
    h: TransferFunction,
    min_value: f32,
    max_value: f32,
}

use crate::shader::{Shader, ShaderBound};
impl SendUniforms for GrayscaleParameter {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.h)
            .attach_uniform("min_value", &self.min_value)
            .attach_uniform("max_value", &self.max_value);

        shader
    }
}

/// List of the different type of surveys
#[derive(Clone, Debug)]
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
    pub fn get_raster_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager, integer_tex: bool) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raster_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                if integer_tex {
                    P::get_raster_shader_gray2colormap_integer(gl, shaders)
                } else {
                    P::get_raster_shader_gray2colormap(gl, shaders)
                }
            },
            Color::Grayscale2Color { .. } => {
                if integer_tex {
                    P::get_raster_shader_gray2color_integer(gl, shaders)
                } else {
                    P::get_raster_shader_gray2color(gl, shaders)
                }
            },
        }
    }

    pub fn get_raytracer_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager, integer_tex: bool) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raytracer_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                if integer_tex {
                    P::get_raytracer_shader_gray2colormap_integer(gl, shaders)
                } else {
                    P::get_raytracer_shader_gray2colormap(gl, shaders)
                }
            },
            Color::Grayscale2Color { .. } => {
                if integer_tex {
                    P::get_raytracer_shader_gray2color_integer(gl, shaders)
                } else {
                    P::get_raytracer_shader_gray2color(gl, shaders)
                }
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

/*#[derive(Clone, Copy)]
enum ImageSurveyType {
    Simple,
    Component
}*/

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
fn add_vertices_grid<P: Projection, E: RecomputeRasterizer>(
    vertices: &mut Vec<f32>,
    idx_positions: &mut IdxVerticesVec,

    cell: &HEALPixCell,
    sphere_sub: &SphereSubdivided,

    uv_0: &TileUVW,
    uv_1: &TileUVW,
    alpha: f32
) {
    let num_subdivision = E::num_subdivision::<P>(cell, sphere_sub);

    let n_segments_by_side: u16 = 1_u16 << num_subdivision;
    let lonlat = cdshealpix::grid_lonlat::<f32>(cell, n_segments_by_side);

    let n_vertices_per_segment = n_segments_by_side + 1;

    let off_idx_vertices = (vertices.len()/12) as u16;
    for i in 0..n_vertices_per_segment {
        for j in 0..n_vertices_per_segment {
            let id_vertex_0 = (j + i * n_vertices_per_segment) as usize;

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

            let (lon, lat) = (lonlat[id_vertex_0].lon().0, lonlat[id_vertex_0].lat().0);
            let position: Vector3<f32> = lonlat[id_vertex_0].vector();

            vertices.push(lon);
            vertices.push(lat);

            vertices.push(position.x);
            vertices.push(position.y);
            vertices.push(position.z);

            vertices.push(uv_s_vertex_0.x);
            vertices.push(uv_s_vertex_0.y);
            vertices.push(uv_s_vertex_0.z);
    
            vertices.push(uv_e_vertex_0.x);
            vertices.push(uv_e_vertex_0.y);
            vertices.push(uv_e_vertex_0.z);

            vertices.push(alpha);
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

use web_sys::{WebGlVertexArrayObject, WebGlBuffer};
pub struct ImageSurvey {
    //color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: HEALPixCellsInView,

    num_idx: usize,

    sphere_sub: SphereSubdivided,
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    ebo: WebGlBuffer,

    gl: WebGl2Context,

    //_type: ImageSurveyType,
    size_vertices_buf: u32,
    size_idx_vertices_buf: u32,
}
use crate::utils;
use crate::camera::UserAction;
use super::view_on_surveys::HEALPixCells;
use web_sys::WebGl2RenderingContext;
impl ImageSurvey {
    fn new(gl: &WebGl2Context,
        camera: &CameraViewPort,
        surveys: &ImageSurveys,
        config: HiPSConfig,
        //color: Color,
        exec: Rc<RefCell<TaskExecutor>>,
        //_type: ImageSurveyType
    ) -> Self {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        let data = vec![0.0_f32; MAX_NUM_FLOATS_TO_DRAW];
        let size_vertices_buf = MAX_NUM_FLOATS_TO_DRAW as u32;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        let num_bytes_per_f32 = mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 12 * num_bytes_per_f32, (0 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec3 position;
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, 12 * num_bytes_per_f32, (2 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec3 uv_start;
        gl.vertex_attrib_pointer_with_i32(2, 3, WebGl2RenderingContext::FLOAT, false, 12 * num_bytes_per_f32, (5 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(2);

        // layout (location = 3) in vec3 uv_end;
        gl.vertex_attrib_pointer_with_i32(3, 3, WebGl2RenderingContext::FLOAT, false, 12 * num_bytes_per_f32, (8 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(3);

        // layout (location = 4) in float time_tile_received;
        gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::FLOAT, false, 12 * num_bytes_per_f32, (11 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(4);

        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        let data = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];
        let size_idx_vertices_buf = MAX_NUM_INDICES_TO_DRAW as u32;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            unsafe { &js_sys::Uint16Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW
        );
        gl.bind_vertex_array(None);

        let num_idx = 0;
        let sphere_sub = SphereSubdivided::new();

        let textures = ImageSurveyTextures::new(gl, config, exec);
        let conf = textures.config();
        let view = HEALPixCellsInView::new(conf.get_tile_size(), conf.get_max_depth(), camera);

        let gl = gl.clone();

        ImageSurvey {
            //color,
            // The image survey texture buffer
            textures,
            // Keep track of the cells in the FOV
            view,
        
            num_idx,
        
            sphere_sub,
            vao,
            vbo,
            ebo,
        
            gl,

            //_type,
            size_vertices_buf,
            size_idx_vertices_buf
        }
    }

    /*pub fn from<T: HiPS>(gl: &WebGl2Context, camera: &CameraViewPort, surveys: &ImageSurveys, hips: T, exec: Rc<RefCell<TaskExecutor>>) -> Result<Self, JsValue> {
        hips.create(gl, camera, surveys, exec)
    }*/

    /*pub fn set_color(&mut self, color: &Color) {
        self.color = color.clone();
    }*/
    /*
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
        crate::log(&format!("{:?} cells to draw", cells_to_draw));
        crate::log(&format!("num coo {:?} ", coo.len()));
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 2 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);
        coo.extend(positions);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 5 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);
        crate::log(&format!("coo {:?} ", coo));
        crate::log(&format!("num coo {:?} ", coo.len()));

        let buf_positions = unsafe { js_sys::Float32Array::view(&coo) };
        crate::log(&format!("buf_positions coo {:?} ", buf_positions.length()));

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
        crate::log(&format!("buf_positions coo2 {:?} ", buf_positions.length()));

    }*/

    pub fn set_vertices<P: Projection>(&mut self, camera: &CameraViewPort) {
        let last_user_action = camera.get_last_user_action();
        match last_user_action {
            UserAction::Unzooming => {
                self.update_vertices::<P, UnZoom>(camera);
            },
            UserAction::Zooming => {
                self.update_vertices::<P, Zoom>(camera);
            },
            UserAction::Moving => {
                self.update_vertices::<P, Move>(camera);
            },
            UserAction::Starting => {
                self.update_vertices::<P, Move>(camera);
            }
        }
    }

    fn update_vertices<P: Projection, T: RecomputeRasterizer>(&mut self, camera: &CameraViewPort) {
        let textures = T::get_textures_from_survey(camera, &mut self.view, &self.textures);

        let mut vertices = vec![];
        let mut idx_vertices = vec![];

        let survey_config = self.textures.config();

        for (cell, state) in textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture, survey_config);
            let uv_1 = TileUVW::new(cell, &state.ending_texture, survey_config);
            let start_time = state.ending_texture.start_time();

            add_vertices_grid::<P, T>(
                &mut vertices,
                &mut idx_vertices,

                &cell,
                &self.sphere_sub,

                &uv_0, &uv_1,
                start_time.as_millis(),
            );
        }

        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.ebo));

        //crate::log(&format!(": {} {}", vertices.len(), self.size_vertices_buf));
        let buf_vertices = unsafe { js_sys::Float32Array::view(&vertices) };
        if vertices.len() > self.size_vertices_buf as usize {
            self.size_vertices_buf =  vertices.len() as u32;
            //crate::log(&format!("realloc num floats: {}", self.size_vertices_buf));

            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &buf_vertices,
                WebGl2RenderingContext::DYNAMIC_DRAW
            );
        } else {

            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                0,
                &buf_vertices
            );
        }

        self.num_idx = idx_vertices.len();
        let buf_idx = unsafe { js_sys::Uint16Array::view(&idx_vertices) };
        if idx_vertices.len() > self.size_idx_vertices_buf as usize {
            self.size_idx_vertices_buf = idx_vertices.len() as u32;
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &buf_idx,
                WebGl2RenderingContext::DYNAMIC_DRAW
            );
        } else {

            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                0,
                &buf_idx
            );
        }
    }

    fn refresh_view(&mut self, camera: &CameraViewPort) {
        let tile_size = self.textures.config().get_tile_size();
        let max_depth = self.textures.config().get_max_depth();

        self.view.refresh_cells(tile_size, max_depth, camera);
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
        &self.get_textures().config.root_url
    }


    /*#[inline]
    fn get_color(&self) -> &Color {
        &self.color
    }

    #[inline]
    fn get_color_mut(&mut self) -> &mut Color {
        &mut self.color
    }*/
}

impl Drop for ImageSurvey {
    fn drop(&mut self) {
        crate::log("drop image survey");
        drop(&mut self.textures);

        // Drop the vertex arrays
        //self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        //self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);

        self.gl.delete_buffer(Some(&self.vbo));
        self.gl.delete_buffer(Some(&self.ebo));

        //self.gl.bind_vertex_array(None);
        self.gl.delete_vertex_array(Some(&self.vao));
    }
}

use std::borrow::Cow;
impl Draw for ImageSurvey {
    fn draw<P: Projection>(&mut self, raytracer: &RayTracer, shaders: &mut ShaderManager, camera: &CameraViewPort, color: &Color, opacity: f32) {
        if !self.textures.is_ready() {
            crate::log("not ready");
            // Do not render while the 12 base cell textures
            // are not loaded
            return;
        }

        let textures_array = self.textures.get_texture_array();
        let survey_storing_integers = self.textures.config.tex_storing_integers == 1;

        let raytracing = camera.get_aperture().0 > P::RASTER_THRESHOLD_ANGLE;
        if raytracing {
            //raytracer.bind();
            let shader = color.get_raytracer_shader::<P>(&self.gl, shaders, survey_storing_integers)
                .bind(&self.gl);

            textures_array.bind_all_textures(&shader);

            let num_tex = textures_array.textures.len();
            shader.attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("current_depth", &(self.view.get_cells().get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniform("num_tex", &(num_tex as i32));

            raytracer.draw(&shader);

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

        let new_cells_added = self.view.is_there_new_cells_added();
        let recompute_positions = new_cells_added;
        {
            self.gl.bind_vertex_array(Some(&self.vao));

            let recompute_vertices = recompute_positions | self.textures.is_there_available_tiles() | camera.has_moved();
            if recompute_vertices {
                self.set_vertices::<P>(camera);
            }

            let shader = color.get_raster_shader::<P>(&self.gl, shaders, survey_storing_integers)
                .bind(&self.gl);
            textures_array.bind_all_textures(&shader);
    
            let num_tex = textures_array.textures.len();
            shader.attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("current_depth", &(self.view.get_cells().get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniform("num_tex", &(num_tex as i32));
            //crate::log("raster");
            // The raster vao is bound at the lib.rs level
            self.gl.draw_elements_with_i32(
                //WebGl2RenderingContext::LINES,
                WebGl2RenderingContext::TRIANGLES,
                self.num_idx as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0
            );
        }
    }
}

use wasm_bindgen::JsValue;
pub trait HiPS {
    fn create(self, gl: &WebGl2Context, camera: &CameraViewPort, surveys: &ImageSurveys, exec: Rc<RefCell<TaskExecutor>>) -> Result<(ImageSurvey, Color), JsValue>;
    fn get_color(&self) -> &HiPSColor;
}

use std::rc::Rc;
use std::cell::RefCell;
use crate::{SimpleHiPS, HiPSColor};

impl HiPS for SimpleHiPS {
    fn get_color(&self) -> &HiPSColor {
        &self.color
    }

    fn create(self, gl: &WebGl2Context, camera: &CameraViewPort, surveys: &ImageSurveys, exec: Rc<RefCell<TaskExecutor>>) -> Result<(ImageSurvey, Color), JsValue> {
        let SimpleHiPS { properties, color } = self;

        let config = HiPSConfig::new(gl, &properties)?;
        Ok(
            (
                ImageSurvey::new(
                    gl,
                    camera,
                    surveys,
                    config,
                    exec,
                    //ImageSurveyType::Simple
                ),
                match color {
                    HiPSColor::Color => Color::Colored,
                    HiPSColor::Grayscale2Color {color, transfer, k} => {
                        Color::Grayscale2Color {
                            color,
                            k,
                            param: GrayscaleParameter {
                                h: transfer.into(),
                                min_value: properties.minCutout.unwrap_or(0.0),
                                max_value: properties.maxCutout.unwrap_or(1.0),
                            }
                        }
                    },
                    HiPSColor::Grayscale2Colormap {colormap, transfer} => {
                        Color::Grayscale2Colormap {
                            colormap: colormap.into(),
                            param: GrayscaleParameter {
                                h: transfer.into(),
                                min_value: properties.minCutout.unwrap_or(0.0),
                                max_value: properties.maxCutout.unwrap_or(1.0),
                            }
                        }
                    }
                },
            )
        )
    }
}

enum ImageSurveyIdx {
    Composite {
        names: Vec<String>,
        colors: Vec<Color>
    },
    Simple {
        name: String,
        color: Color,
    },
    None,
}

impl ImageSurveyIdx {
    fn contains(&self, name: &str) -> bool {
        match self {
            ImageSurveyIdx::Composite { names: cur_names, .. } => {
                cur_names.iter().position(|cur_name| cur_name == name).is_some()
            },
            ImageSurveyIdx::Simple { name: cur_name, .. } => {
                //crate::log(&format!("{} {} res {}", name, cur_name, cur_name == name));
                cur_name == name
            },
            ImageSurveyIdx::None => false
        }
    }
}

const MAX_NUM_LAYERS: usize = 2;
use crate::renderable::view_on_surveys::HEALPixCellsInView;
pub struct ImageSurveys {
    surveys: HashMap<String, ImageSurvey>,
    layers: [ImageSurveyIdx; MAX_NUM_LAYERS],
    // opacity of the primary layer
    opacity: f32,

    raytracer: RayTracer,

    gl: WebGl2Context
}
use crate::buffer::Tiles;
use crate::buffer::{TileArrayBufferImage, TileHTMLImage};
use crate::buffer::{TileResolved, ResolvedTiles, RetrievedImageType};
use cgmath::Vector2;
use crate::buffer::FITSMetaData;
const APERTURE_LIMIT: f32 = 110.0;
use crate::Resources;
impl ImageSurveys {
    pub fn new<P: Projection>(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager, rs: &Resources) -> Self {
        let surveys = HashMap::new();
        let layers = [ImageSurveyIdx::None, ImageSurveyIdx::None];

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new::<P>(&gl, &camera, shaders, rs);

        let opacity = 0.5;
        let gl = gl.clone();
        ImageSurveys {
            surveys,
            layers,
            opacity,

            raytracer,

            gl
        }
    }

    pub fn set_projection<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager, rs: &Resources) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders, rs);
    }

    pub fn set_longitude_reversed<P: Projection>(&mut self, reversed: bool, camera: &CameraViewPort, shaders: &mut ShaderManager, rs: &Resources) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders, rs);
    }

    pub fn set_overlay_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn draw<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        //let raytracing = camera.get_aperture() > APERTURE_LIMIT;
        //let raytracing = !P::is_included_inside_projection(&crate::renderable::projection::ndc_to_clip_space(&Vector2::new(-1.0, -1.0), camera));
        let limit_aperture: Angle<f32> = ArcDeg(APERTURE_LIMIT).into();
        let raytracing = camera.get_aperture().0 > P::RASTER_THRESHOLD_ANGLE;
        if raytracing {
            self.raytracer.bind();
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        } else {
            if camera.is_reversed_longitude() {
                self.gl.cull_face(WebGl2RenderingContext::BACK);
            } else {
                self.gl.cull_face(WebGl2RenderingContext::FRONT);
            }
        }

        let primary_layer = &self.layers[0];
        match &primary_layer {
            ImageSurveyIdx::Simple { name, color } => {
                let mut survey = self.surveys.get_mut(name).unwrap();
                survey.draw::<P>(&self.raytracer, shaders, camera, color, 1.0);
            },
            ImageSurveyIdx::Composite { names, colors } => {
                // Add the first hips on top of the background
                let mut survey = self.surveys.get_mut(names.first().unwrap()).unwrap();
                survey.draw::<P>(&self.raytracer, shaders, camera, colors.first().unwrap(), 1.0);
                
                // Enable the blending for the following HiPSes
                self.gl.enable(WebGl2RenderingContext::BLEND);
                self.gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);

                for (name, color) in names.iter().skip(1).zip(colors.iter().skip(1)) {
                    let mut survey = self.surveys.get_mut(name).unwrap();
                    survey.draw::<P>(&self.raytracer, shaders, camera, color, 1.0);
                }
            },
            _ => unreachable!()
        }
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);

        if self.opacity > 0.0 {
            // Overlay
            let overlay_layer = &self.layers[1];
            match &overlay_layer {
                ImageSurveyIdx::Simple { name, color } => {
                    let mut survey = self.surveys.get_mut(name).unwrap();
                    survey.draw::<P>(&self.raytracer, shaders, camera, color, self.opacity);
                },
                ImageSurveyIdx::Composite { names, colors } => {
                    // All the hipses are plotted blended with the primary one
                    for (name, color) in names.iter().zip(colors.iter()) {
                        let mut survey = self.surveys.get_mut(name).unwrap();
                        survey.draw::<P>(&self.raytracer, shaders, camera, color, self.opacity);
                    }
                },
                // If no HiPS are overlaying we do nothing
                _ => ()
            }
        }
        self.gl.disable(WebGl2RenderingContext::BLEND);
    }

    // Return the layer idx in which the survey is contained
    fn contained_in_any_layer(&self, id: &str) -> Option<HashSet<usize>> {
        let mut layer_indices = HashSet::new();
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            if layer.contains(id) {
                layer_indices.insert(layer_idx);
            }
        }

        if layer_indices.is_empty() {
            None
        } else {
            Some(layer_indices)
        }
    }

    pub fn add_composite_surveys(&mut self, surveys: Vec<ImageSurvey>, colors: Vec<Color>, layer_idx: usize) -> Vec<String> {
        let names = surveys.iter()
            .map(|s| s.get_id().to_string())
            .collect::<Vec<String>>();

        let replaced_survey_names: Vec<String> = {
            let layer = &mut self.layers[layer_idx];
            match layer {
                ImageSurveyIdx::None => {
                    *layer = ImageSurveyIdx::Composite { names: names.clone(), colors };
                    vec![]
                },
                ImageSurveyIdx::Simple { name: cur_name, .. } => {
                    let cur_name = cur_name.clone();
                    *layer = ImageSurveyIdx::Composite { names: names.clone(), colors };

                    vec![cur_name]
                },
                ImageSurveyIdx::Composite { names: cur_names, .. } => {
                    let cur_names = cur_names.clone();
                    *layer = ImageSurveyIdx::Composite { names: names.clone(), colors };
                    cur_names
                }
            }
        };

        for replaced_survey_name in replaced_survey_names.iter() {
            // ensure cur_idx is not contained in any other layers
            if self.contained_in_any_layer(replaced_survey_name).is_none() {
                // if so we can remove it from the surveys hashmap
                self.surveys.remove(replaced_survey_name);
            }
        }

        //crate::log("END ADD");
        // If it is a new survey, insert it
        let mut new_survey_ids = Vec::new();
        for (name, survey) in names.iter().zip(surveys.into_iter()) {
            if !self.surveys.contains_key(name) {
                let id = name.to_string();
                self.surveys.insert(id.clone(), survey);
                new_survey_ids.push(id);
            }
        }

        new_survey_ids
    }

    pub fn remove_overlay(&mut self) {
        let replaced_survey_names: Vec<String> = {
            let layer = &mut self.layers[1];
            match layer {
                ImageSurveyIdx::None => {
                    *layer = ImageSurveyIdx::None;
                    vec![]
                },
                ImageSurveyIdx::Simple { name: cur_name, .. } => {
                    let cur_name = cur_name.clone();
                    *layer = ImageSurveyIdx::None;

                    vec![cur_name]
                },
                ImageSurveyIdx::Composite { names: cur_names, .. } => {
                    let cur_names = cur_names.clone();
                    *layer = ImageSurveyIdx::None;
                    cur_names
                }
            }
        };

        for replaced_survey_name in replaced_survey_names.iter() {
            // ensure cur_idx is not contained in any other layers
            if self.contained_in_any_layer(replaced_survey_name).is_none() {
                // if so we can remove it from the surveys hashmap
                self.surveys.remove(replaced_survey_name);
            }
        }
    }


    pub fn add_simple_survey(&mut self, survey: ImageSurvey, color: Color, layer_idx: usize) -> bool {
        let name = survey.get_id();

        let replaced_survey_names: Vec<String> = {
            let layer = &mut self.layers[layer_idx];
            match layer {
                ImageSurveyIdx::None => {
                    *layer = ImageSurveyIdx::Simple { name: name.to_string(), color };
                    vec![]
                },
                ImageSurveyIdx::Simple { name: cur_name, .. } => {
                    let cur_name = cur_name.clone();
                    *layer = ImageSurveyIdx::Simple { name: name.to_string(), color };

                    vec![cur_name]
                },
                ImageSurveyIdx::Composite { names: cur_names, .. } => {
                    let cur_names = cur_names.clone();
                    *layer = ImageSurveyIdx::Simple { name: name.to_string(), color };
                    cur_names
                }
            }
        };

        for replaced_survey_name in replaced_survey_names.iter() {
            // ensure cur_idx is not contained in any other layers
            if self.contained_in_any_layer(replaced_survey_name).is_none() {
                // if so we can remove it from the surveys hashmap
                self.surveys.remove(replaced_survey_name);
            }
        }

        //crate::log("END ADD");
        // If it is a new survey, insert it
        if !self.surveys.contains_key(name) {
            self.surveys.insert(name.to_string(), survey);
            true
        } else {
            //crate::log("no new");
            false
        }
    }

    pub fn get_view(&self) -> Option<&HEALPixCellsInView> {
        if self.surveys.is_empty() {
            None
        } else {
            let primary_layer = &self.layers[0];

            match primary_layer {
                ImageSurveyIdx::Simple { name, .. } => {
                    Some(self.surveys.get(name).unwrap().get_view())
                },
                ImageSurveyIdx::Composite { names, .. } => {
                    let name = names.first().unwrap();
                    Some(self.surveys.get(name).unwrap().get_view())
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
            if let Some(survey) = self.surveys.get_mut(&tile.root_url) {
                let textures = survey.get_textures_mut();
                match result {
                    TileResolved::Missing { time_req } => {
    
                        let default_image = textures.config().get_black_tile();
                        textures.push::<Rc<TileArrayBufferImage>>(tile, default_image, time_req);
                    },
                    TileResolved::Found { image, time_req } => {
                        match image {
                            RetrievedImageType::FITSImage { image, metadata } => {
                                // Update the metadata found in the header of the
                                // FITS tile received
                                let blank = metadata.blank;
                                //self.set_metadata_fits_surveys(&tile.root_url, metadata);
    
                                textures.config.blank = metadata.blank;
                                textures.config.scale = metadata.bscale;
                                textures.config.offset = metadata.bzero;
                                // Update the blank textures
                                textures.config.set_black_tile_value(metadata.bscale*blank + metadata.bzero);
    
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