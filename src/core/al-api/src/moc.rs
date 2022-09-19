use wasm_bindgen::prelude::wasm_bindgen;
use crate::color::ColorRGBA;

use super::color::{Color, ColorRGB};

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct MOC {
    uuid: String,
    opacity: f32,
    line_width: f32,
    adaptative_display: bool,
    is_showing: bool,
    color: ColorRGB,
}

#[wasm_bindgen]
impl MOC {
    #[wasm_bindgen(constructor)]
    pub fn new(uuid: String, opacity: f32, line_width: f32, adaptative_display: bool, is_showing: bool, hex_color: String) -> Self {
        let color = Color::hexToRgb(hex_color);
        let color = color.into();
        Self {
            uuid,
            opacity,
            line_width,
            adaptative_display,
            color,
            is_showing,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_is_showing(&mut self, is_showing: bool) {
        self.is_showing = is_showing;
    }
}

impl MOC {
    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_color(&self) -> &ColorRGB {
        &self.color
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }

    pub fn is_showing(&self) -> bool {
        self.is_showing
    }
}

impl Default for MOC {
    fn default() -> Self {
        Self {
            uuid: String::from("moc"),
            opacity: 1.0,
            line_width: 1.0,
            adaptative_display: true,
            is_showing: true,
            color: ColorRGB {r: 1.0, g: 0.0, b: 0.0},
        }
    }
}