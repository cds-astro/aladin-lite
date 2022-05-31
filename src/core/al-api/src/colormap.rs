use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
#[wasm_bindgen]
#[derive(Clone, Debug, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Colormap {
    Blues = 0,
    Cubehelix = 1,
    Eosb = 2,
    Grayscale = 3,
    Parula = 4,
    Rainbow = 5,
    Rdbu = 6,
    Rdyibu = 7,
    Redtemperature = 8,
    Spectral = 9,
    Summer = 10,
    Yignbu = 11,
    Yiorbr = 12,
}
