use wasm_bindgen::prelude::wasm_bindgen;

use super::color::{Color, ColorRGBA};

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct MOC {
    uuid: String,
    pub line_width: f32,
    pub perimeter: bool,
    pub filled: bool,
    pub edges: bool,
    pub show: bool,

    pub color: ColorRGBA,
    pub fill_color: ColorRGBA,
}
use crate::{color::ColorRGB, Abort};
use std::convert::TryInto;
#[wasm_bindgen]
impl MOC {
    #[wasm_bindgen(constructor)]
    pub fn new(
        uuid: String,
        opacity: f32,
        line_width: f32,
        perimeter: bool,
        filled: bool,
        edges: bool,
        show: bool,
        hex_color: String,
        fill_color: String,
    ) -> Self {
        let parse_color = |color_hex_str: String, opacity: f32| -> ColorRGBA {
            let rgb = Color::hexToRgb(color_hex_str);
            let rgb: ColorRGB = rgb.try_into().unwrap_abort();
            ColorRGBA {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
                a: opacity,
            }
        };

        let color = parse_color(hex_color, 1.0);
        let fill_color = parse_color(fill_color, opacity);

        Self {
            uuid,
            line_width,
            perimeter,
            filled,
            fill_color,
            edges,
            color,
            show,
        }
    }
}

impl MOC {
    pub fn get_uuid(&self) -> &String {
        &self.uuid
    }
}

impl Default for MOC {
    fn default() -> Self {
        Self {
            uuid: String::from("moc"),
            line_width: 1.0,
            perimeter: false,
            edges: true,
            filled: false,
            show: true,
            color: ColorRGBA {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fill_color: ColorRGBA {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}
