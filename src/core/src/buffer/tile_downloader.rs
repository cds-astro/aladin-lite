use super::{CompressedImageRequest, FitsImageRequest, ResolvedStatus, TileRequest};

use crate::buffer::HiPSConfig;

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 32;

struct Requests {
    reqs: [Option<TileRequest>; NUM_EVENT_LISTENERS],
    //start_fits_req_idx: usize,
    free_slots_idx: Vec<usize>,
}

impl Requests {
    fn new() -> Self {
        let reqs = [None; NUM_EVENT_LISTENERS];
        let free_slots_idx = (0..NUM_EVENT_LISTENERS).iter().collect();
        /*let mut reqs = Vec::with_capacity(NUM_EVENT_LISTENERS);
        for _i in 0..NUM_EVENT_LISTENERS {
            reqs.push(TileRequest(CompressedImageReq::new()));
        }
        let start_fits_req_idx = NUM_EVENT_LISTENERS >> 1;

        for idx in start_fits_req_idx..NUM_EVENT_LISTENERS {
            reqs[idx] = TileRequest(FitsImageReq::new());
        }

        Requests {
            reqs,
            start_fits_req_idx,
        }*/
        Requests {
            reqs,
            free_slots_idx
        }
    }

    fn check_send(&mut self, format: ImageFormatType) -> Option<&mut TileRequest> {
        if self.free_slots_idx.is_empty() {
            return None;
        }

        let free_idx = free_slots_idx.pop().unwrap();

        match format {
            ImageFormatType::RGBA8U => {
                self.reqs[free_idx] = TileRequest(ImageRequestType::PNGRGBA8UImageReq::new(CompressedImageReq::new()));        
            },
            /*ImageFormatType::RGB8U => {
                let mut cur_idx_comp = 0;
                let mut html_image_req_available = true;

                while html_image_req_available && !self.reqs[cur_idx_comp].is_ready() {
                    cur_idx_comp += 1;
                    if cur_idx_comp == self.start_fits_req_idx {
                        html_image_req_available = false;
                    }
                }

                if html_image_req_available {
                    let req = &mut self.reqs[cur_idx_comp];
                    assert!(req.is_ready());
                    Some(req)
                } else {
                    None
                }
            }
            ImageFormatType::R32F | ImageFormatType::R8UI | ImageFormatType::R16I | ImageFormatType::R32I => {
                let mut cur_idx_fits = self.start_fits_req_idx;
                let mut fits_image_req_available = true;

                while fits_image_req_available && !self.reqs[cur_idx_fits].is_ready() {
                    cur_idx_fits += 1;
                    if cur_idx_fits == NUM_EVENT_LISTENERS {
                        fits_image_req_available = false;
                    }
                }

                if fits_image_req_available {
                    let req = &mut self.reqs[cur_idx_fits];
                    assert!(req.is_ready());
                    Some(req)
                } else {
                    None
                }
            }
            _ => unimplemented!(),*/
        }
    }

    /*fn iter_mut<'a>(&'a mut self) -> RequestsIterMut<'a> {
        RequestsIterMut(self.reqs.iter_mut())
    }*/
    fn resolved_requests(&mut self, tiles_uploaded_on_gpu: &HashSet<Tile>, surveys: &ImageSurveys) -> HashMap<Tile, TileResolved> {
        let results = HashMap::new();

        for (idx, req) in self.reqs.iter_mut().enumerate() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            if req.is_resolved() {
                let tile = req.get_tile();

                // A tile request can be reused if its cell texture is available/readable
                // by the GPU
                let available_req = tiles_uploaded_on_gpu.contains(tile);
                if available_req {
                    self.free_slots_idx.push(idx);
                    self.req = None;
                } else {
                    // Time when the tile has been received
                    let time_req = req.get_time_request();

                    let tile_resolved = match req.resolve_status() {
                        ResolvedStatus::Missing => TileResolved::Missing { time_req },
                        ResolvedStatus::Found => {
                            let config =
                                surveys.get(&tile.root_url).unwrap().get_textures().config();

                            let image = req.get_image(config.get_tile_size());
                            if let Ok(image) = image {
                                TileResolved::Found { image, time_req }
                            } else {
                                let err = image.err().unwrap();
                                log!(err);
                                TileResolved::Missing { time_req }
                            }
                        }
                    };

                    results.insert(
                        tile.clone(),
                        tile_resolved
                    );
                }
            }
        }

        results
    }
}

struct RequestsIterMut<'a>(std::slice::IterMut<'a, TileRequest>);

impl<'a> Iterator for RequestsIterMut<'a> {
    type Item = &'a mut TileRequest;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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

use std::collections::{HashSet, VecDeque};
pub struct TileDownloader {
    // Waiting cells to be loaded
    tiles_to_req: Vec<Tile>,
    base_tiles_to_req: Vec<Tile>,

    requests: Requests,

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

use crate::log::*;
use crate::ImageSurveys;
use wasm_bindgen::JsValue;
impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests = Requests::new();
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

        for req in self.requests.iter_mut() {
            req.clear();
        }
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
            self.tiles_to_req.push_back(tile);
        }
    }

    // Retrieve the tiles that have been resolved:
    // Two possibilities:
    // * The image have been found and retrieved
    // * The image is missing
    pub fn get_resolved_tiles(
        &mut self,
        available_tiles: &Tiles,
        surveys: &ImageSurveys,
    ) -> ResolvedTiles {
        let resolved_tiles = self.requests.resolved_tiles();
        for (tile, resolve) in resolved_tiles.iter() {
            assert!(self.requested_tiles.contains(tile));
            self.requested_tiles.remove(tile);
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
                self.tiles_to_req.back().unwrap()
            };

            if let Some(available_req) = self.requests.check_send(tile.format) {
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
