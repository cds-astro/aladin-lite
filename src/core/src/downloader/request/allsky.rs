use crate::downloader::query;
use crate::renderable::CreatorDid;
use al_core::image::fits::FitsImage;
use al_core::image::ImageType;
use al_core::texture::format::PixelType;
use fitsrs::hdu::header::Bitpix;

use super::{Request, RequestType};
use crate::downloader::QueryId;
pub struct AllskyRequest {
    pub hips_cdid: CreatorDid,
    pub url: Url,
    //pub depth_tile: u8,
    pub id: QueryId,
    pub channel: Option<u32>,

    pub request: Request<Vec<ImageType>>,
}

impl AllskyRequest {
    pub fn missing(&self) -> bool {
        self.request.data.borrow().is_none()
    }
}

impl From<AllskyRequest> for RequestType {
    fn from(request: AllskyRequest) -> Self {
        RequestType::Allsky(request)
    }
}

use super::Url;

use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestCredentials, RequestInit, Response};

use al_core::{image::raw::ImageBuffer, texture::pixel::Pixel};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

async fn query_allsky(
    url: &str,
    credentials: RequestCredentials,
) -> Result<ImageBuffer<RGBA8U>, JsValue> {
    let image = super::query_html_image(url, credentials).await?;

    // The image has been received here
    let document = web_sys::window().unwrap_abort().document().unwrap_abort();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_width(image.width());
    canvas.set_height(image.height());
    let context = canvas
        .get_context("2d")?
        .unwrap_abort()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    context.draw_image_with_html_image_element(&image, 0.0, 0.0)?;

    let w = image.width();
    let h = image.height();
    let image_data = context.get_image_data(0.0, 0.0, w as f64, h as f64)?;

    let raw_bytes = image_data.data();

    Ok(ImageBuffer::from_raw_bytes(raw_bytes.0, w, h))
}

impl From<query::Allsky> for AllskyRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Allsky) -> Self {
        let query::Allsky {
            format,
            tile_size,
            url,
            hips_cdid,
            allsky_tile_size,
            id,
            credentials,
            mode,
            channel: slice,
        } = query;

        //let depth_tile = crate::math::utils::log_2_unchecked(texture_size / tile_size) as u8;
        let channel = format.get_pixel_format();
        let url_clone = url.clone();

        let request = Request::new(async move {
            match channel {
                PixelType::RGB8U => {
                    let allsky = query_allsky(&url_clone, credentials).await?;

                    let allsky_tiles =
                        handle_allsky_file::<RGBA8U>(allsky, allsky_tile_size, tile_size)?
                            .map(|image| {
                                let ImageBuffer { data, size } = image;
                                let data = data
                                    .iter()
                                    .enumerate()
                                    .filter(|&(i, _)| i % 4 != 3)
                                    .map(|(_, v)| *v)
                                    .collect::<Vec<_>>();

                                let image = ImageBuffer::new(
                                    data.into_boxed_slice(),
                                    size.0,
                                    size.1,
                                    size.2,
                                );

                                ImageType::RawRgb8u { image }
                            })
                            .collect();

                    Ok(allsky_tiles)
                }
                PixelType::RGBA8U => {
                    let allsky = query_allsky(&url_clone, credentials).await?;

                    let allsky_tiles = handle_allsky_file(allsky, allsky_tile_size, tile_size)?
                        .map(|image| ImageType::RawRgba8u { image })
                        .collect();

                    Ok(allsky_tiles)
                }
                _ => {
                    let mut opts = RequestInit::new();
                    opts.method("GET");
                    opts.mode(mode);
                    opts.credentials(credentials);
                    let window = web_sys::window().unwrap_abort();

                    let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts)?;
                    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                    // `resp_value` is a `Response` object.
                    debug_assert!(resp_value.is_instance_of::<Response>());
                    let resp: Response = resp_value.dyn_into()?;
                    // See https://github.com/MattiasBuelens/wasm-streams/blob/f6dacf58a8826dc67923ab4a3bae87635690ca64/examples/fetch_as_stream.rs#L25-L33
                    /*let raw_body = resp.body().ok_or(JsValue::from_str("Cannot extract readable stream"))?;
                    let body = ReadableStream::from_raw(raw_body.dyn_into()?);

                    // Convert the JS ReadableStream to a Rust stream
                    let mut reader = body.try_into_async_read().map_err(|_| JsValue::from_str("readable stream locked"))?;*/

                    let buf = JsFuture::from(resp.array_buffer()?).await?;
                    let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();

                    let FitsImage {
                        raw_bytes, bitpix, ..
                    } = &FitsImage::from_raw_bytes(raw_bytes.as_slice())?[0];
                    match bitpix {
                        Bitpix::U8 => {
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawR8ui { image })
                                .collect())
                        }
                        Bitpix::I16 => {
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawR16i { image })
                                .collect())
                        }
                        Bitpix::I32 => {
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawR32i { image })
                                .collect())
                        }
                        Bitpix::I64 => {
                            let data = unsafe {
                                std::slice::from_raw_parts(
                                    raw_bytes.as_ptr() as *const i64,
                                    raw_bytes.len() / 8,
                                )
                            };
                            let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
                            let raw_bytes = unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const u8,
                                    data.len() * 4,
                                )
                            };
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawR32i { image })
                                .collect())
                        }
                        Bitpix::F32 => {
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawRgba8u { image })
                                .collect())
                        }
                        Bitpix::F64 => {
                            let data = unsafe {
                                std::slice::from_raw_parts(
                                    raw_bytes.as_ptr() as *const f64,
                                    raw_bytes.len() / 8,
                                )
                            };
                            let data = data.iter().map(|v| *v as f32).collect::<Vec<_>>();
                            let raw_bytes = unsafe {
                                std::slice::from_raw_parts(
                                    data.as_ptr() as *const u8,
                                    data.len() * 4,
                                )
                            };
                            Ok(handle_allsky_fits(raw_bytes, tile_size, allsky_tile_size)?
                                .map(|image| ImageType::RawRgba8u { image })
                                .collect())
                        }
                    }
                }
            }
        });

        Self {
            id,
            hips_cdid,
            url,
            request,
            channel: slice,
        }
    }
}

