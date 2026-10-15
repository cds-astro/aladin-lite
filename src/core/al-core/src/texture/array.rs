use crate::texture::format::PixelType;
use crate::texture::format::TextureFormat;

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

    // Size of one logical tile.
    tile_width: i32,
    tile_height: i32,

    // Atlas dimensions.
    atlas_width: i32,
    atlas_height: i32,
}

impl Texture2DArray {
    // -------------------------------------------------------------------------
    // Atlas layout
    //
    // 64 tiles:
    //
    //     8 columns x 8 rows
    //
    // With 512x512 tiles:
    //
    //     8 * 512 = 4096
    //
    // Therefore:
    //
    //     4096 x 4096 atlas
    // -------------------------------------------------------------------------

    const ATLAS_COLUMNS: i32 = 8;
    const ATLAS_ROWS: i32 = 8;
    const MAX_SLICES: i32 = Self::ATLAS_COLUMNS * Self::ATLAS_ROWS;

    /// Convert a logical slice index into an atlas pixel offset.
    ///
    /// idx = 0  -> (0, 0)
    /// idx = 1  -> (width, 0)
    /// ...
    /// idx = 7  -> (7 * width, 0)
    /// idx = 8  -> (0, height)
    /// ...
    /// idx = 63 -> (7 * width, 7 * height)
    fn slice_offset(&self, idx: i32) -> Result<(i32, i32), JsValue> {
        if idx < 0 || idx >= self.num_slices {
            return Err(JsValue::from_str("Texture slice index out of bounds"));
        }

        if idx >= Self::MAX_SLICES {
            return Err(JsValue::from_str(
                "Texture2DArray atlas supports at most 64 slices",
            ));
        }

        let column = idx % Self::ATLAS_COLUMNS;
        let row = idx / Self::ATLAS_COLUMNS;

        Ok((
            column * self.tile_width,
            row * self.tile_height,
        ))
    }

    pub fn create_empty<F: TextureFormat>(
        gl: &WebGlContext,

        // Width of an individual texture/tile.
        width: i32,

        // Height of an individual texture/tile.
        height: i32,

        // Number of logical texture slices.
        num_slices: i32,

        tex_params: &'static [(u32, u32)],
    ) -> Result<Texture2DArray, JsValue> {
        if num_slices > Self::MAX_SLICES {
            return Err(JsValue::from_str(
                "Texture2DArray atlas supports at most 64 slices",
            ));
        }

        let atlas_width = width * Self::ATLAS_COLUMNS;
        let atlas_height = height * Self::ATLAS_ROWS;

        let texture = gl.create_texture();

        // ---------------------------------------------------------------------
        // This used to be:
        //
        //     TEXTURE_2D_ARRAY
        //
        // It is now one normal 2D texture.
        // ---------------------------------------------------------------------

        gl.bind_texture(
            WebGlRenderingCtx::TEXTURE_2D,
            texture.as_ref(),
        );

        for (pname, param) in tex_params.iter() {
            // Ignore TEXTURE_WRAP_R because a TEXTURE_2D has no R coordinate.
            //
            // This also means existing tex_params containing:
            //
            //     TEXTURE_WRAP_S
            //     TEXTURE_WRAP_T
            //     TEXTURE_WRAP_R
            //
            // will continue to work without having to modify all call sites.
            if *pname == WebGlRenderingCtx::TEXTURE_WRAP_R {
                continue;
            }

            gl.tex_parameteri(
                WebGlRenderingCtx::TEXTURE_2D,
                *pname,
                *param as i32,
            );
        }

        // ---------------------------------------------------------------------
        // Allocate ONE texture instead of a texture array.
        //
        // Example:
        //
        // width       = 512
        // height      = 512
        // num_slices  = 64
        //
        // atlas       = 4096 x 4096
        // ---------------------------------------------------------------------

        gl.tex_storage_2d(
            WebGlRenderingCtx::TEXTURE_2D,
            1,
            F::INTERNAL_FORMAT as u32,
            atlas_width,
            atlas_height,
        );

        let gl = gl.clone();

        let metadata = Some(Rc::new(RefCell::new(Texture2DMeta {
            // IMPORTANT:
            // Metadata continues to describe ONE logical tile, not the
            // complete atlas.
            width: width as u32,
            height: height as u32,
            pixel_type: F::PIXEL_TYPE,
            ty: F::TYPE,
            format: F::FORMAT,
        })));

        Ok(Texture2DArray {
            texture,

            gl: gl.clone(),

            num_slices,

            tile_width: width,
            tile_height: height,

            atlas_width,
            atlas_height,

            metadata,
        })
    }

