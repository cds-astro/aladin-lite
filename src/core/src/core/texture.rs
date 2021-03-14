use std::convert::TryInto;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlImageElement;
use web_sys::WebGl2RenderingContext;

use crate::WebGl2Context;

#[derive(Clone)]
enum TextureType {
    // The image containing the width and height
    // of the texture
    ImageElement(HtmlImageElement),
    // Width and Height
    Bytes(u32, u32),
}

impl TextureType {
    fn get_width(&self) -> u32 {
        match self {
            TextureType::ImageElement(image) => image.width() as u32,
            TextureType::Bytes(width, _) => *width,
        }
    }

    fn get_height(&self) -> u32 {
        match self {
            TextureType::ImageElement(image) => image.height() as u32,
            TextureType::Bytes(_, height) => *height,
        }
    }
}

use web_sys::WebGlTexture;
pub struct Texture2D {
    pub texture: Option<WebGlTexture>,
    pub idx_texture_unit: u32,

    gl: WebGl2Context,

    data: TextureType,

    pub format: u32,
    pub internal_format: i32,
    pub type_: u32,
}

static mut AVAILABLE_TEX_UNITS: [Option<u32>; 16] = [
    Some(WebGl2RenderingContext::TEXTURE0),
    Some(WebGl2RenderingContext::TEXTURE1),
    Some(WebGl2RenderingContext::TEXTURE2),
    Some(WebGl2RenderingContext::TEXTURE3),
    Some(WebGl2RenderingContext::TEXTURE4),
    Some(WebGl2RenderingContext::TEXTURE5),
    Some(WebGl2RenderingContext::TEXTURE6),
    Some(WebGl2RenderingContext::TEXTURE7),
    Some(WebGl2RenderingContext::TEXTURE8),
    Some(WebGl2RenderingContext::TEXTURE9),
    Some(WebGl2RenderingContext::TEXTURE10),
    Some(WebGl2RenderingContext::TEXTURE11),
    Some(WebGl2RenderingContext::TEXTURE12),
    Some(WebGl2RenderingContext::TEXTURE13),
    Some(WebGl2RenderingContext::TEXTURE14),
    Some(WebGl2RenderingContext::TEXTURE15),
];
pub struct IdxTextureUnit;
use wasm_bindgen::JsValue;
impl IdxTextureUnit {
    pub unsafe fn new() -> Result<u32, JsValue> {
        if let Some(idx_texture_unit) = AVAILABLE_TEX_UNITS.iter().find(|idx| idx.is_some()) {
            let idx_texture_unit = idx_texture_unit.unwrap();
            let i = (idx_texture_unit - WebGl2RenderingContext::TEXTURE0) as usize;
            AVAILABLE_TEX_UNITS[i] = None;
            Ok(idx_texture_unit)
        } else {
            Err(JsValue::from_str("No available tex units found"))
        }
    }

    #[allow(dead_code)]
    fn max_combined_texture_image_units(gl: &WebGl2Context) -> u32 {
        gl.get_parameter(WebGl2RenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .unwrap()
            .as_f64()
            .unwrap() as u32
    }
}

use crate::FormatImageType;
use std::path::Path;
impl Texture2D {
    pub fn create<P: AsRef<Path>>(
        gl: &WebGl2Context,
        name: &'static str,
        src: &P,
        tex_params: &'static [(u32, u32)],
        format: FormatImageType,
    ) -> Result<Texture2D, JsValue> {
        let image = HtmlImageElement::new().unwrap();
        let idx_texture_unit = unsafe { IdxTextureUnit::new()? };

        let texture = gl.create_texture();
        let onerror = {
            Closure::wrap(Box::new(move || {
                crate::log(&format!("Cannot load texture located at: {:?}", name));
            }) as Box<dyn Fn()>)
        };
        let internal_format = format.get_internal_format();
        let type_ = format.get_type();
        let format = format.get_format();

        let onload = {
            let image = image.clone();
            let gl = gl.clone();
            let texture = texture.clone();

            Closure::wrap(Box::new(move || {
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

                for (pname, param) in tex_params.iter() {
                    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, *pname, *param as i32);
                }

                gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    internal_format,
                    format,
                    type_,
                    &image,
                )
                .expect("Texture 2D");
                //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
            }) as Box<dyn Fn()>)
        };

