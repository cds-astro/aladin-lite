use al_core::FrameBufferObject;
use {
    js_sys::WebAssembly,
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::{
        WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader,
        WebGlTexture, WebGlVertexArrayObject,
    },
};

use std::borrow::Cow;

use al_core::log::*;
use al_core::{shader::Shader, VecData, VertexArrayObject};
use cgmath::Vector2;
use egui::{
    self,
    emath::vec2,
    epaint::{Color32, Texture},
};
use web_sys::console;
use al_core::WebGl2Context;
pub struct RenderPass {
    gl: WebGl2Context,
    vao: VertexArrayObject,
    shader: Shader,
}

impl RenderPass {
    pub fn new(gl: &WebGl2Context, width: i32, height: i32) -> Result<RenderPass, JsValue> {
        let shader = Shader::new(
            &gl,
            include_str!("../shaders/passes/post_vertex_100es.glsl"),
            include_str!("../shaders/passes/post_fragment_100es.glsl"),
        )?;

        let positions = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let indices = vec![0u8, 1, 2, 1, 3, 2];
        let mut vao = VertexArrayObject::new(&gl);
        shader
            .bind(&gl)
                .bind_vertex_array_object(&mut vao)
                    // positions and texcoords buffers
                    .add_array_buffer(
                        2 * std::mem::size_of::<f32>(),
                        &[2],
                        &[0],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(&positions),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(&indices),
                    )
            // Unbind the buffer
            .unbind();

        Ok(RenderPass {
            gl: gl.clone(),
            vao,
            shader,
        })
    }

    pub fn draw_on_screen(&self, fbo: &FrameBufferObject) {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA); // premultiplied alpha

        self.shader.bind(&self.gl)
            .attach_uniform("fbo_tex", &fbo.texture)
            .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(WebGl2RenderingContext::TRIANGLES, Some(6), WebGl2RenderingContext::UNSIGNED_BYTE, 0);

        self.gl.disable(WebGl2RenderingContext::BLEND);
    }
}
