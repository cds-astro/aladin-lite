/* ------------------------------------------------------ */

#[derive(Debug)]
pub struct HTMLImage<F>
where
    F: ImageFormat,
{
    image: web_sys::HtmlImageElement,
    size: Vector2<i32>,
    format: std::marker::PhantomData<F>,
}
use cgmath::{Vector2, Vector3};
use al_core::{
    format::{
        RGB8U, R8UI, RGBA8U, R16I, R32I, R32F, ImageFormat
    }, 
    Texture2DArray,
    image::Image,
};

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
    ) {
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
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
