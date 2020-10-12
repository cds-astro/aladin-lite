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

    /*pub fn bind(&self) {
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
    }*/
}

impl Drop for Rasterizer {
    fn drop(&mut self) {
        self.gl.delete_vertex_array(Some(&self.vao));
    }
}