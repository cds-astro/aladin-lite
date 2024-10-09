use al_core::FrameBufferObject;
use al_core::WebGlContext;
use al_core::{VecData, VertexArrayObject};
use {wasm_bindgen::prelude::*, web_sys::WebGl2RenderingContext};
pub struct RenderPass {
    gl: WebGlContext,
    vao: VertexArrayObject,
}

use crate::ShaderManager;
impl RenderPass {
    pub fn new(gl: &WebGlContext) -> Result<RenderPass, JsValue> {
        let positions = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let indices = vec![0u8, 1, 2, 1, 3, 2];
        let mut vao = VertexArrayObject::new(gl);
        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            // positions and texcoords buffers
            .add_array_buffer(
                "vertices",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::STATIC_DRAW,
                VecData(&positions),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, VecData(&indices))
            // Unbind the buffer
            .unbind();
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            // positions and texcoords buffers
            .add_array_buffer(
                2,
                "a_pos",
                WebGl2RenderingContext::STATIC_DRAW,
                VecData(&positions),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, VecData(&indices))
            // Unbind the buffer
            .unbind();

        Ok(RenderPass {
            gl: gl.clone(),
            vao,
        })
    }

    pub fn draw_on_screen(
        &self,
        fbo: &FrameBufferObject,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        //self.gl.enable(WebGl2RenderingContext::BLEND);
        /*self.gl.blend_func(
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        ); // premultiplied alpha*/

        let shader = crate::shader::get_shader(
            &self.gl,
            shaders,
            "passes_post_vertex_100es.vert",
            "passes_post_fragment_100es.frag",
        )?;

        shader
            .bind(&self.gl)
            .attach_uniform("fbo_tex", &fbo.texture)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                Some(6),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                0,
            );

        //self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
