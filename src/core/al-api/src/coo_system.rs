use cgmath::Matrix3;

const GAL2ICRS: &'static Matrix3<f64> = &Matrix3::new(
    -0.44482972122205372312012370920248,
    0.74698218398450941835110635824212,
    0.49410943719710765017955928850141,

    -0.19807633727507056817237662907031,
    0.45598381369115237931077906137440,
    -0.86766613755716255824577781583414,

    -0.87343705195577915249273984034980,
    -0.48383507361641838378786914298189,
    -0.05487565771261968232908806948676,
);

const ICRS2GAL: &'static Matrix3<f64> = &Matrix3::new(
    -0.44482972122205372312012370920248,
    -0.19807633727507056817237662907031,
    -0.87343705195577915249273984034980,

    0.74698218398450941835110635824212,
    0.45598381369115237931077906137440,
    -0.48383507361641838378786914298189,

    0.49410943719710765017955928850141,
    -0.86766613755716255824577781583414,
    -0.05487565771261968232908806948676,
);

const ID: &'static Matrix3<f64> = &Matrix3::new(
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
);


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