    pub fn generate_mipmap(&self) {
        self.gl.generate_mipmap(WebGlRenderingCtx::TEXTURE_2D);
    }

    pub fn bind(&self) -> Texture2DArrayBound<'_> {
        self.gl.bind_texture(
            WebGlRenderingCtx::TEXTURE_2D,
            self.texture.as_ref(),
        );

        Texture2DArrayBound { tex: self }
    }

    pub fn active_texture(&self, idx_tex_unit: u8) -> &Self {
        self.gl
            .active_texture(WebGlRenderingCtx::TEXTURE0 + idx_tex_unit as u32);

        self
    }

    pub fn read_pixel(
        &self,
        x: i32,
        y: i32,
        slice_idx: i32,
    ) -> Result<JsValue, JsValue> {
        // ---------------------------------------------------------------------
        // Convert logical tile coordinates into atlas coordinates.
        // ---------------------------------------------------------------------

        if x < 0 || x >= self.tile_width {
            return Err(JsValue::from_str("x coordinate out of bounds"));
        }

        if y < 0 || y >= self.tile_height {
            return Err(JsValue::from_str("y coordinate out of bounds"));
        }

        let (offset_x, offset_y) = self.slice_offset(slice_idx)?;

        let atlas_x = offset_x + x;
        let atlas_y = offset_y + y;

        // Create and bind framebuffer.
        let reader = self.gl.create_framebuffer();

        self.gl
            .bind_framebuffer(
                WebGlRenderingCtx::FRAMEBUFFER,
                reader.as_ref(),
            );

        // ---------------------------------------------------------------------
        // The entire atlas is attached to the framebuffer.
        //
        // Previously framebuffer_texture_layer() selected one array layer.
        // With a normal 2D texture we simply attach the texture itself.
        // ---------------------------------------------------------------------

        self.gl.framebuffer_texture_2d(
            WebGlRenderingCtx::FRAMEBUFFER,
            WebGlRenderingCtx::COLOR_ATTACHMENT0,
            WebGlRenderingCtx::TEXTURE_2D,
            self.texture.as_ref(),
            0,
        );

        let status = self
            .gl
            .check_framebuffer_status(WebGlRenderingCtx::FRAMEBUFFER);

        if status != WebGlRenderingCtx::FRAMEBUFFER_COMPLETE {
            self.gl
                .bind_framebuffer(
                    WebGlRenderingCtx::FRAMEBUFFER,
                    None,
                );

            self.gl.delete_framebuffer(reader.as_ref());

            return Err(JsValue::from_str("incomplete framebuffer"));
        }

        // The FBO is the size of the complete atlas.
        self.gl.viewport(
            0,
            0,
            self.atlas_width,
            self.atlas_height,
        );

        let metadata = self
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        let value = match metadata.pixel_type {
            PixelType::R8U => {
                let p = <[u8; 1]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p[0])?)
            }

            PixelType::R16I => {
                let p = <[i16; 1]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p[0])?)
            }

            PixelType::R32I => {
                let p = <[i32; 1]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p[0])?)
            }

            PixelType::R32F => {
                let p = <[f32; 1]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p[0])?)
            }

            PixelType::RGB8U => {
                let p = <[u8; 3]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p)?)
            }

            PixelType::RGBA8U => {
                let p = <[u8; 4]>::read_pixel(
                    &self.gl,
                    atlas_x,
                    atlas_y,
                )?;

                Ok(serde_wasm_bindgen::to_value(&p)?)
            }
        };

        // Unbind framebuffer.
        self.gl
            .bind_framebuffer(
                WebGlRenderingCtx::FRAMEBUFFER,
                None,
            );

        // Delete framebuffer.
        self.gl.delete_framebuffer(reader.as_ref());

        // Restore canvas viewport.
        let canvas = self
            .gl
            .canvas()
            .unwrap_abort()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap_abort();

        self.gl.viewport(
            0,
            0,
            canvas.width() as i32,
            canvas.height() as i32,
        );

        value
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
    fn uniform(
        gl: &WebGlContext,
        location: Option<&WebGlUniformLocation>,
        tex: &Self,
    ) {
        unsafe {
            let _ = tex
                // 1. Activate the texture unit.
                .active_texture(CUR_IDX_TEX_UNIT)
                // 2. Bind the atlas to that texture unit.
                .bind();

            // The shader will now use sampler2D rather than sampler2DArray.
            gl.uniform1i(
                location,
                CUR_IDX_TEX_UNIT as i32,
            );

            CUR_IDX_TEX_UNIT += 1;
        };
    }
}

