use al_core::image::format::ImageFormatType;

use crate::downloader::{query};
use al_core::image::fits::Fits;

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub blank: f32,
    pub scale: f32,
    pub offset: f32,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            blank: -1.0,
            scale: 1.0,
            offset: 0.0
        }
    }
}

use super::{Request, RequestType};
use crate::downloader::QueryId;

pub struct PixelMetadataRequest {
    pub id: QueryId,
    pub url: Url,
    pub hips_url: Url,
    request: Request<Metadata>,
}

impl From<PixelMetadataRequest> for RequestType {
    fn from(request: PixelMetadataRequest) -> Self {
        RequestType::PixelMetadata(request)
    }
}


use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, RequestInit, RequestMode, Response};
use crate::downloader::query::Query;
use wasm_bindgen::JsCast;
impl From<query::PixelMetadata> for PixelMetadataRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::PixelMetadata) -> Self {
        let id = query.id();
        let query::PixelMetadata {
            format,
            url,
            hips_url,
        } = query;

        let url_clone = url.clone();

        let window = web_sys::window().unwrap();
        let request = match format {
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
                Ok(Metadata {
                    blank: image.blank,
                    scale: image.bscale,
                    offset: image.bzero,
                })
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
                let image = Fits::<al_core::image::format::R32I>::new(&bytes)?;
                Ok(Metadata {
                    blank: image.blank,
                    scale: image.bscale,
                    offset: image.bzero,
                })
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
                let image = Fits::<al_core::image::format::R16I>::new(&bytes)?;
                Ok(Metadata {
                    blank: image.blank,
                    scale: image.bscale,
                    offset: image.bzero,
                })
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
                let image = Fits::<al_core::image::format::R8UI>::new(&bytes)?;
                Ok(Metadata {
                    blank: image.blank,
                    scale: image.bscale,
                    offset: image.bzero,
                })
            }),
            _ => Request::new(async move { Ok(Metadata::default()) }),
        };

        Self {
            id,
            url,
            hips_url,
            request,
        }
    }
}

#[derive(Debug)]
pub struct PixelMetadata {
    pub value: Metadata,
    pub hips_url: String,
    pub url: String,
}

impl<'a> From<&'a PixelMetadataRequest> for Option<PixelMetadata> {
    fn from(request: &'a PixelMetadataRequest) -> Self {
        let PixelMetadataRequest {
            request,
            hips_url,
            url,
            ..
        } = request;
        if request.is_resolved() {
            let Request::<Metadata> {
                data,
                ..
            } = request;
            // It will always be resolved and found as we will request a well know tile (Norder0/Tile0)
            Some(PixelMetadata {
                hips_url: hips_url.clone(),
                url: url.to_string(),
                value: data.lock().unwrap().unwrap().clone(),
            })
        } else {
            None
        }
    }
}
