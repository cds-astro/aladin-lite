pub mod query;
pub mod request;

use crate::survey::Url;
use std::collections::HashSet;
pub struct Downloader {
    // Current requests
    requests: Vec<RequestType>,
    queried_urls: HashSet<Url>,

    cache: Cache<Url, Resource>,
    queried_cached_urls: HashSet<Url>,
}

use crate::fifo_cache::Cache;

use query::Query;
use request::{RequestType, Resource};


impl Downloader {
    pub fn new() -> Downloader {
        let requests = Vec::with_capacity(32);
        let queried_urls = HashSet::with_capacity(64);
        let cache = Cache::new();
        let queried_cached_urls = HashSet::with_capacity(64);
        Self {
            requests,
            queried_urls,
            cache,
            queried_cached_urls
        }
    }

    // Returns true if the fetch has been done
    // Returns false if the query has already been done
    pub fn fetch<T>(&mut self, query: T, force_request: bool) -> bool
    where
        T: Query,
    {
        // Remove the ancient requests
        //self.tiles_to_req.clear();

        let url = query.url();
        if self.cache.contains(url) {
            self.queried_cached_urls.insert(url.to_string());
            false
        } else {
            let not_already_requested = !self.queried_urls.contains(url);

            // The cell is not already requested
            if not_already_requested || force_request {
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
    }

    pub fn get_received_resources(&mut self) -> Vec<Resource> {
        let mut rscs = vec![];

        let mut finished_query_urls = vec![];
        self.requests = self
            .requests
            .drain(..)
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

        for url in self.queried_cached_urls.iter() {
            let rsc = self.cache.extract(url).unwrap();
            rscs.push(rsc);
        }

        self.queried_cached_urls.clear();

        rscs
    }

    pub fn cache_rsc(&mut self, rsc: Resource) {
        self.cache.insert(rsc.url().clone(), rsc);
    }
}
