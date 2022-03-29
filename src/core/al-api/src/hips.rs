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
#[serde(rename_all = "camelCase")]
pub struct SimpleHiPS {
    /// Layer name
    pub layer: String,

    /// The HiPS metadata
    pub properties: HiPSProperties,

    pub meta: ImageSurveyMeta,
}

impl SimpleHiPS {
    pub fn get_layer(&self) -> String {
        self.layer.clone()
    }

    pub fn get_properties(&self) -> &HiPSProperties {
        &self.properties
    }
}

#[derive(Deserialize, Debug)]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct HiPSProperties {
    // Associated with the HiPS
    url: String,
    max_order: u8,
    frame: HiPSFrame,
    tile_size: i32,
    bitpix: Option<i32>,
    format: HiPSTileFormat,

    // Parametrable by the user
    pub longitude_reversed: bool,
    pub min_cutout: Option<f32>,
    pub max_cutout: Option<f32>,
}

#[wasm_bindgen]
impl HiPSProperties {
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, max_order: u8, frame: HiPSFrame, longitude_reversed: bool, tile_size: i32, min_cutout: Option<f32>, max_cutout: Option<f32>, bitpix: Option<i32>, format: HiPSTileFormat) -> Self {
        Self {
            url,
            max_order,
            frame,
            longitude_reversed,
            tile_size,
            format,
            bitpix,
            min_cutout,
            max_cutout
        }
    }

    #[wasm_bindgen(getter)]
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn get_max_order(&self) -> u8 {
        self.max_order
    }

    #[wasm_bindgen(getter)]
    pub fn get_bitpix(&self) -> Option<i32> {
        self.bitpix
    }

    #[wasm_bindgen(getter)]
    pub fn get_format(&self) -> HiPSTileFormat {
        self.format
    }

    #[wasm_bindgen(getter)]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_size
    }
}

#[derive(Deserialize, Debug)]
#[derive(Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub enum HiPSFrame {
    GALACTIC,
    J2000
}

#[derive(Deserialize, Debug)]
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub enum HiPSTileFormat {
    FITS,
    JPG,
    PNG
}

use serde::Serialize;
#[wasm_bindgen]
#[derive(Deserialize, Serialize, Debug)]
#[derive(Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct GrayscaleParameter {
    pub h: TransferFunction,
    pub min_value: f32,
    pub max_value: f32,
}

impl Default for GrayscaleParameter {
    fn default() -> Self {
        Self {
            h: TransferFunction::Asinh,
            min_value: 0.0,
            max_value: 1.0
        }
    }
}

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug)]
#[derive(Deserialize, Serialize)]
pub enum TransferFunction {
    Linear,
    Sqrt,
    Log,
    Asinh,
    Pow2,
}

impl TransferFunction {
    pub fn new(id: &str) -> Self {
        if id.contains("linear") {
            TransferFunction::Linear
        } else if id.contains("pow2") {
            TransferFunction::Pow2
        } else if id.contains("log") {
            TransferFunction::Log
        } else if id.contains("sqrt") {
            TransferFunction::Sqrt
        } else {
            TransferFunction::Asinh
        }
    }
}

impl From<String> for TransferFunction {
    fn from(id: String) -> Self {
        TransferFunction::new(&id)
    }
}

use crate::colormap::Colormap;
#[derive(Deserialize, Debug)]
#[derive(Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HiPSColor {
    Grayscale2Colormap {
        colormap: Colormap,
        param: GrayscaleParameter,
        reversed: bool,
    },
    Grayscale2Color {
        param: GrayscaleParameter,
        color: [f32; 3],
        k: f32,
    },
    Color,
}

impl Default for HiPSColor {
    fn default() -> Self {
        HiPSColor::Grayscale2Color {
            color: [1.0, 0.0, 0.0],
            param: GrayscaleParameter {
                h: TransferFunction::Asinh,
                min_value: 0.0,
                max_value: 1.0,
            },
            k: 1.0
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct ImageSurveyMeta {
    /// Color config
    #[wasm_bindgen(skip)]
    pub color: HiPSColor,

    // Blending config
    #[serde(default)]
    pub blend_cfg: BlendCfg,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_opacity() -> f32 {
    1.0
}

#[wasm_bindgen]
impl ImageSurveyMeta {
    #[wasm_bindgen(setter = color)]
    pub fn set_color(&mut self, color: JsValue) -> std::result::Result<(), JsValue> {
        self.color = color.into_serde()
            .map_err(|e|  JsValue::from_str(&e.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen(getter = color)]
    pub fn color(&self) -> JsValue {
        let js_color_obj = js_sys::Object::new();

        let color = match &self.color {
            HiPSColor::Color => JsValue::from_str("Colored"),
            HiPSColor::Grayscale2Color { param, color, k } => {
                let js_grayscale2clr = js_sys::Object::new();
                
                js_sys::Reflect::set(&js_grayscale2clr, &"param".into(), &JsValue::from_serde(&param).unwrap()).unwrap();
                js_sys::Reflect::set(&js_grayscale2clr, &"color".into(), &JsValue::from_serde(&color).unwrap()).unwrap();
                js_sys::Reflect::set(&js_grayscale2clr, &"strength".into(), &JsValue::from_f64(*k as f64)).unwrap();

                js_grayscale2clr.into()
            },
            HiPSColor::Grayscale2Colormap { colormap, param, reversed } => {
                let js_grayscale2colormap = js_sys::Object::new();
                
                js_sys::Reflect::set(&js_grayscale2colormap, &"colormap".into(), &JsValue::from_serde(&colormap).unwrap()).unwrap();
                js_sys::Reflect::set(&js_grayscale2colormap, &"param".into(), &JsValue::from_serde(&param).unwrap()).unwrap();
                js_sys::Reflect::set(&js_grayscale2colormap, &"reversed".into(), &JsValue::from_bool(*reversed)).unwrap();

                js_grayscale2colormap.into()
            }
        };
        js_sys::Reflect::set(&js_color_obj, &"color".into(), &color).unwrap();

        js_color_obj.into()
    }
}

impl ImageSurveyMeta {
    pub fn visible(&self) -> bool {
        self.opacity > 0.0
    }
}
