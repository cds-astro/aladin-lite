use crate::image::format::ImageFormat;
use crate::texture::pixel::Pixel;
#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageBuffer<T>
where
    T: ImageFormat,
{
    data: Vec<<<T as ImageFormat>::P as Pixel>::Item>,
    size: Vector2<i32>,
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

    pub fn from_raw_bytes(raw_bytes: &[u8], width: i32, height: i32) -> Result<Self, JsValue> {
        let format = <T as ImageFormat>::IMAGE_DECODER_TYPE.ok_or(JsValue::from_str(
            "Format not supported. This image may not be compressed.",
        ))?;
        let mut decoded_bytes = image_decoder::load_from_memory_with_format(raw_bytes, format)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
            .into_bytes();

        let decoded_bytes = unsafe {
            decoded_bytes.set_len(
                decoded_bytes.len() / std::mem::size_of::<<<T as ImageFormat>::P as Pixel>::Item>(),
            );
            std::mem::transmute(decoded_bytes)
        };
        Ok(Self::new(decoded_bytes, width, height))
    }

    pub fn empty() -> Self {
        let size = Vector2::new(0, 0);
        Self { data: vec![], size }
    }

    pub fn allocate(pixel_fill: &<T as ImageFormat>::P, width: i32, height: i32) -> ImageBuffer<T> {
        let size_buf = ((width * height) as usize) * (T::NUM_CHANNELS);

        let mut data = pixel_fill
            .as_ref()
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        ImageBuffer::<T>::new(data, width, height)
    }

    pub fn tex_sub(
        &mut self,
        src: &Self,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
    ) {
        let mut di = dx;
        let mut dj = dy;

        for ix in sx..(sx + sw) {
            for iy in sy..(sy + sh) {
                let s_idx = (iy * src.width() + ix) as usize;
                let d_idx = (di * self.width() + dj) as usize;

                for i in 0..T::NUM_CHANNELS {
                    let si = s_idx * T::NUM_CHANNELS + i;
                    let di = d_idx * T::NUM_CHANNELS + i;
                    let value = src.data[si];
                    self.data[di] = value;
                }

                di += 1;
                if di >= dx + dw {
                    di = dx;
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
    ) {
        let js_array =
            <<<I as ImageFormat>::P as Pixel>::Container as ArrayBuffer>::new(&self.data);
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                offset.x,
                offset.y,
                self.size.x,
                self.size.y,
                Some(js_array.as_ref()),
            );
    }

    // The size of the image
    /*fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }*/
}
