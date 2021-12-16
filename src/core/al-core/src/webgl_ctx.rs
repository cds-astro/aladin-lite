use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[cfg(feature = "webgl2")]
pub type WebGlRenderingCtx = web_sys::WebGl2RenderingContext;
#[cfg(feature = "webgl1")]
pub type WebGlRenderingCtx = web_sys::WebGlRenderingContext;


#[derive(Clone)]
pub struct WebGlContext {
    inner: Rc<WebGlRenderingCtx>,

    #[cfg(feature = "webgl1")]
    pub ext: WebGlExt,
}

#[derive(Clone)]
pub struct WebGlExt {
    #[cfg(feature = "webgl1")]
    pub angles: web_sys::AngleInstancedArrays,
}

impl WebGlContext {
    pub fn new(aladin_div_name: &str) -> Result<WebGlContext, JsValue> {
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

        // See https://stackoverflow.com/a/26790802/13456997
        // preserveDrawingBuffer enabled for exporting the view as a PNG
        let context_options =
            js_sys::JSON::parse(&"{\"antialias\":false, \"preserveDrawingBuffer\": true}")?;
        
        #[cfg(feature = "webgl1")]
        let gl = Rc::new(
            canvas
                .get_context_with_context_options("webgl", context_options.as_ref())?
                .unwrap()
                .dyn_into::<WebGlRenderingCtx>()
                .unwrap(),
        );
        #[cfg(feature = "webgl2")]
        let gl = Rc::new(
            canvas
                .get_context_with_context_options("webgl2", context_options.as_ref())?
                .unwrap()
                .dyn_into::<WebGlRenderingCtx>()
                .unwrap(),
        );

        #[cfg(feature = "webgl2")]
        let _ = get_extension::<web_sys::ExtColorBufferFloat>(&gl, "EXT_color_buffer_float")?;
        #[cfg(feature = "webgl1")]
        let angles_ext = get_extension::<web_sys::AngleInstancedArrays>(&gl, "ANGLE_instanced_arrays")?;
        #[cfg(feature = "webgl1")]
        let _ = get_extension::<web_sys::ExtSRgb>(&gl, "EXT_sRGB")?;

        #[cfg(feature = "webgl1")]
        let ctx = WebGlContext { inner: gl, ext: WebGlExt { angles: angles_ext } };
        #[cfg(feature = "webgl2")]
        let ctx = WebGlContext { inner: gl };

        Ok(ctx)
    }
}

fn get_extension<T>(context: &WebGlRenderingCtx, name: &str) -> Result<T, JsValue>
where
    T: wasm_bindgen::JsCast,
{
    // `unchecked_into` is used here because WebGL extensions aren't actually JS classes
    // these objects are duck-type representations of the actual Rust classes
    // https://github.com/rustwasm/wasm-bindgen/pull/1449
    context
        .get_extension(name)
        .ok()
        .and_then(|maybe_ext| maybe_ext.map(|ext| ext.unchecked_into::<T>()))
        .ok_or(JsValue::from_str(&format!("Failed to load ext: {}", name)))
}



use std::ops::Deref;
impl Deref for WebGlContext {
    type Target = WebGlRenderingCtx;

    fn deref(&self) -> &WebGlRenderingCtx {
        &self.inner
    }
}
