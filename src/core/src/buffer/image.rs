use crate::{healpix_cell::HEALPixCell, time::Time};
use js_sys::Function;
use std::cell::Cell;
use std::rc::Rc;

pub enum RetrievedImageType {
    FitsImage_R32F {
        image: FitsImage<R32F>
    },
    FitsImage_R32I {
        image: FitsImage<R32I>
    },
    FitsImage_R16I {
        image: FitsImage<R16I>
    },
    FitsImage_R8UI {
        image: FitsImage<R8UI>
    },
    PNGImage_RGBA8U {
        image: HTMLImage<RGBA8U>,
    },
    JPGImage_RGB8U {
        image: HTMLImage<RGB8U>,
    }
}

/*pub enum RequestType {
    File,
    HtmlImage,
}*/

use al_core::format::ImageFormatType;
pub trait ImageRequest<F>
where
    F: ImageFormat
{
    type I: Image<T=F>;

    fn new() -> Self;
    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue>;
    fn image(
        &self,
        tile_width: i32,
    ) -> Result<Self::I, JsValue>;
}

enum ImageRequestType {
    FitsR32FImageReq(FitsImageRequest),
    FitsR32IImageReq(FitsImageRequest),
    FitsR16IImageReq(FitsImageRequest),
    FitsR8UIImageReq(FitsImageRequest),
    PNGRGBA8UImageReq(CompressedImageRequest),
    JPGRGB8UImageReq(CompressedImageRequest),
}
impl ImageRequestType {
    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue> {
        match self {
            ImageRequestType::FitsR32FImageReq(r) => r.send(success, fail, url),
            ImageRequestType::FitsR32IImageReq(r) => r.send(success, fail, url),
            ImageRequestType::FitsR16IImageReq(r) => r.send(success, fail, url),
            ImageRequestType::FitsR8UIImageReq(r) => r.send(success, fail, url),
            ImageRequestType::PNGRGBA8UImageReq(r) => r.send(success, fail, url),
            ImageRequestType::JPGRGB8UImageReq(r) => r.send(success, fail, url),
        }
    }
    fn image(
        &self,
        tile_width: i32,
    ) -> Result<RetrievedImageType, JsValue> {
        match self {
            ImageRequestType::FitsR32FImageReq(r) => Ok(RetrievedImageType::FitsImage_R32F { image: r.image(tile_width)? }),
            ImageRequestType::FitsR32IImageReq(r) => Ok(RetrievedImageType::FitsImage_R32I { image: r.image(tile_width)? }),
            ImageRequestType::FitsR16IImageReq(r) => Ok(RetrievedImageType::FitsImage_R16I { image: r.image(tile_width)? }),
            ImageRequestType::FitsR8UIImageReq(r) => Ok(RetrievedImageType::FitsImage_R8UI { image: r.image(tile_width)? }),
            ImageRequestType::PNGRGBA8UImageReq(r) => Ok(RetrievedImageType::PNGImage_RGBA8U { image: r.image(tile_width)? }),
            ImageRequestType::JPGRGB8UImageReq(r) => Ok(RetrievedImageType::JPGImage_RGB8U { image: r.image(tile_width)? })
        }
    }
}

pub struct TileRequest {
    // Is none when it is available for downloading a new fits
    // or image tile
    req: ImageRequestType,
    time_request: Time,
    // Flag telling if the tile has been copied so that
    // the HtmlImageElement can be reused to download another tile
    ready: bool,
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

use super::Tile;
impl TileRequest {
    pub fn new(image_req: ImageRequestType) -> Self {
        // By default, we say the tile is available to be reused
        let resolved = Rc::new(Cell::new(ResolvedStatus::NotResolved));
        let tile = None;
        let closures = [
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
        ];
        let ready = true;
        let time_request = Time::now();
        Self {
            req: image_req,
            resolved,
            ready,
            tile,
            closures,
            time_request,
        }
    }

