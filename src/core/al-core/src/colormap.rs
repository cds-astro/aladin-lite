use std::collections::HashMap;

use colorgrad::Color;

use crate::Texture2D;
use crate::WebGlContext;
use crate::image::format;
use crate::shader::SendUniformsWithParams;

use wasm_bindgen::JsValue;
use crate::webgl_ctx::WebGlRenderingCtx;

const WIDTH_CMAP_TEX: usize = 256;

type Label = String;

pub struct Colormap {
    label: Label,
    grad: colorgrad::Gradient,
}
impl Colormap {
    pub fn new(label: &str, grad: colorgrad::Gradient) -> Self {
        Self { label: label.to_string(), grad }
    }

    pub fn label(&self) -> &Label {
        &self.label
    }
}

fn build_cmaps_texture(gl: &WebGlContext, cmaps: &[Colormap]) -> Result<Texture2D, JsValue> {
    let tex_bytes: Vec<u8> = cmaps.iter()
        .map(|cmap| {
            let mut values = [0_u8; 1024];
            for ix in 0..WIDTH_CMAP_TEX {
                let rgba = cmap.grad.at(ix as f64 / WIDTH_CMAP_TEX as f64).to_rgba8();
                let ptr = values[4*ix..].as_mut_ptr() as *mut [u8; 4];
                unsafe { *ptr = rgba; }
            }

            values
        })
        .flatten()
        .collect();
    let tex_params = &[
        (
            WebGlRenderingCtx::TEXTURE_MIN_FILTER,
            WebGlRenderingCtx::LINEAR,
        ),
        (
            WebGlRenderingCtx::TEXTURE_MAG_FILTER,
            WebGlRenderingCtx::LINEAR,
        ),
        // Prevents s-coordinate wrapping (repeating)
        (
            WebGlRenderingCtx::TEXTURE_WRAP_S,
            WebGlRenderingCtx::CLAMP_TO_EDGE,
        ),
        // Prevents t-coordinate wrapping (repeating)
        (
            WebGlRenderingCtx::TEXTURE_WRAP_T,
            WebGlRenderingCtx::CLAMP_TO_EDGE,
        ),
    ];

    Texture2D::create_from_raw_pixels::<format::RGBA8U>(
        gl,
        WIDTH_CMAP_TEX as i32,
        cmaps.len() as i32,
        tex_params,
        Some(&tex_bytes[..])
    )
}

pub struct Colormaps {
    cmaps: Vec<Colormap>,
    indices: HashMap<Label, i32>,

    cmaps_tex: Texture2D,

    labels: Vec<Label>,

    gl: WebGlContext,
}

