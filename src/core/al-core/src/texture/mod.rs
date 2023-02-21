pub mod texture_array;
pub use texture_array::Texture2DArray;

pub mod pixel;
pub use pixel::*;

use crate::webgl_ctx::WebGlContext;
use crate::webgl_ctx::WebGlRenderingCtx;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;

pub static mut CUR_IDX_TEX_UNIT: u8 = 0;

#[derive(Clone)]
#[allow(dead_code)]
struct Texture2DMeta {
    pub format: u32,
    pub internal_format: i32,
    pub type_: u32,

    pub width: u32,
    pub height: u32,
}

use web_sys::WebGlTexture;
pub struct Texture2D {
    pub texture: Option<WebGlTexture>,

    gl: WebGlContext,

    metadata: Option<Rc<RefCell<Texture2DMeta>>>,
}

use crate::image::format::ImageFormat;
//use super::pixel::PixelType;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
impl Texture2D {
    pub fn create_from_path<P: AsRef<Path>, F: ImageFormat>(
        gl: &WebGlContext,
        name: &'static str,
        src: &P,
        tex_params: &'static [(u32, u32)],
    ) -> Result<Texture2D, JsValue> {
        let image = HtmlImageElement::new().unwrap_abort();

        let texture = gl.create_texture();
        let onerror = {
            Closure::wrap(Box::new(move || {
                println!("Cannot load texture located at: {:?}", name);
            }) as Box<dyn Fn()>)
        };

        let width = image.width();
        let height = image.height();

        let metadata = Rc::new(RefCell::new(Texture2DMeta {
            width: width,
            height: height,
            internal_format: F::INTERNAL_FORMAT,
            format: F::FORMAT,
            type_: F::TYPE,
        }));

        let onload = {
            let image = image.clone();
            let gl = gl.clone();
            let texture = texture.clone();
            let metadata = metadata.clone();

            Closure::wrap(Box::new(move || {
                gl.bind_texture(WebGlRenderingCtx::TEXTURE_2D, texture.as_ref());

                for (pname, param) in tex_params.iter() {
                    gl.tex_parameteri(WebGlRenderingCtx::TEXTURE_2D, *pname, *param as i32);
                }

                #[cfg(feature = "webgl2")]
                gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                    WebGlRenderingCtx::TEXTURE_2D,
                    0,
                    F::INTERNAL_FORMAT,
                    F::FORMAT,
                    F::TYPE,
                    &image,
                )
                .expect("Texture 2D");
                #[cfg(feature = "webgl1")]
                gl.tex_image_2d_with_u32_and_u32_and_image(
                    WebGlRenderingCtx::TEXTURE_2D,
                    0,
                    F::INTERNAL_FORMAT,
                    F::FORMAT,
                    F::TYPE,
                    &image,
                )
                .expect("Texture 2D");

                metadata.borrow_mut().width = image.width();
                metadata.borrow_mut().height = image.height();

                //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
            }) as Box<dyn Fn()>)
        };

        image.set_onload(Some(onload.as_ref().unchecked_ref()));
        image.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        image.set_cross_origin(Some(""));
        image.set_src(src.as_ref().to_str().unwrap_abort());

        onload.forget();
        onerror.forget();