    pub fn send(&mut self, tile: Tile) -> Result<(), JsValue> {
        assert!(self.is_ready());

        self.tile = Some(tile.clone());
        let Tile {
            cell,
            root_url,
            format,
        } = tile;

        self.ready = false;

        let url = {
            let HEALPixCell(depth, idx) = cell;

            let dir_idx = (idx / 10000) * 10000;

            let url = format!(
                "{}/Norder{}/Dir{}/Npix{}.{}",
                root_url,
                depth.to_string(),
                dir_idx.to_string(),
                idx.to_string(),
                format.get_ext_file()
            );

            url
        };

        let success = {
            let resolved = self.resolved.clone();

            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                resolved.set(ResolvedStatus::Found);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        let fail = {
            let resolved = self.resolved.clone();
            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                resolved.set(ResolvedStatus::Missing);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        self.resolved.set(ResolvedStatus::NotResolved);

        self.req.send(
            Some(success.as_ref().unchecked_ref()),
            Some(fail.as_ref().unchecked_ref()),
            &url,
        )?;

        self.closures = [success, fail];
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

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn set_ready(&mut self) {
        self.ready = true;
    }

    pub fn clear(&mut self) {
        self.req.send(None, None, "").unwrap();
        self.ready = true;
        self.resolved.set(ResolvedStatus::NotResolved);
        self.closures = [
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
        ];
        //self.tile = HEALPixCell(0, 13);
        self.time_request = Time::now();
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }

    pub fn get_image(
        &self,
        tile_width: i32,
    ) -> Result<RetrievedImageType, JsValue> {
        assert!(self.is_resolved());
        self.req.image(tile_width)
    }
}

impl Default for TileRequest {
    fn default() -> Self {
        TileRequest::new::<CompressedImageRequest>()
    }
}


impl Drop for TileRequest {
    fn drop(&mut self) {}
}

/* ------------------------------------------------------ */

pub struct HTMLImage<F>
where
    F: FormatImage
{
    image: web_sys::HtmlImageElement,
    size: Vector2<i32>,
    format: std::marker::PhantomData<F>
}
use cgmath::{Vector2, Vector3};
impl<F> Image for HTMLImage<F>
where 
    F: FormatImage
{
    type T = F;

    fn allocate(width: i32, pixel_fill: &<<Self as Image>::T as ImageFormat>::P) -> Self {
        unimplemented!()
    }

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        let _size = self.get_size();
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

pub struct CompressedImageRequest {
    image: web_sys::HtmlImageElement,
}

use al_core::format::{RGBA8U, RGB8U, R32F, R16I, R8UI, R32I};
trait CompressedImageFormat: ImageFormat {}
impl CompressedImageFormat for RGBA8U {}
impl CompressedImageFormat for RGB8U {}

impl<F> ImageRequest<F> for CompressedImageRequest
where
    F: CompressedImageFormat
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
    ) -> Result<(), JsValue> {
        self.image.set_src(&url);
        self.image.set_onload(success);
        self.image.set_onerror(fail);

        Ok(())
    }

    fn image(
        &self,
        _tile_width: i32
    ) -> Result<Self::I, JsValue> {
        let width = self.image.width() as i32;
        let height = self.image.height() as i32;

        let size = Vector2::new(width, height);
        Ok(
            HTMLImage {
                size,
                image: self.image.clone(),
                format: std::marker::PhantomData
            }
        )
    }
}

/* ------------------------------------------------------ */

pub struct FitsImage<F>
where
    F: ImageFormat
{
    pub blank: Option<f32>,
    pub bzero: f32,
    pub bscale: f32,

    pub image: ImageBuffer<F>,
}

use al_core::{texture::Texture2DArray, image::Image};
impl<F> Image for FitsImage<F>
where 
    F: ImageFormat
{
    type T = F;

    fn allocate(width: i32, pixel_fill: &<<Self as Image>::T as ImageFormat>::P) -> ImageBuffer<Self::T> {
        self.image.allocate(width, pixel_fill)
    }

    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        self.image.tex_sub_image_3d(textures, offset);
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        self.image.size()
    }
}

use web_sys::XmlHttpRequest;
pub struct FitsImageRequest {
    image: XmlHttpRequest,
}
use fitsrs::{DataType, Fits};
use fitsrs::{FITSHeaderKeyword, FITSKeywordValue};
use wasm_bindgen::JsValue;
use web_sys::XmlHttpRequestResponseType;

use al_core::{
    format::{R32F, R8UI, R16I, R32I},
    image::ImageBuffer,
};
use al_core::format::ImageFormat;
trait FitsImageFormat: ImageFormat {
    fn extract_image_from_fits(fits_data: fitsrs::DataType<'_>, width: i32) -> ImageBuffer<Self> where Self: Sized;
}

impl FitsImageFormat for R32F {
    fn extract_image_from_fits(fits_data: fitsrs::DataType<'_>, width: i32) -> ImageBuffer<Self> {
        if let DataType::F32(data) = fits_data {
            ImageBuffer::<R32F>::new(
                &data.0,
                width,
            )
        } else if let DataType::F64(data) = fits_data {
            let data = data.0.into_iter().map(|v| v as f32).collect::<Vec<_>>();
            ImageBuffer::<R32F>::new(
                &data,
                width,
            )
        } else {
            unreachable!()
        }
    }
}
impl FitsImageFormat for R32I {
    fn extract_image_from_fits(fits_data: fitsrs::DataType<'_>, width: i32) -> ImageBuffer<Self> {
        if let DataType::I32(data) = fits_data {
            ImageBuffer::<R32I>::new(
                &data.0,
                width,
            )
        } else {
            unreachable!()
        }
    }
}
impl FitsImageFormat for R16I {
    fn extract_image_from_fits(fits_data: fitsrs::DataType<'_>, width: i32) -> ImageBuffer<Self> {
        if let DataType::I16(data) = fits_data {
            ImageBuffer::<R16I>::new(
                &data.0,
                width,
            )
        } else {
            unreachable!()
        }
    }
}
impl FitsImageFormat for R8UI {
    fn extract_image_from_fits(fits_data: fitsrs::DataType<'_>, width: i32) -> ImageBuffer<Self> {
        if let DataType::U8(data) = fits_data {
            ImageBuffer::<R8UI>::new(
                &data.0,
                width,
            )
        } else {
            unreachable!()
        }
    }
}

impl<F> ImageRequest<F> for FitsImageRequest
where
    F: FitsImageFormat
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
    ) -> Result<(), JsValue> {
        self.image.open_with_async("GET", url, true)?;
        self.image.set_onload(success);
        self.image.set_onerror(fail);

        self.image.send().unwrap();

        Ok(())
    }

    fn image(
        &self,
        size: i32,
    ) -> Result<Self::I, JsValue> {
        // We know at this point the request is resolved
        let array_buf = js_sys::Uint8Array::new(self.image.response().unwrap().as_ref());
        let bytes = &array_buf.to_vec();
        let Fits { data, header } = Fits::from_byte_slice(bytes).map_err(|e| {
            JsValue::from_str(&format!(
                "Parsing FITS error of {:?}: {:?}",
                self.image.response_url(),
                e
            ))
        })?;
        
        let image = F::extract_image_from_fits(data, size);

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
            Some(*blank as f32)
        } else {
            Some(std::f32::NAN)
        };

        Ok(FitsImage {
            // data
            image,
            // meta
            blank,
            bscale,
            bzero
        })
    }
}

