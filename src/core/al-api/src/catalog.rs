use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

use crate::angle_fmt::AngleSerializeFmt;

use super::color::ColorRGB;

#[wasm_bindgen]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    #[serde(default = "default_color")]
    pub color: Option<ColorRGB>,
    #[serde(default = "default_opacity")]
    pub opacity: Option<f32>,
    #[serde(default = "default_shape")]
    pub shape: Option<&'static str>,
    #[serde(default = "default_size")]
    pub size: Option<f32>,
}

fn default_color() -> Option<ColorRGB> {
    None
}

fn default_opacity() -> Option<f32> {
    None
}

fn default_shape() -> Option<&'static str> {
    None
}

fn default_size() -> Option<AngleSerializeFmt> {
    None
}
