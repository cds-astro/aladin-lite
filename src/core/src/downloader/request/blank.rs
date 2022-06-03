use al_core::image::format::ImageFormatType;

use crate::downloader::{query};
use al_core::image::fits::Fits;

use super::{Request, RequestType};
pub struct BlankRequest {
    pub url: Url,
    pub hips_url: Url,
    request: Request<f32>,
}

impl From<BlankRequest> for RequestType {
    fn from(request: BlankRequest) -> Self {
        RequestType::Blank(request)
    }
}


use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, RequestInit, RequestMode, Response};

use wasm_bindgen::JsCast;
impl From<query::Blank> for BlankRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Blank) -> Self {
        let query::Blank {
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
                Ok(image.blank)
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
                Ok(image.blank)
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
                Ok(image.blank)
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
                Ok(image.blank)
            }),
            _ => Request::new(async move { Ok(-1.0) }),
        };

        Self {
            url,
            hips_url,
            request,
        }
    }
}

#[derive(Debug)]
pub struct Blank {
    pub value: f32,
    pub hips_url: String,
}

impl<'a> From<&'a BlankRequest> for Option<Blank> {
    fn from(request: &'a BlankRequest) -> Self {
        let BlankRequest {
            request,
            hips_url,
            ..
        } = request;
        if request.is_resolved() {
            let Request::<f32> {
                data,
                ..
            } = request;
            // It will always be resolved and found as we will request a well know tile (Norder0/Tile0)
            Some(Blank {
                hips_url: hips_url.clone(),
                value: data.lock().unwrap().unwrap().clone()
            })
        } else {
            None
        }
    }
}
