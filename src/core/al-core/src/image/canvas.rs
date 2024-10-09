/* ------------------------------------------------------ */
#[derive(Debug)]
pub struct Canvas<F> {
    canvas: web_sys::HtmlCanvasElement,
    format: std::marker::PhantomData<F>,
}

impl<F> Canvas<F>
where
    F: ImageFormat + Clone,
{
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self {
            canvas,
            format: std::marker::PhantomData,
        }
    }
}

use crate::image::format::ImageFormat;
use crate::image::Image;
use crate::texture::Texture2DArray;
use cgmath::Vector3;
use wasm_bindgen::JsValue;
impl<F> Image for Canvas<F>
where
    F: ImageFormat,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        textures.bind().tex_sub_image_3d_with_html_canvas_element(
            offset.z,
            offset.x,
            offset.y,
            &self.canvas,
        );

        Ok(())
    }
}
