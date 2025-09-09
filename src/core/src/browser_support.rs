use js_sys::Reflect;
use wasm_bindgen::JsValue;
use web_sys::window;

pub struct BrowserFeaturesSupport {
    pub create_image_bitmap: bool,
}

impl Default for BrowserFeaturesSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserFeaturesSupport {
    pub fn new() -> Self {
        let window = window().expect("no global `window` exists");
        let create_image_bitmap =
            Reflect::has(&window, &JsValue::from_str("createImageBitmap")).unwrap_or(false);

        Self {
            create_image_bitmap,
        }
    }
}
