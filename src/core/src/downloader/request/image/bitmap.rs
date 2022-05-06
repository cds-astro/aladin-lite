use cgmath::{Vector2, Vector3};
use al_core::{
    format::ImageFormat, 
    Texture2DArray,
    image::Image,
};

#[derive(Debug)]
#[derive(Clone)]
pub struct Bitmap<F>
where
    F: ImageFormat + Clone,
{
    pub image: web_sys::ImageBitmap,
    pub size: Vector2<i32>,
    format: std::marker::PhantomData<F>,
}

use crate::num_traits::Zero;
impl<F> Bitmap<F>
where
    F: ImageFormat + Clone,
{
    pub fn new(image: web_sys::ImageBitmap) -> Self {
        let size = Vector2::new(
            image.width() as i32,
            image.height() as i32
        );
        Self {
            image,
            size,
            format: std::marker::PhantomData
        }
    }
}

impl<F> Image for Bitmap<F>
where
    F: ImageFormat + Clone,
{
    fn tex_sub_image_3d(&self, textures: &Texture2DArray, offset: &Vector3<i32>) {
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_image_bitmap(
                offset.x,
                offset.y,
                &self.image,
            );
    }

    // The size of the image
    /*fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }*/
}