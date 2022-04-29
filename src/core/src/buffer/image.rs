use crate::{healpix_cell::HEALPixCell, time::Time};
use js_sys::{Function};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Debug)]
#[cfg(feature = "webgl2")]
pub enum RetrievedImageType {
    FitsImageR32f { image: FitsImage<R32F> },
    FitsImageR32i { image: FitsImage<R32I> },
    FitsImageR16i { image: FitsImage<R16I> },
    FitsImageR8ui { image: FitsImage<R8UI> },
    PngImageRgba8u { image: ImageBitmap<RGBA8U> },
    JpgImageRgb8u { image: ImageBitmap<RGB8U> },
}

#[cfg(feature = "webgl1")]
pub enum RetrievedImageType {
    FitsImageR32f { image: FitsImage<R32F> },
    PngImageRgba8u { image: ImageBitmap<RGBA8U> },
    JpgImageRgb8u { image: ImageBitmap<RGB8U> },
}

pub trait ImageRequest<F>
where
    F: ImageFormat,
{
    type I: Image<T = F>;

    fn new() -> Self;
    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
        resolved: Rc<Cell<ResolvedStatus>>,
    ) -> Result<(), JsValue>;
    fn image(&self, tile_width: i32) -> Result<Self::I, JsValue>;
}

#[cfg(feature = "webgl2")]
pub enum ImageRequestType {
    FitsR32FImageReq(FitsImageRequest),
    FitsR32IImageReq(FitsImageRequest),
    FitsR16IImageReq(FitsImageRequest),
    FitsR8UIImageReq(FitsImageRequest),
    PNGRGBA8UImageReq(ImageBitmapRequest<RGBA8U>),
    JPGRGB8UImageReq(ImageBitmapRequest<RGB8U>),
}
#[cfg(feature = "webgl1")]
pub enum ImageRequestType {
    FitsR32FImageReq(FitsImageRequest),
    PNGRGBA8UImageReq(ImageBitmapRequest<RGBA8U>),
    JPGRGB8UImageReq(ImageBitmapRequest<RGB8U>),
}

