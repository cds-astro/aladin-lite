use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

use crate::angle_fmt::AngleSerializeFmt;

use super::color::ColorRGB;

#[wasm_bindgen]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridCfg {
    #[serde(default = "default_color")]
    pub color: Option<ColorRGB>,
    pub opacity: Option<f32>,
    #[serde(default = "default_labels")]
    pub show_labels: Option<bool>,
    #[serde(default = "default_label_size")]
    pub label_size: Option<f32>,
    #[serde(default = "default_enabled")]
    pub enabled: Option<bool>,
    #[serde(default = "default_fmt")]
    pub fmt: Option<AngleSerializeFmt>,
}

fn default_labels() -> Option<bool> {
    None
}

fn default_enabled() -> Option<bool> {
    None
}

fn default_color() -> Option<ColorRGB> {
    None
}

fn default_label_size() -> Option<f32> {
    None
}

fn default_fmt() -> Option<AngleSerializeFmt> {
    None
}
