use wasm_bindgen::JsValue;

use super::blend::BlendCfg;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HiPSCfg {
    /// Layer name
    pub layer: String,

    /// The HiPS metadata
    pub properties: HiPSProperties,
    /// Its color
    pub meta: ImageMetadata,
}

impl HiPSCfg {
    pub fn get_layer(&self) -> &str {
        &self.layer
    }

    pub fn get_properties(&self) -> &HiPSProperties {
        &self.properties
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FITSCfg {
    /// Layer name
    pub layer: String,
    pub url: String,
    /// Its color
    pub meta: ImageMetadata,
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
    formats: Vec<HiPSTileFormat>,
    dataproduct_subtype: Option<Vec<String>>,
    hips_body: Option<bool>,

    bitpix: Option<i32>,
    sky_fraction: Option<f32>,
    min_order: Option<u8>,

    hips_initial_fov: Option<f64>,
    hips_initial_ra: Option<f64>,
    hips_initial_dec: Option<f64>,

    // Parametrable by the user
    min_cutout: Option<f32>,
    max_cutout: Option<f32>,
}

impl HiPSProperties {
    #[inline]
    pub fn get_url(&self) -> &str {
        &self.url
    }

    #[inline]
    pub fn get_max_order(&self) -> u8 {
        self.max_order
    }

    #[inline]
    pub fn get_min_order(&self) -> Option<u8> {
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
    pub fn get_sky_fraction(&self) -> Option<f32> {
        self.sky_fraction
    }

    #[inline]
    pub fn get_initial_fov(&self) -> Option<f64> {
        self.hips_initial_fov
    }

    #[inline]
    pub fn get_initial_ra(&self) -> Option<f64> {
        self.hips_initial_ra
    }

    #[inline]
    pub fn get_initial_dec(&self) -> Option<f64> {
        self.hips_initial_dec
    }

    #[inline]
    pub fn get_dataproduct_subtype(&self) -> &Option<Vec<String>> {
        &self.dataproduct_subtype
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
#[serde(rename_all = "camelCase")]
pub enum HiPSTileFormat {
    Fits,
    Jpeg,
    Png,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
#[serde(rename_all = "camelCase")]
pub enum HiPSDataproductSubtype {
    Fits,
    Jpeg,
    Png,
}

use serde::Serialize;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

impl Default for TransferFunction {
    fn default() -> Self {
        TransferFunction::Linear
    }
}

impl From<String> for TransferFunction {
    fn from(id: String) -> Self {
        TransferFunction::new(&id)
    }
}

use crate::colormap::CmapLabel;
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HiPSColor {
    // transfer function called before evaluating the colormap
    pub stretch: TransferFunction,
    // low cut 
    pub min_cut: Option<f32>,
    // high cut
    pub max_cut: Option<f32>,
    // flag to tell the colormap is queried reversed
    pub reversed: bool,
    // the colormap
    pub cmap_name: CmapLabel,
    /// tonal color tuning factors
    pub k_gamma: f32,
    pub k_saturation: f32,
    pub k_contrast: f32,
    pub k_brightness: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct ImageMetadata {
    /// Color config
    #[wasm_bindgen(skip)]
    pub color: HiPSColor,

    // Blending config
    #[serde(default)]
    pub blend_cfg: BlendCfg,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    pub longitude_reversed: bool,
    /// the current format chosen
    pub img_format: HiPSTileFormat,
}

fn default_opacity() -> f32 {
    1.0
}
use crate::Abort;

#[wasm_bindgen]
impl ImageMetadata {
    #[wasm_bindgen(setter = color)]
    pub fn set_color(&mut self, color: JsValue) -> std::result::Result<(), JsValue> {
        self.color = serde_wasm_bindgen::from_value(color)?;

        Ok(())
    }

    #[wasm_bindgen(getter = color)]
    pub fn color(&self) -> JsValue {
        let js_color_obj = js_sys::Object::new();

        let HiPSColor {
            stretch,
            min_cut,
            max_cut,
            reversed,
            cmap_name,
            k_gamma,
            k_saturation,
            k_brightness,
            k_contrast,
        } = &self.color;

        js_sys::Reflect::set(
            &js_color_obj,
            &"stretch".into(),
            &serde_wasm_bindgen::to_value(&stretch).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"minCut".into(),
            &serde_wasm_bindgen::to_value(&min_cut).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"maxCut".into(),
            &serde_wasm_bindgen::to_value(&max_cut).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"kGamma".into(),
            &serde_wasm_bindgen::to_value(&k_gamma).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"kSaturation".into(),
            &serde_wasm_bindgen::to_value(&k_saturation).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"kBrightness".into(),
            &serde_wasm_bindgen::to_value(&k_brightness).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"kContrast".into(),
            &serde_wasm_bindgen::to_value(&k_contrast).unwrap_abort(),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"reversed".into(),
            &JsValue::from_bool(*reversed),
        )
        .unwrap_abort();
        js_sys::Reflect::set(
            &js_color_obj,
            &"colormap".into(),
            &serde_wasm_bindgen::to_value(&cmap_name).unwrap_abort(),
        )
        .unwrap_abort();

        js_color_obj.into()
    }
}

impl ImageMetadata {
    pub fn visible(&self) -> bool {
        self.opacity > 0.0
    }
}
