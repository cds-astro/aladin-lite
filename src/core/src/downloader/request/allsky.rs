use crate::{
    survey::config::HiPSConfig,
    healpix::cell::HEALPixCell
};
use al_core::image::format::ImageFormatType;

use crate::downloader::{request, query};
use al_core::image::{
    ImageType,
    bitmap::Bitmap,
    fits::Fits,
};

use super::{Request, RequestType};
pub struct AllskyRequest {
    pub hips_url: Url,
    pub url: Url,

    request: Request<Vec<ImageType>>,
}

impl From<AllskyRequest> for RequestType {
    fn from(request: AllskyRequest) -> Self {
        RequestType::Allsky(request)
    }
}

use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Response, RequestInit, RequestMode};
use crate::survey::Url;
use super::ResolvedStatus;

use wasm_bindgen::JsCast;
use al_core::{
    image::raw::ImageBuffer,
    texture::Pixel
};

use wasm_bindgen::JsValue;
impl From<query::Allsky> for AllskyRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Allsky) -> Self {
        let query::Allsky { format, tile_size, url, hips_url } = query;

        let url_clone = url.clone();

        let request = Request::new(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);
            let window = web_sys::window().unwrap();
        
            let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts)?;
            if let Ok(resp_value) = JsFuture::from(window.fetch_with_request(&request)).await {
                // `resp_value` is a `Response` object.
                debug_assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into()?;
        
                let buf = JsFuture::from(resp.array_buffer()?).await?;
                const NUM_PIXELS_FITS: usize = 1728*1856;

                let allsky_tiles = match format {
                    ImageFormatType::RGB8U => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                        let bytes = image_decoder::load_from_memory_with_format(
                                &raw_bytes[..],
                                image_decoder::ImageFormat::Jpeg,
                            ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
                            .into_bytes();
        
                        let allsky = ImageBuffer::<RGB8U>::new(bytes, 1728, 1856);
                        handle_allsky_file::<RGB8U>(allsky).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawRgb8u { image }
                            })
                            .collect()
                    },
                    ImageFormatType::RGBA8U => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf).to_vec();
                        let bytes = image_decoder::load_from_memory_with_format(
                                &raw_bytes[..],
                                image_decoder::ImageFormat::Png,
                            ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?
                            .into_bytes();
        
                        let allsky = ImageBuffer::<RGBA8U>::new(bytes, 1728, 1856);
                        handle_allsky_file::<RGBA8U>(allsky).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawRgba8u { image }
                            })
                            .collect()
                    },
                    ImageFormatType::R32F => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R32F>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(image.aligned_data_raw_bytes_ptr, NUM_PIXELS_FITS)
                        };

                        handle_allsky_fits(raw, tile_size).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawR32f { image }
                            })
                            .collect()
                    },
                    ImageFormatType::R32I => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R32I>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(image.aligned_data_raw_bytes_ptr, NUM_PIXELS_FITS)
                        };

                        handle_allsky_fits(raw, tile_size).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawR32i { image }
                            })
                            .collect()
                    },
                    ImageFormatType::R16I => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R16I>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(image.aligned_data_raw_bytes_ptr, NUM_PIXELS_FITS)
                        };

                        handle_allsky_fits(raw, tile_size).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawR16i { image }
                            })
                            .collect()
                    },
                    ImageFormatType::R8UI => {
                        let raw_bytes = js_sys::Uint8Array::new(&buf);
                        // Parsing the raw bytes coming from the received array buffer (Uint8Array)
                        let image = Fits::<R8UI>::new(&raw_bytes)?;
                        let raw = unsafe {
                            std::slice::from_raw_parts(image.aligned_data_raw_bytes_ptr, NUM_PIXELS_FITS)
                        };

                        handle_allsky_fits(raw, tile_size).await?
                            .into_iter()
                            .map(|image| {
                                ImageType::RawR8ui { image }
                            })
                            .collect()
                    },
                    _ => {
                        return Err(js_sys::Error::new("Format not supported").into())
                    }
                };
        
                al_core::log("Completed!");
                Ok(allsky_tiles)
            } else {
                Err(js_sys::Error::new("Allsky not fetched").into())
            }
        });

        Self {
            hips_url,
            url,
            request
        }
    }
}

use al_core::image::format::ImageFormat;

async fn handle_allsky_file<F: ImageFormat>(allsky: ImageBuffer<F>) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    let mut src_idx = 0;
    let mut tiles = Vec::with_capacity(12);

    for idx in 0..12 {
        let mut base_tile = ImageBuffer::<F>::allocate(&<F as ImageFormat>::P::BLACK, 512, 512);
        for idx_tile in 0..64 {
            let (x, y) = crate::utils::unmortonize(idx_tile);
            let dx = x << 6;
            let dy = y << 6;

            let sx = (src_idx % 27) << 6;
            let sy = (src_idx / 27) << 6;
            base_tile.tex_sub(
                &allsky,
                sx, sy, 64, 64,
                dx as i32, dy as i32, 64, 64
            );

            src_idx += 1;
        }

        tiles.push(base_tile);
    }

    Ok(tiles)
}

async fn handle_allsky_fits<F: ImageFormat>(allsky_data: &[<<F as ImageFormat>::P as Pixel>::Item], tile_size: usize) -> Result<Vec<ImageBuffer<F>>, JsValue> {
    // The fits image layout stores rows in reverse
    let reversed_rows_data = allsky_data
        .chunks(1728)
        .rev()
        .flatten()
        .map(|e| *e)
        .collect::<Vec<_>>();

    let allsky = ImageBuffer::<F>::new(reversed_rows_data, 1728, 1856);

    let allsky_tiles = handle_allsky_file::<F>(allsky).await?
        .into_iter()
        .map(|image| {
            // The GPU does a specific transformation on the UV
            // for FITS tiles
            // We must revert this to be compatible with this GPU transformation
            let mut new_image_data = Vec::with_capacity(512);
            for c in image.get_data().chunks(512*tile_size) {
                new_image_data.extend(
                    c.chunks(512)
                    .rev()
                    .flatten()
                );
            }

            ImageBuffer::<F>::new(new_image_data, 512, 512)
        })
        .collect();

    Ok(allsky_tiles)
}

use al_core::image::format::{
    RGB8U, RGBA8U, R32F, R8UI, R16I, R32I
};

use std::sync::{Arc, Mutex};
use crate::time::Time;
pub struct Allsky {
    pub image: Arc<Mutex<Option<Vec<ImageType>>>>,
    pub time_req: Time,

    pub hips_url: Url,
    url: Url,
}

impl Allsky {
    pub fn missing(&self) -> bool {
        self.image.lock()
            .unwrap()
            .is_none()
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
        let AllskyRequest { request, hips_url, url} = request;
        if request.is_resolved() {
            let Request::<Vec<ImageType>> { time_request, data, .. } = request;
            Some(Allsky {
                time_req: *time_request,
                // This is a clone on a Arc, it is supposed to be fast
                image: data.clone(),
                hips_url: hips_url.clone(),
                url: url.clone()
            })
        } else {
            None
        }
    }
}