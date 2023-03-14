use wasm_bindgen::JsValue;

use futures::AsyncReadExt;

use al_core::texture::MAX_TEX_SIZE;
use al_core::texture::TEX_PARAMS;
use al_core::texture::{
    pixel::Pixel,
};
use al_core::Texture2D;
use al_core::WebGlContext;
use al_core::image::format::ImageFormat;



pub async fn build<'a, F, R>(gl: &WebGlContext, width: u64, height: u64, mut reader: R) -> Result<Vec<Texture2D>, JsValue>
where
    F: ImageFormat,
    R: AsyncReadExt + Unpin
{
    let mut buf = vec![0; MAX_TEX_SIZE * std::mem::size_of::<<F::P as Pixel>::Item>()];

    // Subdivision
    let num_textures = ((((width as i32) / (MAX_TEX_SIZE as i32)) + 1) * (((height as i32) / (MAX_TEX_SIZE as i32)) + 1)) as usize;

    let mut tex_chunks = vec![];
    for _ in 0..num_textures {
        tex_chunks.push(Texture2D::create_from_raw_pixels::<F>(gl, MAX_TEX_SIZE as i32, MAX_TEX_SIZE as i32, TEX_PARAMS, None)?);
    }

    let mut pixels_written = 0;
    let num_pixels = width * height;

    let num_texture_x = (((width as i32) / (MAX_TEX_SIZE as i32)) + 1) as u64;
    let num_texture_y = (((height as i32) / (MAX_TEX_SIZE as i32)) + 1) as u64;

    while pixels_written < num_pixels {
        // Get the id of the texture to fill
        let id_tx = (pixels_written % width) / (MAX_TEX_SIZE as u64);
        let id_ty = (pixels_written / width) / (MAX_TEX_SIZE as u64);

        let id_t = id_ty + id_tx*num_texture_y;

        // For textures along the right-x border
        let num_pixels_to_read = if id_tx == num_texture_x - 1 {
            width - (pixels_written % width)
        } else {
            MAX_TEX_SIZE as u64
        };
        let num_bytes_to_read = (num_pixels_to_read as usize) * std::mem::size_of::<<F::P as Pixel>::Item>();
        reader.read_exact(&mut buf[..num_bytes_to_read])
            .await
            .map_err(|_| JsValue::from_str("Read some bytes error"))?;

        // Tell where the data must go inside the texture
        let off_y_px = id_ty * (MAX_TEX_SIZE as u64);

        let dy = (pixels_written / width) - off_y_px;
        let view = unsafe {
            let slice = std::slice::from_raw_parts(
                buf[..num_bytes_to_read].as_ptr() as *const <F::P as Pixel>::Item,
                num_pixels_to_read as usize
            );
            F::view(slice)
        };

        (&mut tex_chunks[id_t as usize])
            .bind_mut()
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                0,
                dy as i32,
                num_pixels_to_read as i32,
                1,
                Some(view.as_ref())
            );

        pixels_written += num_pixels_to_read;
    }

    Ok(tex_chunks)
}
