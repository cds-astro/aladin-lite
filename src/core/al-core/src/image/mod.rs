pub mod bitmap;
pub mod fits;
pub mod format;
pub mod html;
pub mod canvas;
pub mod raw;

pub trait ArrayBuffer: AsRef<js_sys::Object> + std::fmt::Debug {
    type Item: std::cmp::PartialOrd + Clone + Copy + std::fmt::Debug + cgmath::Zero;

    fn new(buf: &[Self::Item]) -> Self;
    fn empty(size: u32, blank_value: Self::Item) -> Self;

    fn to_vec(&self) -> Vec<Self::Item>;

    fn set_index(&self, idx: u32, value: Self::Item);
    fn get(&self, idx: u32) -> Self::Item;
}
#[derive(Debug)]
pub struct ArrayU8(js_sys::Uint8Array);
impl AsRef<js_sys::Object> for ArrayU8 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayU8 {
    type Item = u8;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayU8(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let uint8_arr = js_sys::Uint8Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayU8(uint8_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }

    fn set_index(&self, idx: u32, value: Self::Item) {
        self.0.set_index(idx, value);
    }

    fn get(&self, idx: u32) -> Self::Item {
        self.0.get_index(idx)
    }
}
#[derive(Debug)]
pub struct ArrayI16(js_sys::Int16Array);
impl AsRef<js_sys::Object> for ArrayI16 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayI16 {
    type Item = i16;
    fn new(buf: &[Self::Item]) -> Self {
        ArrayI16(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let int16_arr = js_sys::Int16Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayI16(int16_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }

    fn set_index(&self, idx: u32, value: Self::Item) {
        self.0.set_index(idx, value);
    }

    fn get(&self, idx: u32) -> Self::Item {
        self.0.get_index(idx)
    }
}
#[derive(Debug)]
pub struct ArrayI32(js_sys::Int32Array);
impl AsRef<js_sys::Object> for ArrayI32 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}
impl ArrayBuffer for ArrayI32 {
    type Item = i32;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayI32(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let int32_arr = js_sys::Int32Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayI32(int32_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }

    fn set_index(&self, idx: u32, value: Self::Item) {
        self.0.set_index(idx, value);
    }

    fn get(&self, idx: u32) -> Self::Item {
        self.0.get_index(idx)
    }
}
#[derive(Debug)]
pub struct ArrayF32(js_sys::Float32Array);
impl AsRef<js_sys::Object> for ArrayF32 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayF32 {
    type Item = f32;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayF32(buf.into())
    }
    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let f32_arr = js_sys::Float32Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayF32(f32_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }

    fn set_index(&self, idx: u32, value: Self::Item) {
        self.0.set_index(idx, value);
    }

    fn get(&self, idx: u32) -> Self::Item {
        self.0.get_index(idx)
    }
}

#[derive(Debug)]
pub struct ArrayF64(js_sys::Float64Array);
impl AsRef<js_sys::Object> for ArrayF64 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayF64 {
    type Item = f64;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayF64(buf.into())
    }
    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let f64_arr = js_sys::Float64Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayF64(f64_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }

    fn set_index(&self, idx: u32, value: Self::Item) {
        self.0.set_index(idx, value);
    }

    fn get(&self, idx: u32) -> Self::Item {
        self.0.get_index(idx)
    }
}

use self::html::HTMLImage;
use self::canvas::Canvas;
use wasm_bindgen::JsValue;
use super::Texture2DArray;
pub trait Image {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue>;
}

impl<'a, I> Image for &'a I
where
    I: Image,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let image = &**self;
        image.tex_sub_image_3d(textures, offset)?;

        Ok(())
    }
}

use std::rc::Rc;
impl<I> Image for Rc<I>
where
    I: Image,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let image = &**self;
        image.tex_sub_image_3d(textures, offset)?;

        Ok(())
    }
}
use crate::Abort;

use std::sync::{Arc, Mutex};
impl<I> Image for Arc<Mutex<Option<I>>>
where
    I: Image,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        if let Some(image) = &*self.lock().unwrap_abort() {
            image.tex_sub_image_3d(textures, offset)?;
        }

        Ok(())
    }
}

#[cfg(feature = "webgl2")]
use crate::image::format::{R16I, R32I, R8UI, R64F};
use crate::image::format::{R32F, RGB8U, RGBA8U};

use bitmap::Bitmap;
use fits::Fits;
use raw::ImageBuffer;
#[derive(Debug)]
#[cfg(feature = "webgl2")]
pub enum ImageType {
    FitsImage { raw_bytes: js_sys::Uint8Array },
    Canvas { canvas: Canvas<RGBA8U> },
    PngImageRgba8u { image: Bitmap<RGBA8U> },
    JpgImageRgb8u { image: Bitmap<RGB8U> },
    PngHTMLImageRgba8u { image: HTMLImage<RGBA8U> },
    JpgHTMLImageRgb8u { image: HTMLImage<RGB8U> },
    RawRgb8u { image: ImageBuffer<RGB8U> },
    RawRgba8u { image: ImageBuffer<RGBA8U> },
    RawR32f { image: ImageBuffer<R32F> },
    RawR32i { image: ImageBuffer<R32I> },
    RawR16i { image: ImageBuffer<R16I> },
    RawR8ui { image: ImageBuffer<R8UI> },
}

#[cfg(feature = "webgl1")]
pub enum ImageType {
    FitsImage { raw_bytes: js_sys::Uint8Array },
    Canvas { canvas: Canvas<RGBA8U> },
    PngHTMLImageRgba8u { image: HTMLImage<RGBA8U> },
    JpgHTMLImageRgb8u { image: HTMLImage<RGB8U> },
    PngImageRgba8u { image: Bitmap<RGBA8U> },
    JpgImageRgb8u { image: Bitmap<RGB8U> },
    RawRgb8u { image: ImageBuffer<RGB8U> },
    RawRgba8u { image: ImageBuffer<RGBA8U> },
    RawR32f { image: ImageBuffer<R32F> },
}

use cgmath::Vector3;
impl Image for ImageType {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        match self {
            ImageType::FitsImage { raw_bytes: raw_bytes_buf } => {
                let num_bytes = raw_bytes_buf.length() as usize;
                let mut raw_bytes = Vec::with_capacity(num_bytes);
                unsafe { raw_bytes.set_len(num_bytes); }
                raw_bytes_buf.copy_to(&mut raw_bytes[..]);

                let fits_img = Fits::from_byte_slice(raw_bytes.as_slice())?;
                fits_img.tex_sub_image_3d(textures, offset)?
            },
            ImageType::Canvas { canvas } => canvas.tex_sub_image_3d(textures, offset)?,
            ImageType::PngImageRgba8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::JpgImageRgb8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::PngHTMLImageRgba8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::JpgHTMLImageRgb8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawRgb8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawRgba8u { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawR32f { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawR32i { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawR16i { image } => image.tex_sub_image_3d(textures, offset)?,
            ImageType::RawR8ui { image } => image.tex_sub_image_3d(textures, offset)?,
        }

        Ok(())
    }
}