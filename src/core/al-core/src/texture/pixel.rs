use crate::image::{ArrayBuffer, ArrayF32, ArrayI16, ArrayI32, ArrayU8};
use crate::webgl_ctx::WebGlRenderingCtx;
use wasm_bindgen::JsValue;

use crate::webgl_ctx::WebGlContext;
pub trait Pixel:
    AsRef<[Self::Item]> + Default + std::cmp::PartialEq + std::fmt::Debug + std::clone::Clone + serde::ser::Serialize
{
    type Item: std::cmp::PartialOrd + Clone + Copy + std::fmt::Debug + cgmath::Zero;
    type Container: ArrayBuffer<Item = Self::Item>;

    const BLACK: Self;

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue>;
}

impl Pixel for [f32; 4] {
    type Item = f32;
    type Container = ArrayF32;
    const BLACK: Self = [0.0, 0.0, 0.0, 1.0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Float32Array::new_with_length(4);
        #[cfg(feature = "webgl2")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGBA32F,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;
        #[cfg(feature = "webgl1")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGBA,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;

        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2], pixels[3]])
    }
}
impl Pixel for [f32; 3] {
    type Item = f32;
    type Container = ArrayF32;
    const BLACK: Self = [0.0, 0.0, 0.0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Float32Array::new_with_length(3);
        #[cfg(feature = "webgl2")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGB32F,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;
        #[cfg(feature = "webgl1")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGB,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;

        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2]])
    }
}
impl Pixel for [f32; 1] {
    type Item = f32;
    type Container = ArrayF32;
    const BLACK: Self = [0.0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Float32Array::new_with_length(1);
        #[cfg(feature = "webgl2")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RED,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;
        #[cfg(feature = "webgl1")]
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::LUMINANCE_ALPHA,
            WebGlRenderingCtx::FLOAT,
            Some(&pixels),
        )?;

        Ok([pixels.to_vec()[0]])
    }
}
impl Pixel for [u8; 4] {
    type Item = u8;
    type Container = ArrayU8;
    const BLACK: Self = [0, 0, 0, 255];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(4);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGBA,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&pixels),
        )?;
        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2], pixels[3]])
    }
}
impl Pixel for [u8; 3] {
    type Item = u8;
    type Container = ArrayU8;
    const BLACK: Self = [0, 0, 0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(3);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGB,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&pixels),
        )?;
        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1], pixels[2]])
    }
}
#[cfg(feature = "webgl2")]
impl Pixel for [u8; 1] {
    type Item = u8;
    type Container = ArrayU8;
    const BLACK: Self = [0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RED_INTEGER,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&pixels),
        )?;

        Ok([pixels.to_vec()[0]])
    }
}
#[cfg(feature = "webgl2")]
impl Pixel for [i16; 1] {
    type Item = i16;
    type Container = ArrayI16;
    const BLACK: Self = [0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Int16Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RED_INTEGER,
            WebGlRenderingCtx::SHORT,
            Some(&pixels),
        )?;

        Ok([pixels.to_vec()[0]])
    }
}
#[cfg(feature = "webgl2")]
impl Pixel for [i32; 1] {
    type Item = i32;
    type Container = ArrayI32;
    const BLACK: Self = [0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Int32Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RED_INTEGER,
            WebGlRenderingCtx::INT,
            Some(&pixels),
        )?;

        Ok([pixels.to_vec()[0]])
    }
}

/*impl From<T> for JsValue
where
    T: Pixel
{
    fn from(p: T) -> Self {
        JsValue::from_serde(&p).unwrap()
    }
}*/
/*
pub enum PixelType {
    RU8([u8; 1]),
    RI16([i16; 1]),
    RI32([i32; 1]),
    RF32([f32; 1]),
    RGBU8([u8; 3]),
    RGBAU8([u8; 4]),
}

impl From<PixelType> for JsValue {
    fn from(p: PixelType) -> Self {
        match p {
            PixelType::RU8(v) => JsValue::from_serde(&v).unwrap(),
            PixelType::RI16(v) => JsValue::from_serde(&v).unwrap(),
            PixelType::RI32(v) => JsValue::from_serde(&v).unwrap(),
            PixelType::RF32(v) => JsValue::from_serde(&v).unwrap(),
            PixelType::RGBU8(v) => JsValue::from_serde(&v).unwrap(),
            PixelType::RGBAU8(v) => JsValue::from_serde(&v).unwrap(),
        }
    }
}*/