        image.set_onload(Some(onload.as_ref().unchecked_ref()));
        image.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        image.set_cross_origin(Some(""));
        image.set_src(src.as_ref().to_str().unwrap());

        onload.forget();
        onerror.forget();

        let data = TextureType::ImageElement(image);
        let gl = gl.clone();
        Ok(Texture2D {
            texture,
            idx_texture_unit,

            gl,

            data,
            format,
            internal_format,
            type_,
        })
    }

    pub fn create_empty(
        gl: &WebGl2Context,
        width: i32,
        height: i32,
        tex_params: &'static [(u32, u32)],
        format: FormatImageType,
    ) -> Result<Texture2D, JsValue> {
        let idx_texture_unit = unsafe { IdxTextureUnit::new()? };
        let texture = gl.create_texture();

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, *pname, *param as i32);
        }
        let internal_format = format.get_internal_format();
        let type_ = format.get_type();
        let format = format.get_format();

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            type_,
            None,
        )
        .expect("Texture 2D");
        //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        let gl = gl.clone();
        let data = TextureType::Bytes(width as u32, height as u32);

        Ok(Texture2D {
            texture,
            idx_texture_unit,

            gl,

            data,
            internal_format,
            format,
            type_,
        })
    }

    pub fn create_empty_with_format(
        gl: &WebGl2Context,
        width: i32,
        height: i32,
        tex_params: &'static [(u32, u32)],
        internal_format: i32,
        format: u32,
        type_: u32,
    ) -> Result<Texture2D, JsValue> {
        let idx_texture_unit = unsafe { IdxTextureUnit::new()? };
        let texture = gl.create_texture();

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, *pname, *param as i32);
        }

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            type_,
            None,
        )
        .expect("Texture 2D");
        //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        let gl = gl.clone();
        let data = TextureType::Bytes(width as u32, height as u32);

        Ok(Texture2D {
            texture,
            idx_texture_unit,

            gl,

            data,
            format,
            internal_format,
            type_,
        })
    }

    pub fn attach_to_framebuffer(&self) {
        self.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            self.texture.as_ref(),
            0,
        );
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.data.get_width(), self.data.get_height())
    }

    pub fn active_texture(&self) -> &Self {
        self.gl.active_texture(self.idx_texture_unit);
        self
    }

    pub fn bind(&self) -> Texture2DBound {
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.texture.as_ref());

        Texture2DBound { texture_2d: self }
    }

    pub fn read_pixel(&self, x: i32, y: i32) -> Result<Pixel, JsValue> {
        // Create and bind the framebuffer
        let reader = self.gl.create_framebuffer();
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, reader.as_ref());

        // Attach the texture as the first color attachment
        self.attach_to_framebuffer();

        // set the viewport as the FBO won't be the same dimension as the screen
        self.gl.viewport(
            x,
            y,
            self.data.get_width() as i32,
            self.data.get_height() as i32,
        );

        let value = match (self.format, self.type_) {
            (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::UNSIGNED_BYTE) => {
                let val = u8::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RU8(val))
            }
            (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::SHORT) => {
                let val = i16::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RI16(val))
            }
            (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::INT) => {
                let val = i32::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RI32(val))
            }
            (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT) => {
                let val = f32::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RF32(val))
            }
            (WebGl2RenderingContext::RGB, WebGl2RenderingContext::UNSIGNED_BYTE) => {
                let val = <[u8; 3]>::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RGBU8(val))
            }
            (WebGl2RenderingContext::RGBA, WebGl2RenderingContext::UNSIGNED_BYTE) => {
                let val = <[u8; 4]>::read_pixel(&self.gl, x, y)?;
                Ok(Pixel::RGBAU8(val))
            }
            _ => Err(JsValue::from_str(
                "Pixel retrieval not implemented for that texture format.",
            )),
        };

        // Unbind the framebuffer
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        // Delete the framebuffer
        self.gl.delete_framebuffer(reader.as_ref());

        // set the viewport as the FBO won't be the same dimension as the screen
        let canvas = self
            .gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        self.gl
            .viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        value
    }

    /*pub fn get_idx_sampler(&self) -> i32 {
        let idx_sampler: i32 = (self.idx_texture_unit - WebGl2RenderingContext::TEXTURE0)
            .try_into()
            .unwrap();

        idx_sampler
    }*/
}

