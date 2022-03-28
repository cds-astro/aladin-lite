use al_core::FrameBufferObject;
use {
    wasm_bindgen::{prelude::*},
    web_sys::{
        WebGl2RenderingContext,
    },
};
use al_core::{shader::Shader, VecData, VertexArrayObject};
use al_core::WebGlContext;
pub struct RenderPass {
    gl: WebGlContext,
    vao: VertexArrayObject,
    shader: Shader,
}

impl RenderPass {
    pub fn new(gl: &WebGlContext, _width: i32, _height: i32) -> Result<RenderPass, JsValue> {
        #[cfg(feature = "webgl1")]
        let shader = Shader::new(
            &gl,
            include_str!("../shaders/webgl1/passes/post_vertex_100es.glsl"),
            include_str!("../shaders/webgl1/passes/post_fragment_100es.glsl"),
        )?;
        #[cfg(feature = "webgl2")]
        let shader = Shader::new(
            gl,
            include_str!("../shaders/webgl2/passes/post_vertex_100es.glsl"),
            include_str!("../shaders/webgl2/passes/post_fragment_100es.glsl"),
        )?;

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
            shader,
        })
    }

    pub fn draw_on_screen(&self, fbo: &FrameBufferObject) {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        ); // premultiplied alpha

        self.shader
            .bind(&self.gl)
            .attach_uniform("fbo_tex", &fbo.texture)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                Some(6),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                0,
            );

        self.gl.disable(WebGl2RenderingContext::BLEND);
    }
}
