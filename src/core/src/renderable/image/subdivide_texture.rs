use al_core::texture::format::PixelType;
use wasm_bindgen::JsValue;

use futures::AsyncReadExt;

use super::cuts;
use al_core::texture::format::TextureFormat;
use al_core::texture::pixel::Pixel;
use al_core::webgl_ctx::WebGlRenderingCtx;
use al_core::Texture2D;
use al_core::WebGlContext;
use std::ops::Range;

use al_core::convert::Cast;
type PixelItem<F> = <<F as TextureFormat>::P as Pixel>::Item;

pub fn crop_image<F>(
    gl: &WebGlContext,
    width: u64,
    height: u64,
    bytes: &[u8],
    max_tex_size: u64,
    blank: Option<f32>,
) -> Result<ImagePatches, JsValue>
where
    F: TextureFormat,
{
    let mut tex_chunks = vec![];
    let num_texture_x = ((width / max_tex_size) + 1) as usize;
    let num_texture_y = ((height / max_tex_size) + 1) as usize;
    // Subdivision

    let mut w = Vec::with_capacity(num_texture_x);
    let mut h = Vec::with_capacity(num_texture_x);

    for i in 0..num_texture_x {
        let w_patch = if i == num_texture_x - 1 {
            width % max_tex_size
        } else {
            max_tex_size
        };
        let h_patch = max_tex_size;

        w.push(w_patch as usize);
        h.push(h_patch as usize);
    }

    let create_next_patches = |num_patches_per_row: usize| -> Vec<Vec<u8>> {
        (0..num_patches_per_row)
            .map(|i| {
                vec![0_u8; (max_tex_size as usize) * (max_tex_size as usize) * F::NUM_CHANNELS]
            })
            .collect::<Vec<_>>()
    };

    let mut buf = create_next_patches(num_texture_x);

    let mut pixels_written = 0_usize;
    let num_pixels = (width * height) as usize;

    // Sampled pixels for computing automatic min/max cut values
    const PIXEL_STEP: usize = 256;
    let mut sub_pixels = vec![];

    let step_x_cut = (width as usize) / PIXEL_STEP;
    let step_y_cut = (height as usize) / PIXEL_STEP;
    let step_cut = step_x_cut.max(step_y_cut) + 1 as usize;

    let mut id_tx = 0;
    let mut id_ty = 0;

    while pixels_written < num_pixels {
        let bytes_written = pixels_written * F::NUM_CHANNELS;

        // For textures along the right-x border
        let w_patch = w[id_tx];
        let h_patch = h[id_tx];

        let num_pixels_to_read = w_patch;
        let num_bytes_to_read = (num_pixels_to_read as usize) * F::NUM_CHANNELS;

        // Tell where the data must go inside the texture
        let off_y_px = id_ty * h_patch;

        // line index
        let y = pixels_written / (width as usize);
        let dy = y - off_y_px;

        let off_bytes_src = bytes_written;
        let off_bytes_dst = dy * (max_tex_size as usize) * F::NUM_CHANNELS;

        buf[id_tx][off_bytes_dst..(off_bytes_dst + num_bytes_to_read)]
            .copy_from_slice(&bytes[off_bytes_src..(off_bytes_src + num_bytes_to_read)]);

        pixels_written += num_pixels_to_read as usize;

        if F::PIXEL_TYPE.num_channels() == 1 && y % step_cut == 0 {
            let x = pixels_written % (width as usize);
            // on a good line
            let bytes_line = &buf[id_tx][off_bytes_dst..(off_bytes_dst + num_bytes_to_read)];
            for x_in_patch in (0..w_patch).step_by(step_cut) {
                let x_byte_off = (x_in_patch as usize) * F::NUM_CHANNELS;
                let p = &bytes_line[x_byte_off..(x_byte_off + F::NUM_CHANNELS)];

                let v = match F::PIXEL_TYPE {
                    PixelType::R8U => {
                        let p = p[0] as f32;

                        if let Some(blank) = blank {
                            if p != blank {
                                Some(p)
                            } else {
                                None
                            }
                        } else {
                            Some(p)
                        }
                    }
                    PixelType::R16I => {
                        let p = i16::from_be_bytes([p[0], p[1]]) as f32;

                        if let Some(blank) = blank {
                            if p != blank {
                                Some(p)
                            } else {
                                None
                            }
                        } else {
                            Some(p)
                        }
                    }
                    PixelType::R32I => {
                        let p = i32::from_be_bytes([p[0], p[1], p[2], p[3]]) as f32;

                        if let Some(blank) = blank {
                            if p != blank {
                                Some(p)
                            } else {
                                None
                            }
                        } else {
                            Some(p)
                        }
                    }
                    PixelType::R32F => {
                        let p = f32::from_be_bytes([p[0], p[1], p[2], p[3]]);

                        if p.is_finite() {
                            Some(p)
                        } else {
                            None
                        }
                    }
                    _ => unreachable!(),
                };

                if let Some(v) = v {
                    sub_pixels.push(v);
                }
            }
        }

        if (((dy + 1) % (max_tex_size as usize) == 0) && id_tx == buf.len() - 1)
            || pixels_written >= num_pixels
        {
            // we can create new textures of size max_tex_size
            for i in 0..buf.len() {
                let tex_chunk = Texture2D::create_from_raw_bytes::<F>(
                    gl,
                    max_tex_size as i32,
                    max_tex_size as i32,
                    &[
                        (
                            WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                            WebGlRenderingCtx::NEAREST_MIPMAP_NEAREST,
                        ),
                        (
                            WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                            WebGlRenderingCtx::NEAREST,
                        ),
                        // Prevents s-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_S,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                        // Prevents t-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_T,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                    ],
                    &buf[i],
                )?;

                tex_chunks.push(tex_chunk);
            }
            //buf.clear();
            //buf = create_next_patches(num_texture_x);

            id_ty = (id_ty + 1) % num_texture_y;
        }

        id_tx = (id_tx + 1) % num_texture_x;
    }

    let cuts = if F::PIXEL_TYPE.num_channels() == 1 {
        cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
    } else {
        0.0..1.0
    };

    Ok(ImagePatches {
        pixel_type: F::PIXEL_TYPE,
        texture_patches: tex_chunks,
        initial_cuts: cuts,
    })
}

pub struct ImagePatches {
    pub pixel_type: PixelType,
    pub texture_patches: Vec<Texture2D>,
    pub initial_cuts: Range<f32>,
}

impl ImagePatches {
    pub fn new(
        pixel_type: PixelType,
        texture_patches: Vec<Texture2D>,
        initial_cuts: Range<f32>,
    ) -> Self {
        Self {
            pixel_type,
            texture_patches,
            initial_cuts,
        }
    }
}
/*
pub async fn crop_image<F, R>(
    gl: &WebGlContext,
    width: u64,
    height: u64,
    mut reader: R,
    max_tex_size: u64,
    blank: Option<f32>,
) -> Result<(Vec<Texture2D>, Range<f32>), JsValue>
where
    F: TextureFormat,
    R: AsyncReadExt + Unpin,
{
    let mut tex_chunks = vec![];

    // Subdivision
    let num_textures = ((width / max_tex_size) + 1) * ((height / max_tex_size) + 1);

    let mut buf = vec![
        0;
        (max_tex_size as usize)
            * std::mem::size_of::<<F::P as Pixel>::Item>()
            * F::NUM_CHANNELS
    ];

    for _ in 0..num_textures {
        let tex_chunk = Texture2D::create_empty_with_format::<F>(
            gl,
            max_tex_size as i32,
            max_tex_size as i32,
            &[
                (
                    WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                    WebGlRenderingCtx::NEAREST_MIPMAP_NEAREST,
                ),
                (
                    WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                    WebGlRenderingCtx::NEAREST,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_S,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_T,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
            ],
        )?;
        tex_chunk.generate_mipmap();
        tex_chunks.push(tex_chunk);
    }

    let mut pixels_written = 0;
    let num_pixels = width * height;

    const PIXEL_STEP: u64 = 256;

    let step_x_cut = (width / PIXEL_STEP) as usize;
    let step_y_cut = (height / PIXEL_STEP) as usize;

    let mut sub_pixels = vec![];

    let step_cut = step_x_cut.max(step_y_cut) + 1;

    let num_texture_x = (width / max_tex_size) + 1;
    let num_texture_y = (height / max_tex_size) + 1;

    while pixels_written < num_pixels {
        // Get the id of the texture to fill
        let id_tx = (pixels_written % width) / max_tex_size;
        let id_ty = (pixels_written / width) / max_tex_size;

        let id_t = id_ty + id_tx * num_texture_y;

        // For textures along the right-x border
        let num_pixels_to_read = if id_tx == num_texture_x - 1 {
            width - (pixels_written % width)
        } else {
            max_tex_size
        };

        let num_bytes_to_read = (num_pixels_to_read as usize)
            * std::mem::size_of::<<F::P as Pixel>::Item>()
            * F::NUM_CHANNELS;

        if let Ok(()) = reader.read_exact(&mut buf[..num_bytes_to_read]).await {
            // Tell where the data must go inside the texture
            let off_y_px = id_ty * max_tex_size;

            let dy = (pixels_written / width) - off_y_px;
            let view = unsafe {
                let data = std::slice::from_raw_parts(
                    buf[..num_bytes_to_read].as_ptr() as *const <F::P as Pixel>::Item,
                    (num_pixels_to_read as usize) * F::NUM_CHANNELS,
                );

                // compute the cuts if the pixel is grayscale
                if (pixels_written / width) % (step_cut as u64) == 0 {
                    // We are in a good line
                    let xmin = pixels_written % width;

                    match F::PIXEL_TYPE {
                        PixelType::R32F => {
                            let pixels = std::slice::from_raw_parts(
                                data.as_ptr() as *const f32,
                                data.len() / 4,
                            );

                            for i in (0..width).step_by(step_cut) {
                                if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                    let j = (i - xmin) as usize;

                                    if pixels[j].is_finite() {
                                        sub_pixels.push(pixels[j]);
                                    }
                                }
                            }
                        }
                        PixelType::R8U | PixelType::R16I | PixelType::R32I => {
                            if let Some(blank) = blank {
                                for i in (0..width).step_by(step_cut) {
                                    if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                        let j = (i - xmin) as usize;

                                        let pixel = <PixelItem<F> as Cast<f32>>::cast(data[j]);

                                        if pixel != blank {
                                            sub_pixels.push(pixel);
                                        }
                                    }
                                }
                            } else {
                                for i in (0..width).step_by(step_cut) {
                                    if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                        let j = (i - xmin) as usize;

                                        let pixel = <PixelItem<F> as Cast<f32>>::cast(data[j]);
                                        sub_pixels.push(pixel);
                                    }
                                }
                            }
                        }
                        // colored pixels
                        _ => (),
                    }
                }

                F::view(data)
            };

            tex_chunks[id_t as usize]
                .bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    0,
                    dy as i32,
                    num_pixels_to_read as i32,
                    1,
                    Some(view.as_ref()),
                );

            pixels_written += num_pixels_to_read;
        } else {
            return Err(JsValue::from_str(
                "invalid data with respect to the NAXIS given in the WCS",
            ));
        }
    }

    let cuts = if F::PIXEL_TYPE.num_channels() == 1 {
        cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
    } else {
        0.0..1.0
    };

    Ok((tex_chunks, cuts))
}
*/
