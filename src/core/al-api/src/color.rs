use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../../js/Color.js")]
extern "C" {
    pub type Color;

    #[wasm_bindgen(static_method_of = Color)]
    pub fn hexToRgb(hex: String) -> JsValue;
    #[wasm_bindgen(static_method_of = Color)]
    pub fn hexToRgba(hex: String) -> JsValue;
    #[wasm_bindgen(static_method_of = Color)]
    pub fn rgbToHex(r: u8, g: u8, b: u8) -> String;
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct ColorRGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

use std::ops::Mul;
impl<'a> Mul<f32> for &'a ColorRGB {
    // The multiplication of rational numbers is a closed operation.
    type Output = ColorRGB;

    fn mul(self, rhs: f32) -> Self::Output {
        ColorRGB {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

/*
#[wasm_bindgen]
impl Color {
    #[wasm_bindgen(constructor)]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            red,
            green,
            blue,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}
*/

use std::convert::TryFrom;
impl TryFrom<JsValue> for ColorRGB {
    type Error = JsValue;

    fn try_from(rgb: JsValue) -> Result<Self, JsValue> {
        let mut c: ColorRGB = serde_wasm_bindgen::from_value(rgb)?;
        c.r /= 255.0;
        c.g /= 255.0;
        c.b /= 255.0;

        Ok(c)
    }
}

impl TryFrom<JsValue> for ColorRGBA {
    type Error = JsValue;

    fn try_from(rgb: JsValue) -> Result<Self, JsValue> {
        let mut c: ColorRGBA = serde_wasm_bindgen::from_value(rgb)?;
        c.r /= 255.0;
        c.g /= 255.0;
        c.b /= 255.0;
        c.a /= 255.0;

        Ok(c)
    }
}
