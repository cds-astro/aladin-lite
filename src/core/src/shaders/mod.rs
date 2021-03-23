/*
blackwhite = 0,
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
*/
use std::collections::HashMap;
pub struct Colormaps {
    tex: Texture2D,
    colormaps: HashMap<&'static str, Colormap>,
}

impl Colormaps {
    pub fn new(gl: &WebGl2Context, rs: &Resources) -> Result<Self, JsValue> {
        let colormaps: HashMap<&str, Colormap> = [
            (
                "blackwhite",
                Colormap {
                    name: "blackwhite",
                    id: 0,
                },
            ),
            (
                "blues",
                Colormap {
                    name: "blues",
                    id: 1,
                },
            ),
            (
                "parula",
                Colormap {
                    name: "parula",
                    id: 2,
                },
            ),
            (
                "rainbow",
                Colormap {
                    name: "rainbow",
                    id: 3,
                },
            ),
            (
                "RdBu",
                Colormap {
                    name: "RdBu",
                    id: 4,
                },
            ),
            (
                "RdYiBu",
                Colormap {
                    name: "RdYiBu",
                    id: 5,
                },
            ),
            (
                "redtemperature",
                Colormap {
                    name: "redtemperature",
                    id: 6,
                },
            ),
            (
                "spectral",
                Colormap {
                    name: "spectral",
                    id: 7,
                },
            ),
            (
                "summer",
                Colormap {
                    name: "summer",
                    id: 8,
                },
            ),
            (
                "YIGnBu",
                Colormap {
                    name: "YIGnBu",
                    id: 9,
                },
            ),
            (
                "YIOrBr",
                Colormap {
                    name: "YIOrBr",
                    id: 10,
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let colormap_filename = rs.get_filename("colormaps").unwrap();
        let tex = Texture2D::create(
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
            FormatImageType::PNG,
        )?;

        Ok(Self { colormaps, tex })
    }

    #[inline]
    pub const fn get_list_available_colormaps() -> &'static [&'static str] {
        &[
            "blackwhite",
            "blues",
            "parula",
            "rainbow",
            "RdBu",
            "RdYiBu",
            "redtemperature",
            "spectral",
            "summer",
            "YIGnBu",
            "YIOrBr",
        ]
    }

    pub fn get(&self, name: &str) -> Colormap {
        let c = if let Some(c) = self.colormaps.get(name) {
            c
        } else {
            self.colormaps.get("blackwhite").unwrap()
        };

        *c
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Colormap {
    pub name: &'static str,
    pub id: i32,
}

use crate::core::Texture2D;
use crate::image_fmt::FormatImageType;
use crate::resources::Resources;
use crate::shader::ShaderId;
use crate::WebGl2Context;
use std::borrow::Cow;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
impl Colormap {
    pub fn get_catalog_shader<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> Result<&'a Shader, JsValue> {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("ColormapCatalogVS"),
                    Cow::Borrowed("ColormapCatalogFS"),
                ),
            )
            .map_err(|e| e.into())
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;

impl SendUniforms for Colormaps {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniform("colormaps", &self.tex)
            .attach_uniform("num_colormaps", &(self.colormaps.len() as i32));

        shader
    }
}

impl SendUniforms for Colormap {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniform("colormap_id", &self.id);

        shader
    }
}

use crate::Shader;
use crate::ShaderManager;
