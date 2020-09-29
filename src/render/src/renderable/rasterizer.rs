use crate::{
    WebGl2Context,
    shader::{Shader, ShaderBound, ShaderManager},
    healpix_cell::SphereSubdivided,
    renderable::Angle,
    core::VecData
};
use cgmath::Vector3;

use web_sys::WebGl2RenderingContext;

#[repr(C)]
struct Position {
    lon: Angle<f32>,
    lat: Angle<f32>,

    pos: Vector3<f32>,
}

impl Position {
    fn new(lonlat: &LonLatT<f32>) -> Vertex {
        let pos = lonlat.vector();
        let lon = lonlat.lon();
        let lat = lonlat.lat();
        Vertex {
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
use crate::healpix_cell::HEALPixCell;
use crate::viewport::CameraViewPort;
use crate::renderable::RecomputeRasterizer;
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

            let (lon, lat) = lonlat[id_vertex_0];
            let position = lonlat[id_vertex_0].vector();

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

    let off_idx_vertices = (positions.len()/3) as u16;
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

use std::borrow::Cow;
use crate::renderable::projection::*;
use crate::shader::ShaderId;
pub trait RasterizerProjection {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
}

impl RasterizerProjection for Aitoff {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerFS")
            )
        ).unwrap()
    }
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerFITSFS")
            )
        ).unwrap()
    }
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerFITSIFS")
            )
        ).unwrap()    
    }
}
impl RasterizerProjection for Mollweide {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollVS"),
                Cow::Borrowed("RasterizerFS")
            )
        ).unwrap()
    }
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollVS"),
                Cow::Borrowed("RasterizerFITSFS")
            )
        ).unwrap()
    }
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollVS"),
                Cow::Borrowed("RasterizerFITSIFS")
            )
        ).unwrap()    
    }
}
impl RasterizerProjection for AzimutalEquidistant {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFS")
            )
        ).unwrap()
    }
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFITSFS")
            )
        ).unwrap()
    }
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFITSIFS")
            )
        ).unwrap()    
    }
}
impl RasterizerProjection for Mercator {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerFS")
            )
        ).unwrap()
    }
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerFITSFS")
            )
        ).unwrap()
    }
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerFITSIFS")
            )
        ).unwrap()    
    }
}
impl RasterizerProjection for Orthographic {
    fn get_rasterize_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFS")
            )
        ).unwrap()
    }
    // FITS HiPS are handled by different shaders
    fn get_rasterize_shader_f_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFITSFS")
            )
        ).unwrap()
    }
    fn get_rasterize_shader_i_fits<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFITSIFS")
            )
        ).unwrap()    
    }
}

use crate::core::VertexArrayObject;
pub struct Rasterizer {
    gl: WebGl2Context,
    //vertices: Vec<f32>,
    //idx_vertices: Vec<u16>,
    max_num_vertices: usize,
    max_num_idx: usize,
    //num_vertices: usize,
    //num_idx: u16,

    sphere_sub: SphereSubdivided,

    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
}

