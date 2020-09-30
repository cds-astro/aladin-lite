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
pub trait GetShader {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;

    fn get_raytracer_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RayTracerVS"),
                Cow::Borrowed("RayTracerColorFS")
            )
        ).unwrap();
    }
    fn get_raytracer_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RayTracerVS"),
                Cow::Borrowed("RayTracerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raytracer_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RayTracerVS"),
                Cow::Borrowed("RayTracerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}

impl GetShader for Aitoff {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerColorFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerAitoffVS"),
                Cow::Borrowed("RasterizerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}
impl GetShader for Mollweide {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollweideVS"),
                Cow::Borrowed("RasterizerColorFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollweideVS"),
                Cow::Borrowed("RasterizerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMollweideVS"),
                Cow::Borrowed("RasterizerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}
impl GetShader for AzimutalEquidistant {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerColorFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}
impl GetShader for Mercator {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerColorFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerMercatorVS"),
                Cow::Borrowed("RasterizerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}
impl GetShader for Orthographic {
    fn get_raster_shader_color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerColorFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2colormap<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerGrayscale2ColormapFS")
            )
        ).unwrap();
    }
    fn get_raster_shader_gray2color<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerGrayscale2ColorFS")
            )
        ).unwrap();
    }
}

use crate::core::VertexArrayObject;
pub struct Rasterizer {
    gl: WebGl2Context,

    vao: WebGlVertexArrayObject,
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
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 2 * mem::size_of::<f32>(), 0 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec3 position;
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), MAX_NUM_VERTICES_TO_DRAW * 2 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec3 uv_start;
        gl.vertex_attrib_pointer_with_i32(2, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), MAX_NUM_VERTICES_TO_DRAW * 5 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(2);

        // layout (location = 3) in vec3 uv_end;
        gl.vertex_attrib_pointer_with_i32(3, 3, WebGl2RenderingContext::FLOAT, false, 3 * mem::size_of::<f32>(), MAX_NUM_VERTICES_TO_DRAW * 8 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(3);

        // layout (location = 4) in float time_tile_received;
        gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::FLOAT, false, 1 * mem::size_of::<f32>(), MAX_NUM_VERTICES_TO_DRAW * 11 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(4);
        
        let gl = gl.clone();
        Rasterizer {
            gl,
            vao,
        }
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
    /*pub fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a mut ShaderManager, survey: &ImageSurvey) -> &'a Shader {
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
    }*/

    pub fn bind(&self) {
        self.gl.bind_vertex_array(Some(&self.vao));
    }

    pub fn draw(&self, num_idx: i32) {
        self.gl.draw_elements_with_i32(
            //WebGl2RenderingContext::LINES,
            WebGl2RenderingContext::TRIANGLES,
            Some(num_idx),
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0
        );            
    }
}