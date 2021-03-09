#[derive(Clone, Copy, Debug)]
pub enum Colormap {
    BlackWhiteLinear = 0,
    RedTemperature = 1,
    IDLCBGnBu = 2,
    IDLCBYIGnBu = 3,
    BluePastelRed = 4,
    IDLCBBrBG = 5,
    viridis = 6,
    plasma = 7,
    magma = 8,
    inferno = 9,
    turbo = 10,
    YIOrBr = 11,
    stern = 12,
    EOSB = 13,
    spectral = 14,
    RdBu = 15,
    parula = 16,
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
        } else if id.contains("viridis") {
            Colormap::viridis
        } else if id.contains("plasma") {
            Colormap::plasma
        } else if id.contains("magma") {
            Colormap::magma
        } else if id.contains("inferno") {
            Colormap::inferno
        } else if id.contains("turbo") {
            Colormap::turbo
        } else if id.contains("YIOrBr") {
            Colormap::YIOrBr
        } else if id.contains("stern") {
            Colormap::stern
        } else if id.contains("EOSB") {
            Colormap::EOSB
        } else if id.contains("spectral") {
            Colormap::spectral
        } else if id.contains("RdBu") {
            Colormap::RdBu
        } else if id.contains("parula") {
            Colormap::parula
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
            "viridis",
            "plasma",
            "magma",
            "inferno",
            "turbo",
            "stern",
            "EOSB",
            "spectral",
            "RdBu",
            "parula",
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
            // TODO: update with correct shader
            Colormap::viridis => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::plasma => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::magma => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::inferno => shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapVS"),
                    Cow::Borrowed("ColormapIDL_CB_BrBGFS"),
                ),
            ),
            // TODO: update with correct shader
            Colormap::turbo => shaders.get(
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
            Colormap::stern => shaders.get(
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
            Colormap::spectral => shaders.get(
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
            Colormap::parula => shaders.get(
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
        shader.attach_uniform("colormap", self)
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
