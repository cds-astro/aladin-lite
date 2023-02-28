use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[wasm_bindgen]
pub struct FoV {
    /// Position of the field of view
    pub ra: f64,
    pub dec: f64,

    /// Aperture
    pub fov: f64
}