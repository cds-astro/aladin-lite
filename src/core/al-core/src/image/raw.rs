use crate::image::format::ImageFormat;
use crate::texture::pixel::Pixel;
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageBuffer<T>
where
    T: ImageFormat,
{
    pub data: Vec<<<T as ImageFormat>::P as Pixel>::Item>,
    pub size: Vector2<i32>,
}

use crate::image::format::Bytes;

pub struct ImageBufferView {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
use wasm_bindgen::JsValue;
impl<T> ImageBuffer<T>
where
    T: ImageFormat,
{
    pub fn new(data: Vec<<<T as ImageFormat>::P as Pixel>::Item>, width: i32, height: i32) -> Self {
        let size_buf = width * height * (T::NUM_CHANNELS as i32);
        debug_assert!(size_buf == data.len() as i32);
        //let buf = <<T as ImageFormat>::P as Pixel>::Container::new(buf);
        let size = Vector2::new(width, height);
        Self { data, size }
    }

    pub fn from_encoded_raw_bytes(
        raw_bytes: &[u8],
        width: i32,
        height: i32,
    ) -> Result<Self, JsValue> {
        let mut decoded_bytes = match T::decode(raw_bytes).map_err(|e| JsValue::from_str(e))? {
            Bytes::Borrowed(bytes) => bytes.to_vec(),
            Bytes::Owned(bytes) => bytes,
        };

        let decoded_pixels = unsafe {
            decoded_bytes.set_len(
                decoded_bytes.len() / std::mem::size_of::<<<T as ImageFormat>::P as Pixel>::Item>(),
            );
            std::mem::transmute(decoded_bytes)
        };

        Ok(Self::new(decoded_pixels, width, height))
    }

    pub fn from_raw_bytes(mut raw_bytes: Vec<u8>, width: i32, height: i32) -> Self {
        let size_buf = width * height * (std::mem::size_of::<T::P>() as i32);
        debug_assert!(size_buf == raw_bytes.len() as i32);

        let decoded_pixels = unsafe {
            raw_bytes.set_len(
                raw_bytes.len() / std::mem::size_of::<<<T as ImageFormat>::P as Pixel>::Item>(),
            );
            std::mem::transmute(raw_bytes)
        };

        Self::new(decoded_pixels, width, height)
    }

    pub fn empty() -> Self {
        let size = Vector2::new(0, 0);
        Self { data: vec![], size }
    }

    pub fn allocate(pixel_fill: &<T as ImageFormat>::P, width: i32, height: i32) -> ImageBuffer<T> {
        let size_buf = ((width * height) as usize) * (T::NUM_CHANNELS);

        let data = pixel_fill
            .as_ref()
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        ImageBuffer::<T>::new(data, width, height)
    }

    pub fn tex_sub(&mut self, src: &Self, s: &ImageBufferView, d: &ImageBufferView) {
        let mut di = d.x;
        let mut dj = d.y;

        for ix in s.x..(s.x + s.w) {
            for iy in s.y..(s.y + s.h) {
                let s_idx = (iy * src.width() + ix) as usize;
                let d_idx = (di * self.width() + dj) as usize;

                for i in 0..T::NUM_CHANNELS {
                    let si = s_idx * T::NUM_CHANNELS + i;
                    let di = d_idx * T::NUM_CHANNELS + i;
                    let value = src.data[si];
                    self.data[di] = value;
                }

                di += 1;
                if di >= d.x + d.w {
                    di = d.x;
                    dj += 1;
                }
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &<<T as ImageFormat>::P as Pixel>::Item> {
        self.data.iter()
    }

    pub fn get_data(&self) -> &[<<T as ImageFormat>::P as Pixel>::Item] {
        &self.data
    }

    pub fn width(&self) -> i32 {
        self.size.x
    }

    pub fn height(&self) -> i32 {
        self.size.y
    }
}

use crate::image::format::{R16I, R32F, R32I, R8UI, RGB8U, RGBA8U};
pub enum ImageBufferType {
    JPG(ImageBuffer<RGB8U>),
    PNG(ImageBuffer<RGBA8U>),
    R32F(ImageBuffer<R32F>),
    R8UI(ImageBuffer<R8UI>),
    R16I(ImageBuffer<R16I>),
    R32I(ImageBuffer<R32I>),
}

use crate::image::{ArrayBuffer, Image};
use crate::Texture2DArray;
use cgmath::{Vector2, Vector3};
impl<I> Image for ImageBuffer<I>
where
    I: ImageFormat,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let js_array =
            <<<I as ImageFormat>::P as Pixel>::Container as ArrayBuffer>::new(&self.data);
        textures.bind().tex_sub_image_3d_with_opt_array_buffer_view(
            offset.z,
            offset.x,
            offset.y,
            self.width(),
            self.height(),
            Some(js_array.as_ref()),
        );

        Ok(())
    }

    // The size of the image
    /*fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }*/
}
