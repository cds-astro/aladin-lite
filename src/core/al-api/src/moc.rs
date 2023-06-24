use wasm_bindgen::prelude::wasm_bindgen;

use super::color::{Color, ColorRGBA};

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct MOC {
    uuid: String,
    line_width: f32,
    is_showing: bool,
    color: ColorRGBA,
    adaptative_display: bool,
}
use std::convert::TryInto;
use crate::{Abort, color::ColorRGB};
#[wasm_bindgen]
impl MOC {
    #[wasm_bindgen(constructor)]
    pub fn new(uuid: String, opacity: f32, line_width: f32, is_showing: bool, hex_color: String, adaptative_display: bool) -> Self {
        let color = Color::hexToRgb(hex_color);
        let rgb: ColorRGB = color.try_into().unwrap_abort();
        let rgba = ColorRGBA { r: rgb.r, g: rgb.g, b: rgb.b, a: opacity };

        Self {
            uuid,
            line_width,
            color: rgba,
            is_showing,
            adaptative_display
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_is_showing(&mut self, is_showing: bool) {
        self.is_showing = is_showing;
    }
}

impl MOC {
    pub fn get_uuid(&self) -> &String {
        &self.uuid
    }

    pub fn get_color(&self) -> &ColorRGBA {
        &self.color
    }
    
    pub fn get_line_width(&self) -> f32 {
        self.line_width
    }

    pub fn is_showing(&self) -> bool {
        self.is_showing
    }

    pub fn is_adaptative_display(&self) -> bool {
        self.adaptative_display
    }
}

impl Default for MOC {
    fn default() -> Self {
        Self {
            uuid: String::from("moc"),
            line_width: 1.0,
            is_showing: true,
            color: ColorRGBA {r: 1.0, g: 0.0, b: 0.0, a: 1.0},
            adaptative_display: true,
        }
    }
}