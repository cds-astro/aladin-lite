use cgmath::{Vector2, Vector3};

pub trait ArrayBuffer: AsRef<js_sys::Object> + std::fmt::Debug {
    type Item: std::cmp::PartialOrd + Clone + Copy + std::fmt::Debug + cgmath::Zero;

    fn new(buf: &[Self::Item]) -> Self;
    fn empty(size: u32, blank_value: Self::Item) -> Self;

    fn to_vec(&self) -> Vec<Self::Item>;
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
}

use super::format::ImageFormat;
use super::pixel::Pixel;
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageBuffer<T>
where
    T: ImageFormat,
{
    buf: <<T as ImageFormat>::P as Pixel>::Container,
    size: Vector2<i32>,
    format: std::marker::PhantomData<T>,
}

impl<T> ImageBuffer<T>
where
    T: ImageFormat,
{
    pub fn new(buf: &[<<T as ImageFormat>::P as Pixel>::Item], width: i32) -> Self {
        let size_buf = width * width * (T::NUM_CHANNELS as i32);
        assert_eq!(size_buf, buf.len() as i32);
        let buf = <<T as ImageFormat>::P as Pixel>::Container::new(buf);
        let size = Vector2::new(width, width);
        Self {
            buf,
            size,
            format: std::marker::PhantomData,
        }
    }
}

use super::Texture2DArray;
pub trait Image {
    type T: ImageFormat;

    fn allocate(width: i32, pixel_fill: &<<Self as Image>::T as ImageFormat>::P) -> Self;

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
        size: &Vector2<i32>,
    );

    // The size of the image
    fn get_size(&self) -> &Vector2<i32>;
}
use std::rc::Rc;
impl<I> Image for Rc<I>
where
    I: Image,
{
    type T = I::T;

    fn allocate(width: i32, pixel_fill: &<<Self as Image>::T as ImageFormat>::P) -> Self {
        Rc::new(I::allocate(width, pixel_fill))
    }

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
        size: &Vector2<i32>,
    ) {
        let image = &**self;
        image.tex_sub_image_3d(textures, offset, size);
    }

    fn get_size(&self) -> &Vector2<i32> {
        let image = &**self;
        image.get_size()
    }
}

impl<I> Image for ImageBuffer<I>
where
    I: ImageFormat,
{
    type T = I;

    fn allocate(
        width: i32,
        pixel_fill: &<<Self as Image>::T as ImageFormat>::P,
    ) -> ImageBuffer<Self::T> {
        let size_buf = (width * width * (Self::T::NUM_CHANNELS as i32)) as usize;

        let pixels = pixel_fill
            .as_ref()
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        ImageBuffer::<Self::T>::new(&pixels[..], width)
    }

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
        size: &Vector2<i32>,
    ) {
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                offset.x,
                offset.y,
                size.x,
                size.y,
                Some(self.buf.as_ref()),
            );
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}
