use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::convert::WasmSlice;

use super::blend::BlendCfg;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct CompositeHiPS {
    hipses: Vec<SimpleHiPS>,
}

use std::fmt::Result;
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

#[wasm_bindgen]
impl TransferFunction {
    #[wasm_bindgen(constructor)]
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

use serde::Serialize;
#[wasm_bindgen]
#[derive(Deserialize, Serialize, Clone, Debug)]
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

use crate::colormap::Colormap;
#[derive(Deserialize, Debug, Clone)]
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
#[derive(Clone)]
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
    pub fn visible(&self) -> bool {
        self.opacity > 0.0
    }

    #[wasm_bindgen(getter)]
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

    #[wasm_bindgen(setter)]
    pub fn set_color(&mut self, color: JsValue) -> std::result::Result<(), JsValue> {
        self.color = color.into_serde()
            .map_err(|e|  JsValue::from_str(&e.to_string()))?;

        Ok(())
    }
}
/*
impl PartialEq for ImageSurveyMeta {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}*/