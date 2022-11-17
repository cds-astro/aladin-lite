use crate::image::format::ImageFormat;

use crate::webgl_ctx::WebGlContext;
pub struct Texture2DArray {
    pub textures: Vec<Texture2D>,
}

use std::ops::Index;
impl Index<usize> for Texture2DArray {
    type Output = Texture2D;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.textures[idx]
    }
}
use super::Texture2D;
use wasm_bindgen::prelude::*;
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
        let textures: Result<Vec<_>, _> = (0..num_slices)
            .map(|_| {
                Texture2D::create_from_raw_pixels::<F>(gl, width, height, tex_params, None)
            })
            .collect();

        Ok(Texture2DArray { textures: textures? })
    }
}

const TEX_UNIFORMS_NAME: &[&str] = &["tex1", "tex2", "tex3", "tex4", "tex5"];

use crate::shader::{SendUniforms, ShaderBound};
impl SendUniforms for Texture2DArray {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        let num_tex = self.textures.len();

        for (idx, tex) in self.textures.iter().enumerate() {
            let loc = TEX_UNIFORMS_NAME[idx];
            shader.attach_uniform(loc, tex);
        }
        shader.attach_uniform("num_tex", &(num_tex as i32));

        shader
    }
}
