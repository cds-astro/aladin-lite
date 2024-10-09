use std::io::Cursor;

use crate::downloader::query;
use crate::renderable::CreatorDid;
use al_core::image::format::ChannelType;
use al_core::image::ImageType;

use fitsrs::{fits::Fits, hdu::data::InMemData};

use super::{Request, RequestType};
use crate::downloader::QueryId;
pub struct AllskyRequest {
    pub hips_cdid: CreatorDid,
    pub url: Url,
    pub depth_tile: u8,
    pub id: QueryId,

    request: Request<Vec<ImageType>>,
}

impl From<AllskyRequest> for RequestType {
    fn from(request: AllskyRequest) -> Self {
        RequestType::Allsky(request)
    }
}

use super::Url;

use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response};

use al_core::{image::raw::ImageBuffer, texture::pixel::Pixel};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

async fn query_image(url: &str) -> Result<ImageBuffer<RGBA8U>, JsValue> {
    let image = web_sys::HtmlImageElement::new().unwrap_abort();
    let image_cloned = image.clone();

    let html_img_elt_promise = js_sys::Promise::new(
        &mut (Box::new(move |resolve, reject| {
            image_cloned.set_cross_origin(Some(""));
            image_cloned.set_onload(Some(&resolve));
            image_cloned.set_onerror(Some(&reject));
            image_cloned.set_src(&url);
        }) as Box<dyn FnMut(js_sys::Function, js_sys::Function)>),
    );

    let _ = JsFuture::from(html_img_elt_promise).await?;

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

    Ok(ImageBuffer::from_raw_bytes(raw_bytes.0, w as i32, h as i32))
}

impl From<query::Allsky> for AllskyRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Allsky) -> Self {
        let query::Allsky {
            format,
            tile_size,
            url,
            hips_cdid,
            texture_size,
            id,
        } = query;

        let depth_tile = crate::math::utils::log_2_unchecked(texture_size / tile_size) as u8;
        let channel = format.get_channel();
        let url_clone = url.clone();

        let request = Request::new(async move {
            match channel {
                ChannelType::RGB8U => {
                    let allsky_tile_size = std::cmp::min(tile_size, 64);
                    let allsky = query_image(&url_clone).await?;

                    let allsky_tiles = handle_allsky_file::<RGBA8U>(
                        allsky,
                        allsky_tile_size,
                        texture_size,
                        tile_size,
                    )?
                    .into_iter()
                    .map(|image| {
                        let ImageBuffer { data, size } = image;
                        let data = data
                            .into_iter()
                            .enumerate()
                            .filter(|&(i, _)| i % 4 != 3)
                            .map(|(_, v)| v)
                            .collect();
                        let image = ImageBuffer::new(data, size.x, size.y);

                        ImageType::RawRgb8u { image }
                    })
                    .collect();

                    Ok(allsky_tiles)
                }
                ChannelType::RGBA8U => {
                    let allsky_tile_size = std::cmp::min(tile_size, 64);
                    let allsky = query_image(&url_clone).await?;

                    let allsky_tiles =
                        handle_allsky_file(allsky, allsky_tile_size, texture_size, tile_size)?
                            .into_iter()
                            .map(|image| ImageType::RawRgba8u { image })
                            .collect();

                    Ok(allsky_tiles)
                }
                _ => {
                    let opts = RequestInit::new();
                    opts.set_method("GET");
                    opts.set_mode(RequestMode::Cors);
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

                    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
                    let bytes_buffer = js_sys::Uint8Array::new(&array_buffer);

                    let num_bytes = bytes_buffer.length() as usize;
                    let mut raw_bytes = Vec::with_capacity(num_bytes);
                    unsafe {
                        raw_bytes.set_len(num_bytes);
                    }
                    bytes_buffer.copy_to(&mut raw_bytes[..]);
                    let mut reader = Cursor::new(&raw_bytes[..]);
                    let Fits { hdu } = Fits::from_reader(&mut reader)
                        .map_err(|_| JsValue::from_str("Parsing fits error of allsky"))?;

                    //let width_allsky_px = 27 * std::cmp::min(tile_size, 64) as i32;
                    //let height_allsky_px = 29 * std::cmp::min(tile_size, 64) as i32;
                    let data = hdu.get_data();

                    match data {
                        InMemData::U8(data) => {
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR8ui { image })
                                .collect())
                        }
                        InMemData::I16(data) => {
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR16i { image })
                                .collect())
                        }
                        InMemData::I32(data) => {
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR32i { image })
                                .collect())
                        }
                        InMemData::F32(data) => {
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR32f { image })
                                .collect())
                        }
                        InMemData::I64(data) => {
                            let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR32i { image })
                                .collect())
                        }
                        InMemData::F64(data) => {
                            let data = data.iter().map(|v| *v as f32).collect::<Vec<_>>();
                            Ok(handle_allsky_fits(&data, tile_size, texture_size)?
                                .into_iter()
                                .map(|image| ImageType::RawR32f { image })
                                .collect())
                        }
                    }
                }
            }
        });

        Self {
            id,
            hips_cdid,
            depth_tile,
            url,
            request,
        }
    }
}

