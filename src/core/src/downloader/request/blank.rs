use al_core::image::format::ImageFormatType;
use std::io::Cursor;

use crate::downloader::{query};
use fitsrs::{
    fits::Fits,
};

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


use crate::renderable::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response};
use crate::downloader::query::Query;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
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

        let window = web_sys::window().unwrap_abort();
        let request = match format {
            ImageFormatType::R32F | ImageFormatType::R32I | ImageFormatType::R16I | ImageFormatType::R8UI => Request::new(async move {
                let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);

                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
                // See https://github.com/MattiasBuelens/wasm-streams/blob/f6dacf58a8826dc67923ab4a3bae87635690ca64/examples/fetch_as_stream.rs#L25-L33
                /*let raw_body = resp.body().ok_or(JsValue::from_str("Cannot extract readable stream"))?;
                let body = ReadableStream::from_raw(raw_body.dyn_into()?);

                // Convert the JS ReadableStream to a Rust stream
                let mut reader = body.try_into_async_read().map_err(|_| JsValue::from_str("readable stream locked"))?;
                let image = Fits::new(reader).await?;*/

                let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
                let bytes_buffer = js_sys::Uint8Array::new(&array_buffer);

                let num_bytes = bytes_buffer.length() as usize;
                let mut raw_bytes = Vec::with_capacity(num_bytes);
                unsafe { raw_bytes.set_len(num_bytes); }
                bytes_buffer.copy_to(&mut raw_bytes[..]);

                let mut reader = Cursor::new(&raw_bytes[..]);
                let Fits { hdu } = Fits::from_reader(&mut reader)
                    .map_err(|_| {
                        JsValue::from_str("Parsing fits error")
                    })?;

                let header = hdu.get_header();
                let scale = if let Some(fitsrs::card::Value::Float(bscale)) = header.get(b"BSCALE  ") {
                    *bscale as f32
                } else {
                    1.0
                };
                let offset = if let Some(fitsrs::card::Value::Float(bzero)) = header.get(b"BZERO   ") {
                    *bzero as f32
                } else {
                    0.0
                };
                let blank = if let Some(fitsrs::card::Value::Float(blank)) = header.get(b"BLANK   ") {
                    *blank as f32
                } else {
                    std::f32::NAN
                };

                Ok(Metadata { blank, scale, offset })
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

use std::sync::{Mutex, Arc};
#[derive(Debug)]
pub struct PixelMetadata {
    pub value: Arc<Mutex<Option<Metadata>>>,
    pub hips_url: String,
    pub url: String,
}
use crate::Abort;
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
                value: data.clone(),
            })
        } else {
            None
        }
    }
}
