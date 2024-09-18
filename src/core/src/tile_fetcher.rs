use crate::downloader::{query, Downloader};
use crate::renderable::HiPS;
use crate::time::{DeltaTime, Time};
use crate::Abort;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

const MAX_NUM_TILE_FETCHING: usize = 8;
const MAX_QUERY_QUEUE_LENGTH: usize = 100;

pub struct TileFetcherQueue {
    // A stack of queries to fetch
    queries: VecDeque<query::Tile>,
    base_tile_queries: Vec<query::Tile>,
    tiles_fetched_time: Time,
    num_tiles_fetched: usize,

    hips_local_files: HashMap<CreatorDid, HiPSLocalFiles>,
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct HiPSLocalFiles {
    paths: Box<[HashMap<u64, web_sys::File>]>,
}

use crate::tile_fetcher::query::Tile;
use crate::HEALPixCell;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
impl HiPSLocalFiles {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let paths = vec![HashMap::new(); 30].into_boxed_slice();

        Self { paths }
    }

    pub fn insert(&mut self, depth: u8, ipix: u64, file: web_sys::File) {
        self.paths[depth as usize].insert(ipix, file);
    }

    fn get(&self, cell: &HEALPixCell) -> Option<&web_sys::File> {
        let d = cell.depth() as usize;
        let i = cell.idx();

        return self.paths[d].get(&i);
    }
}
use crate::renderable::CreatorDid;
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
            hips_local_files: HashMap::new(),
        }
    }

    pub fn insert_hips_local_files(&mut self, id: CreatorDid, local_files: HiPSLocalFiles) {
        self.hips_local_files.insert(id, local_files);
    }

    pub fn delete_hips_local_files(&mut self, id: &str) {
        self.hips_local_files.remove(id);
    }

    pub fn clear(&mut self) {
        self.queries.clear();
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

    fn check_in_file_list(&self, mut query: Tile) -> Result<Tile, JsValue> {
        if let Some(files) = self.hips_local_files.get(&query.hips_cdid) {
            if let Some(file) = files.get(&query.cell) {
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(file.as_ref()) {
                    // rewrite the url
                    query.url = url;
                    Ok(query)
                } else {
                    Err(JsValue::from_str("could not create an url from the tile"))
                }
            } else {
                Ok(query)
            }
        } else {
            Ok(query)
        }
    }

    fn fetch(&mut self, downloader: Rc<RefCell<Downloader>>) {
        // Fetch the base tiles with higher priority
        while let Some(query) = self.base_tile_queries.pop() {
            if let Ok(query) = self.check_in_file_list(query) {
                downloader.borrow_mut().fetch(query);
            }
        }

        let mut num_fetched_tile = 0;
        while num_fetched_tile < MAX_NUM_TILE_FETCHING && !self.queries.is_empty() {
            let query = self.queries.pop_back().unwrap_abort();

            if let Ok(query) = self.check_in_file_list(query) {
                if downloader.borrow_mut().fetch(query) {
                    // The fetch has succeded
                    num_fetched_tile += 1;
                }
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

                for tile_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                    if let Ok(query) = self.check_in_file_list(query::Tile::new(
                        tile_cell,
                        hips_cdid.clone(),
                        hips_url.clone(),
                        hips_fmt,
                    )) {
                        let dl = downloader.clone();

                        crate::utils::set_timeout(
                            move || {
                                dl.borrow_mut().fetch(query);
                            },
                            2_000,
                        );
                    }
                }
            }
        }
    }
}
