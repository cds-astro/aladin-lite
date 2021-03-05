use crate::image_fmt::FormatImageType;

#[derive(Clone, Debug)]
struct TileConfig {
    // The size of the tile in the texture
    width: i32,
    default: Rc<TileArrayBufferImage>,
    black_tile_value: f32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TileArrayBufferImage {
    F32(TileArrayBuffer<ArrayF32>),
    F64(TileArrayBuffer<ArrayF64>),
    U8(TileArrayBuffer<ArrayU8>),
    I16(TileArrayBuffer<ArrayI16>),
    I32(TileArrayBuffer<ArrayI32>),
}

use super::TileArrayBuffer;
use std::rc::Rc;

use super::{ArrayF32, ArrayF64, ArrayI16, ArrayI32, ArrayU8};
use crate::image_fmt::{FITS, JPG, PNG};
use crate::WebGl2Context;
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
    }
}

impl TileConfig {
    fn new(width: i32, format: FormatImageType) -> TileConfig {
        assert!(is_power_of_two(width as usize));
        let black_tile_value = 0.0;
        let default = Rc::new(create_black_tile(format, width, black_tile_value));
        TileConfig {
            width,
            black_tile_value,
            default,
        }
    }

    #[inline]
    pub fn get_black_tile(&self) -> Rc<TileArrayBufferImage> {
        self.default.clone()
    }

    #[inline]
    pub fn set_black_tile_value(&mut self, value: f32, format: FormatImageType) {
        if value != self.black_tile_value {
            self.black_tile_value = value;
            self.default = Rc::new(create_black_tile(format, self.width, self.black_tile_value));
        }
    }
}

#[derive(Debug)]
pub struct HiPSConfig {
    pub root_url: String,
    // HiPS image format
    format: FormatImageType,

    tile_config: TileConfig,

    // The size of the texture images
    pub texture_size: i32,
    // Delta depth i.e. log2(texture_size / tile_size)
    delta_depth: u8,
    // Num tiles per texture
    num_tiles_per_texture: usize,
    // Max depth of the current HiPS tiles
    max_depth_tile: u8,
    max_depth_texture: u8,
    num_textures_by_side_slice: i32,
    num_textures_by_slice: i32,
    num_slices: i32,
    num_textures: usize,

    // TODO: store this values in the ImageSurvey
    // These are proper to the survey (FITS one) and not
    // to a specific survey color
    pub scale: f32,
    pub offset: f32,
    pub blank: f32,

    pub tex_storing_integers: i32,
    pub tex_storing_fits: i32,
}

#[inline]
fn is_power_of_two(x: usize) -> bool {
    x & (x - 1) == 0
}

