use {wasm_bindgen::prelude::*, web_sys::WebGlFramebuffer};

use crate::webgl_ctx::WebGlRenderingCtx;
// Internal format used for the framebuffer final texture
use crate::image::format::RGBA8U;

pub struct FrameBufferObject {
    gl: WebGlContext,
    fbo: WebGlFramebuffer,
    pub texture: Texture2D,
}
use crate::texture::Texture2D;
use crate::webgl_ctx::WebGlContext;

impl FrameBufferObject {
    pub fn new(gl: &WebGlContext, width: usize, height: usize) -> Result<Self, JsValue> {
        let fbo = gl
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        gl.bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, Some(&fbo));

        let texture = Texture2D::create_empty_with_format::<RGBA8U>(
            gl,
            width as i32,
            height as i32,
            &[
                (
                    WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                    WebGlRenderingCtx::LINEAR,
                ),
                (
                    WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                    WebGlRenderingCtx::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_S,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_T,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
            ],
        )?;
        texture.attach_to_framebuffer();

        gl.bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, None);

        Ok(Self {
            gl: gl.clone(),
            texture,
            fbo,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if (width, height)
            != (
                self.texture.width() as usize,
                self.texture.height() as usize,
            )
        {
            drop(self.texture.clone());

            self.gl
                .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, Some(&self.fbo));
            self.texture = Texture2D::create_empty_with_format::<RGBA8U>(
                &self.gl,
                width as i32,
                height as i32,
                &[
                    (
                        WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                        WebGlRenderingCtx::LINEAR,
                    ),
                    (
                        WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                        WebGlRenderingCtx::LINEAR,
                    ),
                    // Prevents s-coordinate wrapping (repeating)
                    (
                        WebGlRenderingCtx::TEXTURE_WRAP_S,
                        WebGlRenderingCtx::CLAMP_TO_EDGE,
                    ),
                    // Prevents t-coordinate wrapping (repeating)
                    (
                        WebGlRenderingCtx::TEXTURE_WRAP_T,
                        WebGlRenderingCtx::CLAMP_TO_EDGE,
                    ),
                ],
            )
            .unwrap();
            self.texture.attach_to_framebuffer();
            self.gl
                .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, None);
        }
    }

    pub fn bind(&self) {
        // bind the fbo
        self.gl
            .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, Some(&self.fbo));

        let w = self.texture.width() as i32;
        let h = self.texture.height() as i32;
        self.gl.viewport(0, 0, w, h);
        self.gl.scissor(0, 0, w, h);
    }

    pub fn draw_onto(
        &self,
        f: impl FnOnce() -> Result<(), JsValue>,
        cur_fbo: Option<&Self>,
    ) -> Result<(), JsValue> {
        // bind the fbo
        self.bind();

        // clear the fbo
        self.gl.clear(WebGlRenderingCtx::COLOR_BUFFER_BIT);

        // render all the things onto the fbo
        f()?;

        // restore the fbo to its previous state
        if let Some(prev_fbo) = cur_fbo {
            prev_fbo.bind();
        } else {
            self.gl
                .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, None);
        }

        Ok(())
    }
}

impl Drop for FrameBufferObject {
    fn drop(&mut self) {
        self.gl.delete_framebuffer(Some(&self.fbo));
    }
}
