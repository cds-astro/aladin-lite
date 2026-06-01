use super::blend::BlendCfg;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HiPSCfg {
    /// Layer name
    pub layer: String,

    /// The HiPS metadata
    pub properties: HiPSProperties,
    /// Its color
    pub meta: ImageMetadata,
}

impl HiPSCfg {
    pub fn get_layer(&self) -> &str {
        &self.layer
    }

    pub fn get_properties(&self) -> &HiPSProperties {
        &self.properties
    }
}

use crate::coo_system::CooSystem;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct HiPSProperties {
    // Associated with the HiPS
    url: String,
    max_order: u8,
    coo_frame: CooSystem,
    tile_size: i32,
    formats: Vec<ImageExt>,

    #[allow(unused)]
    is_planetary_body: Option<bool>,

    bitpix: Option<i32>,
    sky_fraction: Option<f32>,
    min_order: Option<u8>,

    initial_fov: Option<f64>,
    initial_ra: Option<f64>,
    initial_dec: Option<f64>,
    // HiPS cube
    cube_depth: Option<u32>,

    // HiPS 3D keywords
    order_freq: Option<u8>,
    tile_depth: Option<u8>,

    /// Start of spectral coordinates (in meters)
    em_min: Option<f32>,
    /// End of spectral coordinates (in meters)
    em_max: Option<f32>,

    // Parametrable by the user
    #[allow(unused)]
    min_cutout: Option<f32>,
    #[allow(unused)]
    max_cutout: Option<f32>,

    dataproduct_type: Option<DataproductType>,

    creator_did: String,

    request_credentials: String,
    request_mode: String,
}

impl HiPSProperties {
    #[inline(always)]
    pub fn get_hips_order_freq(&self) -> Option<u8> {
        self.order_freq
    }
    #[inline(always)]
    pub fn get_hips_tile_depth(&self) -> Option<u8> {
        self.tile_depth
    }

    #[inline(always)]
    pub fn get_dataproduct_type(&self) -> Option<DataproductType> {
        self.dataproduct_type
    }

    #[inline(always)]
    pub fn get_url(&self) -> &str {
        &self.url
    }

    #[inline(always)]
    pub fn get_creator_did(&self) -> &str {
        &self.creator_did
    }

    #[inline(always)]
    pub fn get_max_order(&self) -> u8 {
        self.max_order
    }

    #[inline(always)]
    pub fn get_min_order(&self) -> Option<u8> {
        self.min_order
    }

    #[inline(always)]
    pub fn get_cube_depth(&self) -> Option<u32> {
        self.cube_depth
    }

    #[inline(always)]
    pub fn get_bitpix(&self) -> Option<i32> {
        self.bitpix
    }

    #[inline(always)]
    pub fn get_formats(&self) -> &[ImageExt] {
        &self.formats[..]
    }

    #[inline(always)]
    pub fn get_tile_size(&self) -> i32 {
        self.tile_size
    }

    #[inline(always)]
    pub fn get_frame(&self) -> CooSystem {
        self.coo_frame
    }

    #[inline(always)]
    pub fn get_sky_fraction(&self) -> Option<f32> {
        self.sky_fraction
    }

    #[inline(always)]
    pub fn get_initial_fov(&self) -> Option<f64> {
        self.initial_fov
    }

    #[inline(always)]
    pub fn get_initial_ra(&self) -> Option<f64> {
        self.initial_ra
    }

    #[inline(always)]
    pub fn get_initial_dec(&self) -> Option<f64> {
        self.initial_dec
    }

    #[inline(always)]
    pub fn get_request_credentials(&self) -> &str {
        &self.request_credentials
    }

    #[inline(always)]
    pub fn get_request_mode(&self) -> &str {
        &self.request_mode
    }

    #[inline(always)]
    pub fn get_em_min(&self) -> Option<f32> {
        self.em_min
    }

    #[inline(always)]
    pub fn get_em_max(&self) -> Option<f32> {
        self.em_max
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[wasm_bindgen]
#[serde(rename_all = "camelCase")]
pub enum ImageExt {
    Fits,
    Jpeg,
    Png,
    Webp,
    #[serde(alias = "fits.fz")]
    FitsFz,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[wasm_bindgen]
#[serde(rename_all = "camelCase")]
pub enum DataproductType {
    #[serde(rename = "spectral-cube")]
    SpectralCube,
    Image,
    Cube,
}

impl std::fmt::Display for ImageExt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageExt::FitsFz => write!(f, "fits.fz"),
            ImageExt::Fits => write!(f, "fits"),
            ImageExt::Png => write!(f, "png"),
            ImageExt::Jpeg => write!(f, "jpg"),
            ImageExt::Webp => write!(f, "webp"),
        }
    }
}

use serde::Serialize;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransferFunction {
    #[default]
    Linear,
    Sqrt,
    Log,
    Asinh,
    Pow2,
}

impl TransferFunction {
    pub fn new(id: &str) -> Self {
        if id.contains("linear") {
            TransferFunction::Linear
        } else if id.contains("pow2") {
            TransferFunction::Pow2
        } else if id.contains("log") {
            TransferFunction::Log
        } else if id.contains("sqrt") {
            TransferFunction::Sqrt
        } else {
            TransferFunction::Asinh
        }
    }
}

impl From<String> for TransferFunction {
    fn from(id: String) -> Self {
        TransferFunction::new(&id)
    }
}

use crate::colormap::CmapLabel;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct ImageMetadata {
    /// Color config
    // transfer function called before evaluating the colormap
    pub stretch: TransferFunction,
    // low cut
    pub min_cut: Option<f32>,
    // high cut
    pub max_cut: Option<f32>,
    // flag to tell the colormap is queried reversed
    pub reversed: bool,
    // the colormap
    #[wasm_bindgen(skip)]
    pub colormap: CmapLabel,
    /// tonal color tuning factors
    pub gamma: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub brightness: f32,

    // Blending config
    #[serde(default)]
    pub blending: BlendCfg,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// the current format chosen
    pub img_format: ImageExt,
}

fn default_opacity() -> f32 {
    1.0
}

impl ImageMetadata {
    pub fn visible(&self) -> bool {
        self.opacity > 0.0
    }
}
