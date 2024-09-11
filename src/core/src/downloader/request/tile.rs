use crate::healpix::cell::HEALPixCell;
use crate::renderable::CreatorDid;
use al_core::image::format::{ChannelType, ImageFormatType, RGB8U, RGBA8U};

use crate::downloader::query;
use al_core::image::ImageType;

use super::Url;
use super::{Request, RequestType};
use crate::downloader::QueryId;

pub struct TileRequest {
    pub id: QueryId,

    cell: HEALPixCell,
    hips_cdid: CreatorDid,
    url: Url,
    format: ImageFormatType,

    request: Request<ImageType>,
}

impl From<TileRequest> for RequestType {
    fn from(request: TileRequest) -> Self {
        RequestType::Tile(request)
    }
}

async fn query_html_image(url: &str) -> Result<web_sys::HtmlImageElement, JsValue> {
    let image = web_sys::HtmlImageElement::new().unwrap_abort();
    let image_cloned = image.clone();

    let promise = js_sys::Promise::new(
        &mut (Box::new(move |resolve, reject| {
            // Ask for CORS permissions
            image_cloned.set_cross_origin(Some(""));
            image_cloned.set_onload(Some(&resolve));
            image_cloned.set_onerror(Some(&reject));
            image_cloned.set_src(&url);
        }) as Box<dyn FnMut(js_sys::Function, js_sys::Function)>),
    );

    let _ = JsFuture::from(promise).await?;

    Ok(image)
}

use al_core::image::html::HTMLImage;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response};
impl From<query::Tile> for TileRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Tile) -> Self {
        let query::Tile {
            format,
            cell,
            url,
            hips_cdid,
            id,
        } = query;

        let url_clone = url.clone();
        let channel = format.get_channel();

        let window = web_sys::window().unwrap_abort();
        let request = match channel {
            ChannelType::RGB8U => Request::new(async move {
                /*let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);

                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;*/

                /*/// Bitmap version
                let blob = JsFuture::from(resp.blob()?).await?.into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?)
                    .await?
                    .into();

                let image = Bitmap::new(image);
                Ok(ImageType::JpgImageRgb8u { image })*/
                /*
                /// Raw image decoding

                let buf = JsFuture::from(resp.array_buffer()?).await?;
                let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                let image = ImageBuffer::<RGB8U>::from_raw_bytes(&raw_bytes[..], 512, 512)?;

                Ok(ImageType::RawRgb8u { image })
                */
                // HTMLImageElement
                let image = query_html_image(&url_clone).await?;
                // The image has been resolved
                Ok(ImageType::HTMLImageRgb8u {
                    image: HTMLImage::<RGB8U>::new(image),
                })
            }),
            ChannelType::RGBA8U => Request::new(async move {
                /*let mut opts = RequestInit::new();
                opts.method("GET");
                opts.mode(RequestMode::Cors);

                let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;*/

                /*/// Bitmap version
                let blob = JsFuture::from(resp.blob()?).await?.into();
                let image = JsFuture::from(window.create_image_bitmap_with_blob(&blob)?)
                    .await?
                    .into();

                let image = Bitmap::new(image);
                Ok(ImageType::PngImageRgba8u { image })*/

                /*
                /// Raw image decoding
                let buf = JsFuture::from(resp.array_buffer()?).await?;
                let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                let image = ImageBuffer::<RGBA8U>::from_raw_bytes(&raw_bytes[..], 512, 512)?;

                Ok(ImageType::RawRgba8u { image })
                */
                // HTMLImageElement
                let image = query_html_image(&url_clone).await?;
                // The image has been resolved
                Ok(ImageType::HTMLImageRgba8u {
                    image: HTMLImage::<RGBA8U>::new(image),
                })
            }),
            ChannelType::R32F
            | ChannelType::R64F
            | ChannelType::R32I
            | ChannelType::R16I
            | ChannelType::R8UI => Request::new(async move {
                let opts = RequestInit::new();
                opts.set_method("GET");
                opts.set_mode(RequestMode::Cors);

                let request =
                    web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
                let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
                // See https://github.com/MattiasBuelens/wasm-streams/blob/f6dacf58a8826dc67923ab4a3bae87635690ca64/examples/fetch_as_stream.rs#L25-L33
                /*let raw_body = resp.body().ok_or(JsValue::from_str("Cannot extract readable stream"))?;
                let body = ReadableStream::from_raw(raw_body.dyn_into()?);

                // Convert the JS ReadableStream to a Rust stream
                let mut reader = body.try_into_async_read().map_err(|_| JsValue::from_str("readable stream locked"))?;
                let image = Fits::new(reader).await?;
                */
                if resp.ok() {
                    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
                    let raw_bytes = js_sys::Uint8Array::new(&array_buffer);

                    Ok(ImageType::FitsImage { raw_bytes })
                } else {
                    Err(JsValue::from_str(
                        "Response status code not between 200-299.",
                    ))
                }
            }),
            _ => todo!(),
        };

        Self {
            cell,
            format,
            id,
            hips_cdid,
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
    pub format: ImageFormatType,
    hips_cdid: CreatorDid,
    url: Url,
}

use crate::Abort;
impl Tile {
    #[inline(always)]
    pub fn missing(&self) -> bool {
        self.image.lock().unwrap_abort().is_none()
    }

    #[inline(always)]
    pub fn get_hips_cdid(&self) -> &CreatorDid {
        &self.hips_cdid
    }

    #[inline(always)]
    pub fn get_url(&self) -> &Url {
        &self.url
    }

    #[inline(always)]
    pub fn cell(&self) -> &HEALPixCell {
        &self.cell
    }

    /*#[inline(always)]
    pub fn query(&self) -> query::Tile {
        query::Tile::new(&self.cell, self.hips_url.clone(), self.format)
    }*/
}

impl<'a> From<&'a TileRequest> for Option<Tile> {
    fn from(request: &'a TileRequest) -> Self {
        let TileRequest {
            cell,
            request,
            hips_cdid,
            url,
            format,
            ..
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
                hips_cdid: hips_cdid.clone(),
                url: url.clone(),
                format: *format,
            })
        } else {
            None
        }
    }
}