use crate::math;
use crate::{HiPSFormat, HiPSProperties};
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
impl HiPSConfig {
    pub fn new(_gl: &WebGl2Context, properties: &HiPSProperties) -> Result<HiPSConfig, JsValue> {
        let root_url = properties.url.clone();
        // Define the size of the 2d texture array depending on the
        // characterics of the client
        let num_textures_by_side_slice = 8;
        let num_textures_by_slice = num_textures_by_side_slice * num_textures_by_side_slice;
        let num_slices = 3;
        let num_textures = (num_textures_by_slice * num_slices) as usize;

        // Assert size is a power of two
        // Determine the size of the texture to copy
        // it cannot be > to 512x512px

        let fmt = &properties.format;
        let mut tex_storing_integers = 0;
        let mut tex_storing_fits = 0;
        let format: Result<_, JsValue> = match fmt {
            HiPSFormat::FITSImage { bitpix, .. } => {
                tex_storing_fits = 1;
                // Check the bitpix to determine the internal format of the tiles
                match bitpix {
                    8 => {
                        tex_storing_integers = 1;
                        Ok(FormatImageType::FITS(FITS::new(
                            WebGl2RenderingContext::R8UI as i32,
                        )))
                    }
                    16 => {
                        tex_storing_integers = 1;
                        Ok(FormatImageType::FITS(FITS::new(
                            WebGl2RenderingContext::R16I as i32,
                        )))
                    }
                    32 => {
                        tex_storing_integers = 1;
                        Ok(FormatImageType::FITS(FITS::new(
                            WebGl2RenderingContext::R32I as i32,
                        )))
                    }
                    -32 => {
                        tex_storing_integers = 0;
                        Ok(FormatImageType::FITS(FITS::new(
                            WebGl2RenderingContext::R32F as i32,
                        )))
                    }
                    -64 => {
                        tex_storing_integers = 0;
                        Ok(FormatImageType::FITS(FITS::new(
                            WebGl2RenderingContext::R32F as i32,
                        )))
                    }
                    _ => Err(
                        "Fits tiles exists but the BITPIX is not correct in the property file"
                            .to_string()
                            .into(),
                    ),
                }
            }
            HiPSFormat::Image { format } => {
                tex_storing_integers = 0;

                if format.contains("png") {
                    Ok(FormatImageType::PNG)
                } else if format.contains("jpeg") || format.contains("jpg") {
                    Ok(FormatImageType::JPG)
                } else {
                    Err(format!("{} Unrecognized image format", format).into())
                }
            }
        };
        let format = format?;
        let max_depth_tile = properties.max_order;
        let tile_size = properties.tile_size;

        let tile_config = TileConfig::new(tile_size, format);

        let texture_size = std::cmp::min(512, tile_size << max_depth_tile);
        let num_tile_per_side_texture = (texture_size / tile_size) as usize;

        let delta_depth = math::log_2(num_tile_per_side_texture as i32) as u8;
        let num_tiles_per_texture = num_tile_per_side_texture * num_tile_per_side_texture;

        let max_depth_texture = max_depth_tile - delta_depth;

        let hips_config = HiPSConfig {
            // HiPS name
            root_url,
            format,
            // Tile size & blank tile data
            tile_config,
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
            num_textures_by_side_slice,
            num_textures_by_slice,
            num_slices,
            num_textures,

            scale: 1.0,
            offset: 0.0,
            blank: 0.0,

            tex_storing_fits,
            tex_storing_integers,
        };

        Ok(hips_config)
    }

    #[inline]
    pub fn set_black_tile_value(&mut self, value: f32) {
        self.tile_config.set_black_tile_value(value, self.format);
    }

    #[inline]
    pub fn delta_depth(&self) -> u8 {
        self.delta_depth
    }

    #[inline]
    pub fn num_tiles_per_texture(&self) -> usize {
        self.num_tiles_per_texture
    }

    #[inline]
    pub fn get_texture_size(&self) -> i32 {
        self.texture_size
    }

    #[inline]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_config.width
    }

    #[inline]
    pub fn get_black_tile(&self) -> Rc<TileArrayBufferImage> {
        self.tile_config.get_black_tile()
    }

    #[inline]
    pub fn get_max_depth(&self) -> u8 {
        self.max_depth_texture
    }

    /*#[inline]
    pub fn get_max_tile_depth(&self) -> u8 {
        self.max_depth_tile
    }*/

    #[inline]
    pub fn num_textures(&self) -> usize {
        self.num_textures
    }

    #[inline]
    pub fn num_textures_by_side_slice(&self) -> i32 {
        self.num_textures_by_side_slice
    }

    #[inline]
    pub fn num_textures_by_slice(&self) -> i32 {
        self.num_textures_by_slice
    }

    #[inline]
    pub fn num_slices(&self) -> i32 {
        self.num_slices
    }

    #[inline]
    pub fn format(&self) -> FormatImageType {
        self.format
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;

impl SendUniforms for HiPSConfig {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send max depth
        shader
            .attach_uniform("max_depth", &(self.max_depth_texture as i32))
            .attach_uniform("size_tile_uv", &(1_f32 / ((8 << self.delta_depth) as f32)))
            .attach_uniform("tex_storing_integers", &(self.tex_storing_integers as f32))
            .attach_uniform("tex_storing_fits", &self.tex_storing_fits)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);

        shader
    }
}
