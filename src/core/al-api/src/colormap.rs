use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
#[wasm_bindgen]
#[derive(Clone, Debug, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Colormap {
    Blues = 0,
    Cividis = 1,
    Cubehelix = 2,
    Eosb = 3,
    Grayscale = 4,
    Inferno = 5,
    Magma = 6,
    Parula = 7,
    Plasma = 8,
    Rainbow = 9,
    Rdbu = 10,
    Rdyibu = 11,
    Redtemperature = 12,
    Spectral = 13,
    Summer = 14,
    Viridis = 15,
    Ylgnbu = 16,
    Ylorbr = 17,
}