use al_core::image::raw::ImageBufferView;
use al_core::texture::format::TextureFormat;
fn handle_allsky_file<F: TextureFormat>(
    image: ImageBuffer<F>,
    allsky_tile_size: i32,
    tile_size: i32,
) -> Result<impl Iterator<Item = ImageBuffer<F>>, JsValue> {
    let d3_tile_allsky_size = std::cmp::min(tile_size, 64);

    let mut src_idx = 0;
    let tiles = (0..12).map(move |_| {
        let mut base_tile = ImageBuffer::<F>::allocate(
            &F::P::BLACK,
            allsky_tile_size as u32,
            allsky_tile_size as u32,
        );
        for idx_tile in 0..64 {
            let (x, y) = crate::utils::unmortonize(idx_tile as u64);
            let dx = x * (d3_tile_allsky_size as u32);
            let dy = y * (d3_tile_allsky_size as u32);

            let sx = (src_idx % 27) * d3_tile_allsky_size;
            let sy = (src_idx / 27) * d3_tile_allsky_size;
            let s = ImageBufferView {
                x: sx,
                y: sy,
                w: d3_tile_allsky_size,
                h: d3_tile_allsky_size,
            };
            let d = ImageBufferView {
                x: dx as i32,
                y: dy as i32,
                w: d3_tile_allsky_size,
                h: d3_tile_allsky_size,
            };

            base_tile.tex_sub(&image, &s, &d);

            src_idx += 1;
        }

        base_tile
    });

    Ok(tiles)
}

fn handle_allsky_fits<F: TextureFormat>(
    image: &[<F::P as Pixel>::Item],

    tile_size: i32,
    allsky_tile_size: i32,
) -> Result<impl Iterator<Item = ImageBuffer<F>>, JsValue> {
    let d3_tile_allsky_size = std::cmp::min(tile_size, 64);
    let width_allsky_px = 27 * d3_tile_allsky_size;
    let height_allsky_px = 29 * d3_tile_allsky_size;
    // The fits image layout stores rows in reverse
    let reversed_rows_data = image
        .chunks(width_allsky_px as usize * F::NUM_CHANNELS)
        .rev()
        .flatten()
        .copied()
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let image = ImageBuffer::<F>::new(
        reversed_rows_data,
        width_allsky_px as u32,
        height_allsky_px as u32,
        1,
    );

    let allsky_tiles_iter =
        handle_allsky_file::<F>(image, allsky_tile_size, tile_size)?.map(move |image| {
            // The GPU does a specific transformation on the UV for FITS tiles
            // We must revert this to be compatible with this GPU transformation
            let new_image_data = image
                .get_data()
                .chunks((allsky_tile_size * allsky_tile_size) as usize * F::NUM_CHANNELS)
                .flat_map(|c| {
                    c.chunks(allsky_tile_size as usize * F::NUM_CHANNELS)
                        .rev()
                        .flatten()
                })
                .cloned()
                .collect();

            ImageBuffer::<F>::new(
                new_image_data,
                allsky_tile_size as u32,
                allsky_tile_size as u32,
                1,
            )
        });

    Ok(allsky_tiles_iter)
}

use al_core::texture::format::RGBA8U;

use crate::Abort;
