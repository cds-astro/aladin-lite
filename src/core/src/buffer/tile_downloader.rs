use super::{CompressedImageRequest, FitsImageRequest, ResolvedStatus, TileRequest};

use crate::buffer::HiPSConfig;

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 32;

struct RequestSystem {
    reqs: Box<[Option<TileRequest>; NUM_EVENT_LISTENERS]>,
    //start_fits_req_idx: usize,
    free_slots_idx: Vec<usize>,
}
use super::image::ImageRequestType;
use crate::buffer::image::ImageRequest;
use al_core::format::{R32F, RGB8U, RGBA8U};

#[cfg(feature = "webgl2")]
use al_core::format::{R16I, R32I, R8UI};


impl RequestSystem {
    fn new() -> Self {
        let mut reqs: Vec<Option<TileRequest>> = Vec::with_capacity(NUM_EVENT_LISTENERS);
        for _ in 0..NUM_EVENT_LISTENERS {
            reqs.push(None);
        }

        let reqs = reqs.into_boxed_slice();
        let reqs = unsafe {
            Box::from_raw(Box::into_raw(reqs) as *mut [Option<TileRequest>; NUM_EVENT_LISTENERS])
        };

        let free_slots_idx = (0..NUM_EVENT_LISTENERS).into_iter().collect();

        RequestSystem {
            reqs,
            free_slots_idx,
        }
    }

    fn check_send(&mut self, format: ImageFormatType) -> Option<&mut TileRequest> {
        if self.free_slots_idx.is_empty() {
            return None;
        }

        let free_idx = self.free_slots_idx.pop().unwrap();

        #[cfg(feature = "webgl1")]
        match format {
            ImageFormatType::RGBA8U => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::PNGRGBA8UImageReq(
                    <CompressedImageRequest as ImageRequest<RGBA8U>>::new(),
                )));
            }
            ImageFormatType::RGB8U => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::JPGRGB8UImageReq(
                    <CompressedImageRequest as ImageRequest<RGB8U>>::new(),
                )));
            }
            ImageFormatType::R32F => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::FitsR32FImageReq(
                    <FitsImageRequest as ImageRequest<R32F>>::new(),
                )));
            }
            _ => unimplemented!(),
        }
        #[cfg(feature = "webgl2")]
        match format {
            ImageFormatType::RGBA8U => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::PNGRGBA8UImageReq(
                    <CompressedImageRequest as ImageRequest<RGBA8U>>::new(),
                )));
            }
            ImageFormatType::RGB8U => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::JPGRGB8UImageReq(
                    <CompressedImageRequest as ImageRequest<RGB8U>>::new(),
                )));
            }
            ImageFormatType::R32F => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::FitsR32FImageReq(
                    <FitsImageRequest as ImageRequest<R32F>>::new(),
                )));
            }
            ImageFormatType::R8UI => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::FitsR8UIImageReq(
                    <FitsImageRequest as ImageRequest<R8UI>>::new(),
                )));
            }
            ImageFormatType::R16I => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::FitsR16IImageReq(
                    <FitsImageRequest as ImageRequest<R16I>>::new(),
                )));
            }
            ImageFormatType::R32I => {
                self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::FitsR32IImageReq(
                    <FitsImageRequest as ImageRequest<R32I>>::new(),
                )));
            }
            _ => unimplemented!(),
        }

        self.reqs[free_idx].as_mut()
    }

    fn resolved_tiles(
        &mut self,
        tiles_uploaded_on_gpu: &HashSet<Tile>,
        surveys: &ImageSurveys,
    ) -> HashMap<Tile, TileResolved> {
        let mut results = HashMap::new();

        for (idx, req) in self.reqs.iter_mut().enumerate() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            if let Some(request) = req {
                if request.is_resolved() {
                    let tile = request.get_tile();
                    // A tile request can be reused if its cell texture is available/readable
                    // by the GPU
                    let available_req = tiles_uploaded_on_gpu.contains(tile);
                    if available_req {
                        *req = None;
                        self.free_slots_idx.push(idx);
                    } else {
                        // Time when the tile has been received
                        if let Some(survey) = surveys.get(&tile.root_url) {
                            let time_req = request.get_time_request();

                            let tile_resolved = match request.resolve_status() {
                                ResolvedStatus::Missing => TileResolved::Missing { time_req },
                                ResolvedStatus::Found => {
                                    let config = survey.get_textures().config();
    
                                    let image = request.get_image(config.get_tile_size());
                                    if let Ok(image) = image {
                                        TileResolved::Found { image, time_req }
                                    } else {
                                        let _err = image.err().unwrap();
                                        TileResolved::Missing { time_req }
                                    }
                                }
                                _ => unreachable!(),
                            };
    
                            results.insert(tile.clone(), tile_resolved);
                        }
                    }
                }
            }
        }

        results
    }

    fn clear(&mut self) {
        for req in self.reqs.iter_mut() {
            *req = None;
        }
        self.free_slots_idx = (0..NUM_EVENT_LISTENERS).into_iter().collect();
    }
}

