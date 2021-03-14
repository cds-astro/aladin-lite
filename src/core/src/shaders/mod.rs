#[derive(Clone, Copy, Debug)]
pub enum Colormap {
    BlackWhiteLinear = 0,
    RedTemperature = 1,
    IDLCBGnBu = 2,
    IDLCBYIGnBu = 3,
    BluePastelRed = 4,
    IDLCBBrBG = 5,
    Viridis = 6,
    Plasma = 7,
    Magma = 8,
    Inferno = 9,
    Turbo = 10,
    YIOrBr = 11,
    Stern = 12,
    EOSB = 13,
    Spectral = 14,
    RdBu = 15,
    Parula = 16,
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
        } else if id.contains("Viridis") {
            Colormap::Viridis
        } else if id.contains("Plasma") {
            Colormap::Plasma
        } else if id.contains("Magma") {
            Colormap::Magma
        } else if id.contains("Inferno") {
            Colormap::Inferno
        } else if id.contains("Turbo") {
            Colormap::Turbo
        } else if id.contains("YIOrBr") {
            Colormap::YIOrBr
        } else if id.contains("Stern") {
            Colormap::Stern
        } else if id.contains("EOSB") {
            Colormap::EOSB
        } else if id.contains("Spectral") {
            Colormap::Spectral
        } else if id.contains("RdBu") {
            Colormap::RdBu
        } else if id.contains("Parula") {
            Colormap::Parula
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
            "YIOrBr",
            "Viridis",
            "Plasma",
            "Magma",
            "Inferno",
            "Turbo",
            "Stern",
            "EOSB",
            "Spectral",
            "RdBu",
            "Parula",
            "BlackWhiteLinear",
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
            // TODO: update with correct shader
            Colormap::Viridis => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Plasma => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Magma => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Inferno => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Turbo => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::YIOrBr => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Stern => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::EOSB => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Spectral => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::RdBu => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::Parula => shaders.get(
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
        shader
            .attach_uniform("colormap", self)
            .attach_uniform("reversed", &1);

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
