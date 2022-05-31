/*
grayscale = 0,
blues = 1,
parula = 2,
rainbow = 3,
redtemperature = 4,
RdBu = 5,
RdYiBu = 6,
spectral = 7,
summer = 8,
YIGnBu = 9,
YIOrBr = 10,
cubehelix = 11,
eosb = 12,
*/
use al_core::resources::Resources;
use std::collections::HashMap;

use al_core::Texture2D;
use al_core::WebGlContext;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub struct Colormaps {
    tex: Texture2D,
    colormaps: HashMap<&'static str, Colormap>,
}

use al_api::colormap::Colormap;
impl Colormaps {
    pub fn new(gl: &WebGlContext, rs: &Resources) -> Result<Self, JsValue> {
        let colormaps: HashMap<&str, Colormap> = [
            ("blues", Colormap::Blues),
            ("cubehelix", Colormap::Cubehelix),
            ("eosb", Colormap::Eosb),
            ("grayscale", Colormap::Grayscale),
            ("parula", Colormap::Parula),
            ("rainbow", Colormap::Rainbow),
            ("rdbu", Colormap::Rdbu),
            ("rdyibu", Colormap::Rdyibu),
            ("redtemperature", Colormap::Redtemperature),
            ("spectral", Colormap::Spectral),
            ("summer", Colormap::Summer),
            ("yignbu", Colormap::Yignbu),
            ("yiorbr", Colormap::Yiorbr),
        ]
        .iter()
        .cloned()
        .collect();
        let colormap_filename = rs.get_filename("colormaps").unwrap();
        let tex = Texture2D::create_from_path::<_, al_core::image::format::RGBA8U>(
            gl,
            "colormap",
            &colormap_filename,
            &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
            ],
        )?;

        Ok(Self { colormaps, tex })
    }

    #[inline]
    pub const fn get_list_available_colormaps() -> &'static [&'static str] {
        &[
            "blues",
            "cubehelix",
            "eosb",
            "grayscale",
            "parula",
            "rainbow",
            "rdbu",
            "rdyibu",
            "redtemperature",
            "spectral",
            "summer",
            "yignbu",
            "yiorbr",
        ]
    }

    pub fn get(&self, name: &str) -> Colormap {
        let c = if let Some(c) = self.colormaps.get(name) {
            c
        } else {
            self.colormaps.get("grayscale").unwrap()
        };

        *c
    }
}

use al_core::shader::{SendUniforms, ShaderBound};

impl SendUniforms for Colormaps {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniform("colormaps", &self.tex)
            .attach_uniform("num_colormaps", &(self.colormaps.len() as f32));

        shader
    }
}
