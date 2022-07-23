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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleHiPS {
    /// Layer name
    pub layer: String,

    /// The HiPS metadata
    pub properties: HiPSProperties,

    pub meta: ImageSurveyMeta,

    pub img_format: HiPSTileFormat,
}

/*#[wasm_bindgen]
impl SimpleHiPS {
    #[wasm_bindgen(constructor)]
    pub fn new(layer: String, properties: HiPSProperties, meta: ImageSurveyMeta) -> Self {
        Self {
            layer,
            properties,
            meta,
            backend: None
        }
    }
}*/

impl SimpleHiPS {
    pub fn get_layer(&self) -> String {
        self.layer.clone()
    }

    pub fn get_properties(&self) -> &HiPSProperties {
        &self.properties
    }
}

use crate::coo_system::CooSystem;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HiPSProperties {
    // Associated with the HiPS
    url: String,
    max_order: u8,
    frame: CooSystem,
    tile_size: i32,
    bitpix: Option<i32>,
    formats: Vec<HiPSTileFormat>,
    sky_fraction: f32,
    min_order: u8,

    // Parametrable by the user
    pub min_cutout: Option<f32>,
    pub max_cutout: Option<f32>,
}

impl HiPSProperties {
    pub fn new(
        url: String,
        max_order: u8,
        frame: CooSystem,
        tile_size: i32,
        min_cutout: Option<f32>,
        max_cutout: Option<f32>,
        bitpix: Option<i32>,
        formats: Vec<HiPSTileFormat>,
        sky_fraction: f32,
        min_order: u8
    ) -> Self {
        Self {
            url,
            max_order,
            min_order,
            frame,
            tile_size,
            formats,
            bitpix,
            min_cutout,
            max_cutout,
            sky_fraction
        }
    }

    #[inline]
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    #[inline]
    pub fn get_max_order(&self) -> u8 {
        self.max_order
    }

    #[inline]
    pub fn get_min_order(&self) -> u8 {
        self.min_order
    }

    #[inline]
    pub fn get_bitpix(&self) -> Option<i32> {
        self.bitpix
    }

    #[inline]
    pub fn get_formats(&self) -> &[HiPSTileFormat] {
        &self.formats[..]
    }

    #[inline]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_size
    }

    #[inline]
    pub fn get_frame(&self) -> CooSystem {
        self.frame
    }

    #[inline]
    pub fn get_sky_fraction(&self) -> f32 {
        self.sky_fraction
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum HiPSTileFormat {
    FITS,
    JPEG,
    PNG,
}

use serde::Serialize;
/*#[wasm_bindgen]
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
}*/

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug, Deserialize, Serialize)]
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

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HiPSColor {
    // FITS tile
    Grayscale {
        #[serde(rename = "stretch")]
        tf: TransferFunction,
        #[serde(rename = "minCut")]
        min_cut: Option<f32>,
        #[serde(rename = "maxCut")]
        max_cut: Option<f32>,

        color: GrayscaleColor,
    },
    // JPG/PNG tile
    Color,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy)]
pub enum GrayscaleColor {
    Colormap { reversed: bool, name: Colormap },
    Color([f32; 4]),
}
/*
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
}*/

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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
    pub longitude_reversed: bool,
}

fn default_opacity() -> f32 {
    1.0
}

#[wasm_bindgen]
impl ImageSurveyMeta {
    #[wasm_bindgen(setter = color)]
    pub fn set_color(&mut self, color: JsValue) -> std::result::Result<(), JsValue> {
        self.color = color
            .into_serde()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(())
    }

    #[wasm_bindgen(getter = color)]
    pub fn color(&self) -> JsValue {
        let js_color_obj = js_sys::Object::new();

        let color = match &self.color {
            HiPSColor::Color => JsValue::from_str("Colored"),
            HiPSColor::Grayscale {
                tf,
                min_cut,
                max_cut,
                color,
            } => {
                let js_grayscale = js_sys::Object::new();

                js_sys::Reflect::set(
                    &js_grayscale,
                    &"stretch".into(),
                    &JsValue::from_serde(&tf).unwrap(),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_grayscale,
                    &"minCut".into(),
                    &JsValue::from_serde(min_cut).unwrap(),
                )
                .unwrap();
                js_sys::Reflect::set(
                    &js_grayscale,
                    &"maxCut".into(),
                    &JsValue::from_serde(max_cut).unwrap(),
                )
                .unwrap();

                let js_color = match color {
                    GrayscaleColor::Color(color) => {
                        let js_color = js_sys::Object::new();
                        js_sys::Reflect::set(
                            &js_color,
                            &"color".into(),
                            &JsValue::from_serde(&color).unwrap(),
                        )
                        .unwrap();

                        js_color
                    }
                    GrayscaleColor::Colormap { reversed, name } => {
                        let js_colormap = js_sys::Object::new();
                        js_sys::Reflect::set(
                            &js_colormap,
                            &"reversed".into(),
                            &JsValue::from_bool(*reversed),
                        )
                        .unwrap();
                        js_sys::Reflect::set(
                            &js_colormap,
                            &"colormap".into(),
                            &JsValue::from_serde(name).unwrap(),
                        )
                        .unwrap();

                        js_colormap
                    }
                };
                js_sys::Reflect::set(&js_grayscale, &"color".into(), &js_color).unwrap();

                js_grayscale.into()
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
