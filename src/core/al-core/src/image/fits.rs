use crate::texture::format::TextureFormat;
use crate::texture::format::R8U;
use cgmath::Vector3;
use fitsrs::hdu::data::bintable::data::BinaryTableData;
use fitsrs::hdu::data::bintable::tile_compressed::pixels::Pixels;
use fitsrs::hdu::header::extension::bintable::TileCompressedImage;
use fitsrs::hdu::header::Bitpix;
use fitsrs::WCS;
use fitsrs::{Fits, HDU};
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::Range;
use wasm_bindgen::JsValue;

use fitsrs::hdu::header::ValueMap;

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
    // bytes offset where the data bytes are located inside the fits
    pub data_byte_offset: Range<usize>,
    // raw bytes of the data image (in Big-Endian)
    pub raw_bytes: Cow<'a, [u8]>,

    // keep the header keywords and their values
    pub header: ValueMap,
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
                    let naxis = hdu.get_header().get_xtension().get_naxis();
                    if naxis.len() >= 2 {
                        let width = naxis[0];
                        let height = naxis[1];
                        let depth = if naxis.len() >= 3 { naxis[2] } else { 1 };

                        let header = hdu.get_header();

                        let bscale = header.get_parsed::<f32>("BSCALE").unwrap_or(1.0);
                        let bzero = header.get_parsed::<f32>("BZERO").unwrap_or(0.0);
                        let blank = header.get_parsed::<f32>("BLANK").ok();

                        let trim1 = header.get_parsed::<u32>("TRIM1").unwrap_or(0);
                        let trim2 = header.get_parsed::<u32>("TRIM2").unwrap_or(0);
                        let trim3 = header.get_parsed::<u32>("TRIM3").unwrap_or(0);

                        let bitpix = hdu.get_header().get_xtension().get_bitpix();

                        let off = hdu.get_data_unit_byte_offset() as usize;
                        let len = hdu.get_data_unit_byte_size() as usize;

                        let data_byte_offset = off..(off + len);
                        let raw_bytes = Cow::Borrowed(&bytes[data_byte_offset.clone()]);

                        let wcs = hdu.wcs().ok();

                        let values: &ValueMap = header;
                        images.push(Self {
                            trim1,
                            trim2,
                            trim3,
                            width: width as u32,
                            height: height as u32,
                            depth: depth as u32,
                            bitpix,
                            bscale,
                            wcs,
                            bzero,
                            blank,
                            data_byte_offset,
                            raw_bytes,
                            header: values.clone(),
                        });
                    }
                }
                HDU::XBinaryTable(hdu) => {
                    let header = hdu.get_header();
                    let bin_table = header.get_xtension();

                    if let Some(TileCompressedImage {
                        z_bitpix: bitpix,
                        z_naxisn: naxis,
                        ..
                    }) = &bin_table.get_z_image()
                    {
                        if naxis.len() >= 2 {
                            let width = naxis[0] as u32;
                            let height = naxis[1] as u32;

                            let depth = if naxis.len() >= 3 { naxis[2] as u32 } else { 1 };

                            let bscale = header.get_parsed::<f32>("BSCALE").unwrap_or(1.0);
                            let bzero = header.get_parsed::<f32>("BZERO").unwrap_or(0.0);
                            let blank = header.get_parsed::<f32>("BLANK").ok();

                            let trim1 = header.get_parsed::<u32>("TRIM1").unwrap_or(0);
                            let trim2 = header.get_parsed::<u32>("TRIM2").unwrap_or(0);
                            let trim3 = header.get_parsed::<u32>("TRIM3").unwrap_or(0);

                            let wcs = hdu.wcs().ok();

                            let off = hdu.get_data_unit_byte_offset() as usize;
                            let len = hdu.get_data_unit_byte_size() as usize;

                            let data_byte_offset = off..(off + len);

                            let mut bitpix = *bitpix;
                            let raw_bytes = match fits.get_data(&hdu) {
                                BinaryTableData::TileCompressed(Pixels::U8(pixels)) => {
                                    Some(pixels.collect::<Vec<_>>())
                                }
                                BinaryTableData::TileCompressed(Pixels::I16(pixels)) => {
                                    Some(pixels.flat_map(|p| p.to_be_bytes()).collect::<Vec<_>>())
                                }
                                BinaryTableData::TileCompressed(Pixels::I32(pixels)) => {
                                    Some(pixels.flat_map(|p| p.to_be_bytes()).collect::<Vec<_>>())
                                }
                                BinaryTableData::TileCompressed(Pixels::F32(pixels)) => {
                                    Some(pixels.flat_map(|p| p.to_be_bytes()).collect::<Vec<_>>())
                                }
                                BinaryTableData::TileCompressed(Pixels::F64(pixels)) => {
                                    bitpix = Bitpix::F32;
                                    let raw_bytes =
                                        pixels.flat_map(|p| p.to_be_bytes()).collect::<Vec<_>>();

                                    Some(raw_bytes)
                                }
                                _ => None,
                            };

                            if let Some(raw_bytes) = raw_bytes {
                                let values: &ValueMap = header;

                                images.push(Self {
                                    trim1,
                                    trim2,
                                    trim3,
                                    width,
                                    height,
                                    depth,
                                    bitpix,
                                    bscale,
                                    wcs,
                                    bzero,
                                    blank,
                                    data_byte_offset,
                                    raw_bytes: Cow::Owned(raw_bytes),
                                    header: values.clone(),
                                });
                            }
                        }
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
use std::convert::TryInto;
impl Image for FitsImage<'_> {
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        // The texture array
        textures: &T,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        let view = unsafe {
            match self.bitpix {
                Bitpix::I64 => {
                    // convert to i64 first
                    let new_bytes: Vec<_> = self
                        .raw_bytes
                        .chunks_exact(8)
                        .flat_map(|chunk| {
                            let bytes: [u8; 8] = chunk.try_into().unwrap();
                            let value = i64::from_be_bytes(bytes);

                            (value as i32).to_be_bytes()
                        })
                        .collect();

                    R8U::view(&new_bytes)
                }
                Bitpix::F64 => {
                    // convert to i64 first
                    let new_bytes: Vec<_> = self
                        .raw_bytes
                        .chunks_exact(8)
                        .flat_map(|chunk| {
                            let bytes: [u8; 8] = chunk.try_into().unwrap();
                            let value = f64::from_be_bytes(bytes);

                            (value as f32).to_be_bytes()
                        })
                        .collect();

                    R8U::view(&new_bytes)
                }
                _ => R8U::view(&self.raw_bytes),
            }
        };

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
        // The true image size is given by ONAXISi keywords
        (self.width, self.height, self.depth)
    }
}
