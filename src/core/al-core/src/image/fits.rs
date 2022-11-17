use cgmath::{Vector2, Vector3};

#[derive(Debug)]
pub struct Fits<'a> {
    // Tile size
    size: Vector2<i32>,

    pub data: Data<'a>,
}

use std::borrow::Cow;
use std::io::BufReader;
use std::io::Read;
use std::fmt::Debug;
#[derive(Debug)]
pub enum Data<'a> {
    U8(Cow<'a, [u8]>),
    I16(Cow<'a, [i16]>),
    I32(Cow<'a, [i32]>),
    F32(Cow<'a, [f32]>),
}
use fitsrs::{hdu::{DataRead, AsyncHDU, HDU, data_async::DataOwnedSt}, fits::AsyncFits, card::Value};
use wasm_streams::readable::IntoAsyncRead;
use futures::stream::StreamExt;
use fitsrs::hdu::Header;
impl<'a> Fits<'a> {
    pub fn from_byte_slice(bytes: &'a [u8]) -> Result<Self, JsValue> {
        let fitsrs::fits::Fits { hdu: HDU { data, header } } = fitsrs::fits::Fits::from_reader(bytes)
            .map_err(|err| {
                JsValue::from_str(&"Parsing fits error")
            })?;

        let width = header.get_axis_size(1)
            .ok_or_else(|| JsValue::from_str("NAXIS1 not found in the fits"))?;

        let height = header.get_axis_size(2)
            .ok_or_else(|| JsValue::from_str("NAXIS2 not found in the fits"))?;

        let data = match data {
            fitsrs::hdu::data::DataBorrowed::U8(slice) => {
                Data::U8(Cow::Borrowed(slice))
            },
            fitsrs::hdu::data::DataBorrowed::I16(slice) => {
                Data::I16(Cow::Borrowed(slice))
            },
            fitsrs::hdu::data::DataBorrowed::I32(slice) => {
                Data::I32(Cow::Borrowed(slice))
            },
            fitsrs::hdu::data::DataBorrowed::I64(slice) => {
                let data = slice.iter().map(|v| *v as i32).collect();
                Data::I32(Cow::Owned(data))
            },
            fitsrs::hdu::data::DataBorrowed::F32(slice) => {
                Data::F32(Cow::Borrowed(slice))
            },
            fitsrs::hdu::data::DataBorrowed::F64(slice) => {
                let data = slice.iter().map(|v| *v as f32).collect();
                Data::F32(Cow::Owned(data))
            }
        };

        Ok(Self {
            // Tile size
            size: Vector2::new(*width as i32, *height as i32),

            // Allocation info of the layout            
            data
        })
    }

    pub fn from_reader<R>(reader: BufReader<R>) -> Result<Self, JsValue>
    where
        R: Read + Debug
    {
        let fitsrs::fits::Fits { hdu: HDU { data, header } } = fitsrs::fits::Fits::from_reader(reader)
            .map_err(|err| {
                JsValue::from_str(&"Parsing fits error")
            })?;

        let width = header.get_axis_size(1)
            .ok_or_else(|| JsValue::from_str("NAXIS1 not found in the fits"))?;

        let height = header.get_axis_size(2)
            .ok_or_else(|| JsValue::from_str("NAXIS2 not found in the fits"))?;

        let data = match data {
            fitsrs::hdu::data::DataOwned::U8(it) => {
                Data::U8(Cow::Owned(it.collect()))
            },
            fitsrs::hdu::data::DataOwned::I16(it) => {
                Data::I16(Cow::Owned(it.collect()))
            },
            fitsrs::hdu::data::DataOwned::I32(it) => {
                Data::I32(Cow::Owned(it.collect()))
            },
            fitsrs::hdu::data::DataOwned::I64(it) => {
                let data = it.map(|v| v as i32).collect();
                Data::I32(Cow::Owned(data))
            },
            fitsrs::hdu::data::DataOwned::F32(it) => {
                Data::F32(Cow::Owned(it.collect()))
            },
            fitsrs::hdu::data::DataOwned::F64(it) => {
                let data = it.map(|v| v as f32).collect();
                Data::F32(Cow::Owned(data))
            }
        };

        Ok(Self {
            // Tile size
            size: Vector2::new(*width as i32, *height as i32),

            // Allocation info of the layout            
            data
        })
    }

    pub fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

impl Fits<'static> {
    pub async fn from_async_reader(reader: IntoAsyncRead<'static>) -> Result<Self, JsValue> {
        let fitsrs::fits::AsyncFits { hdu: AsyncHDU { data, header } } = fitsrs::fits::AsyncFits::from_reader(futures::io::BufReader::new(reader))
            .await
            .map_err(|err| {
                JsValue::from_str(&format!("Parsing fits error: {}", err))
            })?;

        let width = header.get_axis_size(1)
            .ok_or_else(|| JsValue::from_str("NAXIS1 not found in the fits"))?;

        let height = header.get_axis_size(2)
            .ok_or_else(|| JsValue::from_str("NAXIS2 not found in the fits"))?;

        let data = match data {
            fitsrs::hdu::data_async::DataOwned::U8(stream) => {
                let data = stream.collect().await;
                Data::U8(Cow::Owned(data))
            },
            fitsrs::hdu::data_async::DataOwned::I16(stream) => {
                let data = stream.collect().await;
                Data::I16(Cow::Owned(data))
            },
            fitsrs::hdu::data_async::DataOwned::I32(stream) => {
                let data = stream.collect().await;
                Data::I32(Cow::Owned(data))
            },
            fitsrs::hdu::data_async::DataOwned::I64(stream) => {
                let data = stream.map(|v| v as i32).collect().await;
                Data::I32(Cow::Owned(data))
            },
            fitsrs::hdu::data_async::DataOwned::F32(stream) => {
                let data = stream.collect().await;
                Data::F32(Cow::Owned(data))
            },
            fitsrs::hdu::data_async::DataOwned::F64(stream) => {
                let data = stream.map(|v| v as f32).collect().await;
                Data::F32(Cow::Owned(data))
            }
        };

        Ok(Self {
            // Tile size
            size: Vector2::new(*width as i32, *height as i32),

            // Allocation info of the layout            
            data
        })
    }
}
use crate::Texture2DArray;
use crate::image::Image;
impl Image for Fits<'_> {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        match &self.data {
            Data::U8(data) => { 
                let view = unsafe { R8UI::view(&data) };
                textures[offset.z as usize]
                    .bind()
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                        offset.x,
                        offset.y,
                        self.size.x,
                        self.size.y,
                        Some(view.as_ref()),
                    );
            }
            Data::I16(data) => { 
                let view = unsafe { R16I::view(&data) };
                textures[offset.z as usize]
                    .bind()
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                        offset.x,
                        offset.y,
                        self.size.x,
                        self.size.y,
                        Some(view.as_ref()),
                    );
            }
            Data::I32(data) => { 
                let view = unsafe { R32I::view(&data) };
                textures[offset.z as usize]
                    .bind()
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                        offset.x,
                        offset.y,
                        self.size.x,
                        self.size.y,
                        Some(view.as_ref()),
                    );
            }
            Data::F32(data) => { 
                let view = unsafe { R32F::view(&data) };
                textures[offset.z as usize]
                    .bind()
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                        offset.x,
                        offset.y,
                        self.size.x,
                        self.size.y,
                        Some(view.as_ref()),
                    );
            }
        }
    }
}




