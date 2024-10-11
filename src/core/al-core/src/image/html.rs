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

use crate::image::format::ImageFormat;
use crate::image::Image;
use crate::texture::{Tex3D, Texture2DArray};
use cgmath::Vector3;
use wasm_bindgen::JsValue;
impl<F> Image for HTMLImage<F>
where
    F: ImageFormat,
{
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        // The texture array
        textures: &T,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        textures.tex_sub_image_3d_with_html_image_element(
            offset.x,
            offset.y,
            offset.z,
            &self.image,
        );

        Ok(())
    }
}
