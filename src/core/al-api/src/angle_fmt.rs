use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use std::fmt;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum AngleSerializeFmt {
    DMM,
    DD,
    DMS,
    HMS
}

impl fmt::Display for AngleSerializeFmt {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let str = match self {
            Self::DMM => "DMM",
            Self::DD => "DD",
            Self::DMS => "DMS",
            Self::HMS => "HMS",
        };
        write!(f, "{}", str)
    }
}