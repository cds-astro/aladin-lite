use wasm_bindgen::JsValue;

use futures::AsyncReadExt;

use super::cuts;
use al_core::image::format::ImageFormat;
use al_core::texture::pixel::Pixel;
use al_core::texture::TEX_PARAMS;
use al_core::Texture2D;
use al_core::WebGlContext;
use std::ops::Range;

pub async fn crop_image<'a, F, R>(
    gl: &WebGlContext,
    width: u64,
    height: u64,
    mut reader: R,
    max_tex_size: u64,
    blank: f32,
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
        tex_chunks.push(Texture2D::create_empty_with_format::<F>(
            gl,
            max_tex_size as i32,
            max_tex_size as i32,
            TEX_PARAMS,
        )?);
    }

    let mut pixels_written = 0;
    let num_pixels = width * height;

    let step_x_cut = (width / 50) as usize;
    let step_y_cut = (height / 50) as usize;

    let mut samples = vec![];

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
                let slice = std::slice::from_raw_parts(
                    buf[..num_bytes_to_read].as_ptr() as *const <F::P as Pixel>::Item,
                    (num_pixels_to_read as usize) * F::NUM_CHANNELS,
                );

                // compute the cuts if the pixel is grayscale
                if F::NUM_CHANNELS == 1 {
                    // fill the samples buffer
                    if (pixels_written / width) % (step_cut as u64) == 0 {
                        // We are in a good line
                        let xmin = pixels_written % width;

                        for i in (0..width).step_by(step_cut) {
                            if (xmin..(xmin + num_pixels_to_read)).contains(&i) {
                                let j = (i - xmin) as usize;

                                let sj: f32 = <<F::P as Pixel>::Item as al_core::convert::Cast<
                                    f32,
                                >>::cast(slice[j]);
                                if !sj.is_nan() {
                                    if blank != sj {
                                        samples.push(sj);
                                    }
                                }
                            }
                        }
                    }
                }

                F::view(slice)
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

    let cuts = if F::NUM_CHANNELS == 1 {
        cuts::first_and_last_percent(&mut samples, 1, 99)
    } else {
        0.0..1.0
    };

    Ok((tex_chunks, cuts))
}
