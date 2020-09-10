#[derive(Clone, Copy, Debug)]
pub enum Colormap {
    BlackWhiteLinear = 0,
    RedTemperature = 1,
    IDLCBGnBu = 2,
    IDLCBYIGnBu = 3,
    BluePastelRed = 4,
    IDLCBBrBG = 5,
}

impl Colormap {
    pub fn new(id: &str) -> Self {
        if id.contains("RedTemperature") {
            Colormap::RedTemperature
        } else if id.contains("BluePastelRed") {
            Colormap::BluePastelRed
        } else if id.contains("IDLCBGnBu") {
            Colormap::IDLCBGnBu
        } else if id.contains("IDLCBYIGnBu") {
            Colormap::IDLCBYIGnBu
        } else if id.contains("IDLCBBrBG") {
            Colormap::IDLCBBrBG
        } else {
            Colormap::BlackWhiteLinear
        }
    }

    pub fn get_shader<'a>(&self, shaders: &'a ShaderManager) -> &'a Shader {
        let shader = match self {
            Colormap::BlackWhiteLinear => shaders.get("black_white_linear"),
            Colormap::RedTemperature => shaders.get("red_temperature"),
            Colormap::IDLCBGnBu => shaders.get("IDL_CB_GnBu"),
            Colormap::IDLCBYIGnBu => shaders.get("IDL_CB_YIGnBu"),
            Colormap::BluePastelRed => shaders.get("BluePastelRed"),
            Colormap::IDLCBBrBG => shaders.get("IDL_CB_BrBG"),
        };

        shader.unwrap()
    }
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;

impl HasUniforms for Colormap {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("colormap_id", self);

        shader
    }
}

use crate::ShaderManager;
use crate::Shader;