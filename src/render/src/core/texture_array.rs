use web_sys::WebGl2RenderingContext;

use crate::image_fmt::FormatImageType;
use crate::WebGl2Context;

pub struct Texture2DArray {
    gl: WebGl2Context,

    pub textures: Vec<Texture2D>,
}

use super::{Texture2D, Texture2DBound};

impl Texture2DArray {
    /*pub fn create_from_slice_images<P: AsRef<Path>>(
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
    }*/

    // Create a Texture2DArray from an image
    //
    // The number of texture is defined from the height of the image.
    /*pub fn create<P: AsRef<Path>>(gl: &WebGl2Context,
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
    }*/

    pub fn create_empty(
        gl: &WebGl2Context,
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
        let mut textures = vec![];
        for _slice_idx in 0..num_slices {
            let texture = Texture2D::create_empty(gl, width, height, tex_params, format);
            textures.push(texture);
        }

        /*let texture = gl.create_texture();
        let idx_texture_unit = unsafe { IdxTextureUnit::new(gl) };

        gl.active_texture(idx_texture_unit);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, texture.as_ref());
        crate::log(&format!("{:?} bound", gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)));

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
        //gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D_ARRAY);*/

        let gl = gl.clone();
        Texture2DArray { gl, textures }
    }

    pub fn bind_texture_slice(&self, idx_texture: i32) -> Texture2DBound {
        let texture = &self.textures[idx_texture as usize];
        texture.bind()
    }

    /*pub fn bind(&self) -> Texture2DArrayBound {
        let mut textures_bound = vec![];
        for texture in self.textures.iter() {
            textures_bound.push(texture.bind());
        }

        Texture2DArrayBound {
            gl: self.gl.clone(),
            format: self.format,
            textures: textures_bound
        }
    }*/
}

/*impl Drop for Texture2DArray {
    fn drop(&mut self) {
        //self.gl.active_texture(self.idx_texture_unit);
        //self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, None);
        self.gl.delete_texture(self.texture.as_ref());
    }
}*/

/*pub struct Texture2DArrayBound<'a> {
    textures: &'a Vec<Texture2D>,
}*/

/*impl<'a> Drop for Texture2DArrayBound<'a> {
    fn drop(&mut self) {
        for (tex_idx, texture) in self.textures.iter().enumerate() {
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE0 + tex_idx as u32);
            self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        }
    }
}*/

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
impl SendUniforms for Texture2DArray {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        let num_tex = self.textures.len();

        let mut textures_bound = Vec::with_capacity(num_tex);
        for texture_idx in 0..num_tex {
            let texture_bound = self.bind_texture_slice(texture_idx as i32);
            textures_bound.push(texture_bound.get_idx_sampler());
        }

        shader
            .attach_uniform("tex[0]", &textures_bound.as_slice())
            .attach_uniform("num_tex", &(num_tex as i32));

        shader
    }
}
