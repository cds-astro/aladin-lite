use cgmath::Matrix4;
pub trait CooBaseFloat: Sized + 'static {
    const GALACTIC_TO_J2000: &'static Matrix4<Self>;
    const J2000_TO_GALACTIC: &'static Matrix4<Self>;
    const ID: &'static Matrix4<Self>;
}

impl CooBaseFloat for f32 {
    const GALACTIC_TO_J2000: &'static Matrix4<Self> = &Matrix4::new(
        -0.444_829_64,
        0.746_982_2,
        0.494_109_42,
        0.0,
        -0.198_076_37,
        0.455_983_8,
        -0.867_666_1,
        0.0,
        -0.873_437_1,
        -0.483_835,
        -0.054_875_56,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    const J2000_TO_GALACTIC: &'static Matrix4<Self> = &Matrix4::new(
        -0.444_829_64,
        -0.198_076_37,
        -0.873_437_1,
        0.0,
        0.746_982_2,
        0.455_983_8,
        -0.483_835,
        0.0,
        0.494_109_42,
        -0.867_666_1,
        -0.054_875_56,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    const ID: &'static Matrix4<Self> = &Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
}
impl CooBaseFloat for f64 {
    const GALACTIC_TO_J2000: &'static Matrix4<Self> = &Matrix4::new(
        -0.4448296299195045,
        0.7469822444763707,
        0.4941094279435681,
        0.0,
        -0.1980763734646737,
        0.4559837762325372,
        -0.867_666_148_981_161,
        0.0,
        -0.873437090247923,
        -0.4838350155267381,
        -0.0548755604024359,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    const J2000_TO_GALACTIC: &'static Matrix4<Self> = &Matrix4::new(
        -0.4448296299195045,
        -0.1980763734646737,
        -0.873437090247923,
        0.0,
        0.7469822444763707,
        0.4559837762325372,
        -0.4838350155267381,
        0.0,
        0.4941094279435681,
        -0.867_666_148_981_161,
        -0.0548755604024359,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    const ID: &'static Matrix4<Self> = &Matrix4::new(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
}

use cgmath::BaseFloat;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize, Hash)]
pub enum CooSystem {
    ICRSJ2000 = 0,
    GAL = 1,
}

pub const NUM_COOSYSTEM: usize = 2;

impl CooSystem {
    #[inline]
    pub fn to<S>(&self, coo_system: &Self) -> &Matrix4<S>
    where
        S: BaseFloat + CooBaseFloat,
    {
        match (self, coo_system) {
            (CooSystem::GAL, CooSystem::ICRSJ2000) => S::GALACTIC_TO_J2000,
            (CooSystem::ICRSJ2000, CooSystem::GAL) => S::J2000_TO_GALACTIC,
            (_, _) => S::ID,
        }
    }
}
