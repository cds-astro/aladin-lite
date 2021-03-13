use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

#[derive(Clone)]
pub struct WebGl2Context {
    inner: Rc<WebGl2RenderingContext>,
}

impl WebGl2Context {
    pub fn new(aladin_div_name: &str) -> Result<WebGl2Context, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document
            // Get the aladin div element
            .get_element_by_id(aladin_div_name)
            .unwrap()
            // Inside it, retrieve the canvas
            .get_elements_by_class_name("aladin-imageCanvas")
            .get_with_index(0)
            .unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context_options = js_sys::JSON::parse(&"{\"antialias\":false}")?;
        let gl = Rc::new(
            canvas
                .get_context_with_context_options("webgl2", context_options.as_ref())?
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap(),
        );

        let ext = gl.get_extension("EXT_color_buffer_float")?;

        Ok(WebGl2Context { inner: gl })
    }
}

use std::ops::Deref;
impl Deref for WebGl2Context {
    type Target = WebGl2RenderingContext;

    fn deref(&self) -> &WebGl2RenderingContext {
        &self.inner
    }
}
