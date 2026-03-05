use cgmath::Matrix3;

const GAL2ICRS: &Matrix3<f64> = &Matrix3::new(
    -0.444_829_721_222_053_7,
    0.746_982_183_984_509_4,
    0.494_109_437_197_107_65,
    -0.198_076_337_275_070_57,
    0.455_983_813_691_152_4,
    -0.867_666_137_557_162_6,
    -0.873_437_051_955_779_1,
    -0.483_835_073_616_418_37,
    -0.054_875_657_712_619_68,
);

const ICRS2GAL: &Matrix3<f64> = &Matrix3::new(
    -0.444_829_721_222_053_7,
    -0.198_076_337_275_070_57,
    -0.873_437_051_955_779_1,
    0.746_982_183_984_509_4,
    0.455_983_813_691_152_4,
    -0.483_835_073_616_418_37,
    0.494_109_437_197_107_65,
    -0.867_666_137_557_162_6,
    -0.054_875_657_712_619_68,
);

const ID: &Matrix3<f64> = &Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

use serde::Deserialize;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Hash)]
pub enum CooSystem {
    ICRS,
    GAL,
}

pub const NUM_COOSYSTEM: usize = 2;

impl CooSystem {
    #[inline]
    pub fn to(&self, coo_system: Self) -> &Matrix3<f64> {
        match (self, coo_system) {
            (CooSystem::GAL, CooSystem::ICRS) => GAL2ICRS,
            (CooSystem::ICRS, CooSystem::GAL) => ICRS2GAL,
            (_, _) => ID,
        }
    }
}
