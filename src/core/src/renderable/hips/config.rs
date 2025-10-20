use al_api::hips::{DataproductType, ImageExt};

use crate::math::spectra::Freq;
use al_core::image::format::ImageFormatType;
use al_core::texture::format::PixelType;
use web_sys::{RequestCredentials, RequestMode};
#[derive(Debug)]
pub struct HiPSConfig {
    pub root_url: String,
    // HiPS image format
    // TODO: Make that independant of the HiPS but of the ImageFormat

    // Size of the tiles
    pub tile_size: i32,

    // Number of slices for HiPS cubes
    pub cube_depth: Option<u32>,

    /// Max depth of the current HiPS tiles
    pub max_depth_tile: u8,
    /// Min depth of the current HiPS tiles
    min_depth_tile: u8,
    /// Max depth in the frequency axis (HiPS3D only)
    pub max_depth_freq: Option<u8>,

    /// Start of spectral coordinates (in meters)
    pub em_min: Option<Freq>,
    /// End of spectral coordinates (in meters)
    pub em_max: Option<Freq>,

    // For HiPS3D
    pub tile_depth: Option<u8>,

    pub is_allsky: bool,

    pub frame: CooSystem,
    // For FITS HiPSes
    pub bitpix: Option<i32>,
    format: ImageFormatType,

    pub dataproduct_type: DataproductType,

    pub creator_did: String,

    pub request_credentials: RequestCredentials,
    pub request_mode: RequestMode,
}

use crate::{math::spectra::Wavelength, HiPSProperties};
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
        let cube_depth = properties.get_cube_depth();
        // Define the size of the 2d texture array depending on the
        // characterics of the client

        let max_depth_tile = properties.get_max_order();
        let tile_size = properties.get_tile_size();
        // Assert size is a power of two
        // Determine the size of the texture to copy
        // it cannot be > to 512x512px

        let bitpix = properties.get_bitpix();

        if !properties.get_formats().contains(&img_ext) {
            return Err(js_sys::Error::new("HiPS format not available").into());
        }

        let format = match img_ext {
            ImageExt::Fits | ImageExt::FitsFz => {
                // Check the bitpix to determine the internal format of the tiles
                if let Some(bitpix) = bitpix {
                    let fmt = (match bitpix {
                        8 => Ok(PixelType::R8U),
                        16 => Ok(PixelType::R16I),
                        32 => Ok(PixelType::R32I),
                        -32 => Ok(PixelType::R32F),
                        -64 => Ok(PixelType::R32F),
                        64 => Ok(PixelType::R32I),
                        _ => Err(JsValue::from_str(
                            "Fits tiles exists but the BITPIX is not correct in the property file",
                        )),
                    })?;

                    Ok(ImageFormatType { ext: img_ext, fmt })
                } else {
                    Err(JsValue::from_str(
                        "Fits tiles exists but the BITPIX is not found",
                    ))
                }
            }
            ImageExt::Png | ImageExt::Webp => Ok(ImageFormatType {
                ext: img_ext,
                fmt: PixelType::RGBA8U,
            }),
            ImageExt::Jpeg => Ok(ImageFormatType {
                ext: img_ext,
                fmt: PixelType::RGB8U,
            }),
        }?;

        let frame = properties.get_frame();
        let sky_fraction = properties.get_sky_fraction().unwrap_or(1.0);

        let is_allsky = sky_fraction >= 1.0;

        let min_depth_tile = properties.get_min_order().unwrap_or(0);

        let request_credentials = match properties.get_request_credentials() {
            "include" => RequestCredentials::Include,
            "same-origin" => RequestCredentials::SameOrigin,
            "omit" => RequestCredentials::Omit,
            _ => RequestCredentials::Omit,
        };

        let request_mode = match properties.get_request_mode() {
            "cors" => RequestMode::Cors,
            "no-cors" => RequestMode::NoCors,
            "same-origin" => RequestMode::SameOrigin,
            _ => RequestMode::Cors,
        };

        let dataproduct_type = properties.get_dataproduct_type().ok_or(JsValue::from_str(
            "dataproduct_type keyword is required in the HiPS properties file",
        ))?;
        let max_depth_freq = properties.get_hips_order_freq();
        let tile_depth = properties.get_hips_tile_depth();

        let em_min: Option<Freq> = properties
            .get_em_max()
            .map(|lambda| Wavelength(lambda as f64).into());
        let em_max: Option<Freq> = properties
            .get_em_min()
            .map(|lambda| Wavelength(lambda as f64).into());

        let hips_config = HiPSConfig {
            creator_did,
            // HiPS name
            root_url: root_url.to_string(),
            max_depth_tile,
            min_depth_tile,

            is_allsky,

            // HiPSCube
            cube_depth,

            em_min,
            em_max,

            // HiPS3D
            tile_depth,
            max_depth_freq,

            frame,
            bitpix,
            format,
            tile_size,
            request_credentials,
            request_mode,
            dataproduct_type,
        };

        Ok(hips_config)
    }

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        let format = match ext {
            ImageExt::Fits | ImageExt::FitsFz => {
                // Check the bitpix to determine the internal format of the tiles
                if let Some(bitpix) = self.bitpix {
                    let fmt = (match bitpix {
                        8 => Ok(PixelType::R8U),
                        16 => Ok(PixelType::R16I),
                        32 => Ok(PixelType::R32I),
                        64 => Ok(PixelType::R32I),
                        -32 => Ok(PixelType::R32F),
                        -64 => Ok(PixelType::R32F),
                        _ => Err(JsValue::from_str(
                            "Fits tiles exists but the BITPIX is not correct in the property file",
                        )),
                    })?;

                    Ok(ImageFormatType { ext, fmt })
                } else {
                    Err(JsValue::from_str(
                        "Fits tiles exists but the BITPIX is not found",
                    ))
                }
            }
            ImageExt::Png | ImageExt::Webp => Ok(ImageFormatType {
                ext,
                fmt: PixelType::RGBA8U,
            }),
            ImageExt::Jpeg => Ok(ImageFormatType {
                ext,
                fmt: PixelType::RGB8U,
            }),
        }?;

        self.format = format;

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
    pub fn get_cube_depth(&self) -> Option<u32> {
        self.cube_depth
    }

    #[inline(always)]
    pub fn allsky_tile_size(&self) -> i32 {
        (self.get_tile_size() << 3).min(512)
    }

    #[inline(always)]
    pub fn get_min_depth_tile(&self) -> u8 {
        self.min_depth_tile
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
    pub fn get_request_credentials(&self) -> RequestCredentials {
        self.request_credentials
    }

    #[inline(always)]
    pub fn get_request_mode(&self) -> RequestMode {
        self.request_mode
    }
}

use al_core::shader::{SendUniforms, ShaderBound};

impl SendUniforms for HiPSConfig {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send max depth
        shader.attach_uniform("max_depth", &(self.max_depth_tile as i32));

        shader
    }
}
