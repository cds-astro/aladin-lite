use crate::downloader::{query, Downloader};
use crate::renderable::HiPS;
use crate::Abort;

use std::collections::VecDeque;

const MAX_NUM_TILE_FETCHING: isize = 8;
const MAX_QUERY_QUEUE_LENGTH: usize = 100;

pub struct TileFetcherQueue {
    num_tiles_fetched: isize,
    // A stack of queries to fetch
    queries: VecDeque<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
}

impl TileFetcherQueue {
    pub fn new() -> Self {
        let queries = VecDeque::new();
        let base_tile_queries = Vec::new();
        Self {
            num_tiles_fetched: 0,
            queries,
            base_tile_queries,
        }
    }

    pub fn clear(&mut self) {
        self.queries.clear();
    }

    pub fn append(&mut self, query: query::Tile, downloader: &mut Downloader) {
        // discard too old tile queries
        // this may not be the best thing to do but
        if self.queries.len() > MAX_QUERY_QUEUE_LENGTH {
            self.queries.pop_front();   
        }
        self.queries.push_back(query);
        self.fetch(downloader);
    }

    pub fn append_base_tile(&mut self, query: query::Tile, downloader: &mut Downloader) {
        self.base_tile_queries.push(query);
        self.fetch(downloader);
    }

    pub fn notify(&mut self, num_tiles_completed: usize, downloader: &mut Downloader) {
        self.num_tiles_fetched -= num_tiles_completed as isize;

        self.fetch(downloader);
    }

    fn fetch(&mut self, downloader: &mut Downloader) {
        // Fetch the base tiles with higher priority
        while self.num_tiles_fetched < MAX_NUM_TILE_FETCHING && !self.base_tile_queries.is_empty() {
            let query = self.base_tile_queries.pop().unwrap_abort();

            if downloader.fetch(query) {
                // The fetch has succeded
                self.num_tiles_fetched += 1;
            }
        }

        while self.num_tiles_fetched < MAX_NUM_TILE_FETCHING && !self.queries.is_empty() {
            let query = self.queries.pop_back().unwrap_abort();

            if downloader.fetch(query) {
                // The fetch has succeded
                self.num_tiles_fetched += 1;
            }
        }
    }

    pub fn launch_starting_hips_requests(&mut self, hips: &HiPS, downloader: &mut Downloader) {
        let cfg = hips.get_config();
        // Request for the allsky first
        // The allsky is not mandatory present in a HiPS service but it is better to first try to search for it
        downloader.fetch(query::PixelMetadata::new(cfg));
        // Try to fetch the MOC
        downloader.fetch(query::Moc::new(format!("{}/Moc.fits", cfg.get_root_url()), al_api::moc::MOC::default()));

        let tile_size = cfg.get_tile_size();
        //Request the allsky for the small tile size or if base tiles are not available
        if tile_size <= 128 || cfg.get_min_depth_tile() > 0 {
            // Request the allsky
            downloader.fetch(query::Allsky::new(cfg));
        } else {
            for texture_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                for cell in texture_cell.get_tile_cells(cfg) {
                    let query = query::Tile::new(&cell, cfg);
                    self.append_base_tile(query, downloader);
                }
            }
        }
    }
}