use crate::Abort;
impl Colormaps {
    pub fn new(gl: &WebGlContext) -> Result<Self, JsValue> {
        let labels: Vec<_> = [
            "blues", "cividis", "cubehelix", "eosb",
            "grayscale", "inferno", "magma", "native",
            "parula", "plasma", "rainbow", "rdbu",
            "rdylbu", "redtemperature", "sinebow", "spectral", "summer",
            "viridis", "ylgnbu", "ylorbr", "red", "green", "blue"
        ]
        .iter()
        .map(|cmap_name| cmap_name.to_string())
        .collect();

        let indices = labels.iter().enumerate()
            .map(|(id, label)| {
                (label.clone(), id as i32)
            })
            .collect();

        let cmaps = vec![
            Colormap::new("blues", colorgrad::blues()),
            Colormap::new("cividis", colorgrad::cividis()),
            Colormap::new("cubehelix", colorgrad::cubehelix_default()),
            Colormap::new("eosb", colorgrad::turbo()),
            Colormap::new("grayscale", {
                colorgrad::CustomGradient::new()
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("inferno", colorgrad::inferno()),
            Colormap::new("magma", colorgrad::magma()),
            Colormap::new("native", {
                colorgrad::CustomGradient::new()
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("parula", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::from_rgba8(61, 38, 168, 255),
                        Color::from_rgba8(71, 87, 247, 255),
                        Color::from_rgba8(39, 150, 235, 255),
                        Color::from_rgba8(24, 191, 181, 255),
                        Color::from_rgba8(128, 203, 88, 255),
                        Color::from_rgba8(253, 189, 60, 255),
                        Color::from_rgba8(249, 250, 20, 255),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("plasma", colorgrad::plasma()),
            Colormap::new("rainbow", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::from_rgba8(127, 0, 255, 255),
                        Color::from_rgba8(0, 0, 255, 255),
                        Color::from_rgba8(0, 127, 255, 255),
                        Color::from_rgba8(0, 255, 255, 255),
                        Color::from_rgba8(0, 255, 127, 255),
                        Color::from_rgba8(0, 255, 0, 255),
                        Color::from_rgba8(127, 255, 0, 255),
                        Color::from_rgba8(255, 255, 0, 255),
                        Color::from_rgba8(255, 127, 0, 255),
                        Color::from_rgba8(255, 0, 0, 255),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("rdbu", colorgrad::rd_bu()),
            Colormap::new("rdylbu", colorgrad::rd_yl_bu()),
            Colormap::new("redtemperature", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        Color::new(0.75, 0.0, 0.0, 1.0),
                        Color::new(1.0, 0.5, 0.0, 1.0),
                        Color::new(1.0, 1.0, 1.0, 1.0),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("sinebow", colorgrad::sinebow()),
            Colormap::new("spectral", colorgrad::spectral()),
            Colormap::new("summer", colorgrad::yl_gn()),
            Colormap::new("viridis", colorgrad::viridis()),
            Colormap::new("ylgnbu", colorgrad::yl_gn_bu()),
            Colormap::new("ylorbr", colorgrad::yl_or_br()),
            Colormap::new("red", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        Color::new(1.0, 0.0, 0.0, 1.0),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("green", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        Color::new(0.0, 1.0, 0.0, 1.0),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
            Colormap::new("blue", {
                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::new(0.0, 0.0, 0.0, 1.0),
                        Color::new(0.0, 0.0, 1.0, 1.0),
                    ])
                    .build()
                    .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?
            }),
        ];

        let cmaps_tex = build_cmaps_texture(gl, &cmaps[..])?;

        let gl = gl.clone();
        Ok(Self { cmaps, cmaps_tex, labels, indices, gl })
    }

    #[inline]
    pub fn get_list_available_colormaps(&self) -> &[Label] {
        &self.labels
    }

    #[inline]
    pub fn get(&self, label: &str) -> &Colormap {
        if let Some(id) = self.get_id(label).map(|id| *id) {
            &self.cmaps[id as usize]
        } else {
            crate::log::console_warn(&format!("{:?} is not a valid colormap, replaced with 'grayscale'.", label));
            let id_greys = self.get_id("grayscale").map(|id| *id).unwrap_abort();
            &self.cmaps[id_greys as usize]
        }
    }

    #[inline]
    pub fn get_id(&self, label: &str) -> Option<&i32> {
        self.indices.get(label)
    }

    pub fn add_cmap(&mut self, label: Label, cmap: Colormap) -> Result<(), JsValue> {
        if let Some(id) = self.get_id(&label).map(|id| *id) {
            let colormap = &mut self.cmaps[id as usize];
            *colormap = cmap;
        } else {
            let num_cmaps = self.labels.len();
            self.labels.push(label.clone());
    
            self.indices.insert(label, num_cmaps as i32);
            self.cmaps.push(cmap);
        }

        // recompute the texture
        self.cmaps_tex = build_cmaps_texture(&self.gl, &self.cmaps)?;

        Ok(())
    }
}

use crate::shader::{ShaderBound, SendUniforms};
impl SendUniforms for Colormaps {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniform("colormaps", &self.cmaps_tex)
            .attach_uniform("num_colormaps", &(self.labels.len() as f32));

        shader
    }
}

impl SendUniformsWithParams<Colormaps> for Colormap {
    fn attach_uniforms_with_params<'a>(&self, shader: &'a ShaderBound<'a>, cmaps: &Colormaps) -> &'a ShaderBound<'a> {
        let cmap_id = cmaps.get_id(&self.label).unwrap_abort();
        shader.attach_uniform("colormap_id", &(*cmap_id as f32));
        shader
    }
}
