pub trait FormatImage {
    const NUM_CHANNELS: usize;
    const EXT: &'static str;
}

use web_sys::WebGl2RenderingContext;
#[derive(Clone, Copy, Debug)]
pub struct JPG;
impl JPG {
    const FORMAT: u32 = WebGl2RenderingContext::RGB as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGB as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;

    pub fn create_black_tile(width: i32) -> TileArrayBuffer<ArrayU8> {
        let num_channels = Self::NUM_CHANNELS as i32;
        let size_buf = (width * width * num_channels) as usize;

        let pixels = [0, 0, 0]
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        TileArrayBuffer::<ArrayU8>::new(&pixels, width, num_channels)
    }
}

impl FormatImage for JPG {
    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";
}

use crate::buffer::{ArrayU8, TileArrayBuffer};
#[derive(Clone, Copy, Debug)]
pub struct PNG;
impl PNG {
    const FORMAT: u32 = WebGl2RenderingContext::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGBA as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;

    pub fn create_black_tile(width: i32) -> TileArrayBuffer<ArrayU8> {
        let num_channels = Self::NUM_CHANNELS as i32;
        let size_buf = (width * width * num_channels) as usize;

        let pixels = [0, 0, 0, 255]
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        TileArrayBuffer::<ArrayU8>::new(&pixels, width, num_channels)
    }
}

impl FormatImage for PNG {
    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FITS {
    format: u32,
    internal_format: i32,
    _type: u32,
}

use crate::buffer::ArrayBuffer;
impl FITS {
    pub fn new(internal_format: i32) -> Self {
        let (format, _type) = match internal_format as u32 {
            WebGl2RenderingContext::RED => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT)
            }
            WebGl2RenderingContext::R32F => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT)
            }
            WebGl2RenderingContext::R8UI => (
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_BYTE,
            ),
            WebGl2RenderingContext::R16I => (
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::SHORT,
            ),
            WebGl2RenderingContext::R32I => (
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::INT,
            ),
            _ => unimplemented!(),
        };

        Self {
            format,
            internal_format,
            _type,
        }
    }

    pub fn create_black_tile<T: ArrayBuffer>(width: i32, value: T::Item) -> TileArrayBuffer<T> {
        let size_buf = (width * width * 1) as usize;

        let pixels = [value]
            .iter()
            .cloned()
            .cycle()
            .take(size_buf)
            .collect::<Vec<_>>();

        TileArrayBuffer::<T>::new(&pixels[..], width, 1)
    }
}

pub trait FITSDataType: std::cmp::PartialOrd + Clone + Copy + std::fmt::Debug {
    fn zero() -> Self;
}
impl FITSDataType for f32 {
    #[inline]
    fn zero() -> Self {
        0.0
    }
}
impl FITSDataType for i32 {
    #[inline]
    fn zero() -> Self {
        0
    }
}
impl FITSDataType for i16 {
    #[inline]
    fn zero() -> Self {
        0
    }
}
impl FITSDataType for u8 {
    #[inline]
    fn zero() -> Self {
        0
    }
}

impl FormatImage for FITS {
    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";
}
/*
impl FITS {
    fn get_texture_blank_pixels(&self, num_pixels: usize) -> BytesImageType<impl ArrayBufferView> {
        match self.internal_format as u32 {
            WebGl2RenderingContext::R32F | WebGl2RenderingContext::RED => {
                let mut pixels = vec![0_f32; num_pixels];
            },
            WebGl2RenderingContext::R8UI => {
                vec![0_u8; num_pixels]
            },
            WebGl2RenderingContext::R16I => {
                let mut pixels = vec![0_i16; num_pixels];
            },
            WebGl2RenderingContext::R32I => {
                let mut pixels = vec![0_i32; num_pixels];
            },
            _ => unimplemented!()
        }
    }
}
*/
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum FormatImageType {
    FITS(FITS),
    PNG,
    JPG,
}
impl FormatImageType {
    pub fn get_num_channels(&self) -> usize {
        match self {
            &FormatImageType::FITS(_) => FITS::NUM_CHANNELS,
            &FormatImageType::PNG => PNG::NUM_CHANNELS,
            &FormatImageType::JPG => JPG::NUM_CHANNELS,
        }
    }

    pub fn get_internal_format(&self) -> i32 {
        match self {
            &FormatImageType::FITS(fits) => fits.internal_format,
            &FormatImageType::PNG => PNG::INTERNAL_FORMAT,
            &FormatImageType::JPG => JPG::INTERNAL_FORMAT,
        }
    }

    pub fn get_format(&self) -> u32 {
        match self {
            &FormatImageType::FITS(fits) => fits.format,
            &FormatImageType::PNG => PNG::FORMAT,
            &FormatImageType::JPG => JPG::FORMAT,
        }
    }

    pub fn get_type(&self) -> u32 {
        match self {
            &FormatImageType::FITS(fits) => fits._type,
            &FormatImageType::PNG => PNG::TYPE,
            &FormatImageType::JPG => JPG::TYPE,
        }
    }

    pub fn get_ext_file(&self) -> &'static str {
        match self {
            &FormatImageType::FITS(_) => FITS::EXT,
            &FormatImageType::PNG => PNG::EXT,
            &FormatImageType::JPG => JPG::EXT,
        }
    }

    /*pub fn is_i_internal_format(&self) -> bool {
        match self {
            &FormatImageType::FITS(fits) => {
                match fits._type {
                    WebGl2RenderingContext::FLOAT | WebGl2RenderingContext::HALF_FLOAT => {
                        false
                    },
                    _ => true
                }
            },
            &FormatImageType::PNG => false,
            &FormatImageType::JPG => false,
        }
    }*/
}