use crate::{
    renderable::TextureStates,
    utils,
    buffer::TileBuffer,
    viewport::LastAction,
    renderable::hips_sphere::{Zoom, UnZoom, Move},
    buffer::HiPSConfig
};
impl Rasterizer {
    pub fn new(gl: &WebGl2Context, shaders: &mut ShaderManager) -> Rasterizer {
        // Compute the size of the VBO in bytes
        // We do want to draw maximum 768 tiles
        let max_hpx_cells = 768;
        // Each cell has 4 vertices
        let max_num_vertices = max_hpx_cells * 4;
        // There is 12 floats per vertices (lonlat, pos, uv_start, uv_end, time_start) = 2 + 3 + 3 + 3 + 1 = 12
        let max_num_floats = max_num_vertices * 12;

        // Define the Vertex Array Object where vertices data will be put
        // Memory reserved from the stack
        //let vertices = vec![0.0; max_num_floats];
        let max_num_idx = max_hpx_cells * 6;
        let idx_vertices = vec![0; max_num_idx];
        //let mut vertex_array_object = VertexArrayObject::new(gl);

        let shader = shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFS"),
            )
        ).unwrap();

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            max_num_floats * std::mem::size_of::<f32>(),
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        {
            let shader_bound = shader.bind(gl);
            
            // layout (location = 0) in vec2 lonlat;
            gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 2 * mem::size_of::<f32>(), 0 * mem::size_of::<f32>());
            gl.enable_vertex_attrib_array(0);

            // layout (location = 1) in vec3 position;
            gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), max_num_vertices * 2 * mem::size_of::<f32>());
            gl.enable_vertex_attrib_array(1);

            // layout (location = 2) in vec3 uv_start;
            gl.vertex_attrib_pointer_with_i32(2, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), max_num_vertices * 5 * mem::size_of::<f32>());
            gl.enable_vertex_attrib_array(2);

            // layout (location = 3) in vec3 uv_end;
            gl.vertex_attrib_pointer_with_i32(3, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), max_num_vertices * 8 * mem::size_of::<f32>());
            gl.enable_vertex_attrib_array(3);

            // layout (location = 4) in float time_tile_received;
            gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::FLOAT, false, 1 * mem::size_of::<f32>(), max_num_vertices * 11 * mem::size_of::<f32>());
            gl.enable_vertex_attrib_array(4);
        }

        // Element buffer
        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            max_num_floats * std::mem::size_of::<u16>(),
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        /*
            .bind_vertex_array_object(&mut vertex_array_object)
            // Store the projeted and 3D vertex positions in a VBO
            .add_array_buffer(
                12 * mem::size_of::<f32>(),
                &[2, 3, 3, 3, 1],    
                &[
                    0,
                    2 * mem::size_of::<f32>(),
                    5 * mem::size_of::<f32>(),
                    8 * mem::size_of::<f32>(),
                    11 * mem::size_of::<f32>(),
                ],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&vertices),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&idx_vertices),
            )
            // Unbind the buffer
            .unbind();
        */

        let sphere_sub = SphereSubdivided::new();
        let gl = gl.clone();
        Rasterizer {
            gl,
            //vertices,
            //idx_vertices,
            max_num_vertices,
            max_num_idx,

            sphere_sub,

            vao,
            vbo
        }
    }

    pub fn set_positions<P: Projection>(&mut self, cells_to_draw: &HEALPixCells, last_user_action: UserAction) {
        match last_user_action {
            UserAction::Unzooming => {
                self.update_positions::<P, UnZoom>(&cells_to_draw);
            },
            UserAction::Zooming => {
                self.update_positions::<P, Zoom>(&cells_to_draw);
            },
            UserAction::Moving => {
                self.update_positions::<P, Move>(&cells_to_draw);
            },
            UserAction::Starting => {
                self.update_positions::<P, Move>(&cells_to_draw);
            }
        }
    }

    pub fn set_UVs<P: Projection>(&mut self, cells_to_draw: &HEALPixCells, survey: &ImageSurvey, last_user_action: UserAction) {
        match last_user_action {
            UserAction::Unzooming => {
                let textures = UnZoom::get_textures_from_survey(cells_to_draw, survey);
                self.update_uvs::<P, UnZoom>(textures);
            },
            UserAction::Zooming => {
                let textures = Zoom::get_textures_from_survey(cells_to_draw, survey);
                self.update_uvs::<P, Zoom>(textures);
            },
            UserAction::Moving => {
                let textures = Move::get_textures_from_survey(cells_to_draw, survey);
                self.update_uvs::<P, Move>(textures);
            },
            UserAction::Starting => {
                let textures = Move::get_textures_from_survey(cells_to_draw, survey);
                self.update_uvs::<P, Move>(textures);
            }
        }
    }

    fn update_positions<P: Projection, T: RecomputeRasterizer>(&mut self, cells_in_fov: &HEALPixCells) {
        let mut lonlats = vec![];
        let mut positions = vec![];
        let mut idx_vertices = vec![];

        for cell in cells_in_fov {
            add_positions_grid::<P, T>(
                &mut lonlats,
                &mut positions,
                &mut idx_vertices,
                &cell,
                &self.sphere_sub,
            );
        }

        self.gl.bind_vertex_array(Some(&self.vao));

        let mut coo = lonlats;
        let num_filling_floats = self.max_num_vertices * 2 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);
        coo.extend(positions);
        let num_filling_floats = self.max_num_vertices * 5 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);

        let buf_positions = unsafe { js_sys::Float32Array::view(&coo) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0 as i32,
            &buf_positions
        );

        self.max_num_idx = idx_vertices.len();
        let buf_idx = unsafe { js_sys::Uint16Array::view(&idx_vertices) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0 as i32,
            &buf_idx
        );
    }

    fn update_uvs<P: Projection, T: RecomputeRasterizer>(&mut self, textures: &ImageSurveyTextures) {
        let mut uv_start = vec![];
        let mut uv_end = vec![];
        let mut start_times = vec![];

        for (cell, state) in textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture);
            let uv_1 = TileUVW::new(cell, &state.ending_texture);
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

        self.gl.bind_vertex_array(Some(&self.vao));

        let mut uv = uv_start;
        let num_filling_floats = self.max_num_vertices * 3 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(uv_end);
        let num_filling_floats = self.max_num_vertices * 6 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(start_time);
        let num_filling_floats = self.max_num_vertices * 7 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        let buf_uvs = unsafe { js_sys::Float32Array::view(&uv) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            self.max_num_vertices * 5 * std::mem::size_of::<f32>() as i32,
            &buf_uvs
        );
    }

    /*fn update_vertex_array_object<P: Projection, T: RecomputeRasterizer>(&mut self, tile_textures: &TextureStates, config: &HiPSConfig) {
        self.vertices.clear();
        self.idx_vertices.clear();

        for (cell, state) in tile_textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture, config);
            let uv_1 = TileUVW::new(cell, &state.ending_texture, config);
            let start_time = state.ending_texture.start_time();

            add_cell_vertices::<P, T>(
                &self.sphere_sub,
                &mut self.vertices,
                &mut self.idx_vertices,
                &cell,
                &uv_0, &uv_1,
                start_time.as_millis(),
            );
        }

        // Update the VAO
        self.vertex_array_object
            .bind_for_update()
            .update_array(
                0, 
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.vertices)
            )
            .update_element_array(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.idx_vertices)
            );
    }*/

    // The rasterizer has several shaders, one for each projection
    pub fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a mut ShaderManager, survey: &ImageSurvey) -> &'a Shader {
        // Fits tiles are handled by other shaders
        if buffer.fits_tiles_requested() {
            if buffer.fits_i_format() {
                P::get_rasterize_shader_i_fits(gl, shaders)
            } else {
                P::get_rasterize_shader_f_fits(gl, shaders)
            }
        } else {
            P::get_rasterize_shader(gl, shaders)
        }
    }

    pub fn draw<P: Projection>(&self, _gl: &WebGl2Context, shader: &ShaderBound) {
        shader.bind_vertex_array_object_ref(&self.vertex_array_object)
            .draw_elements_with_i32(
                //WebGl2RenderingContext::LINES,
                WebGl2RenderingContext::TRIANGLES,
                Some(self.idx_vertices.len() as i32),
                WebGl2RenderingContext::UNSIGNED_SHORT
            );
    }
}