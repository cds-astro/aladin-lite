use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
#[wasm_bindgen]
#[derive(Clone, Debug, Copy, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Colormap {
    Blackwhite = 0,
    Blues = 1,
    Parula = 2,
    Rainbow = 3,
    RdBu = 4,
    RdYiBu = 5,
    RedTemperature = 6,
    Spectral = 7,
    Summer = 8,
    YIGnBu = 9,
    YIOrBr = 10,
}