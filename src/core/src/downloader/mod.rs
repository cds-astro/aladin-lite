pub mod request;
pub mod query;
/*
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

struct Tile {
    time_req: Time,
    cell: HEALPixCell,

    image: ImageResolved,
}

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

    fn handle_received_tiles(&mut self) -> Vec<> {
        /*let mut tiles_received = Vec::new();

        for (idx, request) in self.reqs.iter_mut().enumerate() {
            if let Some(request) = request.take() {
                let TileRequest {
                    tile,
                    time_req,
                    status,
                    ..
                } = request;

                /*let tile = resp.get_tile();
                let time_req = resp.get_time_request();
                let status = resp.resolve_status();*/
                let response = match status {
                    ResolvedStatus::Missing => {
                        Some(TileResolved::Missing { time_req })
                    },
                    ResolvedStatus::Found => {
                        let response = if let Some(survey) = surveys.get(&tile.root_url) {
                            let cfg = survey.get_config();
                            if let Ok(image) = resp.get_image(cfg.get_tile_size()) {
                                TileResolved::Found { image, time_req }
                            } else {
                                TileResolved::Missing { time_req }
                            }
                        } else {
                            TileResolved::Missing { time_req }
                        };

                        Some(response)
                    },
                    ResolvedStatus::NotResolved => None,
                };

                if let Some(resp) = response {
                    // Signals that the tile has been handled (copied for the GPU)
                    self.add_resolved_tile(tile, resp, surveys);
                    tiles_received.push(tile.clone());
    
                    // Free the request to be used to download a new tile
                    self.free_slots_idx.push(idx);
                    *req = None;

                    break; // handle one tile per frame
                }
            }
        }

        tiles_received*/
        let resolved = 
        for request in self.reqs.iter_mut() {
            if let Some(request) = request {
                let response = match request.get_status() {
                    ResolvedStatus::Missing => {
                        Some(ImageResolved::Missing)
                    },
                    ResolvedStatus::Found => {
                        let response = if let Ok(image) = request.get_image() {
                            ImageResolved::Found { image }
                        } else {
                            ImageResolved::Missing
                        };

                        Some(response)
                    },
                    ResolvedStatus::NotResolved => None,
                };

                if let Some(resp) = response {
                    

                    // Free the request to be used to download a new tile
                    self.free_slots_idx.push(idx);
                    *request = None;
                }
            }        
        }

            if let Some(request) = request.take() {
                

                /*let tile = resp.get_tile();
                let time_req = resp.get_time_request();
                let status = resp.resolve_status();*/
                

                if let Some(resp) = response {
                    // Signals that the tile has been handled (copied for the GPU)
                    self.add_resolved_tile(tile, resp, surveys);
                    tiles_received.push(tile.clone());
    
                    // Free the request to be used to download a new tile
                    self.free_slots_idx.push(idx);
                    *req = None;

                    break; // handle one tile per frame
                }
            }
        }
    }

    fn add_resolved_tile(&mut self, tile: &Tile, status: TileResolved, surveys: &'mut ImageSurveys) {
        if let Some(survey) = surveys.get_mut(&tile.root_url) {
            match status {
                TileResolved::Missing { time_req } => {
                    let missing = true;

                    let cfg = survey.get_config();
                    match cfg.get_format() {
                        ImageFormatType::RGBA8U { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<RGBA8U>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                        ImageFormatType::RGB8U { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<RGB8U>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                        ImageFormatType::R32F { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<R32F>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                        #[cfg(feature = "webgl2")]
                        ImageFormatType::R8UI { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<R8UI>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                        #[cfg(feature = "webgl2")]
                        ImageFormatType::R16I { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<R16I>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                        #[cfg(feature = "webgl2")]
                        ImageFormatType::R32I { config } => {
                            let missing_tile_image = config.get_default_tile();
                            survey.add_tile::<Rc<ImageBuffer<R32I>>>(
                                &tile.cell,
                                missing_tile_image,
                                time_req,
                                missing,
                            );
                        }
                    }
                },
                TileResolved::Found { image, time_req } => {
                    let missing = false;
                    let cfg = survey.get_config_mut();
                    match image {
                        RetrievedImageType::FitsImageR32f { image } => {
                            // update the metadata
                            cfg.set_fits_metadata(
                                image.bscale,
                                image.bzero,
                                image.blank,
                            );
                            survey.add_tile::<FitsImage<R32F>>(&tile.cell, image, time_req, missing);
                        }
                        #[cfg(feature = "webgl2")]
                        RetrievedImageType::FitsImageR32i { image } => {
                            cfg.set_fits_metadata(
                                image.bscale,
                                image.bzero,
                                image.blank,
                            );
                            survey.add_tile::<FitsImage<R32I>>(&tile.cell, image, time_req, missing);
                        }
                        #[cfg(feature = "webgl2")]
                        RetrievedImageType::FitsImageR16i { image } => {
                            cfg.set_fits_metadata(
                                image.bscale,
                                image.bzero,
                                image.blank,
                            );
                            survey.add_tile::<FitsImage<R16I>>(&tile.cell, image, time_req, missing);
                        }
                        #[cfg(feature = "webgl2")]
                        RetrievedImageType::FitsImageR8ui { image } => {
                            cfg.set_fits_metadata(
                                image.bscale,
                                image.bzero,
                                image.blank,
                            );
                            survey.add_tile::<FitsImage<R8UI>>(&tile.cell, image, time_req, missing);
                        }
                        RetrievedImageType::PngImageRgba8u { image } => {
                            survey.add_tile::<ImageBitmap<RGBA8U>>(&tile.cell, image, time_req, missing);
                        }
                        RetrievedImageType::JpgImageRgb8u { image } => {
                            survey.add_tile::<ImageBitmap<RGB8U>>(&tile.cell, image, time_req, missing);
                        }
                    }
                }
            }
        }
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
pub enum ImageResolved {
    Missing,
    Found {
        image: RetrievedImageType,
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
        if !already_requested {
            if tile.is_root() {
                self.base_tiles_to_req.push(tile);
            } else {
                self.tiles_to_req.push(tile);
            }
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

    pub fn try_sending_tile_requests(&mut self, surveys: &ImageSurveys) -> Result<(), JsValue> {
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
            //let tile = self.tiles_to_req.last();

            if let Some(available_req) = self.requests.check_send(tile.format.clone()) {
                let tile = if base_tile_requested {
                    // Send in priority requests to get the base tiles
                    self.base_tiles_to_req.pop().unwrap()
                } else {
                    // Otherwise requests the other tiles
                    self.tiles_to_req.pop().unwrap()
                };
                //let tile = self.tiles_to_req.pop().unwrap();

                is_remaining_req =
                    !self.tiles_to_req.is_empty() || !self.base_tiles_to_req.is_empty();
                //is_remaining_req = !self.tiles_to_req.is_empty();
                self.requested_tiles.insert(tile.clone());

                available_req.fetch(&tile, surveys)?;
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;
            }
        }

        Ok(())
    }

    /*pub fn request_base_tiles(&mut self, config: &HiPSConfig) {
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
    }*/
}*/

use crate::{
    healpix::cell::HEALPixCell,
    survey::Url,
};

type TileUrl = Url;

use al_core::image::format::ImageFormatType;
use std::collections::HashSet;
use request::tile::TileRequest;
pub struct Downloader {
    // Waiting cells to be loaded
    //tiles_to_req: Vec<TileQuery>,
    //base_tiles_to_req: Vec<TileQuery>,

    // Current requests
    requests: Vec<RequestType>,
    queried_urls: HashSet<Url>,
}

use crate::time::Time;
use std::collections::HashMap;

use al_core::log::*;
use wasm_bindgen::JsValue;
use request::{
    Request,
    tile::Tile
};
use std::sync::{Arc, Mutex};
use query::Query;
use request::{
    Resource,
    RequestType,
};
impl Downloader {
    pub fn new() -> Downloader {
        let requests = Vec::with_capacity(32);
        let queried_urls = HashSet::with_capacity(64);

        Self {
            requests,
            queried_urls,
        }
    }

    // Returns true if the fetch has been done
    // Returns false if the query has already been done
    pub fn fetch<T>(&mut self, query: T) -> bool
    where
        T: Query
    {
        // Remove the ancient requests
        //self.tiles_to_req.clear();

        let url = query.url();
        let not_already_requested = !self.queried_urls.contains(url);

        // The cell is not already requested
        if not_already_requested {
            /*if tile.is_root() {
                self.base_tiles_to_req.push(tile);
            } else {
                self.tiles_to_req.push(tile);
            }*/
            self.queried_urls.insert(url.to_string());

            let request = T::Request::from(query);
            self.requests.push(request.into());
        }

        not_already_requested
    }

    /*pub fn try_sending_tile_requests(&mut self, surveys: &ImageSurveys) -> Result<(), JsValue> {
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
            //let tile = self.tiles_to_req.last();

            if let Some(available_req) = self.requests.check_send(tile.format.clone()) {
                let tile = if base_tile_requested {
                    // Send in priority requests to get the base tiles
                    self.base_tiles_to_req.pop().unwrap()
                } else {
                    // Otherwise requests the other tiles
                    self.tiles_to_req.pop().unwrap()
                };
                //let tile = self.tiles_to_req.pop().unwrap();

                is_remaining_req =
                    !self.tiles_to_req.is_empty() || !self.base_tiles_to_req.is_empty();
                //is_remaining_req = !self.tiles_to_req.is_empty();
                self.requested_tiles.insert(tile.clone());

                available_req.fetch(&tile, surveys)?;
            } else {
                // We have to wait for more requests
                // to be available
                downloader_overloaded = true;
            }
        }

        Ok(())
    }*/

    pub fn get_received_resources(&mut self) -> Vec<Resource> {
        let mut rscs = vec![];

        let mut finished_query_urls = vec![];
        self.requests = self.requests.drain(..)
            .filter(|request| {
                // If the request resolves into a resource
                if let Some(rsc) = request.into() {
                    rscs.push(rsc);
                    finished_query_urls.push(request.url().clone());

                    false
                } else {
                    // The request is not resolved, we keep it
                    true
                }
            })
            .collect();

        for url in finished_query_urls.into_iter() {
            self.queried_urls.remove(&url);
        }

        rscs
    }
}