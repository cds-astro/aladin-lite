use crate::{
    survey::config::HiPSConfig,
    healpix::cell::HEALPixCell
};
use al_core::format::ImageFormatType;

use crate::downloader::{
    request::{
        self,
        image::{
            ImageType,
            bitmap::Bitmap,
            fits::Fits
        }
    },
    query,
};

use super::{Request, RequestType2};

pub struct TileRequest {
    pub cell: HEALPixCell,
    pub hips_url: Url,
    pub url: Url,

    request: Request<ImageType>,
}

impl From<TileRequest> for RequestType2 {
    fn from(request: TileRequest) -> Self {
        RequestType2::Tile(request)
    }
}

use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Response, RequestInit, RequestMode};
use crate::survey::Url;
use super::ResolvedStatus;

use wasm_bindgen::JsCast;
impl<'a, 'b> From<query::Tile<'a, 'b>> for TileRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Tile<'a, 'b>) -> Self {
        let query::Tile { cfg, cell, url, hips_url } = query;

        // Retrieve the url from the config
        let url_clone = url.clone();
        let format = cfg.get_format();

        let window = web_sys::window().unwrap();
        let request = match format {
            ImageFormatType::RGB8U => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;

                let blob = JsFuture::from(resp.blob()?).await?
                    .into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?).await?
                    .into();

                let image = Bitmap::new(image);
                Ok(ImageType::JpgImageRgb8u { image })
            }),
            ImageFormatType::RGBA8U => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;

                let blob = JsFuture::from(resp.blob()?).await?
                    .into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?).await?
                    .into();

                let image = Bitmap::new(image);
                Ok(ImageType::PngImageRgba8u { image })
            }),
            ImageFormatType::R32F => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
                let array_buffer =  JsFuture::from(resp.array_buffer()?).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::<al_core::format::R32F>::new(&bytes)?;
                Ok(ImageType::FitsImageR32f { image })
            }),
            ImageFormatType::R32I => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;

                let blob: Blob = JsFuture::from(resp.blob()?).await?
                    .into();
                let array_buffer =  JsFuture::from(blob.array_buffer()).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::new(&bytes)?;
                Ok(ImageType::FitsImageR32i { image })
            }),
            ImageFormatType::R16I => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
                let array_buffer =  JsFuture::from(resp.array_buffer()?).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::new(&bytes)?;
                Ok(ImageType::FitsImageR16i { image })
            }),
            ImageFormatType::R8UI => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);
    
                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
                let array_buffer =  JsFuture::from(resp.array_buffer()?).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::new(&bytes)?;
                Ok(ImageType::FitsImageR8ui { image })
            }),
            _ => todo!(),
        };

        Self {
            cell: *cell,
            hips_url,
            url,
            request
        }
    }
}

use std::sync::{Arc, Mutex};
use crate::time::Time;
pub struct Tile {
    pub image: Arc<Mutex<Option<ImageType>>>,
    pub time_req: Time,
    pub cell: HEALPixCell,
    hips_url: Url,
    url: Url,
}

impl Tile {
    pub fn missing(&self) -> bool {
        self.image.lock()
            .unwrap()
            .is_none()
    }

    pub fn get_hips_url(&self) -> &Url {
        &self.hips_url
    }

    pub fn get_url(&self) -> &Url {
        &self.url
    }
}

impl<'a> From<&'a TileRequest> for Option<Tile> {
    fn from(request: &'a TileRequest) -> Self {
        let TileRequest { cell, request, hips_url, url} = request;
        if request.is_resolved() {
            let Request::<ImageType> { time_request, data, .. } = request;
            Some(Tile {
                cell: *cell,
                time_req: *time_request,
                // This is a clone on a Arc, it is supposed to be fast
                image: data.clone(),
                hips_url: hips_url.clone(),
                url: url.clone()
            })
        } else {
            None
        }
    }
}

/*
    FitsImageR32f { image: FitsImage<R32F> },
    FitsImageR32i { image: FitsImage<R32I> },
    FitsImageR16i { image: FitsImage<R16I> },
    FitsImageR8ui { image: FitsImage<R8UI> },
    PngImageRgba8u { image: ImageBitmap<RGBA8U> },
    JpgImageRgb8u { image: ImageBitmap<RGB8U> },
*/

/*
// Order the by importance (lower resolution tiles will be downloaded first)
impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let d0 = self.cell.depth();
        let d1 = other.cell.depth();

        d0.partial_cmp(&d1)
            .map(|ord| {
                match ord {
                    Ordering::Equal => {
                        let idx0 = self.cell.idx();
                        let idx1 = other.cell.idx();

                        idx0.partial_cmp(&idx1).unwrap()
                    },
                    _ => ord
                }
            })
    }
}*/