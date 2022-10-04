const MAX_NUM_TILE_FETCHING: isize = 8;
const MAX_QUERY_QUEUE_LENGTH: usize = 100;
use std::collections::VecDeque;

use crate::downloader::query;
pub struct TileFetcherQueue {
    num_tiles_fetched: isize,
    // A stack of queries to fetch
    queries: VecDeque<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
}

use crate::downloader::Downloader;
use crate::Abort;
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
}
