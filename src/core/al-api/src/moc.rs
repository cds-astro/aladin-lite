use wasm_bindgen::prelude::wasm_bindgen;
use super::color::Color;

#[derive(Clone)]
#[wasm_bindgen]
pub struct MOC {
    uuid: String,
    opacity: f32,
    line_width: f32,
    adaptative_display: bool,
    is_showing: bool,
    color: Color,
}

#[wasm_bindgen]
impl MOC {
    #[wasm_bindgen(constructor)]
    pub fn new(uuid: String, opacity: f32, line_width: f32, adaptative_display: bool, is_showing: bool, color: Color) -> Self {
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

    pub fn get_color(&self) -> &Color {
        &self.color
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
            color: Color::new(1.0, 0.0, 0.0, 1.0),
        }
    }
}