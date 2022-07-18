
use al_core::image::format::ImageFormatType;

use crate::downloader::{query};
use al_core::image::{fits::Fits, ImageType};

use super::{Request, RequestType};
pub struct AllskyRequest {
    pub hips_url: Url,
    pub url: Url,
    pub depth_tile: u8,

    request: Request<Vec<ImageType>>,
}

impl From<AllskyRequest> for RequestType {
    fn from(request: AllskyRequest) -> Self {
        RequestType::Allsky(request)
    }
}

use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response};

use al_core::{image::raw::ImageBuffer, texture::Pixel};
use wasm_bindgen::JsCast;

use wasm_bindgen::JsValue;
impl From<query::Allsky> for AllskyRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Allsky) -> Self {
        let query::Allsky {
            format,
            tile_size,
            url,
            hips_url,
            texture_size,
        } = query;

        let depth_tile = crate::math::utils::log_2_unchecked(texture_size / tile_size) as u8;

        let url_clone = url.clone();

        let request = Request::new(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);
            let window = web_sys::window().unwrap();

            let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts)?;
            if let Ok(resp_value) = JsFuture::from(window.fetch_with_request(&request)).await {
                let tile_size = tile_size as i32;
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;

                let buf = JsFuture::from(resp.array_buffer()?).await?;

                let width_allsky_px = 27 * std::cmp::min(tile_size, 64) as i32;
                let height_allsky_px = 29 * std::cmp::min(tile_size, 64) as i32;

                let num_pixels = (width_allsky_px * height_allsky_px) as usize;

                let allsky_tiles = match format {
                    ImageFormatType::RGB8U => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                        let allsky =
                            ImageBuffer::<RGB8U>::from_raw_bytes(&raw_bytes[..], width_allsky_px, height_allsky_px)?;
                        let allsky_tile_size = std::cmp::min(tile_size, 64);
                        handle_allsky_file::<RGB8U>(allsky, allsky_tile_size, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawRgb8u { image })
                            .collect()
                    }
                    ImageFormatType::RGBA8U => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                        let allsky =
                            ImageBuffer::<RGBA8U>::from_raw_bytes(&raw_bytes[..], width_allsky_px, height_allsky_px)?;
                        let allsky_tile_size = std::cmp::min(tile_size, 64);
                        handle_allsky_file::<RGBA8U>(allsky, allsky_tile_size, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawRgba8u { image })
                            .collect()
                    }
                    ImageFormatType::R32F => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R32F>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(
                                image.aligned_data_raw_bytes_ptr,
                                num_pixels,
                            )
                        };

                        handle_allsky_fits(raw, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawR32f { image })
                            .collect()
                    }
                    ImageFormatType::R32I => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R32I>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(
                                image.aligned_data_raw_bytes_ptr,
                                num_pixels,
                            )
                        };

                        handle_allsky_fits(raw, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawR32i { image })
                            .collect()
                    }
                    ImageFormatType::R16I => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R16I>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(
                                image.aligned_data_raw_bytes_ptr,
                                num_pixels,
                            )
                        };

                        handle_allsky_fits(raw, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawR16i { image })
                            .collect()
                    }
                    ImageFormatType::R8UI => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R8UI>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(
                                image.aligned_data_raw_bytes_ptr,
                                num_pixels,
                            )
                        };

                        handle_allsky_fits(raw, tile_size)
                            .await?
                            .into_iter()
                            .map(|image| ImageType::RawR8ui { image })
                            .collect()
                    }
                    _ => return Err(js_sys::Error::new("Format not supported").into()),
                };

                Ok(allsky_tiles)
            } else {
                Err(js_sys::Error::new("Allsky not fetched").into())
            }
        });

        Self {
            hips_url,
            depth_tile,
            url,
            request,
        }
    }
}

use al_core::image::format::ImageFormat;

async fn handle_allsky_file<F: ImageFormat>(
    allsky: ImageBuffer<F>,
    allsky_tile_size: i32,
    tile_size: i32,
) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    let mut src_idx = 0;

    let num_tiles_per_texture = (512 / tile_size)*(512 / tile_size);
    let num_tiles = num_tiles_per_texture*12;
    let mut tiles = Vec::with_capacity(num_tiles as usize);

    let num_allsky_tiles_per_tile = (tile_size / 64)*(tile_size / 64);

    for _ in 0..num_tiles {
        let mut base_tile = ImageBuffer::<F>::allocate(&<F as ImageFormat>::P::BLACK, tile_size, tile_size);
        for idx_tile in 0..num_allsky_tiles_per_tile {
            let (x, y) = crate::utils::unmortonize(idx_tile as u64);
            let dx = x * (allsky_tile_size as u32);
            let dy = y * (allsky_tile_size as u32);

            let sx = (src_idx % 27) * allsky_tile_size;
            let sy = (src_idx / 27) * allsky_tile_size;
            base_tile.tex_sub(&allsky, sx, sy, allsky_tile_size, allsky_tile_size, dx as i32, dy as i32, allsky_tile_size, allsky_tile_size);

            src_idx += 1;
        }

        tiles.push(base_tile);
    }

    Ok(tiles)
}

async fn handle_allsky_fits<F: ImageFormat>(
    allsky_data: &[<<F as ImageFormat>::P as Pixel>::Item],
    tile_size: i32,
) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    let allsky_tile_size = std::cmp::min(tile_size, 64);
    let width_allsky_px = 27 * allsky_tile_size;
    let height_allsky_px = 29 * allsky_tile_size;
    // The fits image layout stores rows in reverse
    let reversed_rows_data = allsky_data
        .chunks(width_allsky_px as usize)
        .rev()
        .flatten()
        .map(|e| *e)
        .collect::<Vec<_>>();

    let allsky = ImageBuffer::<F>::new(reversed_rows_data, width_allsky_px, height_allsky_px);

    let allsky_tiles = handle_allsky_file::<F>(allsky, allsky_tile_size, tile_size)
        .await?
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

use al_core::image::format::{R16I, R32F, R32I, R8UI, RGB8U, RGBA8U};

use crate::time::Time;
use std::sync::{Arc, Mutex};
pub struct Allsky {
    pub image: Arc<Mutex<Option<Vec<ImageType>>>>,
    pub time_req: Time,
    pub depth_tile: u8,

    pub hips_url: Url,
    url: Url,
}

impl Allsky {
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

impl<'a> From<&'a AllskyRequest> for Option<Allsky> {
    fn from(request: &'a AllskyRequest) -> Self {
        let AllskyRequest {
            request,
            hips_url,
            depth_tile,
            url,
        } = request;
        if request.is_resolved() {
            let Request::<Vec<ImageType>> {
                time_request, data, ..
            } = request;
            Some(Allsky {
                time_req: *time_request,
                // This is a clone on a Arc, it is supposed to be fast
                image: data.clone(),
                hips_url: hips_url.clone(),
                url: url.clone(),
                depth_tile: *depth_tile,
            })
        } else {
            None
        }
    }
}
