use cgmath::Vector3;

#[derive(Debug, Clone)]
pub struct Bitmap<F> {
    pub image: web_sys::ImageBitmap,
    format: std::marker::PhantomData<F>,
}

use crate::image::format::ImageFormat;
use crate::image::Image;
impl<F> Bitmap<F>
where
    F: ImageFormat + Clone,
{
    pub fn new(image: web_sys::ImageBitmap) -> Self {
        Self {
            image,
            format: std::marker::PhantomData,
        }
    }
}

use crate::texture::Texture2DArray;
impl<F> Image for Bitmap<F>
where
    F: ImageFormat + Clone,
{
    fn tex_sub_image_3d(&self, textures: &Texture2DArray, offset: &Vector3<i32>) {
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(offset.x, offset.y, &self.image);
    }
}
