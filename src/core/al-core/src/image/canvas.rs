/* ------------------------------------------------------ */
#[derive(Debug)]
pub struct Canvas<F> {
    canvas: web_sys::HtmlCanvasElement,
    format: std::marker::PhantomData<F>,
}

impl<F> Canvas<F>
where
    F: TextureFormat + Clone,
{
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self {
            canvas,
            format: std::marker::PhantomData,
        }
    }
}

use crate::image::Image;
use crate::texture::format::TextureFormat;
use crate::texture::Tex3D;
use cgmath::Vector3;
use wasm_bindgen::JsValue;
impl<F> Image for Canvas<F>
where
    F: TextureFormat,
{
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        // The texture array
        textures: &T,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        textures.tex_sub_image_3d_with_html_canvas_element(
            offset.x,
            offset.y,
            offset.z,
            &self.canvas,
        );

        Ok(())
    }

    fn get_size(&self) -> (u32, u32, u32) {
        (self.canvas.width(), self.canvas.height(), 1)
    }
}
