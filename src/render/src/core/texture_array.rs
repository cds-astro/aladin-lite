use std::rc::Rc;
use std::convert::TryInto;

use web_sys::WebGl2RenderingContext;
use web_sys::HtmlImageElement;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::console;

use crate::WebGl2Context;
use crate::image_fmt::FormatImageType;

use web_sys::WebGlTexture;
pub struct Texture2DArray {
    gl: WebGl2Context,

    texture: Option<WebGlTexture>, // The texture data
    idx_texture_unit: u32, // Internal index of the texture array
    format: FormatImageType, // The storage format (e.g. RGB, RGBA)

    width: i32, // Width of a texture element
    height: i32, // Height of a texture element
    num_slices: i32 // number of texture elements
}

use crate::core::IdxTextureUnit;

use std::path::Path;

impl Texture2DArray {
    pub fn create_from_slice_images<P: AsRef<Path>>(
        gl: &WebGl2Context,
        // Paths to the same size images
        paths: &[P],
        // The width of the image
        width: i32,
        // The height of the image
        height: i32,
        // Params of the texture 2D array
        tex_params: &'static [(u32, u32)],
        // Texture format
        format: FormatImageType,
    ) -> Rc<Texture2DArray> {
        let num_textures = paths.len();
        let texture_2d_array = Rc::new(Self::create_empty(gl, width, height, num_textures as i32, tex_params, format));

        for (idx_slice, path) in paths.iter().enumerate() {
            let image = HtmlImageElement::new().unwrap();
            let onerror = {
                let path = path.as_ref().to_str().unwrap().to_string();
                Closure::wrap(Box::new(move || {
                    unsafe { crate::log(&format!("Cannot load texture located at: {:?}", path)); }
                }) as Box<dyn Fn()>)
            };

            let onload = {
                let image = image.clone();
                let _gl = gl.clone();
                let texture_2d_array = texture_2d_array.clone();

                Closure::wrap(Box::new(move || {
                    texture_2d_array.bind()
                        .tex_sub_image_3d_with_html_image_element(0, 0, idx_slice as i32, width, height, &image);
                }) as Box<dyn Fn()>)
            };

            image.set_onload(Some(onload.as_ref().unchecked_ref()));
            image.set_onerror(Some(onerror.as_ref().unchecked_ref()));

            image.set_cross_origin(Some(""));
            image.set_src(path.as_ref().to_str().unwrap());

            onload.forget();
            onerror.forget();
        }
        
        texture_2d_array
    }

