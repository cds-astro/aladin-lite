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
use js_sys::Uint16Array;


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
        self.reqs[free_idx] = Some(TileRequest::new(ImageRequestType::new(format)));
        self.reqs[free_idx].as_mut()
    }

    fn handle_received_tiles(
        &mut self,
        surveys: &mut ImageSurveys,
    ) -> Vec<Tile> {
        let mut tiles_received = Vec::new();

        for (idx, req) in self.reqs.iter_mut().enumerate() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            let mut handled_tile = false;

            if let Some(resp) = req {
                let tile = resp.get_tile();
                let time_req = resp.get_time_request();
                let status = resp.resolve_status();
                match status {
                    ResolvedStatus::Missing => {
                        let status = TileResolved::Missing { time_req };

                        surveys.add_resolved_tile(tile, status);
                        handled_tile = true;
                    },
                    ResolvedStatus::Found => {
                        let status = if let Some(survey) = surveys.get(&tile.root_url) {
                            let cfg = survey.get_textures().config();
                            if let Ok(image) = resp.get_image(cfg.get_tile_size()) {
                                TileResolved::Found { image, time_req }
                            } else {
                                TileResolved::Missing { time_req }
                            }
                        } else {
                            TileResolved::Missing { time_req }
                        };

                        // Ensure again if it totally resolved
                        // for image bitmap, it must need further processing to convert the blob received
                        // into a ImageBitmap => this is usually done by a JSPromise that we can't know
                        // when it will be processed.
                        surveys.add_resolved_tile(tile, status);
                        handled_tile = true;
                    },
                    ResolvedStatus::NotResolved => (),
                }

                if handled_tile {
                    // Signals that the tile has been handled (copied for the GPU)
                    tiles_received.push(tile.clone());
    
                    // Free the request to be used to download a new tile
                    self.free_slots_idx.push(idx);
                    *req = None;

                    break; // handle one tile per frame
                }
            }
        }

        tiles_received
    }

    fn clear(&mut self) {
        for req in self.reqs.iter_mut() {
            *req = None;
        }
        self.free_slots_idx = (0..NUM_EVENT_LISTENERS).into_iter().collect();
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
#[derive(Debug)]
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
use crate::request::Request;
use wasm_bindgen::JsCast;
use crate::buffer::ImageBitmap;
use super::tile::Tile;
use crate::healpix_cell::HEALPixCell;
use web_sys::window;
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
        self.base_tiles_to_req.clear();

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
            /*if tile.is_root() {
                self.base_tiles_to_req.push(tile);
            } else {*/
                self.tiles_to_req.push(tile);
            //}
        }
    }

    // Retrieve the tiles that have been resolved:
    // Two possibilities:
    // * The image have been found and retrieved
    // * The image is missing
    pub fn get_resolved_tiles(
        &mut self,
        surveys: &mut ImageSurveys,
    ) -> bool {
        let resolved_tiles = self.requests.handle_received_tiles(surveys);
        for tile in resolved_tiles.iter() {
            self.requested_tiles.remove(tile);
        }

        !resolved_tiles.is_empty()
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

                self.request_tile(tile);
            }
        }
    }
}