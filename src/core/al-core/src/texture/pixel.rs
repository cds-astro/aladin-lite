use crate::image::{ArrayBuffer, ArrayF32, ArrayI16, ArrayI32, ArrayU8};
use crate::webgl_ctx::WebGlRenderingCtx;
use wasm_bindgen::JsValue;

use crate::webgl_ctx::WebGlContext;
pub trait Pixel:
    AsRef<[Self::Item]> + Default + std::cmp::PartialEq + std::fmt::Debug + std::clone::Clone
{
    type Item: std::cmp::PartialOrd
        + Clone
        + Copy
        + std::fmt::Debug
        + cgmath::Zero
        + cgmath::One
        + std::cmp::PartialEq
        + crate::convert::Cast<f32>;
    type Container: ArrayBuffer<Item = Self::Item>;

    const BLACK: Self;

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue>;
}

impl Pixel for [u8; 4] {
    type Item = u8;
    type Container = ArrayU8;
    // Transparency handled
    const BLACK: Self = [0, 0, 0, 0];

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
        Ok([pixels[0], pixels[1], pixels[2]])
    }
}
impl Pixel for [u8; 2] {
    type Item = u8;
    type Container = ArrayU8;
    const BLACK: Self = [0, 0];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let pixels = js_sys::Uint8Array::new_with_length(2);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RG,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&pixels),
        )?;
        let pixels = pixels.to_vec();
        Ok([pixels[0], pixels[1]])
    }
}

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

impl Pixel for [i16; 1] {
    type Item = i16;
    type Container = ArrayI16;
    const BLACK: Self = [i16::MIN];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let p = js_sys::Uint8Array::new_with_length(2);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RG,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&p),
        )?;

        Ok([i16::from_le_bytes([p.at(1).unwrap(), p.at(0).unwrap()])])
    }
}

impl Pixel for [i32; 1] {
    type Item = i32;
    type Container = ArrayI32;
    const BLACK: Self = [i32::MIN];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let p = js_sys::Uint8Array::new_with_length(4);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGBA,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&p),
        )?;

        Ok([i32::from_le_bytes([
            p.at(3).unwrap(),
            p.at(2).unwrap(),
            p.at(1).unwrap(),
            p.at(0).unwrap(),
        ])])
    }
}

impl Pixel for [f32; 1] {
    type Item = f32;
    type Container = ArrayF32;
    const BLACK: Self = [f32::NAN];

    fn read_pixel(gl: &WebGlContext, x: i32, y: i32) -> Result<Self, JsValue> {
        let p = js_sys::Uint8Array::new_with_length(4);
        gl.read_pixels_with_opt_array_buffer_view(
            x,
            y,
            1,
            1,
            WebGlRenderingCtx::RGBA,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            Some(&p),
        )?;

        Ok([f32::from_le_bytes([
            p.at(3).unwrap(),
            p.at(2).unwrap(),
            p.at(1).unwrap(),
            p.at(0).unwrap(),
        ])])
    }
}