    // Create a Texture2DArray from an image
    //
    // The number of texture is defined from the height of the image.
    pub fn create<P: AsRef<Path>>(gl: &WebGl2Context,
        // The path to the image
        path: &'static P,
        // The width of the individual textures
        width: i32,
        // Their height
        height: i32,
        // How many texture slices it contains
        num_slices: i32,
        tex_params: &'static [(u32, u32)],
        // Texture format
        format: FormatImageType,
    ) -> Texture2DArray {
        let image = HtmlImageElement::new().unwrap();

        let texture = gl.create_texture();
        let idx_texture_unit = unsafe { IdxTextureUnit::new(gl) };

        let onerror = {
            Closure::wrap(Box::new(move || {
                unsafe { crate::log(&format!("Cannot load texture located at: {:?}", path.as_ref().to_str())); }
            }) as Box<dyn Fn()>)
        };

        let onload = {
            let image = image.clone();
            let gl = gl.clone();
            let texture = texture.clone();

            Closure::wrap(Box::new(move || {
                gl.active_texture(idx_texture_unit);
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, texture.as_ref());

                for (pname, param) in tex_params.iter() {
                    gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D_ARRAY, *pname, *param as i32);
                }

                let internal_format = format.get_internal_format();
                let _type = format.get_type();
                let format_tex = format.get_format();

                gl.tex_image_3d_with_html_image_element(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target
                    0, // level
                    internal_format, // internalformat
                    width, // width
                    height, // height
                    num_slices, // depth
                    0, // border
                    format_tex, // format
                    _type, // type
                    &image // source
                ).expect("Texture Array 2D");
                //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D_ARRAY);
            }) as Box<dyn Fn()>)
        };

        image.set_onload(Some(onload.as_ref().unchecked_ref()));
        image.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        image.set_cross_origin(Some(""));
        image.set_src(path.as_ref().to_str().unwrap());

        onload.forget();
        onerror.forget();
        
        let gl = gl.clone();
        Texture2DArray {
            gl,

            texture,
            idx_texture_unit,
            format,

            width,
            height,
            num_slices
        }
    }

    pub fn create_empty(gl: &WebGl2Context,
        // The weight of the individual textures
        width: i32,
        // Their height
        height: i32,
        // How many texture slices it contains
        num_slices: i32,
        tex_params: &'static [(u32, u32)],
        // Texture format
        format: FormatImageType,
    ) -> Texture2DArray {
        let texture = gl.create_texture();
        let idx_texture_unit = unsafe { IdxTextureUnit::new(gl) };

        gl.active_texture(idx_texture_unit);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, texture.as_ref());

        for (pname, param) in tex_params.iter() {
            gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D_ARRAY, *pname, *param as i32);
        }
        let internal_format = format.get_internal_format();
        let _type = format.get_type();
        let format_tex = format.get_format();

        gl.tex_image_3d_with_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target
            0, // level
            internal_format, // internalformat
            width, // width
            height, // height
            num_slices, // depth
            0, // border
            format_tex, // format
            _type, // type
            None, // source
        ).expect("Texture 2D Array");
        crate::log(&format!("AAAAA {:?} {:?} {:?}", internal_format, _type, format_tex));
        //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D_ARRAY);
        crate::log(&format!("BBBB {:?} {:?} {:?}", internal_format, _type, format_tex));

        let gl = gl.clone();
        Texture2DArray {
            gl,

            texture,
            idx_texture_unit,
            format,

            width,
            height,
            num_slices
        }
    }

    pub fn bind(&self) -> Texture2DArrayBound {
        let idx_texture_unit = self.idx_texture_unit;

        self.gl.active_texture(idx_texture_unit);
        self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, self.texture.as_ref());

        Texture2DArrayBound {
            texture_2d_array: self
        }
    }

    pub fn is_storing_integer(&self) -> bool {
        self.format.is_i_internal_format()
    }
}

impl Drop for Texture2DArray {
    fn drop(&mut self) {
        unsafe { crate::log(&"Delete texture array!"); }
        self.gl.delete_texture(self.texture.as_ref());
    }
}

pub struct Texture2DArrayBound<'a> {
    texture_2d_array: &'a Texture2DArray,
}

use crate::buffer::{ArrayF32, ArrayI32, ArrayI16, ArrayU8};
use crate::buffer::ArrayBuffer;
impl<'a> Texture2DArrayBound<'a> {
    pub fn get_idx_sampler(&self) -> i32 {
        let idx_sampler: i32 = (self.texture_2d_array.idx_texture_unit - WebGl2RenderingContext::TEXTURE0)
            .try_into()
            .unwrap();
   
        idx_sampler
    }

