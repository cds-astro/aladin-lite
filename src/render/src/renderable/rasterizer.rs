use crate::{
    WebGl2Context,
    shader::{Shader, ShaderBound, ShaderManager},
    healpix_cell::SphereSubdivided,
    renderable::Angle,
    core::VecData
};
use cgmath::Vector3;

use web_sys::WebGl2RenderingContext;

use crate::core::VertexArrayObject;
use web_sys::WebGlVertexArrayObject;
pub struct Rasterizer {
    gl: WebGl2Context,

    vao: WebGlVertexArrayObject,
}

use crate::{
    utils,
    camera::UserAction,
    renderable::image_survey::{Zoom, UnZoom, Move},
    buffer::HiPSConfig
};
use std::mem;
use super::image_survey::{MAX_NUM_VERTICES_TO_DRAW, TexturesToDraw};
impl Rasterizer {
    pub fn new(gl: &WebGl2Context, shaders: &mut ShaderManager) -> Rasterizer {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, (2 * mem::size_of::<f32>()) as i32, (0 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec3 position;
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, (3 * mem::size_of::<f32>()) as i32, (MAX_NUM_VERTICES_TO_DRAW * 2 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec3 uv_start;
        gl.vertex_attrib_pointer_with_i32(2, 3, WebGl2RenderingContext::FLOAT, false, (3 * mem::size_of::<f32>()) as i32, (MAX_NUM_VERTICES_TO_DRAW * 5 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(2);

        // layout (location = 3) in vec3 uv_end;
        gl.vertex_attrib_pointer_with_i32(3, 3, WebGl2RenderingContext::FLOAT, false, (3 * mem::size_of::<f32>()) as i32, (MAX_NUM_VERTICES_TO_DRAW * 8 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(3);

        // layout (location = 4) in float time_tile_received;
        gl.vertex_attrib_pointer_with_i32(4, 1, WebGl2RenderingContext::FLOAT, false, (1 * mem::size_of::<f32>()) as i32, (MAX_NUM_VERTICES_TO_DRAW * 11 * mem::size_of::<f32>()) as i32);
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
            num_idx,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0
        );            
    }
}