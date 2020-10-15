use super::{TileRequest, TileHTMLImage, TileArrayBuffer, ResolvedStatus, FITSImageRequest, CompressedImageRequest};
use crate::WebGl2Context;

use crate::buffer::{
    HiPSConfig,
};

use crate::async_task::TaskExecutor;
// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 16;
const MAX_NUM_CELLS_MEMORY_REQUEST: usize = 100;

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
            reqs[idx] = TileRequest::new::<FITSImageRequest>();
        }

        Requests {
            reqs,
            start_fits_req_idx
        }
    }

    fn check_send(&mut self, tile_format: FormatImageType) -> Option<&mut TileRequest> {
        match tile_format {
            FormatImageType::JPG | FormatImageType::PNG => {
                let mut cur_idx_comp = 0;
                let mut html_image_req_available = true;

                //crate::log(&format!("{}", self.start_fits_req_idx));
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
            },
            FormatImageType::FITS(_) => {
                let mut cur_idx_fits = self.start_fits_req_idx;
                let mut fits_image_req_available = true;

                while fits_image_req_available && !self.reqs[cur_idx_fits].is_ready()  {
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
        }
    }

    fn iter_mut<'a>(&'a mut self) -> RequestsIterMut<'a> {
        RequestsIterMut(self.reqs.iter_mut())
    }

    fn iter<'a>(&'a self) -> RequestsIter<'a> {
        RequestsIter(self.reqs.iter())
    }

    fn get_start_fits_req_idx(&self) -> usize {
        self.start_fits_req_idx
    }
}

struct RequestsIter<'a>(std::slice::Iter<'a, TileRequest>);
struct RequestsIterMut<'a>(std::slice::IterMut<'a, TileRequest>);

impl<'a> Iterator for RequestsIter<'a> {
    type Item = &'a TileRequest;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
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
#[derive(PartialEq, Eq, Hash)]
#[derive(Clone, Debug)]
pub struct Tile {
    pub cell: HEALPixCell,
    pub root_url: String,
    pub format: FormatImageType,
}

impl Tile {
    pub fn new(cell: &HEALPixCell, config: &HiPSConfig) -> Self {
        Tile {
            cell: *cell,
            root_url: config.root_url.to_string(),
            format: config.format()
        }
    }
}

pub type Tiles = HashSet<Tile>;

use std::collections::{HashSet, VecDeque};
pub struct TileDownloader {
    // Waiting cells to be loaded
    fits_tiles_to_req: VecDeque<Tile>,
    html_img_tiles_to_req: VecDeque<Tile>,

    requests: Requests,

    requested_tiles: Tiles,
}

use crate::FormatImageType;

use super::image::RetrievedImageType;
use crate::time::Time;
pub enum TileResolved {
    Missing { time_req: Time },
    Found { image: RetrievedImageType, time_req: Time }
}
use std::collections::HashMap;
pub type ResolvedTiles = HashMap<Tile, TileResolved>;

use crate::ImageSurveys;
impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests = Requests::new();
        let html_img_tiles_to_req = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);
        let fits_tiles_to_req = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);
        let requested_tiles = HashSet::with_capacity(64);

        Self {
            fits_tiles_to_req,
            html_img_tiles_to_req,

            requests,
            requested_tiles,
        }
    }

    pub fn clear_requests(&mut self) {
        //self.html_img_tiles_to_req.clear();
        //self.fits_tiles_to_req.clear();

        for req in self.requests.iter_mut() {
            req.clear();
        }
        self.requested_tiles.clear();
    }

    pub fn request_tile(&mut self, tile: Tile) {
        let already_requested = self.requested_tiles.contains(&tile);
        // The cell is not already requested
        if !already_requested {
            // Add to the tiles requested
            self.requested_tiles.insert(tile.clone());
            self.add_tile_request(tile);
        }
    }

    // Register further tile requests to launch
    fn add_tile_request(&mut self, tile: Tile) {
        match tile.format {
            FormatImageType::JPG | FormatImageType::PNG => {
                self.html_img_tiles_to_req.push_back(tile);

                /*if self.html_img_tiles_to_req.len() > MAX_NUM_CELLS_MEMORY_REQUEST {
                    self.html_img_tiles_to_req.pop_front();
                }*/
            },
            FormatImageType::FITS(_) => {
                self.fits_tiles_to_req.push_back(tile);

                /*if self.fits_tiles_to_req.len() > MAX_NUM_CELLS_MEMORY_REQUEST {
                    self.fits_tiles_to_req.pop_front();
                }*/
            }
        }
    }

    // Retrieve the tiles that have been resolved:
    // Two possibilities:
    // * The image have been found and retrieved
    // * The image is missing
    pub fn get_resolved_tiles(&mut self, available_tiles: &Tiles, surveys: &ImageSurveys) -> ResolvedTiles {
        let mut resolved_tiles = HashMap::new();

        for req in self.requests.iter_mut() {
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
                    let req_just_resolved = self.requested_tiles.contains(tile);
                    // The tile has not been copied
                    if req_just_resolved {
                        // Tile received
                        let time_req = req.get_time_request();
                        // Remove from the requested tile
                        self.requested_tiles.remove(tile);

                        let tile_resolved = match req.resolve_status() {
                            ResolvedStatus::Missing => {
                                TileResolved::Missing { time_req }
                            },
                            ResolvedStatus::Found => {
                                let config = surveys.get(&tile.root_url).unwrap()
                                    .get_textures()
                                    .config();

                                let image = req.get_image(config.get_texture_size(), &config.format());
                                TileResolved::Found { image, time_req }
                            },
                            _ => unreachable!()
                        };

                        resolved_tiles.insert(tile.clone(), tile_resolved);
                    }
                }
            }
        }

        resolved_tiles
    }

    pub fn try_sending_tile_requests(&mut self) {
        // Try sending the fits tile requests
        self.try_sending_fits_tiles();
        // And then the HTML image tile requests
        self.try_sending_html_tiles();
    }

    fn try_sending_fits_tiles(&mut self) {
        let mut is_remaining_req = !self.fits_tiles_to_req.is_empty();

        let mut downloader_overloaded = false;

        while is_remaining_req && !downloader_overloaded {
            let tile = self.fits_tiles_to_req.back().unwrap();

            if let Some(available_req) = self.requests.check_send(tile.format) {
                let tile = self.fits_tiles_to_req.pop_back().unwrap();
                
                is_remaining_req = !self.fits_tiles_to_req.is_empty();
                available_req.send(tile);
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;
            }
        }
    }

    fn try_sending_html_tiles(&mut self) {
        let mut is_remaining_req = !self.html_img_tiles_to_req.is_empty();

        let mut downloader_overloaded = false;

        while is_remaining_req && !downloader_overloaded {
            let tile = self.html_img_tiles_to_req.back().unwrap();

            if let Some(available_req) = self.requests.check_send(tile.format) {
                let tile = self.html_img_tiles_to_req.pop_back().unwrap();

                is_remaining_req = !self.html_img_tiles_to_req.is_empty();
                available_req.send(tile);
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;

            }
        }
    }

    pub fn request_base_tiles(&mut self, url: &str, format: &FormatImageType) {
        // Request base tiles
        for idx in 0..12 {
            let tile = Tile {
                root_url: url.to_string(),
                format: *format,
                cell: HEALPixCell(0, idx)
            };

            self.request_tile(tile);
        }

    }
}