use crate::{
    healpix::cell::HEALPixCell,
    time::Time
};
use js_sys::{Function};
use std::cell::Cell;
use std::rc::Rc;

pub mod fits;
pub mod bitmap;
pub mod html;

pub use bitmap::Bitmap;
pub use fits::Fits;
use al_core::image::ImageBuffer;
#[derive(Debug)]
#[cfg(feature = "webgl2")]
pub enum ImageType {
    FitsImageR32f { image: Fits<R32F> },
    FitsImageR32i { image: Fits<R32I> },
    FitsImageR16i { image: Fits<R16I> },
    FitsImageR8ui { image: Fits<R8UI> },
    PngImageRgba8u { image: Bitmap<RGBA8U> },
    JpgImageRgb8u { image: Bitmap<RGB8U> },
    RawRgb8u { image: ImageBuffer<RGB8U> },
    RawRgba8u { image: ImageBuffer<RGBA8U> },
    RawR32f { image: ImageBuffer<R32F> },
    RawR32i { image: ImageBuffer<R32I> },
    RawR16i { image: ImageBuffer<R16I> },
    RawR8ui { image: ImageBuffer<R8UI> },
}

#[cfg(feature = "webgl1")]
pub enum ImageType {
    FitsImageR32f { image: Fits<R32F> },
    PngImageRgba8u { image: Bitmap<RGBA8U> },
    JpgImageRgb8u { image: Bitmap<RGB8U> },
    RawRgb8u { image: ImageBuffer<RGB8U> },
    RawRgba8u { image: ImageBuffer<RGBA8U> },
    RawR32f { image: ImageBuffer<R32F> },
}

use al_core::{
    format::{
        RGB8U, R8UI, RGBA8U, R16I, R32I, R32F
    }, 
    Texture2DArray,
    image::Image,
};

use cgmath::Vector3;
impl Image for ImageType {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        match self {
            ImageType::FitsImageR32f { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::FitsImageR32i { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::FitsImageR16i { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::FitsImageR8ui { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::PngImageRgba8u { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::JpgImageRgb8u { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::RawRgb8u { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::RawRgba8u { image} => image.tex_sub_image_3d(textures, offset),
            ImageType::RawR32f { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::RawR32i { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::RawR16i { image } => image.tex_sub_image_3d(textures, offset),
            ImageType::RawR8ui { image } => image.tex_sub_image_3d(textures, offset),
        }
    }

    // The size of the image
    /*fn get_size(&self) -> &Vector2<i32> {
        match self {
            ImageType::FitsImageR32f { image } => image.get_size(),
            ImageType::FitsImageR32i { image } => image.get_size(),
            ImageType::FitsImageR16i { image } => image.get_size(),
            ImageType::FitsImageR8ui { image } => image.get_size(),
            ImageType::PngImageRgba8u { image } => image.get_size(),
            ImageType::JpgImageRgb8u { image } => image.get_size(),
        }
    }*/
}


/*
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
    fn image(&self) -> Result<Self::I, JsValue>;
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

    fn image(&self) -> Result<RetrievedImageType, JsValue> {
        match self {
            ImageRequestType::FitsR32FImageReq(r) => Ok(RetrievedImageType::FitsImageR32f {
                image: r.image()?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR32IImageReq(r) => Ok(RetrievedImageType::FitsImageR32i {
                image: r.image()?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR16IImageReq(r) => Ok(RetrievedImageType::FitsImageR16i {
                image: r.image()?,
            }),
            #[cfg(feature = "webgl2")]
            ImageRequestType::FitsR8UIImageReq(r) => Ok(RetrievedImageType::FitsImageR8ui {
                image: r.image()?,
            }),
            ImageRequestType::PNGRGBA8UImageReq(r) => Ok(RetrievedImageType::PngImageRgba8u {
                image: r.image()?,
            }),
            ImageRequestType::JPGRGB8UImageReq(r) => Ok(RetrievedImageType::JpgImageRgb8u {
                image: r.image()?,
            }),
        }
    }
}
use super::tile::Tile;

pub struct TileRequest {
    // Is none when it is available for downloading a new fits
    // or image tile
    req: ImageRequestType,
    time_request: Time,
    // Flag telling if the tile has been copied so that
    // the HtmlImageElement can be reused to download another tile
    //ready: bool,
    resolved: Rc<Cell<ResolvedStatus>>,
    //pub tile: Option<Tile<'a>>,
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
        //let tile = None;

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
            //tile,
            closures,
            time_request,
        }
    }

    pub fn send(&mut self, tile: &Tile) -> Result<(), JsValue> {
        //assert!(self.is_ready());

        //self.tile = Some(tile.clone());
        /*let Tile {
            cell,
            root_url,
            format,
        } = tile;*/

        //self.ready = false;

        let url = {
            let HEALPixCell(depth, idx) = cell;

            let dir_idx = (idx / 10000) * 10000;

            let url = format!(
                "{}/Norder{}/Dir{}/Npix{}.{}",
                tile.cfg.root_url,
                depth,
                dir_idx,
                idx,
                tile.cfg.get_format().get_ext_file()
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

    /*pub fn get_tile(&self) -> &Tile {
        self.tile.as_ref().unwrap()
    }*/

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

    pub fn get_image(&self) -> Result<RetrievedImageType, JsValue> {
        debug_assert!(self.is_resolved());
        self.req.image()
    }
}

impl Drop for TileRequest {
    fn drop(&mut self) {
        //self.req.send(None, None, "").unwrap();
    }
}

/*-----------------------------------------------------*/

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

    fn image(&self) -> Result<Self::I, JsValue> {
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
*/
/* ------------------------------------------------------ */
/*
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

    fn image(&self) -> Result<Self::I, JsValue> {
        // We know at this point the request is resolved
        let fits_raw_bytes = js_sys::Uint8Array::new(self.image.response().unwrap().as_ref());
        FitsImage::new(&fits_raw_bytes)
    }
}
*/

/*
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

    fn image(&self) -> Result<Self::I, JsValue> {
        Ok(self.result.clone())
    }
} */