use wasm_bindgen::JsValue;

use crate::image::format::ImageFormat;

pub trait FitsImageFormat: ImageFormat {
    type Type: Clone;
    type ArrayBufferView: AsRef<js_sys::Object>;

    const BITPIX: i8;
    /// Creates a JS typed array which is a view into wasm's linear memory at the slice specified.
    /// This function returns a new typed array which is a view into wasm's memory. This view does not copy the underlying data.
    ///
    /// # Safety
    ///
    /// Views into WebAssembly memory are only valid so long as the backing buffer isn't resized in JS. Once this function is called any future calls to Box::new (or malloc of any form) may cause the returned value here to be invalidated. Use with caution!
    ///
    /// Additionally the returned object can be safely mutated but the input slice isn't guaranteed to be mutable.
    ///
    /// Finally, the returned object is disconnected from the input slice's lifetime, so there's no guarantee that the data is read at the right time.
    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView;
}

use crate::image::R32F;
impl FitsImageFormat for R32F {
    const BITPIX: i8 = -32;
    type Type = f32;
    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
use crate::image::{R16I, R32I, R8UI, R64F};
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R64F {
    const BITPIX: i8 = -64;
    type Type = f64;

    type ArrayBufferView = js_sys::Float64Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
impl FitsImageFormat for R32I {
    const BITPIX: i8 = 32;
    type Type = i32;

    type ArrayBufferView = js_sys::Int32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R16I {
    const BITPIX: i8 = 16;
    type Type = i16;

    type ArrayBufferView = js_sys::Int16Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R8UI {
    const BITPIX: i8 = 8;
    type Type = u8;

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
