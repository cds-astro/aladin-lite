use crate::image::format::ImageFormat;

use crate::webgl_ctx::WebGlContext;
pub struct Texture2DArray {
    pub textures: Vec<Texture2D>,
}

use super::pixel::Pixel;
use std::ops::Index;
impl Index<usize> for Texture2DArray {
    type Output = Texture2D;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.textures[idx]
    }
}
use crate::image::raw::ImageBuffer;
use super::texture::Texture2D;
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
        let mut textures = vec![];
        

        let raw_image = ImageBuffer::<F>::allocate(
            &<<F as ImageFormat>::P as Pixel>::BLACK,
            width,
            height,
        );

        let raw_pixels = raw_image.get_data();
        let raw_bytes = unsafe {
            std::slice::from_raw_parts(raw_pixels.as_ptr() as *const u8, raw_pixels.len() * std::mem::size_of::<<<F as ImageFormat>::P as Pixel>::Item>())
        };
        for _ in 0..num_slices {
            let texture =
                Texture2D::create_from_raw_pixels::<F>(gl, width, height, tex_params, Some(raw_bytes))?;
            textures.push(texture);
        }

        Ok(Texture2DArray { textures })
    }
}

use crate::shader::{SendUniforms, ShaderBound};
impl SendUniforms for Texture2DArray {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        let num_tex = self.textures.len();

        for (idx, tex) in self.textures.iter().enumerate() {
            let loc = &format!("tex{}", idx + 1);
            shader.attach_uniform(loc, tex);
        }
        shader.attach_uniform("num_tex", &(num_tex as i32));

        shader
    }
}
