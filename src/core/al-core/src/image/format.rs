use crate::texture::pixel::Pixel;
use al_api::hips::ImageExt;

pub enum Bytes<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

pub trait ImageFormat {
    type P: Pixel;
    type ArrayBufferView: AsRef<js_sys::Object>;

    const NUM_CHANNELS: usize;

    const FORMAT: u32;
    const INTERNAL_FORMAT: i32;
    const TYPE: u32;

    const CHANNEL_TYPE: ChannelType;

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
    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str>;
}
use crate::webgl_ctx::WebGlRenderingCtx;
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB8U;
impl ImageFormat for RGB8U {
    type P = [u8; 3];

    const NUM_CHANNELS: usize = 3;

    const FORMAT: u32 = WebGlRenderingCtx::RGB as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const CHANNEL_TYPE: ChannelType = ChannelType::RGB8U;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder
            .decode()
            .map_err(|_| "Cannot decoder jpeg. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA8U;
#[cfg(feature = "webgl2")]
impl ImageFormat for RGBA8U {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;

    const FORMAT: u32 = WebGlRenderingCtx::RGBA as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const CHANNEL_TYPE: ChannelType = ChannelType::RGBA8U;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder
            .decode()
            .map_err(|_| "Cannot decoder png. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA32F;
impl ImageFormat for RGBA32F {
    type P = [f32; 4];

    const NUM_CHANNELS: usize = 4;

    const FORMAT: u32 = WebGlRenderingCtx::RGBA as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA as i32;

    const CHANNEL_TYPE: ChannelType = ChannelType::RGBA32F;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGB32F;
impl ImageFormat for RGB32F {
    type P = [f32; 3];

    const NUM_CHANNELS: usize = 3;

    const FORMAT: u32 = WebGlRenderingCtx::RGB as u32;
    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB as i32;

    const CHANNEL_TYPE: ChannelType = ChannelType::RGB32F;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32F;
impl ImageFormat for R32F {
    type P = [f32; 1];

    const NUM_CHANNELS: usize = 1;

    #[cfg(feature = "webgl2")]
    const FORMAT: u32 = WebGlRenderingCtx::RED as u32;
    #[cfg(feature = "webgl1")]
    const FORMAT: u32 = WebGlRenderingCtx::LUMINANCE as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::LUMINANCE as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    const CHANNEL_TYPE: ChannelType = ChannelType::R32F;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R64F;
#[cfg(feature = "webgl2")]
impl ImageFormat for R64F {
    type P = [f32; 1];

    const NUM_CHANNELS: usize = 1;

    #[cfg(feature = "webgl2")]
    const FORMAT: u32 = WebGlRenderingCtx::RED as u32;
    #[cfg(feature = "webgl1")]
    const FORMAT: u32 = WebGlRenderingCtx::LUMINANCE as u32;

    #[cfg(feature = "webgl2")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32F as i32;
    #[cfg(feature = "webgl1")]
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::LUMINANCE as i32;

    const TYPE: u32 = WebGlRenderingCtx::FLOAT;

    const CHANNEL_TYPE: ChannelType = ChannelType::R64F;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R8UI;
#[cfg(feature = "webgl2")]
impl ImageFormat for R8UI {
    type P = [u8; 1];

    const NUM_CHANNELS: usize = 1;

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R8UI as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const CHANNEL_TYPE: ChannelType = ChannelType::R8UI;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R16I;
#[cfg(feature = "webgl2")]
impl ImageFormat for R16I {
    type P = [i16; 1];

    const NUM_CHANNELS: usize = 1;

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R16I as i32;
    const TYPE: u32 = WebGlRenderingCtx::SHORT;
    const CHANNEL_TYPE: ChannelType = ChannelType::R16I;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Int16Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32I;
#[cfg(feature = "webgl2")]
impl ImageFormat for R32I {
    type P = [i32; 1];

    const NUM_CHANNELS: usize = 1;

    const FORMAT: u32 = WebGlRenderingCtx::RED_INTEGER as u32;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R32I as i32;
    const TYPE: u32 = WebGlRenderingCtx::INT;

    const CHANNEL_TYPE: ChannelType = ChannelType::R32I;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Int32Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum ChannelType {
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

pub const NUM_CHANNELS: usize = 9;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ImageFormatType {
    pub ext: ImageExt,
    pub channel: ChannelType,
}

impl ImageFormatType {
    pub fn get_ext_file(&self) -> &ImageExt {
        &self.ext
    }

    pub fn get_channel(&self) -> ChannelType {
        self.channel
    }

    pub fn is_colored(&self) -> bool {
        match self.channel {
            ChannelType::RGBA32F
            | ChannelType::RGB32F
            | ChannelType::RGBA8U
            | ChannelType::RGB8U => true,
            _ => false,
        }
    }
}