use al_core::image::format::ImageFormat;
use al_core::image::raw::ImageBufferView;
fn handle_allsky_file<F: ImageFormat>(
    allsky: ImageBuffer<F>,
    allsky_tile_size: i32,
    texture_size: i32,
    tile_size: i32,
) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    let num_tiles_per_texture = (texture_size / tile_size) * (texture_size / tile_size);
    let num_tiles = num_tiles_per_texture * 12;
    let mut tiles = Vec::with_capacity(num_tiles as usize);

    let num_allsky_tiles_per_tile = (tile_size / allsky_tile_size) * (tile_size / allsky_tile_size);

    let mut src_idx = 0;
    for _ in 0..num_tiles {
        let mut base_tile =
            ImageBuffer::<F>::allocate(&<F as ImageFormat>::P::BLACK, tile_size, tile_size);
        for idx_tile in 0..num_allsky_tiles_per_tile {
            let (x, y) = crate::utils::unmortonize(idx_tile as u64);
            let dx = x * (allsky_tile_size as u32);
            let dy = y * (allsky_tile_size as u32);

            let sx = (src_idx % 27) * allsky_tile_size;
            let sy = (src_idx / 27) * allsky_tile_size;
            let s = ImageBufferView {
                x: sx as i32,
                y: sy as i32,
                w: allsky_tile_size as i32,
                h: allsky_tile_size as i32,
            };
            let d = ImageBufferView {
                x: dx as i32,
                y: dy as i32,
                w: allsky_tile_size as i32,
                h: allsky_tile_size as i32,
            };

            base_tile.tex_sub(&allsky, &s, &d);

            src_idx += 1;
        }

        tiles.push(base_tile);
    }

    Ok(tiles)
}

fn handle_allsky_fits<F: ImageFormat>(
    allsky_data: &[<<F as ImageFormat>::P as Pixel>::Item],
    tile_size: i32,
    texture_size: i32,
) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    let allsky_tile_size = std::cmp::min(tile_size, 64);
    let width_allsky_px = 27 * allsky_tile_size;
    let height_allsky_px = 29 * allsky_tile_size;
    // The fits image layout stores rows in reverse
    let reversed_rows_data = allsky_data
        .chunks(width_allsky_px as usize)
        .rev()
        .flatten()
        .copied()
        .collect::<Vec<_>>();

    let allsky = ImageBuffer::<F>::new(reversed_rows_data, width_allsky_px, height_allsky_px);

    let allsky_tiles = handle_allsky_file::<F>(allsky, allsky_tile_size, texture_size, tile_size)?
        .into_iter()
        .map(|image| {
            // The GPU does a specific transformation on the UV
            // for FITS tiles
            // We must revert this to be compatible with this GPU transformation
            let mut new_image_data = Vec::with_capacity(tile_size as usize);
            for c in image.get_data().chunks((tile_size * tile_size) as usize) {
                new_image_data.extend(c.chunks(tile_size as usize).rev().flatten());
            }

            ImageBuffer::<F>::new(new_image_data, tile_size, tile_size)
        })
        .collect();

    Ok(allsky_tiles)
}

use al_core::image::format::RGBA8U;

use crate::time::Time;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
pub struct Allsky {
    pub image: Rc<RefCell<Option<Vec<ImageType>>>>,
    pub time_req: Time,
    pub depth_tile: u8,

    pub hips_cdid: CreatorDid,
    url: Url,
}

use crate::Abort;

impl Allsky {
    pub fn missing(&self) -> bool {
        self.image.borrow().is_none()
    }

    pub fn get_hips_cdid(&self) -> &CreatorDid {
        &self.hips_cdid
    }

    pub fn get_url(&self) -> &Url {
        &self.url
    }
}

impl<'a> From<&'a AllskyRequest> for Option<Allsky> {
    fn from(request: &'a AllskyRequest) -> Self {
        let AllskyRequest {
            request,
            hips_cdid,
            depth_tile,
            url,
            ..
        } = request;
        if request.is_resolved() {
            let Request::<Vec<ImageType>> {
                time_request, data, ..
            } = request;
            Some(Allsky {
                time_req: *time_request,
                // This is a clone on a Arc, it is supposed to be fast
                image: data.clone(),
                hips_cdid: hips_cdid.clone(),
                url: url.clone(),
                depth_tile: *depth_tile,
            })
        } else {
            None
        }
    }
}
