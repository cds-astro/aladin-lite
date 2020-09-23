use super::{TileRequest, TileHTMLImage, TileArrayBuffer, ResolvedStatus, FITSImageRequest, CompressedImageRequest, ImageRequest};
use crate::WebGl2Context;

use crate::buffer::{
    ImageSurvey,
    HiPSConfig,
};

use crate::async_task::AladinTaskExecutor;

struct Requests {
    reqs: [TileRequest; NUM_EVENT_LISTENERS],
    start_fits_req_idx: usize,
}

impl Requests {
    fn new() -> Self {
        let mut reqs: [TileRequest; NUM_EVENT_LISTENERS] = Default::default();
        let start_fits_req_idx = NUM_EVENT_LISTENERS >> 1;

        for idx in start_fits_req_idx..NUM_EVENT_LISTENERS {
            reqs[idx] = TileRequest::<FITSImageRequest>::new();
        }

        Requests {
            reqs,
            start_fits_req_idx
        }
    }

    fn iter_mut<'a>(&'a mut self) -> RequestsIterMut<'a> {
        RequestsIterMut(self.reqs.iter_mut())
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

use super::buffer_tiles::Tile;
use std::collections::{VecDeque, HashSet};
pub struct TileDownloader {
    // Waiting cells to be loaded
    tiles_to_request: VecDeque<Tile>,
    requests: Requests,
}

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 16;
const MAX_NUM_CELLS_MEMORY_REQUEST: usize = 100;
use crate::FormatImageType;
use crate::healpix_cell::HEALPixCell;

enum TileResolved {
    Missing,
    Found { image: RetrievedTileImage }
}
struct RetrievedTiles {
    tiles: HashMap<Tile, TileResolved>
}

impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests: Requests::new();
        let tiles_to_request = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);
        Self {
            tiles_to_request,
            requests,
        }
    }

    pub fn reset(&mut self) {
        self.tiles_to_request.clear();

        for req in self.requests.iter_mut() {
            req.clear();
        }
    }

    pub fn register_tile_request(&mut self, cell: &HEALPixCell) {
        self.tiles_to_request.push_back(*cell);
    }

    pub fn checks_for_finished_requests(&mut self, tiles_finished: &Tiles, requested_tiles: &mut Tiles, surveys: &[&ImageSurvey]) -> Vec<TileResolved> {
        let mut tiles_resolved = Vec::new();

        for req in self.requests.iter_mut() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            if req.is_resolved() {
                let cell = *req.get_cell();

                // A tile request can be reused if its cell texture is available/readable
                // by the GPU
                let available_req = tiles_finished.contains(&cell);
                if available_req {
                    req.set_ready();
                } else {
                    let req_just_resolved = requested_tiles.contains(&cell);
                    // The tile has not been copied
                    if req_just_resolved {
                        // Tile received
                        let time_of_request = req.get_time_request();
                        requested_tiles.remove(&cell);
        
                        let tile = match req.resolve_status() {
                            ResolvedStatus::Missing => {
                                TileResolved::Missing
                            },
                            ResolvedStatus::Found => {
                                let image = req.get_image();
                                TileResolved::Found { image }
                            },
                            _ => unreachable!()
                        }

                        tiles_resolved.push(tile);
                    }
                }
            }

            // Then, send new requests for available ones
            if !self.tiles_to_request.is_empty() {
                if req.is_ready() {
                    // Launch requests if the tile has yet not been used (start)
                    let cell = self.tiles_to_request.pop_front().unwrap();
                    req.send(&cell, survey);
                }
            }
        }

        tiles_resolved
    }
}
