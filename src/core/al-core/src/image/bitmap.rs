use cgmath::Vector3;

#[derive(Debug, Clone)]
pub struct Bitmap<F> {
    pub image: web_sys::ImageBitmap,
    format: std::marker::PhantomData<F>,
}

use crate::image::Image;
use crate::texture::format::TextureFormat;
impl<F> Bitmap<F>
where
    F: TextureFormat + Clone,
{
    pub fn new(image: web_sys::ImageBitmap) -> Self {
        Self {
            image,
            format: std::marker::PhantomData,
        }
    }
}
use crate::texture::Tex3D;
use wasm_bindgen::JsValue;
impl<F> Image for Bitmap<F>
where
    F: TextureFormat + Clone,
{
    fn insert_into_3d_texture<T: Tex3D>(
        &self,
        textures: &T,
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        textures.tex_sub_image_3d_with_image_bitmap(offset.x, offset.y, offset.z, &self.image);

        Ok(())
    }

    fn get_size(&self) -> (u32, u32, u32) {
        (self.image.width(), self.image.height(), 1)
    }
}