        let gl = gl.clone();
        Ok(Texture2D {
            texture,

            gl,

            metadata: Some(metadata),
        })
    }

    pub fn create_from_raw_pixels<F: ImageFormat>(
        gl: &WebGlContext,
        width: i32,
        height: i32,
        tex_params: &'static [(u32, u32)],
        data: Option<&[<F::P as Pixel>::Item]>,
    ) -> Result<Texture2D, JsValue> {
        let texture = Texture2D::create_empty_with_format::<F>(
            gl,
            width,
            height,
            tex_params
        )?;

        if let Some(data) = data {
            let buf_data = unsafe { F::view(data) };
            texture.bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    0,
                    0,
                    width,
                    height,
                    Some(buf_data.as_ref()),
                );
        }

        Ok(texture)
    }

    pub fn create_empty_unsized(
        gl: &WebGlContext,
        tex_params: &'static [(u32, u32)],
    ) -> Result<Texture2D, JsValue> {
        let texture = gl.create_texture();

        gl.bind_texture(WebGlRenderingCtx::TEXTURE_2D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGlRenderingCtx::TEXTURE_2D, *pname, *param as i32);
        }

        let gl = gl.clone();

        let metadata = None;
        Ok(Texture2D {
            texture,

            gl,

            metadata,
        })
    }

    pub fn create_empty_with_format<F: ImageFormat>(
        gl: &WebGlContext,
        width: i32,
        height: i32,
        tex_params: &'static [(u32, u32)],
    ) -> Result<Texture2D, JsValue> {
        let texture = gl.create_texture();

        gl.bind_texture(WebGlRenderingCtx::TEXTURE_2D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGlRenderingCtx::TEXTURE_2D, *pname, *param as i32);
        }

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGlRenderingCtx::TEXTURE_2D,
            0,
            F::INTERNAL_FORMAT,
            width,
            height,
            0,
            F::FORMAT,
            F::TYPE,
            None,
        )
        .expect("Texture 2D");
        //gl.generate_mipmap(WebGlRenderingCtx::TEXTURE_2D);

        let gl = gl.clone();
        let metadata = Some(Rc::new(RefCell::new(Texture2DMeta {
            width: width as u32,
            height: height as u32,
            internal_format: F::INTERNAL_FORMAT,
            format: F::FORMAT,
            type_: F::TYPE,
        })));
        Ok(Texture2D {
            texture,

            gl,

            metadata,
        })
    }

    pub fn attach_to_framebuffer(&self) {
        self.gl.framebuffer_texture_2d(
            WebGlRenderingCtx::FRAMEBUFFER,
            WebGlRenderingCtx::COLOR_ATTACHMENT0,
            WebGlRenderingCtx::TEXTURE_2D,
            self.texture.as_ref(),
            0,
        );
    }

    pub fn get_size(&self) -> (u32, u32) {
        (
            self.metadata.as_ref().unwrap_abort().borrow().width,
            self.metadata.as_ref().unwrap_abort().borrow().height,
        )
    }

    pub fn width(&self) -> u32 {
        self.metadata.as_ref().unwrap_abort().borrow().width
    }

    pub fn height(&self) -> u32 {
        self.metadata.as_ref().unwrap_abort().borrow().height
    }

    pub fn active_texture(&self, idx_tex_unit: u8) -> &Self {
        self.gl
            .active_texture(WebGlRenderingCtx::TEXTURE0 + idx_tex_unit as u32);
        self
    }

    pub fn bind(&self) -> Texture2DBound {
        self.gl
            .bind_texture(WebGlRenderingCtx::TEXTURE_2D, self.texture.as_ref());

        Texture2DBound { texture_2d: self }
    }

    pub fn bind_mut(&mut self) -> Texture2DBoundMut {
        self.gl
            .bind_texture(WebGlRenderingCtx::TEXTURE_2D, self.texture.as_ref());

        Texture2DBoundMut { texture_2d: self }
    }

    pub fn read_pixel(&self, x: i32, y: i32) -> Result<JsValue, JsValue> {
        // Create and bind the framebuffer
        let reader = self.gl.create_framebuffer();
        self.gl
            .bind_framebuffer(WebGlRenderingCtx::FRAMEBUFFER, reader.as_ref());

        // Attach the texture as the first color attachment
        //self.attach_to_framebuffer();
        self.gl.framebuffer_texture_2d(
            WebGlRenderingCtx::READ_FRAMEBUFFER,
            WebGlRenderingCtx::COLOR_ATTACHMENT0,
            WebGlRenderingCtx::TEXTURE_2D,
            self.texture.as_ref(),
            0,
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
            self.gl.viewport(x, y, metadata.width as i32, metadata.height as i32);
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
            #[cfg(feature = "webgl1")]
            let value = match (*format, *type_) {
                (WebGlRenderingCtx::LUMINANCE_ALPHA, WebGlRenderingCtx::FLOAT) => {
                    let p = <[f32; 1]>::read_pixel(&self.gl, x, y)?;
                    Ok(serde_wasm_bindgen::to_value(&p)?)
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

impl Drop for Texture2D {
    fn drop(&mut self) {
        self.gl.delete_texture(self.texture.as_ref());

        // free the texture unit
        /*let i = (self.idx_texture_unit - WebGl2RenderingContext::TEXTURE0) as usize;
        unsafe {
            AVAILABLE_TEX_UNITS[i] = Some(self.idx_texture_unit);
        }*/
    }
}
use crate::Abort;

pub struct Texture2DBound<'a> {
    texture_2d: &'a Texture2D,
    //idx_tex_unit: u8
}

impl<'a> Texture2DBound<'a> {
    /*pub fn get_idx_sampler(&self) -> i32 {
        self.idx_tex_unit as i32
    }*/

    pub fn tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
        &self,
        dx: i32,
        dy: i32,
        image: &HtmlImageElement,
    ) {
        let metadata = self.texture_2d.metadata.as_ref().unwrap_abort().borrow();

        #[cfg(feature = "webgl2")]
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
        #[cfg(feature = "webgl1")]
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_image(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
        //self.texture_2d.gl.flush();
    }

    pub fn tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
        &self,
        dx: i32,
        dy: i32,
        image: &web_sys::ImageBitmap,
    ) {
        let metadata = self.texture_2d.metadata.as_ref().unwrap_abort().borrow();

        #[cfg(feature = "webgl2")]
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
        #[cfg(feature = "webgl1")]
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
    }

    pub fn tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
        &self,
        dx: i32,
        dy: i32,
        width: i32,  // Width of the image
        height: i32, // Height of the image
        image: Option<&js_sys::Object>,
    ) {
        let metadata = self.texture_2d.metadata.as_ref().unwrap_abort().borrow();

        self.texture_2d
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                width,
                height,
                metadata.format,
                metadata.type_,
                image,
            )
            .expect("Sub texture 2d");
    }

    #[allow(dead_code)]
    pub fn tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
        &self,
        dx: i32,
        dy: i32,
        width: i32,  // Width of the image
        height: i32, // Height of the image
        pixels: Option<&[u8]>,
    ) {
        let metadata = self.texture_2d.metadata.as_ref().unwrap_abort().borrow();
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                dx,
                dy,
                width,
                height,
                metadata.format,
                metadata.type_,
                pixels,
            )
            .expect("Sub texture 2d");
    }
}

pub struct Texture2DBoundMut<'a> {
    texture_2d: &'a mut Texture2D,
}

impl<'a> Texture2DBoundMut<'a> {
    #[allow(dead_code)]
    pub fn tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        &mut self,
        width: i32,  // Width of the image
        height: i32, // Height of the image
        internal_format: i32,
        src_format: u32,
        src_type: u32,
        pixels: Option<&[u8]>,
    ) {
        //let Texture2DMeta {format, type_, ..} = self.texture_2d.metadata.unwrap_abort();
        /*self.texture_2d
        .gl
        .pixel_storei(WebGlRenderingCtx::UNPACK_ALIGNMENT, 1);*/
        self.texture_2d
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGlRenderingCtx::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                src_format,
                src_type,
                pixels,
            )
            .expect("Sub texture 2d");
        //self.texture_2d.gl.generate_mipmap(WebGlRenderingCtx::TEXTURE_2D);

        self.texture_2d.metadata = Some(Rc::new(RefCell::new(Texture2DMeta {
            format: src_format,
            internal_format,
            type_: src_type,
            width: width as u32,
            height: height as u32,
        })));
    }
}