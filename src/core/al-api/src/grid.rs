use wasm_bindgen::prelude::*;

use serde::{Serialize, Deserialize};

use super::color::Color;
#[wasm_bindgen]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridCfg {
    #[serde(default)]
    pub color: Color,
    #[serde(default = "default_labels")]
    pub labels: bool,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_labels() -> bool {
    true
}

fn default_enabled() -> bool {
    false
}