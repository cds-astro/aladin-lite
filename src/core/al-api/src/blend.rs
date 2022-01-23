use al_core::WebGlContext;
use serde::Deserialize;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

#[cfg(feature = "webgl2")]
pub type WebGlRenderingCtx = web_sys::WebGl2RenderingContext;
#[cfg(feature = "webgl1")]
pub type WebGlRenderingCtx = web_sys::WebGlRenderingContext;

#[derive(Deserialize, Debug)]
#[derive(Clone)]
pub struct BlendCfg {
    pub src_color_factor: BlendFactor,
    pub dst_color_factor: BlendFactor,
    pub func: BlendFunc,
}

#[derive(Deserialize, Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum BlendFactor {
    Zero = WebGlRenderingCtx::ZERO as isize,
    One = WebGlRenderingCtx::ONE as isize,
    
    SrcColor = WebGlRenderingCtx::SRC_COLOR as isize,
    OneMinusSrcColor = WebGlRenderingCtx::ONE_MINUS_SRC_COLOR as isize,

    DstColor = WebGlRenderingCtx::DST_COLOR as isize,
    OneMinusDstColor = WebGlRenderingCtx::ONE_MINUS_DST_COLOR as isize,

    SrcAlpha = WebGlRenderingCtx::SRC_ALPHA as isize,
    OneMinusSrcAlpha = WebGlRenderingCtx::ONE_MINUS_SRC_ALPHA as isize,

    DstAlpha = WebGlRenderingCtx::DST_ALPHA as isize,
    OneMinusDstAlpha = WebGlRenderingCtx::ONE_MINUS_DST_ALPHA as isize,

    ConstantColor = WebGlRenderingCtx::CONSTANT_COLOR as isize,
    OneMinusConstantColor = WebGlRenderingCtx::ONE_MINUS_CONSTANT_COLOR as isize,
    ConstantAlpha = WebGlRenderingCtx::CONSTANT_ALPHA as isize,
    OneMinusConstantAlpha = WebGlRenderingCtx::ONE_MINUS_CONSTANT_ALPHA as isize,
}

#[derive(Deserialize, Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum BlendFunc {
    FuncAdd = WebGlRenderingCtx::FUNC_ADD as isize,
    FuncSubstract = WebGlRenderingCtx::FUNC_SUBTRACT as isize,
    FuncReverseSubstract = WebGlRenderingCtx::FUNC_REVERSE_SUBTRACT as isize,
    Min = WebGlRenderingCtx::MIN as isize,
    Max = WebGlRenderingCtx::MAX as isize
}

impl BlendCfg {
    pub fn active_blend_cfg(&self, gl: &WebGlContext, f: impl FnOnce() -> ()) {
        gl.blend_equation(self.func as u32);
        gl.blend_func_separate(
            self.src_color_factor as u32,
            self.dst_color_factor as u32,
            WebGlRenderingCtx::ONE,
            WebGlRenderingCtx::ONE,
        );

        f();

        gl.blend_equation(BlendFunc::FuncAdd as u32);
    }
}

use std::fmt;
impl fmt::Display for BlendFactor {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let str = match self {
            BlendFactor::ConstantAlpha => "ConstantAlpha",
            BlendFactor::ConstantColor => "ConstantColor",
            BlendFactor::Zero => "Zero",
            BlendFactor::One => "One",
            BlendFactor::DstAlpha => "DstAlpha",
            BlendFactor::DstColor => "DstColor",
            BlendFactor::OneMinusConstantAlpha => "OneMinusConstantAlpha",
            BlendFactor::OneMinusDstColor => "OneMinusDstColor",
            BlendFactor::OneMinusDstAlpha => "OneMinusDstAlpha",
            BlendFactor::SrcAlpha => "SrcAlpha",
            BlendFactor::SrcColor => "SrcColor",
            BlendFactor::OneMinusSrcColor => "OneMinusSrcColor",
            BlendFactor::OneMinusSrcAlpha => "OneMinusSrcAlpha",
            BlendFactor::OneMinusConstantColor => "OneMinusConstantColor",
        };
        write!(f, "{}", str)
    }
}
impl fmt::Display for BlendFunc {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let str = match self {
            BlendFunc::FuncAdd => "Add",
            BlendFunc::FuncSubstract => "Subtract",
            BlendFunc::FuncReverseSubstract => "Reverse Subtract",
            BlendFunc::Min => "Min",
            BlendFunc::Max => "Max",
        };
        write!(f, "{}", str)
    }
}