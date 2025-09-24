use al_core::texture::format::PixelType;
use wasm_bindgen::JsValue;

use super::cuts;
use al_core::texture::format::TextureFormat;
use al_core::webgl_ctx::WebGlRenderingCtx;
use al_core::Texture2D;
use al_core::WebGlContext;
use std::ops::Range;

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
            .map(|_| {
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
    let step_cut = step_x_cut.max(step_y_cut) + 1_usize;

    let mut id_tx = 0;
    let mut id_ty = 0;

    while pixels_written < num_pixels {
        let bytes_written = pixels_written * F::NUM_CHANNELS;

        // For textures along the right-x border
        let w_patch = w[id_tx];
        let h_patch = h[id_tx];

        let num_pixels_to_read = w_patch;
        let num_bytes_to_read = num_pixels_to_read * F::NUM_CHANNELS;

        // Tell where the data must go inside the texture
        let off_y_px = id_ty * h_patch;

        // line index
        let y = pixels_written / (width as usize);
        let dy = y - off_y_px;

        let off_bytes_src = bytes_written;
        let off_bytes_dst = dy * (max_tex_size as usize) * F::NUM_CHANNELS;

        buf[id_tx][off_bytes_dst..(off_bytes_dst + num_bytes_to_read)]
            .copy_from_slice(&bytes[off_bytes_src..(off_bytes_src + num_bytes_to_read)]);

        pixels_written += num_pixels_to_read;

        if F::PIXEL_TYPE.num_channels() == 1 && y.is_multiple_of(step_cut) {
            // on a good line
            let bytes_line = &buf[id_tx][off_bytes_dst..(off_bytes_dst + num_bytes_to_read)];
            for x_in_patch in (0..w_patch).step_by(step_cut) {
                let x_byte_off = x_in_patch * F::NUM_CHANNELS;
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

        if ((dy + 1).is_multiple_of(max_tex_size as usize) && id_tx == buf.len() - 1)
            || pixels_written >= num_pixels
        {
            // we can create new textures of size max_tex_size
            for patch_buf in &buf {
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
                    patch_buf,
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
        w_patch: max_tex_size as usize,
        h_patch: max_tex_size as usize,
    })
}

pub struct ImagePatches {
    pub pixel_type: PixelType,
    pub texture_patches: Vec<Texture2D>,
    pub initial_cuts: Range<f32>,
    pub w_patch: usize,
    pub h_patch: usize,
}

impl ImagePatches {
    pub fn new(
        pixel_type: PixelType,
        texture_patches: Vec<Texture2D>,
        initial_cuts: Range<f32>,
        w_patch: usize,
        h_patch: usize,
    ) -> Self {
        Self {
            pixel_type,
            texture_patches,
            initial_cuts,
            w_patch,
            h_patch,
        }
    }
}
