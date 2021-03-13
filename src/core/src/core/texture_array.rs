use crate::image_fmt::FormatImageType;
use crate::WebGl2Context;

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
use crate::JsValue;
impl Texture2DArray {
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
    ) -> Result<Texture2DArray, JsValue> {
        let mut textures = vec![];
        for _slice_idx in 0..num_slices {
            let texture = Texture2D::create_empty(gl, width, height, tex_params, format)?;
            textures.push(texture);
        }

        Ok(Texture2DArray { textures })
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
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
