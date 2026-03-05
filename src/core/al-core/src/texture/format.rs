use crate::texture::pixel::Pixel;

pub type Bytes<'a> = std::borrow::Cow<'a, [u8]>;

pub trait TextureFormat {
    type P: Pixel;
    type ArrayBufferView: AsRef<js_sys::Object>;

    const NUM_CHANNELS: usize;

    const FORMAT: u32;
    const INTERNAL_FORMAT: i32;
    const TYPE: u32;

    const PIXEL_TYPE: PixelType;

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
impl TextureFormat for RGB8U {
    type P = [u8; 3];

    const NUM_CHANNELS: usize = 3;

    const FORMAT: u32 = WebGlRenderingCtx::RGB;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGB8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const PIXEL_TYPE: PixelType = PixelType::RGB8U;

    fn decode(_raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        todo!()
        /*let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder
            .decode()
            .map_err(|_| "Cannot decoder jpeg. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))*/
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RGBA8U;
impl TextureFormat for RGBA8U {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;

    const FORMAT: u32 = WebGlRenderingCtx::RGBA;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const PIXEL_TYPE: PixelType = PixelType::RGBA8U;

    fn decode(_raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        /*let mut decoder = jpeg::Decoder::new(raw_bytes);
        let bytes = decoder
            .decode()
            .map_err(|_| "Cannot decoder png. This image may not be compressed.")?;

        Ok(Bytes::Owned(bytes))
        */
        todo!()
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32F;
impl TextureFormat for R32F {
    type P = [u8; 4];

    const NUM_CHANNELS: usize = 4;

    const FORMAT: u32 = WebGlRenderingCtx::RGBA;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const PIXEL_TYPE: PixelType = PixelType::R32F;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R8U;
impl TextureFormat for R8U {
    type P = [u8; 1];
    const FORMAT: u32 = WebGlRenderingCtx::RED;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::R8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const NUM_CHANNELS: usize = 1;
    const PIXEL_TYPE: PixelType = PixelType::R8U;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R16I;
impl TextureFormat for R16I {
    type P = [u8; 2];

    const NUM_CHANNELS: usize = 2;

    const FORMAT: u32 = WebGlRenderingCtx::RG;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RG8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;

    const PIXEL_TYPE: PixelType = PixelType::R16I;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct R32I;
impl TextureFormat for R32I {
    type P = [u8; 4];

    const FORMAT: u32 = WebGlRenderingCtx::RGBA;
    const INTERNAL_FORMAT: i32 = WebGlRenderingCtx::RGBA8 as i32;
    const TYPE: u32 = WebGlRenderingCtx::UNSIGNED_BYTE;
    const NUM_CHANNELS: usize = 4;

    const PIXEL_TYPE: PixelType = PixelType::R32I;

    fn decode(raw_bytes: &[u8]) -> Result<Bytes<'_>, &'static str> {
        Ok(Bytes::Borrowed(raw_bytes))
    }

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[<Self::P as Pixel>::Item]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PixelType {
    R8U,
    R16I,
    R32I,
    R32F,
    RGB8U,
    RGBA8U,
}

impl PixelType {
    pub const fn num_channels(&self) -> usize {
        match self {
            Self::RGB8U => 3,
            Self::RGBA8U => 4,
            _ => 1,
        }
    }
}
