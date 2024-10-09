use al_api::blend::BlendFunc;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;

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
use crate::Abort;

impl WebGlContext {
    pub fn new(aladin_div: &HtmlElement) -> Result<WebGlContext, JsValue> {
        let canvas = aladin_div
            // Inside it, retrieve the canvas
            .get_elements_by_class_name("aladin-imageCanvas")
            .get_with_index(0)
            .unwrap_abort();
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_abort();

        // See https://stackoverflow.com/a/26790802/13456997
        // preserveDrawingBuffer enabled for exporting the view as a PNG
        let context_options =
            js_sys::JSON::parse("{\"antialias\":false, \"preserveDrawingBuffer\": true}")?;
        //js_sys::JSON::parse("{\"antialias\":false}")?;

        #[cfg(feature = "webgl2")]
        let gl = Rc::new(
            canvas
                .get_context_with_context_options("webgl2", context_options.as_ref())?
                .unwrap_abort()
                .dyn_into::<WebGlRenderingCtx>()
                .unwrap_abort(),
        );
        // https://webgl2fundamentals.org/webgl/lessons/webgl-data-textures.html
        #[cfg(feature = "webgl2")]
        gl.pixel_storei(WebGlRenderingCtx::UNPACK_ALIGNMENT, 1);

        #[cfg(feature = "webgl2")]
        {
            if let Ok(r) =
                get_extension::<web_sys::ExtColorBufferFloat>(&gl, "EXT_color_buffer_float")
            {
                let _ = r;
            }

            let ctx = WebGlContext { inner: gl };
            Ok(ctx)
        }

        #[cfg(feature = "webgl1")]
        {
            let angles_ext =
                get_extension::<web_sys::AngleInstancedArrays>(&gl, "ANGLE_instanced_arrays")?;
            let _ = get_extension::<web_sys::OesTextureFloat>(&gl, "OES_texture_float")?;
            let _ = get_extension::<web_sys::ExtSRgb>(&gl, "EXT_sRGB")?;

            Ok(WebGlContext {
                inner: gl,
                ext: WebGlExt { angles: angles_ext },
            })
        }
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
        .ok_or_else(|| JsValue::from_str("Failed to load ext"))
}

use std::ops::Deref;
impl Deref for WebGlContext {
    type Target = WebGlRenderingCtx;

    fn deref(&self) -> &WebGlRenderingCtx {
        &self.inner
    }
}

pub trait GlWrapper {
    fn enable(
        &self,
        gl: &WebGlContext,
        f: impl FnOnce() -> Result<(), JsValue>,
    ) -> Result<(), JsValue>;
}

use al_api::blend::{BlendCfg, BlendFactor};
impl GlWrapper for BlendCfg {
    fn enable(
        &self,
        gl: &WebGlContext,
        f: impl FnOnce() -> Result<(), JsValue>,
    ) -> Result<(), JsValue> {
        let blend_factor_f = |f: &BlendFactor| -> u32 {
            match f {
                BlendFactor::ConstantAlpha => WebGlRenderingCtx::CONSTANT_ALPHA,
                BlendFactor::ConstantColor => WebGlRenderingCtx::CONSTANT_COLOR,
                BlendFactor::Zero => WebGlRenderingCtx::ZERO,
                BlendFactor::One => WebGlRenderingCtx::ONE,
                BlendFactor::DstAlpha => WebGlRenderingCtx::DST_ALPHA,
                BlendFactor::DstColor => WebGlRenderingCtx::DST_COLOR,
                BlendFactor::OneMinusConstantAlpha => WebGlRenderingCtx::ONE_MINUS_CONSTANT_ALPHA,
                BlendFactor::OneMinusDstColor => WebGlRenderingCtx::ONE_MINUS_DST_COLOR,
                BlendFactor::OneMinusDstAlpha => WebGlRenderingCtx::ONE_MINUS_DST_ALPHA,
                BlendFactor::SrcAlpha => WebGlRenderingCtx::SRC_ALPHA,
                BlendFactor::SrcColor => WebGlRenderingCtx::SRC_COLOR,
                BlendFactor::OneMinusSrcColor => WebGlRenderingCtx::ONE_MINUS_SRC_COLOR,
                BlendFactor::OneMinusSrcAlpha => WebGlRenderingCtx::ONE_MINUS_SRC_ALPHA,
                BlendFactor::OneMinusConstantColor => WebGlRenderingCtx::ONE_MINUS_CONSTANT_ALPHA,
            }
        };

        let blend_func_f = |f: &BlendFunc| -> u32 {
            match f {
                BlendFunc::FuncAdd => WebGlRenderingCtx::FUNC_ADD,
                BlendFunc::FuncReverseSubstract => WebGlRenderingCtx::FUNC_REVERSE_SUBTRACT,
                BlendFunc::FuncSubstract => WebGlRenderingCtx::FUNC_SUBTRACT,
            }
        };

        gl.blend_equation(blend_func_f(&self.func));
        gl.blend_func_separate(
            blend_factor_f(&self.src_color_factor),
            blend_factor_f(&self.dst_color_factor),
            WebGlRenderingCtx::ONE,
            WebGlRenderingCtx::ONE,
        );

        f()?;

        gl.blend_equation(blend_func_f(&BlendFunc::FuncAdd));

        Ok(())
    }
}
