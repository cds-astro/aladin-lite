pub mod config;

pub mod d2;
pub mod d3;
pub mod raytracing;
pub mod tile_heap;
mod triangulation;
pub mod uv;

pub use d2::HiPS2D;

use crate::browser_support::BrowserFeaturesSupport;
use crate::renderable::HiPSConfig;
use crate::tile_fetcher::TileFetcherQueue;
use crate::CameraViewPort;
use crate::WebGlContext;
use al_api::hips::ImageExt;
use wasm_bindgen::JsValue;

mod subdivide;

pub(crate) trait HpxTileBuffer {
    type T;
    type C;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue>
    where
        Self: Sized;

    fn set_image_ext(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue>;

    // Return if tiles did become available
    fn reset_available_tiles(&mut self) -> bool;

    /// Accessors
    fn get(&self, cell: &Self::C) -> Option<&Self::T>;

    fn contains(&self, cell: &Self::C) -> bool;

    fn config_mut(&mut self) -> &mut HiPSConfig;
    fn config(&self) -> &HiPSConfig;
}

use crate::renderable::hips::HiPS::{D2, D3};
use crate::renderable::HiPS3D;
use crate::ProjectionType;

#[allow(clippy::large_enum_variant)]
pub enum HiPS {
    D2(HiPS2D),
    D3(HiPS3D),
}

impl HiPS {
    pub fn look_for_new_tiles(
        &mut self,
        tile_fetcher: &mut TileFetcherQueue,
        camera: &CameraViewPort,
        browser_features_support: &BrowserFeaturesSupport,
    ) {
        match self {
            D2(hips) => hips.look_for_new_tiles(tile_fetcher, camera, browser_features_support),
            D3(hips) => hips.look_for_new_tiles(tile_fetcher, camera, browser_features_support),
        }
    }

    // Position given is in the camera space
    pub fn read_pixel(
        &self,
        x: f64,
        y: f64,
        camera: &CameraViewPort,
        proj: &ProjectionType,
    ) -> Result<JsValue, JsValue> {
        match self {
            D2(hips) => hips.read_pixel(x, y, camera, proj),
            // FIXME todo
            D3(_) => Ok(JsValue::null()),
        }
    }

    #[inline]
    pub fn get_config(&self) -> &HiPSConfig {
        match self {
            D2(hips) => hips.get_config(),
            D3(hips) => hips.get_config(),
        }
    }

    #[inline]
    pub fn set_root_url(&mut self, root_url: String) {
        match self {
            D2(hips) => hips.get_config_mut().set_root_url(root_url),
            D3(hips) => hips.get_config_mut().set_root_url(root_url),
        }
    }

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        match self {
            D2(hips) => hips.set_image_ext(ext),
            D3(hips) => hips.set_image_ext(ext),
        }
    }

    pub fn is_allsky(&self) -> bool {
        self.get_config().is_allsky
    }

    pub fn set_fits_params(&mut self, bscale: f32, bzero: f32, blank: Option<f32>) {
        match self {
            HiPS::D2(hips) => hips.set_fits_params(bscale, bzero, blank),
            HiPS::D3(hips) => hips.set_fits_params(bscale, bzero, blank),
        }
    }

    pub(crate) fn get_fits_params(&self) -> &Option<FitsParams> {
        match self {
            HiPS::D2(hips) => &hips.fits_params,
            HiPS::D3(hips) => &hips.fits_params,
        }
    }
}

pub(crate) struct FitsParams {
    pub bscale: f32,
    pub bzero: f32,
    pub blank: Option<f32>,
}

use al_core::shader::{SendUniforms, ShaderBound};
impl SendUniforms for FitsParams {
    // Send only the allsky textures
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
            .attach_uniform("scale", &self.bscale)
            .attach_uniform("offset", &self.bzero);

        if let Some(blank) = &self.blank {
            shader.attach_uniform("blank", blank);
        }

        shader
    }
}