pub struct Texture2DArrayBound<'a> {
    tex: &'a Texture2DArray,
}

impl Texture2DArrayBound<'_> {
    pub fn tex_sub_image_3d_with_html_image_element(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        image: &HtmlImageElement,
    ) {
        let metadata = self
            .tex
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        // Convert the logical slice into an atlas position.
        let (slice_x, slice_y) = self
            .tex
            .slice_offset(idx)
            .expect("Invalid texture slice");

        let atlas_x = slice_x + dx;
        let atlas_y = slice_y + dy;

        self.tex
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                atlas_x,
                atlas_y,
                metadata.format,
                metadata.ty,
                image,
            )
            .expect("Sub texture 2d");
    }

    pub fn tex_sub_image_3d_with_html_canvas_element(
        &self,
        dx: i32,
        dy: i32,
        idx: i32,
        canvas: &HtmlCanvasElement,
    ) {
        let metadata = self
            .tex
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        let (slice_x, slice_y) = self
            .tex
            .slice_offset(idx)
            .expect("Invalid texture slice");

        let atlas_x = slice_x + dx;
        let atlas_y = slice_y + dy;

        self.tex
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_html_canvas_element(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                atlas_x,
                atlas_y,
                metadata.format,
                metadata.ty,
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
        let metadata = self
            .tex
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        let (slice_x, slice_y) = self
            .tex
            .slice_offset(idx)
            .expect("Invalid texture slice");

        let atlas_x = slice_x + dx;
        let atlas_y = slice_y + dy;

        self.tex
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                atlas_x,
                atlas_y,
                metadata.format,
                metadata.ty,
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
        let metadata = self
            .tex
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        let (slice_x, slice_y) = self
            .tex
            .slice_offset(idx)
            .expect("Invalid texture slice");

        let atlas_x = slice_x + dx;
        let atlas_y = slice_y + dy;

        self.tex
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                atlas_x,
                atlas_y,
                w,
                h,
                metadata.format,
                metadata.ty,
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
        let metadata = self
            .tex
            .metadata
            .as_ref()
            .unwrap_abort()
            .borrow();

        let (slice_x, slice_y) = self
            .tex
            .slice_offset(idx)
            .expect("Invalid texture slice");

        let atlas_x = slice_x + dx;
        let atlas_y = slice_y + dy;

        self.tex
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                atlas_x,
                atlas_y,
                w,
                h,
                metadata.format,
                metadata.ty,
                pixels,
            )
            .expect("Sub texture 2d");
    }
}