pub enum Pixel {
    RU8(u8),
    RI16(i16),
    RI32(i32),
    RF32(f32),
    RGBU8([u8; 3]),
    RGBAU8([u8; 4]),
}

impl From<Pixel> for JsValue {
    fn from(p: Pixel) -> Self {
        match p {
            Pixel::RU8(v) => JsValue::from_serde(&v).unwrap(),
            Pixel::RI16(v) => JsValue::from_serde(&v).unwrap(),
            Pixel::RI32(v) => JsValue::from_serde(&v).unwrap(),
            Pixel::RF32(v) => JsValue::from_serde(&v).unwrap(),
            Pixel::RGBU8(v) => JsValue::from_serde(&v).unwrap(),
            Pixel::RGBAU8(v) => JsValue::from_serde(&v).unwrap(),
        }
    }
}

trait TextureData: std::marker::Sized {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue>;
}

impl TextureData for u8 {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&pixels),
        )?;

        Ok(pixels.to_vec()[0])
    }
}
impl TextureData for [u8; 3] {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(3);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RGB,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&pixels),
        )?;
        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2]])
    }
}
impl TextureData for [u8; 4] {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(4);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&pixels),
        )?;
        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2], pixels[3]])
    }
}
impl TextureData for i16 {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Int16Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::SHORT,
            Some(&pixels),
        )?;

        Ok(pixels.to_vec()[0])
    }
}
impl TextureData for i32 {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Int32Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::INT,
            Some(&pixels),
        )?;

        Ok(pixels.to_vec()[0])
    }
}
impl TextureData for f32 {
    fn read_pixel(gl: &WebGl2RenderingContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Float32Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGl2RenderingContext::RED,
            WebGl2RenderingContext::FLOAT,
            Some(&pixels),
        )?;

        Ok(pixels.to_vec()[0])
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        self.gl.delete_texture(self.texture.as_ref());

        // free the texture unit
        let i = (self.idx_texture_unit - WebGl2RenderingContext::TEXTURE0) as usize;
        unsafe {
            AVAILABLE_TEX_UNITS[i] = Some(self.idx_texture_unit);
        }
    }
}

pub struct Texture2DBound<'a> {
    texture_2d: &'a Texture2D,
}

impl<'a> Texture2DBound<'a> {
    pub fn get_idx_sampler(&self) -> i32 {
        let idx_sampler: i32 = (self.texture_2d.idx_texture_unit
            - WebGl2RenderingContext::TEXTURE0)
            .try_into()
            .unwrap();

        idx_sampler
    }

    pub fn tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
        &self,
        dx: i32,
        dy: i32,
        image: &HtmlImageElement,
    ) {
        let _type = self.texture_2d.type_;
        let format = self.texture_2d.format;

        self.texture_2d
            .gl
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                dx,
                dy,
                format,
                _type,
                &image,
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
        let _type = self.texture_2d.type_;
        let format = self.texture_2d.format;
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                dx,
                dy,
                width,
                height,
                format,
                _type,
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
        let _type = self.texture_2d.type_;
        let format = self.texture_2d.format;
        self.texture_2d
            .gl
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                dx,
                dy,
                width,
                height,
                format,
                _type,
                pixels,
            )
            .expect("Sub texture 2d");
    }
}