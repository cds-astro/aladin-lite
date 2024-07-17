use serde::{Deserialize, Serialize};

use crate::fov::CenteredFoV;
// This struct is intended to be returned
// to the javascript to create a layer based on it
#[derive(Deserialize, Serialize, Clone)]
pub struct ImageParams {
    pub centered_fov: CenteredFoV,

    pub min_cut: f32,
    pub max_cut: f32,
}
