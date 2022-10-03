use crate::texture::pixel::Pixel;

pub enum Bytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

pub trait ImageFormat {
    type P: Pixel;

    const NUM_CHANNELS: usize;
    const EXT: &'static str;

    const FORMAT: u32;
    const INTERNAL_FORMAT: i32;
    const TYPE: u32;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str>;
}
use crate::webgl_ctx::WebGlRenderingCtx;
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB8U;
impl ImageFormat for RGB8U {
    type P = [u8; 3];

    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";

    const FORMAT: u32 = WebGlRenderingCtx::RGB as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder.decode().map_err(|_| "Cannot decoder jpeg. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA8U;
#[cfg(feature = "webgl2")]
impl ImageFormat for RGBA8U {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";

    const FORMAT: u32 = WebGlRenderingCtx::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder.decode().map_err(|_| "Cannot decoder png. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))
    }
}
#[cfg(feature = "webgl1")]
impl ImageFormat for RGBA8U {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";

    const FORMAT: u32 = WebGlRenderingCtx::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA32F;
impl ImageFormat for RGBA32F {
    type P = [f32; 4];

    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";

    const FORMAT: u32 = WebGlRenderingCtx::RGBA as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB32F;
impl ImageFormat for RGB32F {
    type P = [f32; 3];

    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";

    const FORMAT: u32 = WebGlRenderingCtx::RGB as u32;
    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32F;
impl ImageFormat for R32F {
    type P = [f32; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    #[cfg(feature = "webgl2")]
    const FORMAT: u32 = WebGlRenderingCtx::RED as u32;
    #[cfg(feature = "webgl1")]
    const FORMAT: u32 = WebGlRenderingCtx::LUMINANCE as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::LUMINANCE as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}


#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R64F;
#[cfg(feature = "webgl2")]
impl ImageFormat for R64F {
    type P = [f32; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    #[cfg(feature = "webgl2")]
    const FORMAT: u32 = WebGlRenderingCtx::RED as u32;
    #[cfg(feature = "webgl1")]
    const FORMAT: u32 = WebGlRenderingCtx::LUMINANCE as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::LUMINANCE as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R8UI;
#[cfg(feature = "webgl2")]
impl ImageFormat for R8UI {
    type P = [u8; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R8UI as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R16I;
#[cfg(feature = "webgl2")]
impl ImageFormat for R16I {
    type P = [i16; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R16I as i32;
    const TYPE: u32 = WebGlRenderingCtx::SHORT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32I;
#[cfg(feature = "webgl2")]
impl ImageFormat for R32I {
    type P = [i32; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32I as i32;
    const TYPE: u32 = WebGlRenderingCtx::INT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ImageFormatType {
    RGBA32F,
    RGB32F,
    RGBA8U,
    RGB8U,
    R32F,
    #[cfg(feature = "webgl2")]
    R64F,
    #[cfg(feature = "webgl2")]
    R8UI,
    #[cfg(feature = "webgl2")]
    R16I,
    #[cfg(feature = "webgl2")]
    R32I,
}

impl ImageFormatType {
    pub fn get_ext_file(&self) -> &'static str {
        match self {
            ImageFormatType::RGBA32F => unimplemented!(),
            ImageFormatType::RGB32F => unimplemented!(),
            ImageFormatType::RGBA8U => RGBA8U::EXT,
            ImageFormatType::RGB8U => RGB8U::EXT,
            ImageFormatType::R32F => R32F::EXT,
            ImageFormatType::R64F => R64F::EXT,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R8UI => R8UI::EXT,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R16I => R16I::EXT,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R32I => R32I::EXT,
        }
    }
}
