/* ------------------------------------------------------ */
#[derive(Debug)]
pub struct HTMLImage<F> {
    image: web_sys::HtmlImageElement,
    format: std::marker::PhantomData<F>,
}

impl<F> HTMLImage<F>
where
    F: ImageFormat + Clone,
{
    pub fn new(image: web_sys::HtmlImageElement) -> Self {
        Self {
            image,
            format: std::marker::PhantomData,
        }
    }
}

use cgmath::Vector3;
use wasm_bindgen::JsValue;
use crate::image::format::ImageFormat;
use crate::image::Image;
use crate::texture::Texture2DArray;
impl<F> Image for HTMLImage<F>
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
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                offset.x,
                offset.y,
                &self.image,
            );

        Ok(())
    }
}
