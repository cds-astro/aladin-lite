use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use std::fmt;

#[wasm_bindgen(raw_module = "../../js/libs/astro/coo.js")]
extern "C" {
    #[wasm_bindgen(js_name = Format)]
    pub type Format;

    /**
     * Convert a decimal coordinate into sexagesimal string, according to the given precision<br>
     * 8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
     * @param num number (integer or decimal)
     * @param prec precision (= number of decimal digit to keep or append)
     * @param plus if true, the '+' sign is displayed
     * @return a string with the formatted sexagesimal number
     */
    #[wasm_bindgen(static_method_of = Format)]
    pub fn toSexagesimal(num: f64, prec: u8, plus: bool) -> String;
    /**
     * Convert a decimal coordinate into a decimal string, according to the given precision
     * @param num number (integer or decimal)
     * @param prec precision (= number of decimal digit to keep or append)
     * @return a string with the formatted sexagesimal number
     */
    #[wasm_bindgen(static_method_of = Format)]
    pub fn toDecimal(num: f64, prec: u8) -> String;
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
#[wasm_bindgen]
pub enum AngleSerializeFmt {
    DMM,
    DD,
    DMS,
    HMS,
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
