use crate::texture::format::PixelType;
use al_api::hips::ImageExt;
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ImageFormatType {
    pub ext: ImageExt,
    pub fmt: PixelType,
}

impl ImageFormatType {
    pub fn get_ext_file(&self) -> &ImageExt {
        &self.ext
    }

    pub fn get_pixel_format(&self) -> PixelType {
        self.fmt
    }

    pub fn is_colored(&self) -> bool {
        !matches!(self.ext, ImageExt::Fits)
    }
}
