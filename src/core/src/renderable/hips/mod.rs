pub mod config;

pub mod d2;
pub mod d3;
pub mod raytracing;
mod triangulation;
pub mod uv;

pub use d2::HiPS2D;

use crate::downloader::request::allsky::Allsky;
use crate::renderable::HiPSConfig;
use crate::time::Time;
use crate::CameraViewPort;
use crate::HEALPixCell;
use crate::HEALPixCoverage;
use crate::LonLatT;
use crate::WebGlContext;
use al_api::hips::ImageExt;
use al_core::image::Image;
use wasm_bindgen::JsValue;

mod subdivide;

trait HpxTile {
    // Getter
    // Returns the current time if the texture is not full
    fn start_time(&self) -> Time;

    fn time_request(&self) -> Time;

    fn cell(&self) -> &HEALPixCell;
}

pub trait HpxTileBuffer {
    type T: HpxTile;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue>
    where
        Self: Sized;

    fn set_image_ext(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue>;

    // Return if tiles did become available
    fn reset_available_tiles(&mut self) -> bool;

    /// Accessors
    fn get(&self, cell: &HEALPixCell) -> Option<&Self::T>;

    fn contains(&self, cell: &HEALPixCell) -> bool;

    // Get the nearest parent tile found in the CPU buffer
    fn get_nearest_parent(&self, cell: &HEALPixCell) -> Option<HEALPixCell> {
        /*if cell.is_root() {
            // Root cells are in the buffer by definition
            Some(*cell)
        } else {*/
        let mut parent_cell = cell.parent();

        while !self.contains(&parent_cell) && !parent_cell.is_root() {
            parent_cell = parent_cell.parent();
        }

        if self.contains(&parent_cell) {
            Some(parent_cell)
        } else {
            None
        }
        //}
    }

    fn config_mut(&mut self) -> &mut HiPSConfig;
    fn config(&self) -> &HiPSConfig;

    fn read_pixel(&self, pos: &LonLatT<f64>, camera: &CameraViewPort) -> Result<JsValue, JsValue>;
}

use crate::downloader::query;
use crate::renderable::hips::HiPS::{D2, D3};
use crate::renderable::HiPS3D;
use crate::ProjectionType;
pub enum HiPS {
    D2(HiPS2D),
    D3(HiPS3D),
}

impl HiPS {
    pub fn look_for_new_tiles(
        &mut self,
        camera: &CameraViewPort,
        proj: &ProjectionType,
    ) -> Option<Vec<HEALPixCell>> {
        match self {
            D2(hips) => hips.look_for_new_tiles(camera, proj).map(|it| it.collect()),
            D3(hips) => hips.look_for_new_tiles(camera, proj).map(|it| it.collect()),
        }
    }

    // Position given is in the camera space
    pub fn read_pixel(
        &self,
        p: &LonLatT<f64>,
        camera: &CameraViewPort,
    ) -> Result<JsValue, JsValue> {
        match self {
            D2(hips) => hips.read_pixel(p, camera),
            D3(hips) => hips.read_pixel(p, camera),
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
    pub fn get_config_mut(&mut self) -> &mut HiPSConfig {
        match self {
            D2(hips) => hips.get_config_mut(),
            D3(hips) => hips.get_config_mut(),
        }
    }

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        match self {
            D2(hips) => hips.set_image_ext(ext),
            D3(hips) => hips.set_image_ext(ext),
        }
    }

    #[inline]
    pub fn set_moc(&mut self, moc: HEALPixCoverage) {
        match self {
            D2(hips) => hips.set_moc(moc),
            D3(hips) => hips.set_moc(moc),
        }
    }

    #[inline]
    pub fn get_tile_query(&self, cell: &HEALPixCell) -> query::Tile {
        match self {
            HiPS::D2(hips) => hips.get_tile_query(cell),
            HiPS::D3(hips) => hips.get_tile_query(cell),
        }
    }

    #[inline]
    pub fn add_allsky(&mut self, allsky: Allsky) -> Result<(), JsValue> {
        match self {
            HiPS::D2(hips) => hips.add_allsky(allsky),
            HiPS::D3(hips) => hips.add_allsky(allsky),
        }
    }

    pub fn is_allsky(&self) -> bool {
        self.get_config().is_allsky
    }
}