    pub fn clear(&self) {
        let format = &self.texture_2d_array.format;
        let format_tex = format.get_format();

        let size = (self.texture_2d_array.height as usize) * (self.texture_2d_array.width as usize) * (self.texture_2d_array.num_slices as usize) * format.get_num_channels();

        let _type = format.get_type();


        match _type {
            WebGl2RenderingContext::FLOAT => {
                let buf = ArrayF32::new(&vec![0.0; size]);
                self.texture_2d_array.gl.tex_sub_image_3d_with_opt_array_buffer_view(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
                    0, // level: i32,
                    0, // xoffset: i32,
                    0, // yoffset: i32,
                    0, // zoffset: i32,
        
                    self.texture_2d_array.width, // width: i32,
                    self.texture_2d_array.height, // height: i32,
                    self.texture_2d_array.num_slices, // depth: i32,
        
                    format_tex, // format: u32,
                    _type, // type: u32
                    Some(buf.as_ref()),
                )
                .expect("Sub texture 2d");
            },
            WebGl2RenderingContext::INT => {
                let buf = ArrayI32::new(&vec![0; size]);
                self.texture_2d_array.gl.tex_sub_image_3d_with_opt_array_buffer_view(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
                    0, // level: i32,
                    0, // xoffset: i32,
                    0, // yoffset: i32,
                    0, // zoffset: i32,
        
                    self.texture_2d_array.width, // width: i32,
                    self.texture_2d_array.height, // height: i32,
                    self.texture_2d_array.num_slices, // depth: i32,
        
                    format_tex, // format: u32,
                    _type, // type: u32
                    Some(buf.as_ref()),
                )
                .expect("Sub texture 2d");
            },
            WebGl2RenderingContext::SHORT => {
                let buf = ArrayI16::new(&vec![0; size]);
                self.texture_2d_array.gl.tex_sub_image_3d_with_opt_array_buffer_view(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
                    0, // level: i32,
                    0, // xoffset: i32,
                    0, // yoffset: i32,
                    0, // zoffset: i32,
        
                    self.texture_2d_array.width, // width: i32,
                    self.texture_2d_array.height, // height: i32,
                    self.texture_2d_array.num_slices, // depth: i32,
        
                    format_tex, // format: u32,
                    _type, // type: u32
                    Some(buf.as_ref()),
                )
                .expect("Sub texture 2d");
            },
            WebGl2RenderingContext::UNSIGNED_BYTE => {
                let buf = ArrayU8::new(&vec![0; size]);
                self.texture_2d_array.gl.tex_sub_image_3d_with_opt_array_buffer_view(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
                    0, // level: i32,
                    0, // xoffset: i32,
                    0, // yoffset: i32,
                    0, // zoffset: i32,
        
                    self.texture_2d_array.width, // width: i32,
                    self.texture_2d_array.height, // height: i32,
                    self.texture_2d_array.num_slices, // depth: i32,
        
                    format_tex, // format: u32,
                    _type, // type: u32
                    Some(buf.as_ref()),
                )
                .expect("Sub texture 2d");
            },
            _ => unimplemented!()
        };


    }

    pub fn tex_sub_image_3d_with_opt_array_buffer_view(&self,
        xoffset: i32, yoffset: i32,
        idx_texture: i32, // Idx of the texture to replace
        width: i32, // Width of the image
        height: i32, // Height of the image
        image: Option<&js_sys::Object> // image data
    ) {
        let format = &self.texture_2d_array.format;

        let format_tex = format.get_format();
        let _type = format.get_type();

        self.texture_2d_array.gl.tex_sub_image_3d_with_opt_array_buffer_view(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
            0, // level: i32,
            xoffset, // xoffset: i32,
            yoffset, // yoffset: i32,
            idx_texture, // zoffset: i32,
            width, // width: i32,
            height, // height: i32,
            1, // depth: i32,
            format_tex, // format: u32,
            _type, // type: u32
            image,
        ).expect("Sub texture 2d");
    }

    pub fn tex_sub_image_3d_with_html_image_element(&self,
        xoffset: i32, yoffset: i32,
        idx_texture: i32, // Idx of the texture to replace
        width: i32, // Width of the image
        height: i32, // Height of the image
        image: &HtmlImageElement // image data
    ) {
        let format = &self.texture_2d_array.format;

        let format_tex = format.get_format();
        let _type = format.get_type();

        self.texture_2d_array.gl.tex_sub_image_3d_with_html_image_element(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
            0, // level: i32,
            xoffset, // xoffset: i32,
            yoffset, // yoffset: i32,
            idx_texture, // zoffset: i32,
            width, // width: i32,
            height, // height: i32,
            1, // depth: i32,
            format_tex, // format: u32,
            _type, // type: u32
            image,
        ).expect("Sub texture 2d");
    }

    pub fn tex_sub_image_3d_with_opt_u8_array(&self,
        xoffset: i32, yoffset: i32,
        idx_texture: i32, // Idx of the texture to replace
        width: i32, // Width of the image
        height: i32, // Height of the image
        src_data: Option<&[u8]> // image data
    ) {
        let format = &self.texture_2d_array.format;

        let format_tex = format.get_format();
        let _type = format.get_type();

        self.texture_2d_array.gl.tex_sub_image_3d_with_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY, // target: u32,
            0, // level: i32,
            xoffset, // xoffset: i32,
            yoffset, // yoffset: i32,
            idx_texture, // zoffset: i32,
            width, // width: i32,
            height, // height: i32,
            1, // depth: i32,
            format_tex, // format: u32,
            _type as u32, // type: u32
            src_data,
        )
        .expect("Sub texture 2d");
    }
}