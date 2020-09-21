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
use crate::viewport::ViewPort;
use crate::renderable::RecomputeRasterizer;
use crate::time::Time;
fn add_cell_vertices<P: Projection, E: RecomputeRasterizer>(
    sphere_sub: &SphereSubdivided,
    vertices: &mut Vec<f32>,
    idx_vertices: &mut Vec<u16>,
    //num_vertices: &mut usize,
    //num_idx: &mut u16,
    cell: &HEALPixCell,
    uv_0: &TileUVW,
    uv_1: &TileUVW,
    alpha: f32,
) {
    let num_subdivision = E::num_subdivision::<P>(cell, sphere_sub);
    add_vertices_grid(
        vertices,
        idx_vertices,
        //num_vertices,
        //num_idx,
        cell,
        num_subdivision,
        uv_0,
        uv_1,
        alpha,
    );
}

use crate::cdshealpix;
fn add_vertices_grid(
    vertices: &mut Vec<f32>,
    idx_vertices: &mut Vec<u16>,
    //num_vertices: &mut usize,
    //num_idx: &mut u16,
    cell: &HEALPixCell,
    num_subdivision: u8,
    uv_0: &TileUVW,
    uv_1: &TileUVW,
    alpha: f32
) {
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

            Vertex::new(&lonlat[id_vertex_0], uv_s_vertex_0, uv_e_vertex_0, alpha)
                .add_to_vertices(vertices);
        }
    }

    for i in 0..n_segments_by_side {
        for j in 0..n_segments_by_side {
            let idx_0 = (j + i * n_vertices_per_segment) as u16;
            let idx_1 = (j + 1 + i * n_vertices_per_segment) as u16;
            let idx_2 = (j + (i + 1) * n_vertices_per_segment) as u16;
            let idx_3 = (j + 1 + (i + 1) * n_vertices_per_segment) as u16;

            idx_vertices.push(off_idx_vertices + idx_0);
            idx_vertices.push(off_idx_vertices + idx_1);
            idx_vertices.push(off_idx_vertices + idx_2);

            idx_vertices.push(off_idx_vertices + idx_1);
            idx_vertices.push(off_idx_vertices + idx_3);
            idx_vertices.push(off_idx_vertices + idx_2);
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
    vertices: Vec<f32>,
    idx_vertices: Vec<u16>,
    //num_vertices: usize,
    //num_idx: u16,

    sphere_sub: SphereSubdivided,

    vertex_array_object: VertexArrayObject,
}

use crate::{
    renderable::TextureStates,
    utils,
    buffer::BufferTextures,
    viewport::LastAction,
    renderable::hips_sphere::{Zoom, UnZoom, Move},
    buffer::HiPSConfig
};
impl Rasterizer {
    pub fn new(gl: &WebGl2Context, shaders: &mut ShaderManager) -> Rasterizer {
        // Define the Vertex Array Object where vertices data will be put
        // Memory reserved from the stack
        let vertices = vec![];
        let idx_vertices = vec![];
        let mut vertex_array_object = VertexArrayObject::new(gl);

        let shader = shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("RasterizerOrthoVS"),
                Cow::Borrowed("RasterizerFS"),
            )
        ).unwrap();
        shader.bind(gl)
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

        let sphere_sub = SphereSubdivided::new();
        Rasterizer {
            vertices,
            idx_vertices,

            sphere_sub,

            vertex_array_object,
        }
    }

    pub fn update<P: Projection>(&mut self, buffer: &mut BufferTextures, viewport: &ViewPort, config: &HiPSConfig) {
        let last_user_action = viewport.last_user_action();

        match last_user_action {
            LastAction::Unzooming => {
                let tile_textures = UnZoom::compute_texture_buffer::<P>(buffer, viewport);
                self.update_vertex_array_object::<P, UnZoom>(&tile_textures, config);
            },
            LastAction::Zooming => {
                let tile_textures = Zoom::compute_texture_buffer::<P>(buffer, viewport);
                self.update_vertex_array_object::<P, Zoom>(&tile_textures, config);
            },
            LastAction::Moving => {
                let tile_textures = Move::compute_texture_buffer::<P>(buffer, viewport);
                self.update_vertex_array_object::<P, Move>(&tile_textures, config);
            },
            LastAction::Starting => {
                let tile_textures = Move::compute_texture_buffer::<P>(buffer, viewport);
                self.update_vertex_array_object::<P, Move>(&tile_textures, config);
            }
        }
    }

    fn update_vertex_array_object<P: Projection, T: RecomputeRasterizer>(&mut self, tile_textures: &TextureStates, config: &HiPSConfig) {
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
    }

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