use {
    js_sys::WebAssembly,
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::{
        WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader,
        WebGlTexture, WebGlVertexArrayObject,
    },
};

pub struct FrameBufferObject {
    gl: WebGl2Context,
    fbo: WebGlFramebuffer,
    pub texture: Texture2D,
}
use crate::Texture2D;
use crate::WebGl2Context;

impl FrameBufferObject {
    pub fn new(gl: &WebGl2Context, width: usize, height: usize) -> Result<Self, JsValue> {
        let fbo = gl
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fbo));

        let texture = Texture2D::create_empty_with_format::<crate::format::RGBA8U>(
            &gl,
            width as i32,
            height as i32,
            &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
            ],
        )?;
        texture.attach_to_framebuffer();

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        Ok(Self {
            gl: gl.clone(),
            texture,
            fbo,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if (width, height) != (self.texture.width() as usize, self.texture.height() as usize) {
            let pixels = [0, 0, 0, 0].iter().cloned().cycle().take(4*height*width).collect::<Vec<_>>();
            self.texture.bind_mut()
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    width as i32,
                    height as i32,
                    WebGl2RenderingContext::SRGB8_ALPHA8 as i32,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    Some(&pixels.as_slice())
                );
        }
    }

    pub fn draw_onto(&self, f: impl FnOnce() -> Result<(), JsValue> ) -> Result<(), JsValue> {
        // bind the fbo
        self.gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.fbo));

        // enable the blending 
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA); // premultiplied alpha

        // clear the fbo
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        //self.gl.viewport(0, 0, self.texture.width() as i32, self.texture.height() as i32);

        // render all the things onto the fbo
        f()?;

        // disable blending
        self.gl.disable(WebGl2RenderingContext::BLEND);
        // unbind the fbo
        self.gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        Ok(())
    }
}

impl Drop for FrameBufferObject {
    fn drop(&mut self) {
        self.gl.delete_framebuffer(Some(&self.fbo));
    }
}