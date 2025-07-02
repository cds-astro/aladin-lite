use crate::texture::format::TextureFormat;
use crate::texture::format::R8U;
use cgmath::Vector3;
use fitsrs::card::Value;
use fitsrs::hdu::header::extension::image::Image as XImage;
use fitsrs::hdu::header::Bitpix;
use fitsrs::hdu::header::Header;
use fitsrs::hdu::header::Xtension;
use fitsrs::WCS;
use fitsrs::{Fits, HDU};
use std::fmt::Debug;
use std::io::Cursor;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct FitsImage<'a> {
    // Margin values for HiPS3D cubic tiles
    pub trim1: u32,
    pub trim2: u32,
    pub trim3: u32,
    // Image/cube size
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    // Bitpix
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

fn parse_keyword_as_number<X: Xtension + Debug>(header: &Header<X>, keyword: &str) -> Option<f32> {
    match header.get(keyword) {
        Some(Value::Integer { value, .. }) => Some(*value as f32),
        Some(Value::Float { value, .. }) => Some(*value as f32),
        _ => None,
    }
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

                        let bscale = parse_keyword_as_number(header, "BSCALE").unwrap_or(1.0);
                        let bzero = parse_keyword_as_number(header, "BZERO").unwrap_or(0.0);
                        let blank = parse_keyword_as_number(header, "BLANK");

                        let trim1 = parse_keyword_as_number(header, "TRIM1").unwrap_or(0.0) as u32;
                        let trim2 = parse_keyword_as_number(header, "TRIM2").unwrap_or(0.0) as u32;
                        let trim3 = parse_keyword_as_number(header, "TRIM3").unwrap_or(0.0) as u32;

                        let off = hdu.get_data_unit_byte_offset() as usize;
                        let len = hdu.get_data_unit_byte_size() as usize;

                        let raw_bytes = &bytes[off..(off + len)];

                        let bitpix = hdu.get_header().get_xtension().get_bitpix();

                        let wcs = hdu.wcs().ok();

                        images.push(Self {
                            trim1,
                            trim2,
                            trim3,
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
            offset.x + self.trim1 as i32,
            offset.y + self.trim2 as i32,
            offset.z + self.trim3 as i32,
            self.width as i32,
            self.height as i32,
            self.depth as i32,
            Some(view.as_ref()),
        );

        Ok(())
    }

    fn get_size(&self) -> (u32, u32, u32) {
        (self.width, self.height, self.depth)
    }
}
