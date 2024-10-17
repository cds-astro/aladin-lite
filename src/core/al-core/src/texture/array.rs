use crate::image::format::ImageFormat;
use web_sys::HtmlCanvasElement;
use web_sys::WebGlTexture;

use crate::texture::pixel::Pixel;
use crate::texture::Texture2DMeta;
use crate::webgl_ctx::WebGlContext;
use crate::webgl_ctx::WebGlRenderingCtx;
use crate::Abort;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;
pub struct Texture2DArray {
    gl: WebGlContext,

    texture: Option<WebGlTexture>,

    metadata: Option<Rc<RefCell<Texture2DMeta>>>,
    pub num_slices: i32,
}

impl Texture2DArray {
    pub fn create_empty<F: ImageFormat>(
        gl: &WebGlContext,
        // The weight of the individual textures
        width: i32,
        // Their height
        height: i32,
        // How many texture slices it contains
        num_slices: i32,
        tex_params: &'static [(u32, u32)],
    ) -> Result<Texture2DArray, JsValue> {
        let texture = gl.create_texture();

        gl.bind_texture(WebGlRenderingCtx::TEXTURE_2D_ARRAY, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGlRenderingCtx::TEXTURE_2D_ARRAY, *pname, *param as i32);
        }

        gl.tex_storage_3d(
            WebGlRenderingCtx::TEXTURE_2D_ARRAY,
            1,
            F::INTERNAL_FORMAT as u32,
            width,
            height,
            num_slices,
        );

        let gl = gl.clone();
        let metadata = Some(Rc::new(RefCell::new(Texture2DMeta {
            width: width as u32,
            height: height as u32,
            internal_format: F::INTERNAL_FORMAT,
            format: F::FORMAT,
            type_: F::TYPE,
        })));

        Ok(Texture2DArray {
            texture,
            gl: gl.clone(),
            num_slices,
            metadata,
        })
    }

    pub fn generate_mipmap(&self) {
        self.gl.generate_mipmap(WebGlRenderingCtx::TEXTURE_2D_ARRAY);
    }

    pub fn bind(&self) -> Texture2DArrayBound {
        self.gl
            .bind_texture(WebGlRenderingCtx::TEXTURE_2D_ARRAY, self.texture.as_ref());

        Texture2DArrayBound { tex: self }
    }

    pub fn active_texture(&self, idx_tex_unit: u8) -> &Self {
        self.gl
            .active_texture(WebGlRenderingCtx::TEXTURE0 + idx_tex_unit as u32);
        self
    }

    pub fn read_pixel(&self, x: i32, y: i32, slice_idx: i32) -> Result<JsValue, JsValue> {
        // Create and bind the framebuffer
        let reader = self.gl.create_framebuffer();
        self.gl
            .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, reader.as_ref());

        // Attach the texture as the first color attachment
        self.gl.framebuffer_texture_layer(
            WebGlRenderingCtx::READ_FRAMEBUFFER,
            WebGlRenderingCtx::COLOR_ATTACHMENT0,
            self.texture.as_ref(),
            0,
            slice_idx,
        );

        let status = self
            .gl
            .check_framebuffer_status(WebGlRenderingCtx::FRAMEBUFFER);
        if status != WebGlRenderingCtx::FRAMEBUFFER_COMPLETE {
            // Unbind the framebuffer
            self.gl
                .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, None);
            // Delete the framebuffer
            self.gl.delete_framebuffer(reader.as_ref());

            Err(JsValue::from_str("incomplete framebuffer"))
        } else {
            // set the viewport as the FBO won't be the same dimension as the screen
            let metadata = self.metadata.as_ref().unwrap_abort().borrow();
            self.gl
                .viewport(0, 0, metadata.width as i32, metadata.height as i32);

            #[cfg(feature = "webgl2")]
            let value = match (metadata.format, metadata.type_) {
                (WebGlRenderingCtx::RED_INTEGER, WebGlRenderingCtx::UNSIGNED_BYTE) => {
                    let p = <[u8; 1]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p[0])?)
                }
                (WebGlRenderingCtx::RED_INTEGER, WebGlRenderingCtx::SHORT) => {
                    let p = <[i16; 1]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p[0])?)
                }
                (WebGlRenderingCtx::RED_INTEGER, WebGlRenderingCtx::INT) => {
                    let p = <[i32; 1]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p[0])?)
                }
                (WebGlRenderingCtx::RED, WebGlRenderingCtx::FLOAT) => {
                    let p = <[f32; 1]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p[0])?)
                }
                (WebGlRenderingCtx::RGB, WebGlRenderingCtx::UNSIGNED_BYTE) => {
                    let p = <[u8; 3]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p)?)
                }
                (WebGlRenderingCtx::RGBA, WebGlRenderingCtx::UNSIGNED_BYTE) => {
                    let p = <[u8; 4]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p)?)
                }
                _ => Err(JsValue::from_str(
                    "Pixel retrieval not implemented for that texture format.",
                )),
            };

            // Unbind the framebuffer
            self.gl
                .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, None);
            // Delete the framebuffer
            self.gl.delete_framebuffer(reader.as_ref());

            // set the viewport as the FBO won't be the same dimension as the screen
            let canvas = self
                .gl
                .canvas()
                .unwrap_abort()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap_abort();
            self.gl
                .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

            value
        }
    }
}

