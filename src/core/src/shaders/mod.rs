#[derive(Clone, Copy, Debug)]
pub enum Colormap {
    BlackWhiteLinear = 0,
    RedTemperature = 1,
    IDLCBGnBu = 2,
    IDLCBYIGnBu = 3,
    BluePastelRed = 4,
    IDLCBBrBG = 5,
}
use std::borrow::Cow;

use crate::{shader::ShaderId, WebGl2Context};
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

    #[inline]
    pub const fn get_list_available_colormaps() -> &'static [&'static str] {
        &[
            "RedTemperature",
            "BluePastelRed",
            "IDLCBGnBu",
            "IDLCBYIGnBu",
            "IDLCBBrBG",
            "BlackWhiteLinear"
        ]
    }

    pub fn get_shader<'a>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        let shader = match self {
            Colormap::BlackWhiteLinear => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapBlackWhiteFS"),
                ),
            ),
            Colormap::RedTemperature => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapRedTemperatureFS"),
                ),
            ),
            Colormap::IDLCBGnBu => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_GnBuFS"),
                ),
            ),
            Colormap::IDLCBYIGnBu => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_YIGnBuFS"),
                ),
            ),
            Colormap::BluePastelRed => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapBluePastelRedFS"),
                ),
            ),
            Colormap::IDLCBBrBG => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
        };

        shader.unwrap()
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;

impl SendUniforms for Colormap {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("colormap", self);

        shader
    }
}

impl From<String> for Colormap {
    fn from(id: String) -> Self {
        Colormap::new(&id)
    }
}

use crate::Shader;
use crate::ShaderManager;
