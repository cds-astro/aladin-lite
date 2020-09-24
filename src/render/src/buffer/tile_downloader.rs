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

type TileSentFlag = bool;
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

    fn try_send(&mut self, tile: &Tile) -> TileSentFlag {
        match tile.format {
            FormatImageType::JPG | FormatImageType::PNG => {
                let cur_idx_comp = 0;
                let mut req = &mut self.reqs[cur_idx_comp];
                let mut html_image_req_available = true;

                while !req.is_ready() && html_image_req_available  {
                    cur_idx_comp += 1;
                    if cur_idx_comp == self.start_fits_req_idx {
                        html_image_req_available = false;
                    } else {
                        req = &mut self.reqs[cur_idx_comp];
                    }
                }

                if req.is_ready() {
                    req.send(tile);
                    true
                } else {
                    false
                }
            },
            FormatImageType::FITS(_) => {
                let cur_idx_fits = self.start_fits_req_idx;
                let mut req = &mut self.reqs[cur_idx_fits];
                let mut fits_image_req_available = true;

                while !req.is_ready() && fits_image_req_available {
                    cur_idx_fits += 1;
                    if cur_idx_fits == NUM_EVENT_LISTENERS {
                        fits_image_req_available = false;
                    } else {
                        req = &mut self.reqs[cur_idx_fits];
                    }
                }

                if req.is_ready() {
                    req.send(tile);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn iter_mut<'a>(&'a mut self) -> RequestsIterMut<'a> {
        RequestsIterMut(self.reqs.iter_mut())
    }

    fn iter<'a>(&'a mut self) -> RequestsIter<'a> {
        RequestsIter(self.reqs.iter_mut())
    }

    fn get_start_fits_req_idx(&self) -> usize {
        self.start_fits_req_idx
    }
}

struct RequestsIter<'a>(std::slice::Iter<'a, TileRequest>);
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
    fits_tiles_to_req: VecDeque<Tile>,
    html_img_tiles_to_req: VecDeque<Tile>,

    requests: Requests,
}

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 16;
const MAX_NUM_CELLS_MEMORY_REQUEST: usize = 100;
use crate::FormatImageType;

use super::image::RetrievedImageType;
enum TileResolved {
    Missing,
    Found { image: RetrievedImageType }
}
type RetrievedTiles = HashMap<Tile, TileResolved>;

impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests: Requests::new();
        let html_img_tiles_to_req = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);
        let fits_tiles_to_req = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);

        Self {
            fits_tiles_to_req,
            html_img_tiles_to_req,

            requests,
        }
    }

    pub fn reset(&mut self) {
        self.html_img_tiles_to_req.clear();
        self.fits_tiles_to_req.clear();

        for req in self.requests.iter_mut() {
            req.clear();
        }
    }

    // Register further tile requests to launch
    pub fn add_tile_request(&mut self, tile: Tile) {
        match tile.format {
            FormatImageType::JPG | FormatImageType::PNG => {
                self.html_img_tiles_to_req.push_back(tile);
            },
            FormatImageType::FITS(_) => {
                self.fits_tiles_to_req.push_back(tile);
            }
        }
    }

    // Retrieve the tiles that have been resolved:
    // Two possibilities:
    // * The image have been found and retrieved
    // * The image is missing
    pub fn retrieve_tile_resolved(&self, available_tiles: &Tiles, requested_tiles: &mut Tiles) -> RetrievedTiles {
        let mut resolved_tiles = HashMap::new();

        for req in self.requests.iter() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            if req.is_resolved() {
                let tile = req.get_tile();

                // A tile request can be reused if its cell texture is available/readable
                // by the GPU
                let available_req = available_tiles.contains(tile);
                if available_req {
                    req.set_ready();
                } else {
                    let req_just_resolved = requested_tiles.contains(tile);
                    // The tile has not been copied
                    if req_just_resolved {
                        // Tile received
                        let time_of_request = req.get_time_request();
                        requested_tiles.remove(tile);
        
                        let tile_resolved = match req.resolve_status() {
                            ResolvedStatus::Missing => {
                                TileResolved::Missing
                            },
                            ResolvedStatus::Found => {
                                let image = req.get_image();
                                TileResolved::Found { image }
                            },
                            _ => unreachable!()
                        }

                        resolved_tiles.insert(*tile, tile_resolved);
                    }
                }
            }
        }

        resolved_tiles
    }

    pub fn try_sending_tile_requests(&mut self) {
        // Try sending the fits tile requests
        self.try_sending_tiles(&mut self.fits_tiles_to_req);
        // And then the HTML image tile requests
        self.try_sending_tiles(&mut self.html_img_tiles_to_req);
    }

    fn try_sending_tiles(&mut self, tiles_to_req: &mut VecDeque<Tile>) {
        let mut is_remaining_req = !tiles_to_req.is_empty();

        let mut downloader_overloaded = false;

        while is_remaining_req && !downloader_overloaded {
            let tile = tiles_to_req.front().unwrap();

            let tile_sent = self.requests.try_send(tile);
            if tile_sent {
                tiles_to_req.pop_front().unwrap();
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;
            }
        }
    }
}