impl Drop for Texture2DArray {
    fn drop(&mut self) {
        self.gl.delete_texture(self.texture.as_ref());
    }
}

use super::CUR_IDX_TEX_UNIT;
use crate::shader::UniformType;
use web_sys::WebGlUniformLocation;
impl UniformType for Texture2DArray {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, tex: &Self) {
        unsafe {
            let _ = tex
                // 1. Active the texture unit of the texture
                .active_texture(CUR_IDX_TEX_UNIT)
                // 2. Bind the texture to that texture unit
                .bind();

            gl.uniform1i(location, CUR_IDX_TEX_UNIT as i32);
            CUR_IDX_TEX_UNIT += 1;
        };
    }
}

pub struct Texture2DArrayBound<'a> {
    tex: &'a Texture2DArray,
}

impl<'a> Texture2DArrayBound<'a> {
    pub fn tex_sub_image_3d_with_html_image_element(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        image: &HtmlImageElement,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_html_image_element(
                WebGlRenderingCtx::TEXTURE_2D_ARRAY,
                0,
                dx,
                dy,
                idx,
                image.width() as i32,
                image.height() as i32,
                1,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 3d");
    }

    pub fn tex_sub_image_3d_with_html_canvas_element(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        canvas: &HtmlCanvasElement,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_html_canvas_element(
                WebGlRenderingCtx::TEXTURE_2D_ARRAY,
                0,
                dx,
                dy,
                idx,
                canvas.width() as i32,
                canvas.height() as i32,
                1,
                metadata.format,
                metadata.type_,
                canvas,
            )
            .expect("Sub texture 2d");
    }

    pub fn tex_sub_image_3d_with_image_bitmap(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        image: &web_sys::ImageBitmap,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_image_bitmap(
                WebGlRenderingCtx::TEXTURE_2D_ARRAY,
                0,
                dx,
                dy,
                idx,
                image.width() as i32,
                image.height() as i32,
                1,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
    }

    pub fn tex_sub_image_3d_with_opt_array_buffer_view(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        w: i32,
        h: i32,
        image: Option<&js_sys::Object>,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_opt_array_buffer_view(
                WebGlRenderingCtx::TEXTURE_2D_ARRAY,
                0,
                dx,
                dy,
                idx,
                w,
                h,
                1,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
    }

    #[allow(dead_code)]
    pub fn tex_sub_image_3d_with_opt_u8_array(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        w: i32,
        h: i32,
        pixels: Option<&[u8]>,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();
        self.tex
            .gl
            .tex_sub_image_3d_with_opt_u8_array(
                WebGlRenderingCtx::TEXTURE_2D_ARRAY,
                0,
                dx,
                dy,
                idx,
                w,
                h,
                1,
                metadata.format,
                metadata.type_,
                pixels,
            )
            .expect("Sub texture 2d");
    }
}
