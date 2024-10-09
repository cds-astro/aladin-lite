use cgmath::{Vector2, Vector3};

#[derive(Debug)]
pub struct Fits<'a> {
    // Tile size
    size: Vector2<i32>,

    pub data: Data<'a>,
}

use std::borrow::Cow;
use std::fmt::Debug;
#[derive(Debug)]
pub enum Data<'a> {
    U8(Cow<'a, [u8]>),
    I16(Cow<'a, [i16]>),
    I32(Cow<'a, [i32]>),
    F32(Cow<'a, [f32]>),
}
use fitsrs::{fits::Fits as FitsData, hdu::data::InMemData};
use std::io::Cursor;

impl<'a> Fits<'a> {
    pub fn from_byte_slice(bytes_reader: &'a mut Cursor<&[u8]>) -> Result<Self, JsValue> {
        let FitsData { hdu } = FitsData::from_reader(bytes_reader)
            .map_err(|_| JsValue::from_str(&"Parsing fits error"))?;

        let header = hdu.get_header();
        let xtension = header.get_xtension();
        let width = xtension
            .get_naxisn(1)
            .ok_or_else(|| JsValue::from_str("NAXIS1 not found in the fits"))?;

        let height = xtension
            .get_naxisn(2)
            .ok_or_else(|| JsValue::from_str("NAXIS2 not found in the fits"))?;

        let data = hdu.get_data();
        let data = match *data {
            InMemData::U8(slice) => Data::U8(Cow::Borrowed(slice)),
            InMemData::I16(slice) => Data::I16(Cow::Borrowed(slice)),
            InMemData::I32(slice) => Data::I32(Cow::Borrowed(slice)),
            InMemData::I64(slice) => {
                let data = slice.iter().map(|v| *v as i32).collect();
                Data::I32(Cow::Owned(data))
            }
            InMemData::F32(slice) => Data::F32(Cow::Borrowed(slice)),
            InMemData::F64(slice) => {
                let data = slice.iter().map(|v| *v as f32).collect();
                Data::F32(Cow::Owned(data))
            }
        };

        Ok(Self {
            // Tile size
            size: Vector2::new(*width as i32, *height as i32),

            // Allocation info of the layout
            data,
        })
    }

    pub fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

/*impl Fits<'static> {
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
}*/

use crate::image::Image;
use crate::Texture2DArray;
impl Image for Fits<'_> {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        match &self.data {
            Data::U8(data) => {
                let view = unsafe { R8UI::view(&data) };
                textures.bind().tex_sub_image_3d_with_opt_array_buffer_view(
                    offset.z,
                    offset.x,
                    offset.y,
                    self.size.x,
                    self.size.y,
                    Some(view.as_ref()),
                );
            }
            Data::I16(data) => {
                let view = unsafe { R16I::view(&data) };
                textures.bind().tex_sub_image_3d_with_opt_array_buffer_view(
                    offset.z,
                    offset.x,
                    offset.y,
                    self.size.x,
                    self.size.y,
                    Some(view.as_ref()),
                );
            }
            Data::I32(data) => {
                let view = unsafe { R32I::view(&data) };
                textures.bind().tex_sub_image_3d_with_opt_array_buffer_view(
                    offset.z,
                    offset.x,
                    offset.y,
                    self.size.x,
                    self.size.y,
                    Some(view.as_ref()),
                );
            }
            Data::F32(data) => {
                let view = unsafe { R32F::view(&data) };
                textures.bind().tex_sub_image_3d_with_opt_array_buffer_view(
                    offset.z,
                    offset.x,
                    offset.y,
                    self.size.x,
                    self.size.y,
                    Some(view.as_ref()),
                );
            }
        }

        Ok(())
    }
}

use crate::image::format::ImageFormat;
use wasm_bindgen::JsValue;

pub trait FitsImageFormat: ImageFormat {
    const BITPIX: i8;
}

use crate::image::R32F;
impl FitsImageFormat for R32F {
    const BITPIX: i8 = -32;
}

#[cfg(feature = "webgl2")]
use crate::image::{R16I, R32I, R64F, R8UI};
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R64F {
    const BITPIX: i8 = -64;
}

#[cfg(feature = "webgl2")]
impl FitsImageFormat for R32I {
    const BITPIX: i8 = 32;
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R16I {
    const BITPIX: i8 = 16;
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R8UI {
    const BITPIX: i8 = 8;
}
