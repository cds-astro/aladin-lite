use std::sync::{Arc, Mutex};
use std::cell::Cell;
use std::rc::Rc;
use crate::buffer::{ImageBitmap, HTMLImage};
use crate::healpix_cell::HEALPixCell;
use crate::time::Time;
pub struct Request<R> {
    data: Arc<Mutex<Option<R>>>,
    time_request: Time,
    // Flag telling if the tile has been copied so that
    // the HtmlImageElement can be reused to download another tile
    //ready: bool,
    resolved: Rc<Cell<ResolvedStatus>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ResolvedStatus {
    NotResolved,
    Missing,
    Found,
}
use wasm_bindgen::JsValue;
use std::future::Future;
impl<R> Request<R>
where
    R: 'static
{
    pub fn new<F>(f: F) -> Self
    where
        F: Future< Output = Result<R, JsValue> > + 'static
    {
        // By default, we say the tile is available to be reused
        let resolved = Rc::new(Cell::new(ResolvedStatus::NotResolved));
        let time_request = Time::now();

        let data = Arc::new(Mutex::new(None));

        {
            let data_cloned = data.clone();
            let resolved_cloned = resolved.clone();

            let fut = async move {
                if let Ok(resp) = f.await {
                    *(data_cloned.lock().unwrap()) = Some(resp);
                    resolved_cloned.set(ResolvedStatus::Found);
                } else {
                    al_core::log(&format!("Error"));
                    resolved_cloned.set(ResolvedStatus::Missing);
                }
            };

            wasm_bindgen_futures::spawn_local(fut);
        }

        Self {
            data,
            resolved,
            time_request,
        }
    }

    pub fn get_time_request(&self) -> Time {
        self.time_request
    }

    pub fn is_resolved(&self) -> bool {
        let resolved = self.resolve_status();
        resolved == ResolvedStatus::Found || resolved == ResolvedStatus::Missing
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }

    pub fn get(&self) -> Arc<Mutex<Option<R>>> {
        self.data.clone()
    }
}

use crate::buffer::RetrievedImageType;
pub enum RequestType {
    Allsky {
        survey_root_url: String,
        req: Request<AllskyTilesType>
    },
    Tile {
        cell: HEALPixCell,
        req: Request<RetrievedImageType>,
    }
}

impl RequestType {
    fn resolve_status(&self) -> ResolvedStatus {
        match self {
            RequestType::Allsky { req, .. } => {
                req.resolve_status()
            },
            RequestType::Tile { req, .. } => {
                req.resolve_status()
            }
        }
    }
}

use crate::renderable::survey::AllskyTilesType;
pub enum Resolve {
    Allsky {
        survey_root_url: String,
        tiles: Arc<Mutex<Option<AllskyTilesType>>>,
    },
    Tile {
        cell: HEALPixCell,
        image: Arc<Mutex<Option<RetrievedImageType>>>,
    }
}

pub struct RequestSender {
    requests: Vec<RequestType>,
}

impl RequestSender {
    pub fn new() -> Self {
        Self {
            requests: vec![]
        }
    }

    pub fn push(&mut self, request: RequestType) {
        self.requests.push(request);
    }

    pub fn poll(&mut self) -> Vec<Resolve> {
        let mut resolved = vec![];
        self.requests = self.requests.drain(..)
            .filter(|r| {
                let status = r.resolve_status();
                if status == ResolvedStatus::Found {
                    resolved.push(
                        match r {
                            RequestType::Tile { req, cell } => {
                                Resolve::Tile {
                                    cell: *cell,
                                    image: req.get()
                                }
                            },
                            RequestType::Allsky { req, survey_root_url } => {
                                Resolve::Allsky {
                                    survey_root_url: survey_root_url.clone(),
                                    tiles: req.get()
                                }
                            }
                        }
                    );

                    true
                } else if status == ResolvedStatus::Missing {
                    false
                } else {
                    // Future still in process
                    true
                }
            })
            .collect();
        
        resolved
    }
}

