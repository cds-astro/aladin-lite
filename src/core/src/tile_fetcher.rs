use crate::downloader::{query, Downloader};
use crate::renderable::HiPS;
use crate::time::{DeltaTime, Time};
use crate::Abort;

use std::collections::VecDeque;

const MAX_NUM_TILE_FETCHING: isize = 8;
const MAX_QUERY_QUEUE_LENGTH: usize = 100;

pub struct TileFetcherQueue {
    // A stack of queries to fetch
    queries: VecDeque<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
    tiles_fetched_time: Time,
}

impl TileFetcherQueue {
    pub fn new() -> Self {
        let queries = VecDeque::new();
        let base_tile_queries = Vec::new();
        let tiles_fetched_time = Time::now();
        Self {
            queries,
            base_tile_queries,
            tiles_fetched_time,
        }
    }

    pub fn clear(&mut self) {
        self.queries.clear();
        //self.query_set.clear();
    }

    pub fn append(&mut self, query: query::Tile, _downloader: &mut Downloader) {
        // Check if the query has already been done
        //if !self.query_set.contains(&query) {
        // discard too old tile queries
        // this may not be the best thing to do but
        if self.queries.len() > MAX_QUERY_QUEUE_LENGTH {
            self.queries.pop_front();
        }
        self.queries.push_back(query.clone());
    }

    // fetch the base tile
    pub fn append_base_tile(&mut self, query: query::Tile, _downloader: &mut Downloader) {
        self.base_tile_queries.push(query);
    }

    pub fn notify(&mut self, downloader: &mut Downloader, dt: Option<DeltaTime>) {
        // notify all the x ms
        let now = Time::now();

        if let Some(dt) = dt {
            if now - self.tiles_fetched_time >= dt {
                self.tiles_fetched_time = now;
                self.fetch(downloader);
            }
        } else {
            self.tiles_fetched_time = now;
            self.fetch(downloader);
        }
    }

    fn fetch(&mut self, downloader: &mut Downloader) {
        // Fetch the base tiles with higher priority
        while let Some(query) = self.base_tile_queries.pop() {
            //if downloader.fetch(query) {
            // The fetch has succeded
            //self.num_tiles_fetched += 1;
            //}
            downloader.fetch(query);
        }

        let mut num_fetched_tile = 0;
        while num_fetched_tile < MAX_NUM_TILE_FETCHING && !self.queries.is_empty() {
            let query = self.queries.pop_back().unwrap_abort();

            if downloader.fetch(query) {
                // The fetch has succeded
                num_fetched_tile += 1;
            }
        }
    }

    pub fn launch_starting_hips_requests(&mut self, hips: &HiPS, downloader: &mut Downloader) {
        let cfg = hips.get_config();
        // Request for the allsky first
        // The allsky is not mandatory present in a HiPS service but it is better to first try to search for it
        downloader.fetch(query::PixelMetadata::new(cfg));
        // Try to fetch the MOC
        downloader.fetch(query::Moc::new(
            format!("{}/Moc.fits", cfg.get_root_url()),
            al_api::moc::MOC::default(),
        ));

        let tile_size = cfg.get_tile_size();
        //Request the allsky for the small tile size or if base tiles are not available
        if tile_size <= 128 || cfg.get_min_depth_tile() > 0 {
            // Request the allsky
            downloader.fetch(query::Allsky::new(cfg));
        } else {
            for texture_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                for cell in texture_cell.get_tile_cells(cfg.delta_depth()) {
                    let hips_url = cfg.get_root_url();
                    let format = cfg.get_format();
                    let query = query::Tile::new(&cell, hips_url.to_string(), format);
                    self.append_base_tile(query, downloader);
                }
            }
        }
    }
}
