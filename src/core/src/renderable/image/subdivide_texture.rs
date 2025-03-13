use al_core::image::format::ChannelType;
use wasm_bindgen::JsValue;

use futures::AsyncReadExt;

use super::cuts;
use al_core::image::format::ImageFormat;
use al_core::texture::pixel::Pixel;
use al_core::webgl_ctx::WebGlRenderingCtx;
use al_core::Texture2D;
use al_core::WebGlContext;
use std::ops::Range;

use al_core::convert::Cast;
type PixelItem<F> = <<F as ImageFormat>::P as Pixel>::Item;

pub async fn crop_image<'a, F, R>(
    gl: &WebGlContext,
    width: u64,
    height: u64,
    mut reader: R,
    max_tex_size: u64,
    blank: Option<f32>,
) -> Result<(Vec<Texture2D>, Range<f32>), JsValue>
where
    F: ImageFormat,
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

                    match F::CHANNEL_TYPE {
                        ChannelType::R32F | ChannelType::R64F => {
                            let pixels = std::slice::from_raw_parts(data.as_ptr() as *const f32, data.len() / 4);

                            for i in (0..width).step_by(step_cut) {
                                if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                    let j = (i - xmin) as usize;

                                    if pixels[j].is_finite() {
                                        sub_pixels.push(pixels[j]);
                                    }
                                }
                            }
                        },
                        ChannelType::R8UI | ChannelType::R16I | ChannelType::R32I => {
                            if let Some(blank) = blank {
                                for i in (0..width).step_by(step_cut) {
                                    if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                        let j = (i - xmin) as usize;
    
                                        let pixel = <PixelItem::<F> as Cast<f32>>::cast(data[j]);

                                        if pixel != blank {
                                            sub_pixels.push(pixel);
                                        }
                                    }
                                }
                            } else {
                                for i in (0..width).step_by(step_cut) {
                                    if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                        let j = (i - xmin) as usize;
    
                                        let pixel = <PixelItem::<F> as Cast<f32>>::cast(data[j]);
                                        sub_pixels.push(pixel);                                        
                                    }
                                }
                            }
                        },
                        // colored pixels 
                        _ => (),
                    }
                }

                F::view(data)
            };

            (&mut tex_chunks[id_t as usize])
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

    let cuts = if F::CHANNEL_TYPE.is_colored() {
        cuts::first_and_last_percent(&mut sub_pixels, 1, 99)
    } else {
        0.0..1.0
    };

    Ok((tex_chunks, cuts))
}
