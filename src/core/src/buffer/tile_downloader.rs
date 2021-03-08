use super::{CompressedImageRequest, FITSImageRequest, ResolvedStatus, TileRequest};

use crate::buffer::HiPSConfig;

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 32;

struct Requests {
    reqs: Vec<TileRequest>,
    start_fits_req_idx: usize,
}

impl Requests {
    fn new() -> Self {
        let mut reqs = Vec::with_capacity(NUM_EVENT_LISTENERS);
        for _i in 0..NUM_EVENT_LISTENERS {
            reqs.push(TileRequest::new::<CompressedImageRequest>());
        }
        let start_fits_req_idx = NUM_EVENT_LISTENERS >> 1;

        for idx in start_fits_req_idx..NUM_EVENT_LISTENERS {
            reqs[idx] = TileRequest::new::<FITSImageRequest>();
        }

        Requests {
            reqs,
            start_fits_req_idx,
        }
    }

    fn check_send(&mut self, tile_format: FormatImageType) -> Option<&mut TileRequest> {
        match tile_format {
            FormatImageType::JPG | FormatImageType::PNG => {
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
            FormatImageType::FITS(_) => {
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
            },
            _ => unimplemented!()
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

use crate::healpix_cell::HEALPixCell;
// A tile is described by an image survey
// and an HEALPix cell
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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
            format: config.format(),
        }
    }

    fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}

pub type Tiles = HashSet<Tile>;

use std::collections::{HashSet, VecDeque};
pub struct TileDownloader {
    // Waiting cells to be loaded
    tiles_to_req: VecDeque<Tile>,
    base_tiles_to_req: Vec<Tile>,

    requests: Requests,

    requested_tiles: Tiles,
}

use crate::FormatImageType;

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
use wasm_bindgen::JsValue;
impl TileDownloader {
    pub fn new() -> TileDownloader {
        let requests = Requests::new();
        let tiles_to_req = VecDeque::new();
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
                            ResolvedStatus::Missing => TileResolved::Missing { time_req },
                            ResolvedStatus::Found => {
                                let config =
                                    surveys.get(&tile.root_url).unwrap().get_textures().config();

                                if let Some(image) =
                                    req.get_image(config.get_tile_size(), &config.format())
                                {
                                    TileResolved::Found { image, time_req }
                                } else {
                                    TileResolved::Missing { time_req }
                                }
                            }
                            _ => unreachable!(),
                        };

                        resolved_tiles.insert(tile.clone(), tile_resolved);
                    }
                }
            }
        }

        resolved_tiles
    }

    pub fn try_sending_tile_requests(&mut self) -> Result<(), JsValue> {
        // Try sending the fits tile requests
        self.try_sending_tiles()?;

        Ok(())
    }

    fn try_sending_tiles(&mut self) -> Result<(), JsValue> {
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
                    self.base_tiles_to_req.pop().unwrap()
                } else {
                    self.tiles_to_req.pop_back().unwrap()
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
