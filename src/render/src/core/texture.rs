

use std::convert::TryInto;

use web_sys::WebGl2RenderingContext;
use web_sys::HtmlImageElement;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::ImageData;
use web_sys::console;

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
            TextureType::Bytes(_, height) => *height
        }
    }
}

use web_sys::WebGlTexture;
pub struct Texture2D {
    pub texture: Option<WebGlTexture>,
    pub idx_texture_unit: u32,

    gl: WebGl2Context,

    data: TextureType,

    pub format: FormatImageType,
}

static mut NUM_TEXTURE_UNIT: u32 = WebGl2RenderingContext::TEXTURE0;
pub struct IdxTextureUnit;
impl IdxTextureUnit {
    pub unsafe fn new(gl: &WebGl2Context) -> u32 {
        let max_combined_texture_image_units = Self::max_combined_texture_image_units(gl);
        if max_combined_texture_image_units == NUM_TEXTURE_UNIT {
            panic!(format!("Number of texture image units excedeed. The limit is {:?}", max_combined_texture_image_units));
        }
        let idx_texture_unit = NUM_TEXTURE_UNIT;
        NUM_TEXTURE_UNIT += 1;

        idx_texture_unit
    }

    fn max_combined_texture_image_units(gl: &WebGl2Context) -> u32 {
        gl.get_parameter(WebGl2RenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS).unwrap().as_f64().unwrap() as u32
    }
}

use crate::FormatImageType;
use std::path::Path;
impl Texture2D {
    pub fn create<P: AsRef<Path>>(gl: &WebGl2Context, name: &'static str, src: &P, tex_params: &'static [(u32, u32)], format: FormatImageType) -> Texture2D {
        let image = HtmlImageElement::new().unwrap();
        let idx_texture_unit = unsafe { IdxTextureUnit::new(gl) };

        let texture = gl.create_texture();
        let onerror = {
            Closure::wrap(Box::new(move || {
                unsafe { crate::log(&format!("Cannot load texture located at: {:?}", name)); }
            }) as Box<dyn Fn()>)
        };

        let onload = {
            let image = image.clone();
            let gl = gl.clone();
            let texture = texture.clone();

            Closure::wrap(Box::new(move || {
                gl.active_texture(idx_texture_unit);
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

                for (pname, param) in tex_params.iter() {
                    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, *pname, *param as i32);
                }

                let internal_format = format.get_internal_format();
                let _type = format.get_type();
                let format_tex = format.get_format();

                gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                    WebGl2RenderingContext::TEXTURE_2D,
                    0,
                    internal_format,
                    format_tex,
                    _type,
                    &image
                ).expect("Texture 2D");
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
        Texture2D {
            texture,
            idx_texture_unit,

            gl,

            data,
            format
        }
    }

    pub fn create_empty(gl: &WebGl2Context, width: i32, height: i32, tex_params: &'static [(u32, u32)], format: FormatImageType) -> Texture2D {
        let idx_texture_unit = unsafe { IdxTextureUnit::new(gl) };
        let texture = gl.create_texture();

        gl.active_texture(idx_texture_unit);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, *pname, *param as i32);
        }

        let internal_format = format.get_internal_format();
        let _type = format.get_type();
        let format_tex = format.get_format();

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format_tex,
            _type,
            None
        ).expect("Texture 2D");
        //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        let gl = gl.clone();
        let data = TextureType::Bytes(width as u32, height as u32);

        Texture2D {
            texture,
            idx_texture_unit,

            gl,

            data,
            format
        }
    }

    pub fn attach_to_framebuffer(&self) {
        self.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            self.texture.as_ref(),
            0
        );
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.data.get_width(), self.data.get_height())
    }

    pub fn bind(&self) -> Texture2DBound {
        let texture_unit = self.idx_texture_unit;

        self.gl.active_texture(texture_unit);
        self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, self.texture.as_ref());

        Texture2DBound {
            texture_2d: self
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        self.gl.delete_texture(self.texture.as_ref());
    }
}


pub struct Texture2DBound<'a> {
    texture_2d: &'a Texture2D,
}

/*impl<'a> Drop for Texture2DBound<'a> {
    fn drop(&mut self) {
        self.texture_2d.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }
}*/

impl<'a> Texture2DBound<'a> {
    pub fn unbind(&self) {
        let texture_unit = self.texture_2d.idx_texture_unit;

        self.texture_2d.gl.active_texture(texture_unit);
        self.texture_2d.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }

    pub fn get_idx_sampler(&self) -> i32 {
        let idx_sampler: i32 = (self.texture_2d.idx_texture_unit - WebGl2RenderingContext::TEXTURE0)
            .try_into()
            .unwrap();
   
        idx_sampler
    }

    /*pub fn _clear(&self) {
        let (width, height) = (self.texture_2d.data.get_width(), self.texture_2d.data.get_height());

        let data = vec![0 as u8; 3 * (height as usize) * (width as usize)];
        self.texture_2d.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            0,
            0,

            width as i32,
            height as i32,

            WebGl2RenderingContext::RGB,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&data),
        )
        .expect("Sub texture 2d");
    }*/

    pub fn tex_sub_image_2d_with_u32_and_u32_and_html_image_element(&self, dx: i32, dy: i32, image: &HtmlImageElement) {
        let _type = self.texture_2d.format.get_type();
        let format = self.texture_2d.format.get_format();

        self.texture_2d.gl.tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
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
        width: i32, // Width of the image
        height: i32, // Height of the image
        image: Option<&js_sys::Object>
    ) {
        let _type = self.texture_2d.format.get_type();
        let format = self.texture_2d.format.get_format();
        self.texture_2d.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
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

    pub fn tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
        &self,
        dx: i32,
        dy: i32,
        width: i32, // Width of the image
        height: i32, // Height of the image
        pixels: Option<&[u8]>
    ) {
        let _type = self.texture_2d.format.get_type();
        let format = self.texture_2d.format.get_format();
        self.texture_2d.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
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