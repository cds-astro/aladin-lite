use al_api::resources::Resources;
use std::collections::HashMap;

use al_core::Texture2D;
use al_core::WebGlContext;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub struct Colormaps {
    tex: Texture2D,
    colormaps: HashMap<&'static str, Colormap>,
}

use crate::Abort;
use al_api::colormap::Colormap;
impl Colormaps {
    pub fn new(gl: &WebGlContext, rs: &Resources) -> Result<Self, JsValue> {
        let colormaps: HashMap<&str, Colormap> = [
            ("blues", Colormap::Blues),
            ("cividis", Colormap::Cividis),
            ("cubehelix", Colormap::Cubehelix),
            ("eosb", Colormap::Eosb),
            ("grayscale", Colormap::Grayscale),
            ("inferno", Colormap::Inferno),
            ("magma", Colormap::Magma),
            ("parula", Colormap::Parula),
            ("plasma", Colormap::Plasma),
            ("rainbow", Colormap::Rainbow),
            ("rdbu", Colormap::Rdbu),
            ("rdyibu", Colormap::Rdyibu),
            ("redtemperature", Colormap::Redtemperature),
            ("spectral", Colormap::Spectral),
            ("summer", Colormap::Summer),
            ("viridis", Colormap::Viridis),
            ("yignbu", Colormap::Yignbu),
            ("yiorbr", Colormap::Yiorbr),
        ]
        .iter()
        .cloned()
        .collect();
        let colormap_filename = rs.get_filename("colormaps").unwrap_abort();
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
            "cividis",
            "cubehelix",
            "eosb",
            "grayscale",
            "inferno",
            "magma",
            "parula",
            "plasma",
            "rainbow",
            "rdbu",
            "rdyibu",
            "redtemperature",
            "spectral",
            "summer",
            "viridis",
            "yignbu",
            "yiorbr",
        ]
    }

    pub fn get(&self, name: &str) -> Colormap {
        let c = if let Some(c) = self.colormaps.get(name) {
            c
        } else {
            self.colormaps.get("grayscale").unwrap_abort()
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
