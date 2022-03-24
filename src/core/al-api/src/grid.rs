use wasm_bindgen::prelude::*;

use super::color::Color;
#[wasm_bindgen]
pub struct GridCfg {
    pub color: Color,
    pub labels: bool,
    pub enabled: bool,
}