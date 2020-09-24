
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
}

impl FormatImage for JPG {
    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";
}

#[derive(Clone, Copy, Debug)]
pub struct PNG;
impl PNG {
    const FORMAT: u32 = WebGl2RenderingContext::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGBA as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;
}

impl FormatImage for PNG {
    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FITS { format: u32, internal_format: i32, _type: u32 }

impl FITS {
    pub fn new(internal_format: i32) -> Self {
        let (format, _type) = match internal_format as u32 {
            WebGl2RenderingContext::R32F => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT)
            },
            WebGl2RenderingContext::RGBA32F => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT)
            },
            WebGl2RenderingContext::R16F => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::HALF_FLOAT)
            },
            WebGl2RenderingContext::R8UI => {
                (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::UNSIGNED_BYTE)
            },
            WebGl2RenderingContext::R16I => {
                (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::SHORT)
            },
            WebGl2RenderingContext::R32I => {
                (WebGl2RenderingContext::RED_INTEGER, WebGl2RenderingContext::INT)
            },
            WebGl2RenderingContext::RED => {
                (WebGl2RenderingContext::RED, WebGl2RenderingContext::FLOAT)
            },
            _ => unimplemented!()
        };

        Self {
            format,
            internal_format,
            _type
        }
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

    pub fn is_i_internal_format(&self) -> bool {
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
    }
}
