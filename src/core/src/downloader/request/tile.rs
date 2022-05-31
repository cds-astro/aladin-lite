use crate::{healpix::cell::HEALPixCell, survey::config::HiPSConfig};
use al_core::image::format::ImageFormatType;

use crate::downloader::{query, request};
use al_core::image::{bitmap::Bitmap, fits::Fits, ImageType};

use super::{Request, RequestType};
pub struct TileRequest {
    pub cell: HEALPixCell,
    pub hips_url: Url,
    pub url: Url,

    request: Request<ImageType>,
}

impl From<TileRequest> for RequestType {
    fn from(request: TileRequest) -> Self {
        RequestType::Tile(request)
    }
}

use super::ResolvedStatus;
use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, RequestInit, RequestMode, Response};

use wasm_bindgen::JsCast;
impl From<query::Tile> for TileRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Tile) -> Self {
        let query::Tile {
            format,
            cell,
            url,
            hips_url,
        } = query;

        let url_clone = url.clone();

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

                let blob = JsFuture::from(resp.blob()?).await?.into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?)
                    .await?
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

                let blob = JsFuture::from(resp.blob()?).await?.into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?)
                    .await?
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
                let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::<al_core::image::format::R32F>::new(&bytes)?;
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

                let blob: Blob = JsFuture::from(resp.blob()?).await?.into();
                let array_buffer = JsFuture::from(blob.array_buffer()).await?;

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
                let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

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
                let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

                let bytes = js_sys::Uint8Array::new(&array_buffer);
                let image = Fits::new(&bytes)?;
                Ok(ImageType::FitsImageR8ui { image })
            }),
            _ => todo!(),
        };

        Self {
            cell,
            hips_url,
            url,
            request,
        }
    }
}

use crate::time::Time;
use std::sync::{Arc, Mutex};
pub struct Tile {
    pub image: Arc<Mutex<Option<ImageType>>>,
    pub time_req: Time,
    pub cell: HEALPixCell,
    hips_url: Url,
    url: Url,
}

impl Tile {
    pub fn missing(&self) -> bool {
        self.image.lock().unwrap().is_none()
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
        let TileRequest {
            cell,
            request,
            hips_url,
            url,
        } = request;
        if request.is_resolved() {
            let Request::<ImageType> {
                time_request, data, ..
            } = request;
            Some(Tile {
                cell: *cell,
                time_req: *time_request,
                // This is a clone on a Arc, it is supposed to be fast
                image: data.clone(),
                hips_url: hips_url.clone(),
                url: url.clone(),
            })
        } else {
            None
        }
    }
}
