use super::pixel::Pixel;

pub trait ImageFormat {
    type P: Pixel;

    const NUM_CHANNELS: usize;
    const EXT: &'static str;

    const FORMAT: u32;
    const INTERNAL_FORMAT: i32;
    const TYPE: u32;
}

use web_sys::WebGl2RenderingContext;
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB8U;
impl ImageFormat for RGB8U {    
    type P = [u8; 3];

    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";

    const FORMAT: u32 = WebGl2RenderingContext::RGB as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGB as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA8U;
impl ImageFormat for RGBA8U {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";

    const FORMAT: u32 = WebGl2RenderingContext::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGBA as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA32F;
impl ImageFormat for RGBA32F {
    type P = [f32; 4];

    const NUM_CHANNELS: usize = 4;
    const EXT: &'static str = "png";

    const FORMAT: u32 = WebGl2RenderingContext::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGBA32F as i32;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB32F;
impl ImageFormat for RGB32F {
    type P = [f32; 3];

    const NUM_CHANNELS: usize = 3;
    const EXT: &'static str = "jpg";

    const FORMAT: u32 = WebGl2RenderingContext::RGB as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::RGB32F as i32;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32F;
impl ImageFormat for R32F {
    type P = [f32; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGl2RenderingContext::RED as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::R32F as i32;
    const TYPE: u32 = WebGl2RenderingContext::FLOAT;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R8UI;
impl ImageFormat for R8UI {
    type P = [u8; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGl2RenderingContext::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::R8UI as i32;
    const TYPE: u32 = WebGl2RenderingContext::UNSIGNED_BYTE;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R16I;
impl ImageFormat for R16I {
    type P = [i16; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGl2RenderingContext::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::R16I as i32;
    const TYPE: u32 = WebGl2RenderingContext::SHORT;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32I;
impl ImageFormat for R32I {
    type P = [i32; 1];

    const NUM_CHANNELS: usize = 1;
    const EXT: &'static str = "fits";

    const FORMAT: u32 = WebGl2RenderingContext::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGl2RenderingContext::R32I as i32;
    const TYPE: u32 = WebGl2RenderingContext::INT;
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ImageFormatType {
    RGBA32F,
    RGB32F,
    RGBA8U,
    RGB8U,
    R8UI,
    R16I,
    R32I,
    R32F
}