use crate::healpix_cell::HEALPixCell;
// A tile is described by an image survey
// and an HEALPix cell
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Tile {
    pub cell: HEALPixCell,
    pub root_url: String,
    pub format: ImageFormatType,
}

impl Tile {
    pub fn new(cell: &HEALPixCell, config: &HiPSConfig) -> Self {
        Tile {
            cell: *cell,
            root_url: config.root_url.to_string(),
            format: config.format(),
        }
    }

    fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}

use std::collections::{HashSet};
pub struct TileDownloader {
    // Waiting cells to be loaded
    tiles_to_req: Vec<Tile>,
    base_tiles_to_req: Vec<Tile>,

    requests: RequestSystem,

    requested_tiles: HashSet<Tile>,
}

use al_core::format::ImageFormatType;

use super::image::RetrievedImageType;
use crate::time::Time;
pub enum TileResolved {
    Missing {
        time_req: Time,
    },
    Found {
        image: RetrievedImageType,
        time_req: Time,
    },
}
use std::collections::HashMap;
pub type ResolvedTiles = HashMap<Tile, TileResolved>;

use crate::ImageSurveys;
use al_core::log::*;
use wasm_bindgen::JsValue;
impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests = RequestSystem::new();
        let tiles_to_req = Vec::new();
        let requested_tiles = HashSet::with_capacity(64);
        let base_tiles_to_req = vec![];

        Self {
            tiles_to_req,
            base_tiles_to_req,

            requests,
            requested_tiles,
        }
    }

    pub fn clear_requests(&mut self) {
        self.tiles_to_req.clear();

        self.requests.clear();
        self.requested_tiles.clear();
    }

    // Register further tile requests to launch
    pub fn request_tiles(&mut self, tiles: Vec<Tile>) {
        // Remove the ancient requests
        self.tiles_to_req.clear();

        for tile in tiles.into_iter() {
            self.request_tile(tile);
        }
    }

    fn request_tile(&mut self, tile: Tile /*, max_num_requested_tiles: usize*/) {
        let already_requested = self.requested_tiles.contains(&tile);
        // The cell is not already requested
        if !already_requested && !tile.is_root() {
            self.tiles_to_req.push(tile);
        }
    }

    // Retrieve the tiles that have been resolved:
    // Two possibilities:
    // * The image have been found and retrieved
    // * The image is missing
    pub fn get_resolved_tiles(
        &mut self,
        available_tiles: &HashSet<Tile>,
        surveys: &ImageSurveys,
    ) -> ResolvedTiles {
        let resolved_tiles = self.requests.resolved_tiles(available_tiles, surveys);
        for (tile, _resolve) in resolved_tiles.iter() {
            if self.requested_tiles.contains(tile) {
                self.requested_tiles.remove(tile);
            }
        }

        resolved_tiles
    }

    pub fn try_sending_tile_requests(&mut self) -> Result<(), JsValue> {
        // Try sending the fits tile requests
        let mut is_remaining_req =
            !self.tiles_to_req.is_empty() || !self.base_tiles_to_req.is_empty();

        let mut downloader_overloaded = false;

        while is_remaining_req && !downloader_overloaded {
            let mut base_tile_requested = false;
            let tile = if let Some(base_tile) = self.base_tiles_to_req.last() {
                base_tile_requested = true;
                base_tile
            } else {
                self.tiles_to_req.last().unwrap()
            };

            if let Some(available_req) = self.requests.check_send(tile.format.clone()) {
                let tile = if base_tile_requested {
                    // Send in priority requests to get the base tiles
                    self.base_tiles_to_req.pop().unwrap()
                } else {
                    // Otherwise requests the other tiles
                    self.tiles_to_req.pop().unwrap()
                };

                is_remaining_req =
                    !self.tiles_to_req.is_empty() || !self.base_tiles_to_req.is_empty();
                self.requested_tiles.insert(tile.clone());

                available_req.send(tile)?;
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;
            }
        }

        Ok(())
    }

    pub fn request_base_tiles(&mut self, config: &HiPSConfig) {
        // Request base tiles
        for idx in 0..12 {
            let texture_cell = HEALPixCell(0, idx);
            for cell in texture_cell.get_tile_cells(config) {
                let tile = Tile {
                    root_url: config.root_url.clone(),
                    format: config.format(),
                    cell,
                };

                self.request_base_tile(tile);
            }
        }
    }

    fn request_base_tile(&mut self, tile: Tile /*, max_num_requested_tiles: usize*/) {
        // Resize the requested tiles
        let already_requested = self.requested_tiles.contains(&tile);
        // The cell is not already requested
        if !already_requested {
            // Add to the tiles requested
            self.base_tiles_to_req.push(tile);
        }
    }
}
