use serde::{Deserialize, Serialize};

use crate::fov::CenteredFoV;
// This struct is intended to be returned
// to the javascript to create a layer based on it
#[derive(Deserialize, Serialize)]
#[derive(Clone)]
pub struct ImageParams {
    pub centered_fov: CenteredFoV,

    // a new layer
    pub layer: String,
    // and its url
    pub url: String,

    pub automatic_min_cut: f32,
    pub automatic_max_cut: f32,
}