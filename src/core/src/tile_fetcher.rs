use crate::downloader::{query, Downloader};
use crate::renderable::HiPS;
use crate::time::{DeltaTime, Time};
use crate::Abort;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const MAX_NUM_TILE_FETCHING: usize = 8;
const MAX_QUERY_QUEUE_LENGTH: usize = 100;

pub struct TileFetcherQueue {
    // A stack of queries to fetch
    queries: VecDeque<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
    tiles_fetched_time: Time,
    num_tiles_fetched: usize,
}

impl TileFetcherQueue {
    pub fn new() -> Self {
        let queries = VecDeque::new();
        let base_tile_queries = Vec::new();
        let tiles_fetched_time = Time::now();
        let num_tiles_fetched = 0;
        Self {
            queries,
            base_tile_queries,
            tiles_fetched_time,
            num_tiles_fetched,
        }
    }

    pub fn clear(&mut self) {
        self.queries.clear();
        //self.query_set.clear();
    }

    pub fn append(&mut self, query: query::Tile) {
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
    pub fn append_base_tile(&mut self, query: query::Tile) {
        self.base_tile_queries.push(query);
    }

    pub fn notify(&mut self, downloader: Rc<RefCell<Downloader>>, dt: Option<DeltaTime>) {
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

    pub fn get_num_tile_fetched(&self) -> usize {
        self.num_tiles_fetched
    }

    fn fetch(&mut self, downloader: Rc<RefCell<Downloader>>) {
        // Fetch the base tiles with higher priority
        while let Some(query) = self.base_tile_queries.pop() {
            //if downloader.fetch(query) {
            // The fetch has succeded
            //self.num_tiles_fetched += 1;
            //}
            downloader.borrow_mut().fetch(query);
        }

        let mut num_fetched_tile = 0;
        while num_fetched_tile < MAX_NUM_TILE_FETCHING && !self.queries.is_empty() {
            let query = self.queries.pop_back().unwrap_abort();

            if downloader.borrow_mut().fetch(query) {
                // The fetch has succeded
                num_fetched_tile += 1;
            }
        }

        self.num_tiles_fetched += num_fetched_tile;
    }

    pub fn launch_starting_hips_requests(
        &mut self,
        hips: &HiPS,
        downloader: Rc<RefCell<Downloader>>,
    ) {
        let cfg = hips.get_config();
        // Request for the allsky first
        // The allsky is not mandatory present in a HiPS service but it is better to first try to search for it
        //downloader.fetch(query::PixelMetadata::new(cfg));
        // Try to fetch the MOC
        downloader.borrow_mut().fetch(query::Moc::new(
            format!("{}/Moc.fits", cfg.get_root_url()),
            cfg.get_creator_did().to_string(),
            al_api::moc::MOC::default(),
        ));

        let tile_size = cfg.get_tile_size();
        //Request the allsky for the small tile size or if base tiles are not available
        if tile_size <= 128 || cfg.get_min_depth_tile() > 0 {
            // Request the allsky
            downloader.borrow_mut().fetch(query::Allsky::new(cfg));
        } else if cfg.get_min_depth_tile() == 0 {
            #[cfg(target_arch = "wasm32")]
            {
                let hips_cdid = cfg.get_creator_did().to_string();
                let hips_url = cfg.get_root_url().to_string();
                let hips_fmt = cfg.get_format();
                let min_order = cfg.get_min_depth_texture();

                let dl = downloader.clone();
                crate::utils::set_timeout(
                    move || {
                        for tile_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                            dl.borrow_mut().fetch(query::Tile::new(
                                tile_cell,
                                hips_cdid.clone(),
                                hips_url.clone(),
                                hips_fmt,
                            ));
                        }
                    },
                    2_000,
                );
            }
        }
    }
}
