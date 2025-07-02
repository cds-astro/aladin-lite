use crate::healpix::cell::HEALPixCell;
use crate::renderable::CreatorDid;
use al_core::image::format::ImageFormatType;
use al_core::texture::format::{PixelType, RGB8U, RGBA8U};

use crate::downloader::query;
use al_core::image::ImageType;

use super::Url;
use super::{Request, RequestType};
use crate::downloader::request::query_html_image;
use crate::downloader::QueryId;
pub struct TileRequest {
    pub request: Request<ImageType>,
    pub id: QueryId,

    pub cell: HEALPixCell,
    pub hips_cdid: CreatorDid,
    pub url: Url,
    pub format: ImageFormatType,
    pub channel: Option<u32>,
}

impl From<TileRequest> for RequestType {
    fn from(request: TileRequest) -> Self {
        RequestType::Tile(request)
    }
}

use al_core::image::html::HTMLImage;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Response};
impl From<query::Tile> for TileRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Tile) -> Self {
        let query::Tile {
            format,
            cell,
            url,
            hips_cdid,
            credentials,
            mode,
            id,
            channel,
            size,
            depth,
        } = query;

        let url_clone = url.clone();
        let pixel_format = format.get_pixel_format();

        let window = web_sys::window().unwrap_abort();
        let request = match pixel_format {
            PixelType::RGB8U => Request::new(async move {
                // HTMLImageElement
                let image = query_html_image(&url_clone, credentials).await?;
                // The image has been resolved
                Ok(ImageType::HTMLImageRgb8u {
                    image: HTMLImage::<RGB8U>::new(image),
                })
            }),
            PixelType::RGBA8U => Request::new(async move {
                // HTMLImageElement
                let image = query_html_image(&url_clone, credentials).await?;
                // The image has been resolved
                Ok(ImageType::HTMLImageRgba8u {
                    image: HTMLImage::<RGBA8U>::new(image),
                })
            }),
            PixelType::R32F | PixelType::R32I | PixelType::R16I | PixelType::R8U => {
                Request::new(async move {
                    let mut opts = RequestInit::new();
                    opts.method("GET");
                    opts.mode(mode);
                    opts.credentials(credentials);

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

                        Ok(ImageType::FitsRawBytes {
                            raw_bytes,
                            size: (size, size, depth),
                        })
                    } else {
                        Err(JsValue::from_str(
                            "Response status code not between 200-299.",
                        ))
                    }
                })
            }
        };

        Self {
            cell,
            format,
            id,
            hips_cdid,
            url,
            request,
            channel,
        }
    }
}

use crate::time::Time;
use std::cell::RefCell;
use std::rc::Rc;

use crate::Abort;
