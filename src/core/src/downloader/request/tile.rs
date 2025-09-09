use crate::renderable::CreatorDid;
use al_core::image::format::ImageFormatType;
use al_core::texture::format::PixelType;

use crate::downloader::query;
use al_core::image::ImageType;

use super::super::query::CellDesc;
use super::Url;
use super::{Request, RequestType};
use crate::downloader::request::query_html_image;
use crate::downloader::QueryId;

pub struct TileRequest {
    pub request: Request<ImageType>,
    pub id: QueryId,

    pub cell: CellDesc,
    pub hips_cdid: CreatorDid,
    pub url: Url,
    pub format: ImageFormatType,
}

impl From<TileRequest> for RequestType {
    fn from(request: TileRequest) -> Self {
        RequestType::Tile(request)
    }
}

use crate::downloader::request::query_bitmap_from_blob;
use al_core::image::bitmap::Bitmap;
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
            create_bitmap_support,
        } = query;

        let url_clone = url.clone();
        let pixel_format = format.get_pixel_format();

        let size = match cell {
            CellDesc::HiPS2D { tile_size, .. } | CellDesc::HiPSCube { tile_size, .. } => {
                (tile_size, tile_size, 1)
            }
            CellDesc::HiPS3D {
                tile_size,
                tile_depth,
                ..
            } => (tile_size, tile_size, tile_depth),
        };

        let request = match pixel_format {
            PixelType::RGB8U => Request::new(async move {
                if create_bitmap_support {
                    // optimized download of tile for GPU (using Blob + Bitmap) without creating any DOM structure
                    let image_bitmap =
                        query_bitmap_from_blob(&url_clone, mode, credentials).await?;
                    Ok(ImageType::ImageRgb8u {
                        image: Bitmap::new(image_bitmap),
                    })
                } else {
                    // HTMLImageElement
                    let image = query_html_image(&url_clone, credentials).await?;
                    // The image has been resolved
                    Ok(ImageType::HTMLImageRgb8u {
                        image: HTMLImage::new(image),
                    })
                }
            }),
            PixelType::RGBA8U => Request::new(async move {
                if create_bitmap_support {
                    // optimized download of tile for GPU (using Blob + Bitmap) without creating any DOM structure
                    let image_bitmap =
                        query_bitmap_from_blob(&url_clone, mode, credentials).await?;
                    Ok(ImageType::ImageRgba8u {
                        image: Bitmap::new(image_bitmap),
                    })
                } else {
                    // HTMLImageElement
                    let image = query_html_image(&url_clone, credentials).await?;
                    // The image has been resolved
                    Ok(ImageType::HTMLImageRgba8u {
                        image: HTMLImage::new(image),
                    })
                }
            }),
            PixelType::R32F | PixelType::R32I | PixelType::R16I | PixelType::R8U => {
                Request::new(async move {
                    let window = web_sys::window().unwrap_abort();

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

                        Ok(ImageType::FitsRawBytes { raw_bytes, size })
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
        }
    }
}

use crate::Abort;
