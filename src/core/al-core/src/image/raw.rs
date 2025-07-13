use crate::texture::format::TextureFormat;

use crate::texture::pixel::Pixel;
use crate::texture::Tex3D;
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageBuffer<T>
where
    T: TextureFormat,
{
    pub data: Box<[<<T as TextureFormat>::P as Pixel>::Item]>,
    pub size: (u32, u32, u32),
}

use crate::texture::format::Bytes;

pub struct ImageBufferView {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
use wasm_bindgen::JsValue;
impl<T> ImageBuffer<T>
where
    T: TextureFormat,
{
    pub fn new(
        data: Box<[<<T as TextureFormat>::P as Pixel>::Item]>,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Self {
        let size_buf = width * height * depth * (T::NUM_CHANNELS as u32);
        debug_assert!(size_buf == data.len() as u32);
        //let buf = <<T as ImageFormat>::P as Pixel>::Container::new(buf);
        let size = (width, height, depth);
        Self { data, size }
    }

    pub fn from_encoded_raw_bytes(
        raw_bytes: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Self, JsValue> {
        let mut decoded_bytes = match T::decode(raw_bytes).map_err(JsValue::from_str)? {
            Bytes::Borrowed(bytes) => bytes.to_vec(),
            Bytes::Owned(bytes) => bytes,
        };

        let decoded_pixels = unsafe {
            decoded_bytes.set_len(
                decoded_bytes.len()
                    / std::mem::size_of::<<<T as TextureFormat>::P as Pixel>::Item>(),
            );
            std::mem::transmute::<Vec<u8>, Vec<<<T as TextureFormat>::P as Pixel>::Item>>(
                decoded_bytes,
            )
            .into_boxed_slice()
        };

        Ok(Self::new(decoded_pixels, width, height, 1))
    }

    pub fn from_raw_bytes(mut raw_bytes: Vec<u8>, width: u32, height: u32) -> Self {
        let size_buf = width * height * (std::mem::size_of::<T::P>() as u32);
        debug_assert!(size_buf == raw_bytes.len() as u32);

        let decoded_pixels = unsafe {
            raw_bytes.set_len(raw_bytes.len() / std::mem::size_of::<<T::P as Pixel>::Item>());
            std::mem::transmute::<Vec<u8>, Vec<<T::P as Pixel>::Item>>(raw_bytes).into_boxed_slice()
        };

        Self::new(decoded_pixels, width, height, 1)
    }

    pub fn empty() -> Self {
        let size = (0, 0, 0);
        Self {
            data: Box::new([]),
            size,
        }
    }

    pub fn allocate(pixel_fill: &T::P, width: u32, height: u32) -> ImageBuffer<T> {
        let size_buf = ((width * height) as usize) * (T::NUM_CHANNELS);

        let data = pixel_fill
            .as_ref()
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        ImageBuffer::<T>::new(data, width, height, 1)
    }

    pub fn tex_sub(&mut self, src: &Self, s: &ImageBufferView, d: &ImageBufferView) {
        let mut di = d.x;
        let mut dj = d.y;

        for ix in s.x..(s.x + s.w) {
            for iy in s.y..(s.y + s.h) {
                let s_idx = ((iy * src.width() as i32) + ix) as usize;
                let d_idx = ((di * self.width() as i32) + dj) as usize;

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

    pub fn iter(&self) -> impl Iterator<Item = &<T::P as Pixel>::Item> {
        self.data.iter()
    }

    pub fn get_data(&self) -> &[<T::P as Pixel>::Item] {
        &self.data
    }

    pub fn width(&self) -> u32 {
        self.size.0
    }

    pub fn height(&self) -> u32 {
        self.size.1
    }
}

use crate::texture::format::{R16I, R32F, R32I, R8U, RGB8U, RGBA8U};
pub enum ImageBufferType {
    JPG(ImageBuffer<RGB8U>),
    PNG(ImageBuffer<RGBA8U>),
    R32F(ImageBuffer<R32F>),
    R8UI(ImageBuffer<R8U>),
    R16I(ImageBuffer<R16I>),
    R32I(ImageBuffer<R32I>),
}

use crate::image::{ArrayBuffer, Image};
use cgmath::Vector3;
impl<I> Image for ImageBuffer<I>
where
    I: TextureFormat,
{
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        // The texture array
        textures: &T,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let js_array = <<I::P as Pixel>::Container as ArrayBuffer>::new(&self.data);
        textures.tex_sub_image_3d_with_opt_array_buffer_view(
            offset.x,
            offset.y,
            offset.z,
            self.width() as i32,
            self.height() as i32,
            self.size.2 as i32,
            Some(js_array.as_ref()),
        );

        Ok(())
    }

    // The size of the image
    fn get_size(&self) -> (u32, u32, u32) {
        self.size
    }
}
