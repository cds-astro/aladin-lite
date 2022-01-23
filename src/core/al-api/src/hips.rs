use al_core::WebGlContext;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use super::blend::BlendCfg;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct CompositeHiPS {
    hipses: Vec<SimpleHiPS>,
}

use std::iter::IntoIterator;
impl IntoIterator for CompositeHiPS {
    type Item = SimpleHiPS;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.hipses.into_iter()
    }
}

#[derive(Deserialize, Debug)]
/// The JSON image survey object
pub struct SimpleHiPS {
    /// All that is found in the properties file of the HiPS
    pub properties: HiPSProperties,

    /// The color of the HiPS, can be a direct map to one color, a colormap or the color images
    /// for compressed JPG/PNG tiles
    pub color: HiPSColor,

    // Blending config
    pub blend_cfg: BlendCfg,
    pub opacity: f32,

    /// Layer name in which the survey will be placed in
    pub layer: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HiPSProperties {
    pub url: String,

    pub max_order: u8,
    pub frame: Frame,
    pub tile_size: i32,
    pub min_cutout: Option<f32>,
    pub max_cutout: Option<f32>,
    pub format: HiPSFormat,
}

#[derive(Deserialize, Debug)]
pub struct Frame {
    pub label: String,
    pub system: String,
}

#[derive(Deserialize, Debug)]
pub enum HiPSFormat {
    FITSImage { bitpix: i32 },
    Image { format: String },
}

#[derive(Deserialize, Debug, Clone)]
pub enum HiPSColor {
    Grayscale2Colormap {
        colormap: String,
        transfer: String,
        reversed: bool,
    },
    Grayscale2Color {
        color: [f32; 3],
        transfer: String,
        k: f32,
    },
    Color,
}
