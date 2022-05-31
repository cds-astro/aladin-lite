const MAX_NUM_TILE_FETCHING: isize = 32;
use crate::downloader::query;
pub struct TileFetcherQueue {
    num_tiles_fetched: isize,
    // A stack of queries to fetch
    queries: Vec<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
}

use crate::downloader::Downloader;

impl TileFetcherQueue {
    pub fn new() -> Self {
        let queries = Vec::new();
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
        self.queries.push(query);
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
            let query = self.base_tile_queries.pop().unwrap();

            if downloader.fetch(query) {
                // The fetch has succed
                self.num_tiles_fetched += 1;
            }
        }

        while self.num_tiles_fetched < MAX_NUM_TILE_FETCHING && !self.queries.is_empty() {
            let query = self.queries.pop().unwrap();

            if downloader.fetch(query) {
                self.num_tiles_fetched += 1;
            }
        }
    }
}
