use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HEALPixCellProjeted {
    pub ipix: u64,
    pub vx: [f64; 4],
    pub vy: [f64; 4],
}
