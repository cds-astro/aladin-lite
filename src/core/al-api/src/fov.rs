use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct FoV {
    /// Position of the field of view
    pub ra: f64,
    pub dec: f64,

    /// Aperture
    pub fov: f64
}