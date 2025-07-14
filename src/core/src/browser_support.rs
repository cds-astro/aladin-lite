use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window};
use js_sys::Reflect;

pub struct BrowserFeaturesSupport {
    pub create_image_bitmap: bool,
}

impl BrowserFeaturesSupport {
    pub fn new() -> Self {
        let window = window().expect("no global `window` exists");
        let create_image_bitmap = Reflect::has(&window, &JsValue::from_str("createImageBitmap"))
            .unwrap_or(false);

        Self {
            create_image_bitmap
        }
    }
}