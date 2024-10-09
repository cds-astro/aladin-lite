use al_api::hips::ImageExt;

use al_core::{image::format::ImageFormat, image::raw::ImageBuffer};

#[derive(Debug)]
pub struct EmptyTileImage {
    inner: ImageType,
}

use al_core::{image::ImageType, pixel::Pixel};
impl EmptyTileImage {
    fn new(size: i32, channel: ChannelType) -> EmptyTileImage {
        debug_assert!(math::utils::is_power_of_two(size));
        let inner = match channel {
            ChannelType::RGBA8U => {
                let image = ImageBuffer::<RGBA8U>::allocate(
                    &<<RGBA8U as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawRgba8u { image }
            }
            ChannelType::RGB8U => {
                let image = ImageBuffer::<RGB8U>::allocate(
                    &<<RGB8U as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawRgb8u { image }
            }
            ChannelType::R32F => {
                let image = ImageBuffer::<R32F>::allocate(
                    &<<R32F as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawR32f { image }
            }
            ChannelType::R64F => {
                let image = ImageBuffer::<R32F>::allocate(
                    &<<R32F as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawR32f { image }
            }
            #[cfg(feature = "webgl2")]
            ChannelType::R8UI => {
                let image = ImageBuffer::<R8UI>::allocate(
                    &<<R8UI as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawR8ui { image }
            }
            #[cfg(feature = "webgl2")]
            ChannelType::R16I => {
                let image = ImageBuffer::<R16I>::allocate(
                    &<<R16I as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawR16i { image }
            }
            #[cfg(feature = "webgl2")]
            ChannelType::R32I => {
                let image = ImageBuffer::<R32I>::allocate(
                    &<<R32I as ImageFormat>::P as Pixel>::BLACK,
                    size,
                    size,
                );
                ImageType::RawR32i { image }
            }
            _ => todo!(),
        };
        EmptyTileImage {
            inner,
            //pixel_fill,
        }
    }
}

use al_core::{
    image::{
        format::{R16I, R32F, R32I, R8UI},
        Image,
    },
    Texture2DArray,
};
use cgmath::Vector3;

impl Image for EmptyTileImage {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) -> Result<(), JsValue> {
        self.inner.tex_sub_image_3d(textures, offset)
    }
}

use al_core::image::format::{ChannelType, ImageFormatType, RGB8U, RGBA8U};

//use super::TileArrayBuffer;

/*use super::{ArrayF32, ArrayF64, ArrayI16, ArrayI32, ArrayU8};
fn create_black_tile(format: FormatImageType, width: i32, value: f32) -> TileArrayBufferImage {
    let _num_channels = format.get_num_channels() as i32;
    match format {
        FormatImageType::JPG => TileArrayBufferImage::U8(JPG::create_black_tile(width)),
        FormatImageType::PNG => TileArrayBufferImage::U8(PNG::create_black_tile(width)),
        FormatImageType::FITS(_fits) => match format.get_type() {
            WebGl2RenderingContext::FLOAT => {
                TileArrayBufferImage::F32(FITS::create_black_tile(width, value))
            }
            WebGl2RenderingContext::INT => {
                TileArrayBufferImage::I32(FITS::create_black_tile(width, value as i32))
            }
            WebGl2RenderingContext::SHORT => {
                TileArrayBufferImage::I16(FITS::create_black_tile(width, value as i16))
            }
            WebGl2RenderingContext::UNSIGNED_BYTE => {
                TileArrayBufferImage::U8(FITS::create_black_tile(width, value as u8))
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}*/

#[derive(Debug)]
pub struct HiPSConfig {
    pub root_url: String,
    // HiPS image format
    // TODO: Make that independant of the HiPS but of the ImageFormat
    pub empty_image: EmptyTileImage,

    // The size of the texture images
    pub texture_size: i32,
    tile_size: i32,

    // Delta depth i.e. log2(texture_size / tile_size)
    delta_depth: u8,
    min_depth_tile: u8,
    min_depth_texture: u8,
    // Num tiles per texture
    num_tiles_per_texture: usize,
    // Max depth of the current HiPS tiles
    max_depth_texture: u8,
    max_depth_tile: u8,

    pub is_allsky: bool,

    // TODO: store this values in the ImageSurvey
    // These are proper to the survey (FITS one) and not
    // to a specific survey color
    pub fits_metadata: bool,
    pub scale: f32,
    pub offset: f32,
    pub blank: f32,

    pub tex_storing_integers: bool,
    pub tex_storing_fits: bool,
    pub tex_storing_unsigned_int: bool,

    pub size_tile_uv: f32,
    pub frame: CooSystem,
    pub bitpix: Option<i32>,
    format: ImageFormatType,
    //dataproduct_subtype: Option<Vec<String>>,
    //colored: bool,
    pub creator_did: String,
}

use crate::math;
use crate::HiPSProperties;
use al_api::coo_system::CooSystem;
use wasm_bindgen::JsValue;

impl HiPSConfig {
    /// Define a HiPS configuration
    ///
    /// # Arguments
    ///
    /// * `properties` - A description of the HiPS, its metadata, available formats  etc...
    /// * `img_format` - Image format wanted by the user
    pub fn new(properties: &HiPSProperties, img_ext: ImageExt) -> Result<HiPSConfig, JsValue> {
        let root_url = properties.get_url();
        let creator_did = properties.get_creator_did().to_string();
        // Define the size of the 2d texture array depending on the
        // characterics of the client

        let max_depth_tile = properties.get_max_order();
        let tile_size = properties.get_tile_size();
        // Assert size is a power of two
        // Determine the size of the texture to copy
        // it cannot be > to 512x512px

        let _fmt = properties.get_formats();
        let bitpix = properties.get_bitpix();
        let mut tex_storing_unsigned_int = false;
        let mut tex_storing_integers = false;

        let mut tex_storing_fits = false;

        if !properties.get_formats().contains(&img_ext) {
            return Err(js_sys::Error::new("HiPS format not available").into());
        }

        let format = match img_ext {
            ImageExt::Fits => {
                // Check the bitpix to determine the internal format of the tiles
                if let Some(bitpix) = bitpix {
                    let channel = (match bitpix {
                        #[cfg(feature = "webgl2")]
                        8 => {
                            tex_storing_fits = true;
                            tex_storing_unsigned_int = true;
                            Ok(ChannelType::R8UI)
                        }
                        #[cfg(feature = "webgl2")]
                        16 => {
                            tex_storing_fits = true;
                            tex_storing_integers = true;
                            Ok(ChannelType::R16I)
                        }
                        #[cfg(feature = "webgl2")]
                        32 => {
                            tex_storing_fits = true;
                            tex_storing_integers = true;
                            Ok(ChannelType::R32I)
                        }
                        -32 => {
                            tex_storing_fits = true;
                            tex_storing_integers = false;
                            Ok(ChannelType::R32F)
                        }
                        -64 => {
                            tex_storing_fits = true;
                            tex_storing_integers = false;
                            //Err(JsValue::from_str("f64 FITS files not supported"))
                            Ok(ChannelType::R64F)
                        }
                        _ => Err(JsValue::from_str(
                            "Fits tiles exists but the BITPIX is not correct in the property file",
                        )),
                    })?;

                    Ok(ImageFormatType {
                        ext: img_ext,
                        channel,
                    })
                } else {
                    Err(JsValue::from_str(
                        "Fits tiles exists but the BITPIX is not found",
                    ))
                }
            }
            ImageExt::Png | ImageExt::Webp => Ok(ImageFormatType {
                ext: img_ext,
                channel: ChannelType::RGBA8U,
            }),
            ImageExt::Jpeg => Ok(ImageFormatType {
                ext: img_ext,
                channel: ChannelType::RGB8U,
            }),
        }?;

        /*let dataproduct_subtype = properties.get_dataproduct_subtype().clone();
        let colored = if tex_storing_fits {
            false
        } else {
            if let Some(subtypes) = &dataproduct_subtype {
                subtypes.iter().any(|subtype| subtype == "color")
            } else {
                false
            }
        };*/

        let empty_image = EmptyTileImage::new(tile_size, format.get_channel());

        let texture_size = std::cmp::min(512, tile_size << max_depth_tile);
        //let texture_size = tile_size;
        let num_tile_per_side_texture = (texture_size / tile_size) as usize;

        let delta_depth = math::utils::log_2_unchecked(num_tile_per_side_texture) as u8;
        let num_tiles_per_texture = num_tile_per_side_texture * num_tile_per_side_texture;

        let max_depth_texture = max_depth_tile - delta_depth;
        let size_tile_uv = 1_f32 / ((1 << delta_depth) as f32);

        let frame = properties.get_frame();
        let sky_fraction = properties.get_sky_fraction().unwrap_or(1.0);

        let is_allsky = sky_fraction >= 1.0;

        let min_depth_tile = properties.get_min_order().unwrap_or(0);
        let min_depth_texture = if min_depth_tile >= delta_depth {
            min_depth_tile - delta_depth
        } else {
            0
        };
        let hips_config = HiPSConfig {
            creator_did,
            // HiPS name
            root_url: root_url.to_string(),
            // Tile size & blank tile data
            empty_image,
            // Texture config
            // The size of the texture images
            texture_size,
            // Delta depth i.e. log2(texture_size / tile_size)
            delta_depth,
            // Num tiles per texture
            num_tiles_per_texture,
            // Max depth of the current HiPS tiles
            max_depth_texture,
            max_depth_tile,
            min_depth_texture,
            min_depth_tile,

            is_allsky,

            fits_metadata: false,
            scale: 1.0,
            offset: 0.0,
            blank: -1.0, // by default, set it to -1

            tex_storing_fits,
            tex_storing_integers,
            tex_storing_unsigned_int,

            size_tile_uv,
            frame,
            bitpix,
            format,
            tile_size,
            //dataproduct_subtype,
            //colored,
        };

        Ok(hips_config)
    }

    pub fn set_image_fmt(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        let format = match ext {
            ImageExt::Fits => {
                // Check the bitpix to determine the internal format of the tiles
                if let Some(bitpix) = self.bitpix {
                    let channel = (match bitpix {
                        #[cfg(feature = "webgl2")]
                        8 => {
                            self.tex_storing_fits = true;
                            self.tex_storing_unsigned_int = true;
                            Ok(ChannelType::R8UI)
                        }
                        #[cfg(feature = "webgl2")]
                        16 => {
                            self.tex_storing_fits = true;
                            self.tex_storing_integers = true;
                            Ok(ChannelType::R16I)
                        }
                        #[cfg(feature = "webgl2")]
                        32 => {
                            self.tex_storing_fits = true;
                            self.tex_storing_integers = true;
                            Ok(ChannelType::R32I)
                        }
                        -32 => {
                            self.tex_storing_fits = true;
                            self.tex_storing_integers = false;
                            Ok(ChannelType::R32F)
                        }
                        -64 => {
                            self.tex_storing_fits = true;
                            self.tex_storing_integers = false;
                            //Err(JsValue::from_str("f64 FITS files not supported"))
                            Ok(ChannelType::R64F)
                        }
                        _ => Err(JsValue::from_str(
                            "Fits tiles exists but the BITPIX is not correct in the property file",
                        )),
                    })?;

                    Ok(ImageFormatType { ext, channel })
                } else {
                    Err(JsValue::from_str(
                        "Fits tiles exists but the BITPIX is not found",
                    ))
                }
            }
            ImageExt::Png | ImageExt::Webp => {
                self.tex_storing_fits = false;
                self.tex_storing_unsigned_int = false;
                self.tex_storing_integers = false;
                Ok(ImageFormatType {
                    ext,
                    channel: ChannelType::RGBA8U,
                })
            }
            ImageExt::Jpeg => {
                self.tex_storing_fits = false;
                self.tex_storing_unsigned_int = false;
                self.tex_storing_integers = false;
                Ok(ImageFormatType {
                    ext,
                    channel: ChannelType::RGB8U,
                })
            }
        }?;

        self.format = format;

        // Recompute the empty image
        self.empty_image = EmptyTileImage::new(self.tile_size, self.format.get_channel());

        // Recompute if the survey will be colored or not
        /*self.colored = if self.tex_storing_fits {
            false
        } else {
            if let Some(subtypes) = &self.dataproduct_subtype {
                subtypes.iter().any(|subtype| subtype == "color")
            } else {
                false
            }
        };*/

        Ok(())
    }

    #[inline(always)]
    pub fn get_root_url(&self) -> &str {
        &self.root_url
    }

    #[inline(always)]
    pub fn set_root_url(&mut self, root_url: String) {
        self.root_url = root_url;
    }

    #[inline(always)]
    pub fn set_fits_metadata(&mut self, bscale: f32, bzero: f32, blank: f32) {
        self.scale = bscale;
        self.offset = bzero;
        self.blank = blank;
        self.fits_metadata = true;
    }

    #[inline(always)]
    pub fn delta_depth(&self) -> u8 {
        self.delta_depth
    }

    #[inline(always)]
    pub fn num_tiles_per_texture(&self) -> usize {
        self.num_tiles_per_texture
    }

    #[inline(always)]
    pub fn get_texture_size(&self) -> i32 {
        self.texture_size
    }

    #[inline(always)]
    pub fn get_min_depth_tile(&self) -> u8 {
        self.min_depth_tile
    }

    #[inline(always)]
    pub fn get_min_depth_texture(&self) -> u8 {
        self.min_depth_texture
    }

    #[inline(always)]
    pub fn get_creator_did(&self) -> &str {
        &self.creator_did
    }

    #[inline(always)]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_size
    }

    #[inline(always)]
    pub fn get_max_depth_texture(&self) -> u8 {
        self.max_depth_texture
    }

    #[inline(always)]
    pub fn get_max_depth_tile(&self) -> u8 {
        self.max_depth_tile
    }

    #[inline(always)]
    pub fn get_frame(&self) -> CooSystem {
        self.frame
    }

    #[inline(always)]
    pub fn get_format(&self) -> ImageFormatType {
        self.format
    }

    #[inline(always)]
    pub fn is_colored(&self) -> bool {
        self.format.is_colored()
    }

    #[inline(always)]
    pub fn get_default_image(&self) -> &EmptyTileImage {
        &self.empty_image
    }
}

use al_core::shader::{SendUniforms, ShaderBound};

impl SendUniforms for HiPSConfig {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send max depth
        shader
            .attach_uniform("max_depth", &(self.max_depth_texture as i32))
            .attach_uniform("size_tile_uv", &self.size_tile_uv)
            .attach_uniform("tex_storing_fits", &self.tex_storing_fits)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);

        shader
    }
}