use al_core::format::ImageFormatType;
impl ImageRequestType {
    pub fn new(fmt: ImageFormatType) -> Self {
        #[cfg(feature = "webgl2")]
        match fmt {
            ImageFormatType::RGBA8U => {
                ImageRequestType::PNGRGBA8UImageReq(
                    <ImageBitmapRequest<RGBA8U> as ImageRequest<RGBA8U>>::new(),
                )
            }
            ImageFormatType::RGB8U => {
                ImageRequestType::JPGRGB8UImageReq(
                    <ImageBitmapRequest<RGB8U> as ImageRequest<RGB8U>>::new(),
                )
            }
            ImageFormatType::R32F => {
                ImageRequestType::FitsR32FImageReq(
                    <FitsImageRequest as ImageRequest<R32F>>::new(),
                )
            }
            ImageFormatType::R8UI => {
                ImageRequestType::FitsR8UIImageReq(
                    <FitsImageRequest as ImageRequest<R8UI>>::new(),
                )
            }
            ImageFormatType::R16I => {
                ImageRequestType::FitsR16IImageReq(
                    <FitsImageRequest as ImageRequest<R16I>>::new(),
                )
            }
            ImageFormatType::R32I => {
                ImageRequestType::FitsR32IImageReq(
                    <FitsImageRequest as ImageRequest<R32I>>::new(),
                )
            }
            _ => unimplemented!(),
        }
        #[cfg(feature = "webgl1")]
        match fmt {
            ImageFormatType::RGBA8U => {
                ImageRequestType::PNGRGBA8UImageReq(
                    <ImageBitmapRequest as ImageRequest<RGBA8U>>::new()
                )
            }
            ImageFormatType::RGB8U => {
                ImageRequestType::JPGRGB8UImageReq(
                    <ImageBitmapRequest as ImageRequest<RGB8U>>::new()
                )
            }
            ImageFormatType::R32F => {
                ImageRequestType::FitsR32FImageReq(
                    <FitsImageRequest as ImageRequest<R32F>>::new()
                )
            }
        }
    }

    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
        resolved: Rc<Cell<ResolvedStatus>>
    ) -> Result<(), JsValue> {
        match self {
            ImageRequestType::FitsR32FImageReq(r) => {
                <FitsImageRequest as ImageRequest<R32F>>::send(r, success, fail, url, resolved)
            }
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR32IImageReq(r) => {
                <FitsImageRequest as ImageRequest<R32I>>::send(r, success, fail, url, resolved)
            }
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR16IImageReq(r) => {
                <FitsImageRequest as ImageRequest<R16I>>::send(r, success, fail, url, resolved)
            }
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR8UIImageReq(r) => {
                <FitsImageRequest as ImageRequest<R8UI>>::send(r, success, fail, url, resolved)
            }
            ImageRequestType::PNGRGBA8UImageReq(r) => {
                <ImageBitmapRequest<RGBA8U> as ImageRequest<RGBA8U>>::send(r, success, fail, url, resolved)
            }
            ImageRequestType::JPGRGB8UImageReq(r) => {
                <ImageBitmapRequest<RGB8U> as ImageRequest<RGB8U>>::send(r, success, fail, url, resolved)
            }
        }
    }

    fn image(&self, tile_width: i32) -> Result<RetrievedImageType, JsValue> {
        match self {
            ImageRequestType::FitsR32FImageReq(r) => Ok(RetrievedImageType::FitsImageR32f {
                image: r.image(tile_width)?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR32IImageReq(r) => Ok(RetrievedImageType::FitsImageR32i {
                image: r.image(tile_width)?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR16IImageReq(r) => Ok(RetrievedImageType::FitsImageR16i {
                image: r.image(tile_width)?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR8UIImageReq(r) => Ok(RetrievedImageType::FitsImageR8ui {
                image: r.image(tile_width)?,
            }),
            ImageRequestType::PNGRGBA8UImageReq(r) => Ok(RetrievedImageType::PngImageRgba8u {
                image: r.image(tile_width)?,
            }),
            ImageRequestType::JPGRGB8UImageReq(r) => Ok(RetrievedImageType::JpgImageRgb8u {
                image: r.image(tile_width)?,
            }),
        }
    }
}
use super::Tile;

pub struct TileRequest {
    // Is none when it is available for downloading a new fits
    // or image tile
    req: ImageRequestType,
    time_request: Time,
    // Flag telling if the tile has been copied so that
    // the HtmlImageElement can be reused to download another tile
    //ready: bool,
    resolved: Rc<Cell<ResolvedStatus>>,
    pub tile: Option<Tile>,
    closures: [Closure<dyn FnMut(&web_sys::Event)>; 2],
}
#[derive(Clone, Copy, PartialEq)]
pub enum ResolvedStatus {
    NotResolved,
    Missing,
    Found,
}
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

impl TileRequest {
    pub fn new(image_req: ImageRequestType) -> Self {
        // By default, we say the tile is available to be reused
        let resolved = Rc::new(Cell::new(ResolvedStatus::NotResolved));
        let tile = None;

        let time_request = Time::now();
        let success = {
            let r = resolved.clone();

            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                r.set(ResolvedStatus::Found);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        let fail = {
            let r = resolved.clone();
            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                r.set(ResolvedStatus::Missing);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        let closures = [
            success, fail
        ];

        Self {
            req: image_req,
            resolved,
            //ready,
            tile,
            closures,
            time_request,
        }
    }

    pub fn send(&mut self, tile: Tile) -> Result<(), JsValue> {
        //assert!(self.is_ready());

        self.tile = Some(tile.clone());
        let Tile {
            cell,
            root_url,
            format,
        } = tile;

        //self.ready = false;

        let url = {
            let HEALPixCell(depth, idx) = cell;

            let dir_idx = (idx / 10000) * 10000;

            let url = format!(
                "{}/Norder{}/Dir{}/Npix{}.{}",
                root_url,
                depth,
                dir_idx,
                idx,
                format.get_ext_file()
            );

            url
        };

        //self.resolved.set(ResolvedStatus::NotResolved);

        self.req.send(
            Some(self.closures[0].as_ref().unchecked_ref()),
            Some(self.closures[1].as_ref().unchecked_ref()),
            &url,
            self.resolved.clone()
        )?;

        //self.closures = [success, fail];
        self.time_request = Time::now();

        Ok(())
    }

    pub fn get_tile(&self) -> &Tile {
        self.tile.as_ref().unwrap()
    }

    pub fn get_time_request(&self) -> Time {
        self.time_request
    }

    pub fn is_resolved(&self) -> bool {
        let resolved = self.resolve_status();
        resolved == ResolvedStatus::Found || resolved == ResolvedStatus::Missing
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }

    pub fn get_image(&self, tile_width: i32) -> Result<RetrievedImageType, JsValue> {
        debug_assert!(self.is_resolved());
        self.req.image(tile_width)
    }
}

impl Drop for TileRequest {
    fn drop(&mut self) {
        //self.req.send(None, None, "").unwrap();
    }
}
/* ------------------------------------------------------ */

#[derive(Debug)]
pub struct HTMLImage<F>
where
    F: ImageFormat,
{
    image: web_sys::HtmlImageElement,
    size: Vector2<i32>,
    format: std::marker::PhantomData<F>,
}
use cgmath::{Vector2, Vector3};
impl<F> Image for HTMLImage<F>
where
    F: ImageFormat,
{
    type T = F;

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                offset.x,
                offset.y,
                &self.image,
            );
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

impl<F> Drop for HTMLImage<F>
where
    F: ImageFormat
{
    fn drop(&mut self) {
        //al_core::log("HTML Image dropped!");
    }
}

/*-----------------------------------------------------*/
#[derive(Debug)]
#[derive(Clone)]
pub struct ImageBitmap<F>
where
    F: ImageFormat + Clone,
{
    pub image: Arc<Mutex<Option<web_sys::ImageBitmap>>>,
    pub size: Vector2<i32>,
    format: std::marker::PhantomData<F>,
}
use crate::num_traits::Zero;
impl<F> ImageBitmap<F>
where
    F: ImageFormat + Clone,
{
    pub fn empty() -> Self {
        //al_core::log(&format!("size image bitmap received: {:?}", size));
        Self {
            image: Arc::new(Mutex::new(None)),
            size: Vector2::zero(),
            format: std::marker::PhantomData
        }
    }

    pub fn new(bmp: Arc<Mutex<Option<web_sys::ImageBitmap>>>, size: Vector2<i32>) -> Self {
        al_core::log(&format!("size image bitmap received: {:?}", size));
        Self {
            image: bmp,
            size: size,
            format: std::marker::PhantomData
        }
    }

    pub fn set(&mut self, bmp: web_sys::ImageBitmap) {
        self.size = Vector2::new(
            bmp.width() as i32,
            bmp.height() as i32
        );

        *(self.image.lock().unwrap()) = Some(bmp);
    }
}

impl<F> Image for ImageBitmap<F>
where
    F: ImageFormat + Clone,
{
    type T = F;

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        if let Some(bitmap) = &*self.image.lock().unwrap() {
            textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                offset.x,
                offset.y,
                bitmap,
            );
        } else {
            unreachable!();
        }
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

impl<F> Drop for ImageBitmap<F>
where
    F: ImageFormat + Clone
{
    fn drop(&mut self) {
        //al_core::log("HTML Image dropped!");
    }
}


pub struct CompressedImageRequest {
    image: web_sys::HtmlImageElement,
}

#[cfg(feature = "webgl2")]
use al_core::format::{R16I, R32I, R8UI};
use al_core::format::{R32F, RGB8U, RGBA8U};
pub trait CompressedImageFormat: ImageFormat {}
impl CompressedImageFormat for RGBA8U {}
impl CompressedImageFormat for RGB8U {}

impl<F> ImageRequest<F> for CompressedImageRequest
where
    F: CompressedImageFormat,
{
    type I = HTMLImage<F>;

    fn new() -> Self {
        let image = web_sys::HtmlImageElement::new().unwrap();
        image.set_cross_origin(Some(""));

        Self { image }
    }

    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
        resolved: Rc<Cell<ResolvedStatus>>,
    ) -> Result<(), JsValue> {
        self.image.set_onload(success);
        self.image.set_onerror(fail);
        self.image.set_src(url);
        //self.image.send().unwrap();

        Ok(())
    }

    fn image(&self, _tile_width: i32) -> Result<Self::I, JsValue> {
        let width = self.image.width() as i32;
        let height = self.image.height() as i32;

        let size = Vector2::new(width, height);
        Ok(HTMLImage {
            size,
            image: self.image.clone(),
            format: std::marker::PhantomData,
        })
    }
}

/* ------------------------------------------------------ */

#[derive(Debug)]
pub struct FitsImage<F>
where
    F: FitsImageFormat,
{
    // Fits header properties
    pub blank: f32,
    pub bzero: f32,
    pub bscale: f32,

    // Tile size
    size: Vector2<i32>,

    // Aligned allocation layout
    layout: std::alloc::Layout,
    // Raw pointer to the fits in memory
    aligned_raw_bytes_ptr: *mut u8,
    // Raw pointer to the data part of the fits
    aligned_data_raw_bytes_ptr: *const F::Type,
}

impl<F> FitsImage<F>
where 
    F: FitsImageFormat,
{
    pub fn new(fits_raw_bytes: &js_sys::Uint8Array, size: i32) -> Result<Self, JsValue> {
        // Create a correctly aligned buffer to the type F
        let align = std::mem::size_of::<F::Type>();
        let layout = Layout::from_size_align(fits_raw_bytes.length() as usize, align)
            .expect("Cannot create sized aligned memory layout");
        // 1. Alloc the aligned memory buffer
        let aligned_raw_bytes_ptr = unsafe { alloc(layout) };

        let FitsMemAligned { data, header } = unsafe {
            // 2. Copy the raw fits bytes into that aligned memory space
            fits_raw_bytes.raw_copy_to_ptr(aligned_raw_bytes_ptr);        

            // 3. Convert to a slice of bytes
            let aligned_raw_bytes = std::slice::from_raw_parts(aligned_raw_bytes_ptr, fits_raw_bytes.length() as usize);
            // 4. Parse the fits file to extract its data (big endianness is handled inside fitsrs and is O(n))
            FitsMemAligned::<F::Type>::from_byte_slice(aligned_raw_bytes)
                .map_err(|e| {
                    JsValue::from_str(&format!(
                        "Parsing FITS error: {:?}", e
                    ))
                })?
        };
        let num_pixels = (size*size) as usize;
        assert_eq!(data.len(), num_pixels);

        let bscale = if let Some(FITSHeaderKeyword::Other { value, .. }) = header.get("BSCALE") {
            if let FITSKeywordValue::FloatingPoint(bscale) = value {
                *bscale as f32
            } else {
                1.0
            }
        } else {
            1.0
        };

        let bzero = if let Some(FITSHeaderKeyword::Other { value, .. }) = header.get("BZERO") {
            if let FITSKeywordValue::FloatingPoint(bzero) = value {
                *bzero as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        let blank = if let Some(FITSHeaderKeyword::Blank(blank)) = header.get("BLANK") {
            *blank as f32
        } else {
            std::f32::NAN
        };

        Ok(Self {
            // Metadata fits header properties
            blank,
            bzero,
            bscale,
            // Tile size
            size: Vector2::new(size, size),

            // Allocation info of the layout
            layout,
            aligned_raw_bytes_ptr,

            aligned_data_raw_bytes_ptr: data.as_ptr(),
        })
    }
}

use al_core::{image::Image, texture::Texture2DArray};
impl<F> Image for FitsImage<F>
where
    F: FitsImageFormat,
{
    type T = F;

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        let num_pixels = self.size.x * self.size.y;
        let slice_raw_bytes = unsafe {
            std::slice::from_raw_parts(
                self.aligned_data_raw_bytes_ptr as *const _, 
                num_pixels as usize
            )
        };

        let array = unsafe { F::view(slice_raw_bytes) };
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                offset.x,
                offset.y,
                self.size.x,
                self.size.y,
                Some(array.as_ref()),
            );
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

impl<F> Drop for FitsImage<F>
where
    F: FitsImageFormat,
{
    fn drop(&mut self) {
        //al_core::log("dealloc fits tile");
        unsafe { std::alloc::dealloc(self.aligned_raw_bytes_ptr, self.layout); }
    }
}

use fitsrs::{FITSHeaderKeyword, FITSKeywordValue};
use wasm_bindgen::JsValue;
use web_sys::XmlHttpRequestResponseType;

use al_core::format::ImageFormat;

use fitsrs::ToBigEndian;
pub trait FitsImageFormat: ImageFormat {
    type Type: ToBigEndian;
    type ArrayBufferView: AsRef<js_sys::Object>;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView;
}

impl FitsImageFormat for R32F {
    type Type = f32;
    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R32I {
    type Type = i32;

    type ArrayBufferView = js_sys::Int32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R16I {
    type Type = i16;

    type ArrayBufferView = js_sys::Int16Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R8UI {
    type Type = u8;

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

use web_sys::XmlHttpRequest;
use std::alloc::{alloc, Layout};
use fitsrs::FitsMemAligned;
pub struct FitsImageRequest {
    image: XmlHttpRequest,
}

impl<F> ImageRequest<F> for FitsImageRequest
where
    F: FitsImageFormat,
{
    type I = FitsImage<F>;

    fn new() -> Self {
        let image = XmlHttpRequest::new().unwrap();
        image.set_response_type(XmlHttpRequestResponseType::Arraybuffer);

        Self { image }
    }

    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
        _resolved: Rc<Cell<ResolvedStatus>>,
    ) -> Result<(), JsValue> {
        self.image.open_with_async("GET", url, true)?;
        self.image.set_onload(success);
        self.image.set_onerror(fail);

        self.image.send().unwrap();

        Ok(())
    }

    fn image(&self, size: i32) -> Result<Self::I, JsValue> {
        // We know at this point the request is resolved
        let fits_raw_bytes = js_sys::Uint8Array::new(self.image.response().unwrap().as_ref());
        FitsImage::new(&fits_raw_bytes, size)
    }
}

pub struct ImageBitmapRequest<F>
where
    F: CompressedImageFormat + Clone + 'static
{
    //image: XmlHttpRequest,
    result: ImageBitmap<F>,
}

use std::sync::{Mutex, Arc};
use std::cell::RefCell;
impl<F> ImageRequest<F> for ImageBitmapRequest<F>
where
    F: CompressedImageFormat + Clone + 'static,
{
    type I = ImageBitmap<F>;

    fn new() -> Self {
        /*let image = XmlHttpRequest::new().unwrap();
        image.set_response_type(XmlHttpRequestResponseType::Blob);

        Self { image }*/
        Self {
            result: ImageBitmap::<F>::empty()
        }
    }

    fn send(
        &self,
        _success: Option<&Function>,
        _fail: Option<&Function>,
        url: &str,
        resolved: Rc<Cell<ResolvedStatus>>,
    ) -> Result<(), JsValue> {
        // Define the future to execute
        let window = web_sys::window().unwrap();

        let mut bmp = self.result.clone();
        let url = String::from(url);
        let fut = async move {
            use wasm_bindgen_futures::JsFuture;
            use web_sys::{Blob, Response, Request, RequestInit, RequestMode};
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts).unwrap();
            if let Ok(resp_value) = JsFuture::from(window.fetch_with_request(&request)).await {
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into().unwrap();

                let blob = JsFuture::from(resp.blob().unwrap()).await.unwrap().into();

                let image_bmp = JsFuture::from(window.create_image_bitmap_with_blob(&blob).unwrap()).await
                    .unwrap()
                    .into();

                bmp.set(image_bmp);
                resolved.set(ResolvedStatus::Found);
            } else {
                resolved.set(ResolvedStatus::Missing);
            }
        };

        wasm_bindgen_futures::spawn_local(fut);

        Ok(())
    }

    fn image(&self, _size: i32) -> Result<Self::I, JsValue> {
        Ok(self.result.clone())
    }
}