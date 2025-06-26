use crate::texture::format::TextureFormat;
use crate::texture::format::R8U;
use cgmath::Vector3;
use fitsrs::card::Value;
use fitsrs::gz::GzReader;
use fitsrs::hdu::header::Bitpix;
use fitsrs::WCS;
use fitsrs::{Fits, HDU};
use std::fmt::Debug;
use std::io::Cursor;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct FitsImage<'a> {
    // image size
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    // bitpix
    pub bitpix: Bitpix,
    // 1.0 by default
    pub bscale: f32,
    // 0.0 by default
    pub bzero: f32,
    // blank
    pub blank: Option<f32>,
    // optional wcs
    pub wcs: Option<WCS>,
    // raw bytes of the data image (in Big-Endian)
    pub raw_bytes: &'a [u8],
}

impl<'a> FitsImage<'a> {
    /// Get all the hdu images from a fits file
    pub fn from_raw_bytes(bytes: &'a [u8]) -> Result<Vec<Self>, JsValue> {
        let mut fits = Fits::from_reader(Cursor::new(bytes));
        let mut images = vec![];

        while let Some(Ok(hdu)) = fits.next() {
            match hdu {
                HDU::XImage(hdu) | HDU::Primary(hdu) => {
                    // Prefer getting the dimension directly from NAXIS1/NAXIS2 instead of from the WCS
                    // because it may not exist in all HDU images
                    let width = hdu.get_header().get_xtension().get_naxisn(1);
                    let height = hdu.get_header().get_xtension().get_naxisn(2);

                    if let (Some(&width), Some(&height)) = (width, height) {
                        let depth =
                            *hdu.get_header().get_xtension().get_naxisn(3).unwrap_or(&1) as u32;

                        let header = hdu.get_header();

                        let bscale = match header.get("BSCALE") {
                            Some(Value::Integer { value, .. }) => *value as f32,
                            Some(Value::Float { value, .. }) => *value as f32,
                            _ => 1.0,
                        };

                        let bzero = match header.get("BZERO") {
                            Some(Value::Integer { value, .. }) => *value as f32,
                            Some(Value::Float { value, .. }) => *value as f32,
                            _ => 0.0,
                        };

                        let blank = match header.get("BLANK") {
                            Some(Value::Integer { value, .. }) => Some(*value as f32),
                            Some(Value::Float { value, .. }) => Some(*value as f32),
                            _ => None,
                        };

                        let off = hdu.get_data_unit_byte_offset() as usize;
                        let len = hdu.get_data_unit_byte_size() as usize;

                        let raw_bytes = &bytes[off..(off + len)];

                        let bitpix = hdu.get_header().get_xtension().get_bitpix();

                        let wcs = hdu.wcs().ok();

                        images.push(Self {
                            width: width as u32,
                            height: height as u32,
                            depth,
                            bitpix,
                            bscale,
                            wcs,
                            bzero,
                            blank,
                            raw_bytes,
                        });
                    }
                }
                _ => (),
            }
        }

        if !images.is_empty() {
            Ok(images)
        } else {
            Err(JsValue::from_str("Image HDU not found in the FITS"))
        }
    }
}

use crate::{image::Image, texture::Tex3D};
impl Image for FitsImage<'_> {
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        // The texture array
        textures: &T,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let view = unsafe { R8U::view(self.raw_bytes) };
        textures.tex_sub_image_3d_with_opt_array_buffer_view(
            offset.x,
            offset.y,
            offset.z,
            self.width as i32,
            self.height as i32,
            self.depth as i32,
            Some(view.as_ref()),
        );

        Ok(())
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
