use crate::image::format::ImageFormat;
use web_sys::HtmlCanvasElement;
use web_sys::WebGlTexture;

use crate::Abort;

use crate::texture::Texture2DMeta;
use crate::webgl_ctx::WebGlContext;
use crate::webgl_ctx::WebGlRenderingCtx;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlImageElement;

#[derive(Clone)]
pub struct Texture3D {
    gl: WebGlContext,

    texture: Option<WebGlTexture>,

    metadata: Option<Rc<RefCell<Texture2DMeta>>>,
    depth: i32,
}

impl Texture3D {
    pub fn create_empty<F: ImageFormat>(
        gl: &WebGlContext,
        // The weight of the individual textures
        width: i32,
        // Their height
        height: i32,
        // The depth of the cube
        depth: i32,
        tex_params: &'static [(u32, u32)],
    ) -> Result<Self, JsValue> {
        let texture = gl.create_texture();

        gl.bind_texture(WebGlRenderingCtx::TEXTURE_3D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGlRenderingCtx::TEXTURE_3D, *pname, *param as i32);
        }

        gl.tex_storage_3d(
            WebGlRenderingCtx::TEXTURE_3D,
            1,
            F::INTERNAL_FORMAT as u32,
            width,
            height,
            depth,
        );

        let gl = gl.clone();
        let metadata = Some(Rc::new(RefCell::new(Texture2DMeta {
            width: width as u32,
            height: height as u32,
            internal_format: F::INTERNAL_FORMAT,
            format: F::FORMAT,
            type_: F::TYPE,
        })));

        Ok(Texture3D {
            texture,
            gl: gl.clone(),
            depth,
            metadata,
        })
    }

    pub fn generate_mipmap(&self) {
        self.gl.generate_mipmap(WebGlRenderingCtx::TEXTURE_3D);
    }

    pub fn bind(&self) -> Texture3DBound {
        self.gl
            .bind_texture(WebGlRenderingCtx::TEXTURE_3D, self.texture.as_ref());

        Texture3DBound { tex: self }
    }

    pub fn active_texture(&self, idx_tex_unit: u8) -> &Self {
        self.gl
            .active_texture(WebGlRenderingCtx::TEXTURE0 + idx_tex_unit as u32);
        self
    }
}

impl Drop for Texture3D {
    fn drop(&mut self) {
        self.gl.delete_texture(self.texture.as_ref());
    }
}

use super::CUR_IDX_TEX_UNIT;
use crate::shader::UniformType;
use web_sys::WebGlUniformLocation;
impl UniformType for Texture3D {
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

pub struct Texture3DBound<'a> {
    tex: &'a Texture3D,
}

impl<'a> Texture3DBound<'a> {
    pub fn tex_sub_image_3d_with_html_image_element(
        &self,
        dx: i32,
        dy: i32,
        dz: i32,
        image: &HtmlImageElement,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_html_image_element(
                WebGlRenderingCtx::TEXTURE_3D,
                0,
                dx,
                dy,
                dz,
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
        dz: i32,
        canvas: &HtmlCanvasElement,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_html_canvas_element(
                WebGlRenderingCtx::TEXTURE_3D,
                0,
                dx,
                dy,
                dz,
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
        dz: i32,
        image: &web_sys::ImageBitmap,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_image_bitmap(
                WebGlRenderingCtx::TEXTURE_3D,
                0,
                dx,
                dy,
                dz,
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
        dz: i32,
        w: i32,
        h: i32,
        d: i32,
        image: Option<&js_sys::Object>,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();

        self.tex
            .gl
            .tex_sub_image_3d_with_opt_array_buffer_view(
                WebGlRenderingCtx::TEXTURE_3D,
                0,
                dx,
                dy,
                dz,
                w,
                h,
                d,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
    }

    #[allow(dead_code)]
    pub fn tex_sub_image_3d_with_opt_u8_array(
        &self,
        idx: i32,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        d: i32,
        pixels: Option<&[u8]>,
    ) {
        let metadata = self.tex.metadata.as_ref().unwrap_abort().borrow();
        self.tex
            .gl
            .tex_sub_image_3d_with_opt_u8_array(
                WebGlRenderingCtx::TEXTURE_3D,
                0,
                dx,
                dy,
                idx,
                w,
                h,
                d,
                metadata.format,
                metadata.type_,
                pixels,
            )
            .expect("Sub texture 2d");
    }
}
