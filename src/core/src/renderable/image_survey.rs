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
            ending_texture,
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

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for TexturesToDraw<'a> {
    fn deref_mut(&'_ mut self) -> &'_ mut Self::Target {
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

impl RecomputeRasterizer for Move {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        _camera: &CameraViewPort,
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
        let mut textures = TexturesToDraw::new(cells_to_draw.len());

        for cell in cells_to_draw.iter() {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
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
    fn get_textures_from_survey<'a>(
        _camera: &CameraViewPort,
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
        let mut textures = TexturesToDraw::new(cells_to_draw.len());

        for cell in cells_to_draw.iter() {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
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
    fn get_textures_from_survey<'a>(
        camera: &CameraViewPort,
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let depth = view.get_depth();
        let max_depth = survey.config().get_max_depth();

        // We do not draw the parent cells if the depth has not decreased by at least one
        let cells_to_draw = if depth < max_depth && view.has_depth_decreased_while_unzooming(camera)
        {
            Cow::Owned(crate::renderable::view_on_surveys::get_cells_in_camera(
                depth + 1,
                camera,
            ))
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

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let starting_cell = if survey.contains(&cell) {
                    *cell
                } else {
                    survey.get_nearest_parent(&parent_cell)
                };

                let ending_cell = starting_cell;

                let starting_cell_in_tex = survey.get(&starting_cell).unwrap();
                let ending_cell_in_tex = survey.get(&ending_cell).unwrap();

                textures.insert(
                    *cell,
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
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

use crate::shaders::Colormap;

trait Draw {
    fn draw<P: Projection>(
        &mut self,
        raytracer: &RayTracer,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &Color,
        opacity: f32,
        blank_pixel_color: &color::Color
    );
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
        shader
            .attach_uniforms_from(&self.h)
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
    },
}

impl Color {
    pub fn get_raster_shader<'a, P: Projection>(
        &self,
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
        integer_tex: bool,
    ) -> &'a Shader {
        match self {
            Color::Colored => P::get_raster_shader_color(gl, shaders),
            Color::Grayscale2Colormap { .. } => {
                if integer_tex {
                    P::get_raster_shader_gray2colormap_integer(gl, shaders)
                } else {
                    P::get_raster_shader_gray2colormap(gl, shaders)
                }
            }
            Color::Grayscale2Color { .. } => {
                if integer_tex {
                    P::get_raster_shader_gray2color_integer(gl, shaders)
                } else {
                    P::get_raster_shader_gray2color(gl, shaders)
                }
            }
        }
    }

    pub fn get_raytracer_shader<'a, P: Projection>(
        &self,
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
        integer_tex: bool,
    ) -> &'a Shader {
        match self {
            Color::Colored => P::get_raytracer_shader_color(gl, shaders),
            Color::Grayscale2Colormap { .. } => {
                if integer_tex {
                    P::get_raytracer_shader_gray2colormap_integer(gl, shaders)
                } else {
                    P::get_raytracer_shader_gray2colormap(gl, shaders)
                }
            }
            Color::Grayscale2Color { .. } => {
                if integer_tex {
                    P::get_raytracer_shader_gray2color_integer(gl, shaders)
                } else {
                    P::get_raytracer_shader_gray2color(gl, shaders)
                }
            }
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
            }
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
// There is 13 floats per vertices (lonlat, pos, uv_start, uv_end, time_start, m0, m1) = 2 + 2 + 3 + 3 + 1 + 1 + 1 = 13
const MAX_NUM_FLOATS_TO_DRAW: usize = MAX_NUM_VERTICES_TO_DRAW * 13;
const MAX_NUM_INDICES_TO_DRAW: usize = MAX_NUM_CELLS_TO_DRAW * 6;

use cgmath::{Vector3, Vector4};
use std::mem;

use crate::renderable::uv::{TileCorner, TileUVW};

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
    miss_0: f32,
    miss_1: f32,

    alpha: f32,

    camera: &CameraViewPort,
) {
    let num_subdivision = E::num_subdivision::<P>(cell, sphere_sub);

    let n_segments_by_side: usize = 1 << num_subdivision;
    let lonlat = cdshealpix::grid_lonlat::<f64>(cell, n_segments_by_side as u16);

    let n_vertices_per_segment = n_segments_by_side + 1;

    let off_idx_vertices = (vertices.len() / 13) as u16;
    //let mut valid = vec![vec![true; n_vertices_per_segment]; n_vertices_per_segment];
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
                uv_0[TileCorner::BottomLeft].z,
            );

            let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
            let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;
            let uv_e_vertex_0 = Vector3::new(
                uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                uv_1[TileCorner::BottomLeft].z,
            );

            let (lon, lat) = (lonlat[id_vertex_0].lon().0, lonlat[id_vertex_0].lat().0);
            let model_pos: Vector4<f64> = lonlat[id_vertex_0].vector();
            // The projection is defined whatever the projection is
            // because this code is executed for small fovs (~<100deg depending
            // of the projection).
            if let Some(ndc_pos) = P::model_to_ndc_space(&model_pos, camera) {
                vertices.extend(
                    [
                        lon as f32,
                        lat as f32,
                        ndc_pos.x as f32,
                        ndc_pos.y as f32,
                        uv_s_vertex_0.x,
                        uv_s_vertex_0.y,
                        uv_s_vertex_0.z,
                        uv_e_vertex_0.x,
                        uv_e_vertex_0.y,
                        uv_e_vertex_0.z,
                        alpha,
                        miss_0,
                        miss_1,
                    ]
                    .iter(),
                );
            } else {
                //valid[i][j] = false;
                vertices.extend(
                    [
                        lon as f32,
                        lat as f32,
                        1.0,
                        0.0,
                        uv_s_vertex_0.x,
                        uv_s_vertex_0.y,
                        uv_s_vertex_0.z,
                        uv_e_vertex_0.x,
                        uv_e_vertex_0.y,
                        uv_e_vertex_0.z,
                        alpha,
                        miss_0,
                        miss_1,
                    ]
                    .iter(),
                );
            }
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

use web_sys::{WebGlBuffer, WebGlVertexArrayObject};
pub struct ImageSurvey {
    //color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: HEALPixCellsInView,

    // The projected vertices data
    vertices: Vec<f32>,
    idx_vertices: Vec<u16>,

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
use crate::camera::UserAction;
use crate::utils;

use web_sys::WebGl2RenderingContext;
impl ImageSurvey {
    fn new(
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        _surveys: &ImageSurveys,
        config: HiPSConfig,
        //color: Color,
        exec: Rc<RefCell<TaskExecutor>>,
        //_type: ImageSurveyType
    ) -> Result<Self, JsValue> {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        let data = vec![0.0_f32; MAX_NUM_FLOATS_TO_DRAW];
        let size_vertices_buf = MAX_NUM_FLOATS_TO_DRAW as u32;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let num_bytes_per_f32 = mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (0 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec2 position;
        gl.vertex_attrib_pointer_with_i32(
            1,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (2 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec3 uv_start;
        gl.vertex_attrib_pointer_with_i32(
            2,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (4 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(2);

        // layout (location = 3) in vec3 uv_end;
        gl.vertex_attrib_pointer_with_i32(
            3,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (7 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(3);

        // layout (location = 4) in float time_tile_received;
        gl.vertex_attrib_pointer_with_i32(
            4,
            1,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (10 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(4);

        // layout (location = 5) in float m0;
        gl.vertex_attrib_pointer_with_i32(
            5,
            1,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (11 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(5);

        // layout (location = 6) in float m1;
        gl.vertex_attrib_pointer_with_i32(
            6,
            1,
            WebGl2RenderingContext::FLOAT,
            false,
            13 * num_bytes_per_f32,
            (12 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(6);

        let ebo = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        let data = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];
        let size_idx_vertices_buf = MAX_NUM_INDICES_TO_DRAW as u32;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            unsafe { &js_sys::Uint16Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
        gl.bind_vertex_array(None);

        let num_idx = 0;
        let sphere_sub = SphereSubdivided {};

        let textures = ImageSurveyTextures::new(gl, config, exec)?;
        let conf = textures.config();
        let view = HEALPixCellsInView::new(conf.get_tile_size(), conf.get_max_depth(), camera);

        let gl = gl.clone();

        let vertices = vec![];
        let idx_vertices = vec![];

        Ok(ImageSurvey {
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
            vertices,
            idx_vertices,

            //_type,
            size_vertices_buf,
            size_idx_vertices_buf,
        })
    }

    pub fn set_vertices<P: Projection>(&mut self, camera: &CameraViewPort) {
        let last_user_action = camera.get_last_user_action();
        match last_user_action {
            UserAction::Unzooming => {
                self.update_vertices::<P, UnZoom>(camera);
            }
            UserAction::Zooming => {
                self.update_vertices::<P, Zoom>(camera);
            }
            UserAction::Moving => {
                self.update_vertices::<P, Move>(camera);
            }
            UserAction::Starting => {
                self.update_vertices::<P, Move>(camera);
            }
        }
    }

    fn update_vertices<P: Projection, T: RecomputeRasterizer>(&mut self, camera: &CameraViewPort) {
        let textures = T::get_textures_from_survey(camera, &mut self.view, &self.textures);

        self.vertices.clear();
        self.idx_vertices.clear();

        let survey_config = self.textures.config();

        for (cell, state) in textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture, survey_config);
            let uv_1 = TileUVW::new(cell, &state.ending_texture, survey_config);
            let start_time = state.ending_texture.start_time();
            let miss_0 = state.starting_texture.is_missing() as f32;
            let miss_1 = state.ending_texture.is_missing() as f32;

            add_vertices_grid::<P, T>(
                &mut self.vertices,
                &mut self.idx_vertices,
                &cell,
                &self.sphere_sub,
                &uv_0,
                &uv_1,
                miss_0,
                miss_1,
                start_time.as_millis(),
                camera,
            );
        }

        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
        self.gl.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&self.ebo),
        );

        let buf_vertices = unsafe { js_sys::Float32Array::view(&self.vertices) };
        if self.vertices.len() > self.size_vertices_buf as usize {
            self.size_vertices_buf = self.vertices.len() as u32;

            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &buf_vertices,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        } else {
            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                0,
                &buf_vertices,
            );
        }

        self.num_idx = self.idx_vertices.len();
        let buf_idx = unsafe { js_sys::Uint16Array::view(&self.idx_vertices) };
        if self.idx_vertices.len() > self.size_idx_vertices_buf as usize {
            self.size_idx_vertices_buf = self.idx_vertices.len() as u32;
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &buf_idx,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        } else {
            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                0,
                &buf_idx,
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
}

impl Drop for ImageSurvey {
    fn drop(&mut self) {
        drop(&mut self.textures);

        // Drop the vertex arrays
        self.gl.delete_buffer(Some(&self.vbo));
        self.gl.delete_buffer(Some(&self.ebo));

        self.gl.delete_vertex_array(Some(&self.vao));
    }
}

use std::borrow::Cow;
use crate::color;
impl Draw for ImageSurvey {
    fn draw<P: Projection>(
        &mut self,
        raytracer: &RayTracer,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &Color,
        opacity: f32,
        blank_pixel_color: &color::Color,
    ) {
        if !self.textures.is_ready() {
            // Do not render while the 12 base cell textures
            // are not loaded
            return;
        }

        let textures_array = self.textures.get_texture_array();
        let survey_storing_integers = self.textures.config.tex_storing_integers == 1;

        let raytracing = camera.get_aperture() > P::RASTER_THRESHOLD_ANGLE;
        //let raytracing = camera.is_allsky();
        if raytracing {
            //raytracer.bind();
            let shader = color
                .get_raytracer_shader::<P>(&self.gl, shaders, survey_storing_integers)
                .bind(&self.gl);

            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(&*textures_array)
                .attach_uniforms_from(color)
                .attach_uniform("blank_color", &blank_pixel_color)
                .attach_uniform("current_depth", &(self.view.get_cells().get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity);

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
            let recompute_vertices =
                recompute_positions | self.textures.is_there_available_tiles() | camera.has_moved();
            self.gl.bind_vertex_array(Some(&self.vao));
            if recompute_vertices {
                self.set_vertices::<P>(camera);
            }

            let shader = color
                .get_raster_shader::<P>(&self.gl, shaders, survey_storing_integers)
                .bind(&self.gl);

            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(&*textures_array)
                .attach_uniforms_from(color)
                .attach_uniform("blank_color", &blank_pixel_color)
                .attach_uniform("current_depth", &(self.view.get_cells().get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity);

            // The raster vao is bound at the lib.rs level
            self.gl.draw_elements_with_i32(
                //WebGl2RenderingContext::LINES,
                WebGl2RenderingContext::TRIANGLES,
                self.num_idx as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );
        }
    }
}

use wasm_bindgen::JsValue;
pub trait HiPS {
    fn create(
        self,
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        surveys: &ImageSurveys,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<ImageSurvey, JsValue>;
    fn color(&self) -> Color;
}

use crate::{HiPSColor, SimpleHiPS};
use std::cell::RefCell;
use std::rc::Rc;

impl HiPS for SimpleHiPS {
    fn color(&self) -> Color {
        let color = match self.color.clone() {
            HiPSColor::Color => Color::Colored,
            HiPSColor::Grayscale2Color { color, transfer, k } => Color::Grayscale2Color {
                color,
                k,
                param: GrayscaleParameter {
                    h: transfer.into(),
                    min_value: self.properties.min_cutout.unwrap_or(0.0),
                    max_value: self.properties.max_cutout.unwrap_or(1.0),
                },
            },
            HiPSColor::Grayscale2Colormap { colormap, transfer } => Color::Grayscale2Colormap {
                colormap: colormap.into(),
                param: GrayscaleParameter {
                    h: transfer.into(),
                    min_value: self.properties.min_cutout.unwrap_or(0.0),
                    max_value: self.properties.max_cutout.unwrap_or(1.0),
                },
            },
        };

        color
    }

    fn create(
        self,
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        surveys: &ImageSurveys,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<ImageSurvey, JsValue> {
        let SimpleHiPS { properties, .. } = self;

        let config = HiPSConfig::new(gl, &properties)?;
        let survey = ImageSurvey::new(
            gl, camera, surveys, config, exec,
        )?;

        Ok(survey)
    }
}

#[derive(Debug)]
struct ImageSurveyLayer {
    opacity: f32,
    names: Vec<String>,
    colors: Vec<Color>,
    name_most_precised_survey: String,
}

type LayerName = String;
use crate::renderable::view_on_surveys::HEALPixCellsInView;
pub struct ImageSurveys {
    surveys: HashMap<String, ImageSurvey>,

    ordered_layer_names: Vec<LayerName>,
    layers: HashMap<LayerName, ImageSurveyLayer>,

    raytracer: RayTracer,
    gl: WebGl2Context,
}

use crate::buffer::Tiles;
use crate::buffer::{ResolvedTiles, RetrievedImageType, TileResolved};
use crate::buffer::{TileArrayBufferImage, TileHTMLImage};

use crate::Resources;
use crate::coo_conversion::CooSystem;
impl ImageSurveys {
    pub fn new<P: Projection>(
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        rs: &Resources,
        system: &CooSystem,
    ) -> Self {
        let surveys = HashMap::new();
        let layers = HashMap::new();

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new::<P>(&gl, &camera, shaders, rs, system);

        let gl = gl.clone();

        let ordered_layer_names = vec![];
        ImageSurveys {
            surveys,
            ordered_layer_names,

            layers,

            raytracer,

            gl,
        }
    }

    pub fn set_projection<P: Projection>(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        rs: &Resources,
        system: &CooSystem,
    ) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders, rs, system);
    }

    pub fn set_longitude_reversed<P: Projection>(
        &mut self,
        _reversed: bool,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        rs: &Resources,
        system: &CooSystem,
    ) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl, camera, shaders, rs, system);
    }

    pub fn set_opacity_layer(&mut self, layer: &str, opacity: f32) -> Result<(), JsValue> {
        if let Some(layer) = self.layers.get_mut(layer) {
            layer.opacity = opacity;
            Ok(())
        } else {
            Err(JsValue::from_str(&format!("layer {} not found", layer)))
        }
    }

    pub fn move_image_surveys_layer_forward(&mut self, layer: &str) -> Result<(), JsValue> {
        let pos = self.ordered_layer_names.iter()
            .position(|l| l == layer);

        if let Some(pos) = pos {
            let forward_pos = self.ordered_layer_names.len() - 1;
            self.ordered_layer_names.swap(pos, forward_pos);

            Ok(())
        } else {
            Err(JsValue::from_str(&format!("layer {} not found", layer)))
        }
    }

    pub fn draw<P: Projection>(&mut self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        let raytracing = camera.get_aperture() > P::RASTER_THRESHOLD_ANGLE;

        if raytracing {
            self.raytracer.bind();
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        } else if camera.is_reversed_longitude() {
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        } else {
            self.gl.cull_face(WebGl2RenderingContext::FRONT);
        }

        let mut idx_survey = 0;
        for (layer_idx, layer_name) in self.ordered_layer_names.iter().enumerate() {
            let ImageSurveyLayer { names, colors, opacity, .. } = &self.layers[layer_name];
            if layer_idx == 0 {
                // The base layer
                self.gl.blend_func(
                    WebGl2RenderingContext::ONE,
                    WebGl2RenderingContext::ONE
                );
            } else {
                // The following layers
                self.gl.blend_func_separate(
                    WebGl2RenderingContext::SRC_ALPHA,
                    WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
                    WebGl2RenderingContext::ONE,
                    WebGl2RenderingContext::ONE,
                );
            }

            if opacity > &0.0 {
                for (name, color) in names.iter().zip(colors.iter()) {
                    // Enable the blending for the following HiPSes
                    let blank_pixel_color = if idx_survey == 0 {
                        // The very first survey has the blending disabled
                        self.gl.disable(WebGl2RenderingContext::BLEND);
                        color::Color::new(0.0, 0.0, 0.0, 1.0)
                    } else {
                        self.gl.enable(WebGl2RenderingContext::BLEND);
                        color::Color::new(0.0, 0.0, 0.0, 0.0)
                    };

                    let survey = self.surveys.get_mut(name).unwrap();
                    survey.draw::<P>(&self.raytracer, shaders, camera, color, *opacity, &blank_pixel_color);

                    idx_survey += 1;
                }
            }
        }

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        self.gl.disable(WebGl2RenderingContext::BLEND);
    }

    pub fn set_image_surveys(
        &mut self,
        hipses: Vec<SimpleHiPS>,
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<Vec<String>, JsValue> {
        // retrieve the max depth of the surveys composing the layer

        let mut layers = HashMap::new();
        let mut new_survey_ids = Vec::new();

        let mut current_needed_surveys = HashSet::new();
        self.ordered_layer_names.clear();
        for hips in hipses.into_iter() {
            let layer_name = &hips.layer;
            if !self.ordered_layer_names.contains(layer_name) {
                if layer_name == "base" {
                    self.ordered_layer_names.insert(0, layer_name.to_string());
                } else {
                    self.ordered_layer_names.push(layer_name.to_string());
                }
            }

            let url = hips.properties.url.clone();

            if !layers.contains_key(layer_name) {
                let opacity = 1.0;
                layers.insert(
                    layer_name.clone(),
                    ImageSurveyLayer {
                        names: vec![url.clone()],
                        colors: vec![hips.color()],
                        opacity,
                        name_most_precised_survey: url.clone()
                    }
                );
            } else {
                let layer = layers.get_mut(layer_name).unwrap();

                if layer.names.contains(&url) {
                    continue;
                } else {
                    layer.names.push(url.clone());
                    layer.colors.push(hips.color());
                }
            }

            // Add the new surveys
            if !self.surveys.contains_key(&url) {
                // create the survey
                let survey = hips.create(gl, camera, self, exec.clone())?;
                self.surveys.insert(url.clone(), survey);
                new_survey_ids.push(url.clone());
            }

            current_needed_surveys.insert(url);
        }
        self.layers = layers;

        // loop over the surveys to remove the one that are not needed anymore
        let mut surveys_to_remove = vec![];
        for (name, _) in self.surveys.iter() {
            if !current_needed_surveys.contains(name) {
                surveys_to_remove.push(name.clone());
            }
        }

        for survey_to_remove in &surveys_to_remove {
            self.surveys.remove(survey_to_remove);
        }

        crate::log(&format!("layers {:?}", self.layers));
        crate::log(&format!("list of surveys {:?}", self.surveys.keys()));

        Ok(new_survey_ids)
    }

    pub fn is_ready(&self) -> bool {
        let ready = self.surveys
            .iter()
            .map(|(_, survey)| {
                survey.textures.is_ready()
            })
            .fold(true, |acc, x| acc & x);
        
        ready
    }

    pub fn get_view(&self) -> Option<&HEALPixCellsInView> {
        if self.surveys.is_empty() {
            None
        } else {
            let (_, layer) = &self.layers.iter().next().unwrap();
            let name = &layer.name_most_precised_survey;
            Some(
                self.surveys.get(name)
                    .unwrap()
                    .get_view()
            )
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
            let textures = &mut self
                .surveys
                .get_mut(&tile.root_url)
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
                        let missing = true;
                        let default_image = textures.config().get_black_tile();
                        textures.push::<Rc<TileArrayBufferImage>>(
                            tile,
                            default_image,
                            time_req,
                            missing,
                        );
                    }
                    TileResolved::Found { image, time_req } => {
                        let missing = false;
                        match image {
                            RetrievedImageType::FITSImage { image, metadata } => {
                                // Update the metadata found in the header of the
                                // FITS tile received
                                if let Some(blank) = metadata.blank {
                                    textures.config.blank = blank;
                                    textures.config.set_black_tile_value(blank);
                                }
                                textures.config.scale = metadata.bscale;
                                textures.config.offset = metadata.bzero;
                                //self.set_metadata_fits_surveys(&tile.root_url, metadata);
                                // Update the blank textures
                                //textures.config.set_black_tile_value(0.0);
                                textures
                                    .push::<TileArrayBufferImage>(tile, image, time_req, missing);
                            }
                            RetrievedImageType::CompressedImage { image } => {
                                textures.push::<TileHTMLImage>(tile, image, time_req, missing);
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

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, String, ImageSurvey> {
        self.surveys.iter_mut()
    }
}

use crate::{
    async_task::TaskExecutor,
    buffer::HiPSConfig,
    shader::ShaderManager,
};
use std::collections::hash_map::IterMut;

use crate::TransferFunction;