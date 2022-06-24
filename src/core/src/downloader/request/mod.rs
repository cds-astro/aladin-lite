// A request image should not be used outside this module
// but contained inside a more specific type of query (e.g. for a tile or allsky)
pub mod allsky;
pub mod tile;
pub mod blank;

/* ------------------------------------- */

use crate::{time::Time};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
use std::future::Future;
use wasm_bindgen::JsValue;
impl<R> Request<R>
where
    R: 'static
{
    pub fn new<F>(f: F) -> Self
    where
        F: Future<Output = Result<R, JsValue>> + 'static,
    {
        // By default, we say the tile is available to be reused
        let resolved = Rc::new(Cell::new(ResolvedStatus::NotResolved));
        let time_request = Time::now();

        let data = Arc::new(Mutex::new(None));

        {
            let data_cloned = data.clone();
            let resolved_cloned = resolved.clone();

            let fut = async move {
                let resp = f.await;

                if let Ok(resp) = resp {
                    *(data_cloned.lock().unwrap()) = Some(resp);
                    resolved_cloned.set(ResolvedStatus::Found);
                } else if let Err(_err) = resp {
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

    pub fn is_resolved(&self) -> bool {
        let resolved = self.resolve_status();
        resolved == ResolvedStatus::Found || resolved == ResolvedStatus::Missing
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }
}

use allsky::AllskyRequest;
use tile::TileRequest;
use blank::PixelMetadataRequest;
pub enum RequestType {
    Tile(TileRequest),
    Allsky(AllskyRequest),
    PixelMetadata(PixelMetadataRequest),
    //..
}

use super::query::Url;
impl RequestType {
    pub fn url(&self) -> &Url {
        match self {
            RequestType::Tile(request) => &request.url,
            RequestType::Allsky(request) => &request.url,
            RequestType::PixelMetadata(request) => &request.url,
        }
    }
}

impl<'a> From<&'a RequestType> for Option<Resource> {
    fn from(request: &'a RequestType) -> Self {
        match request {
            RequestType::Tile(request) => {
                Option::<Tile>::from(request).map(|tile| Resource::Tile(tile))
            }
            RequestType::Allsky(request) => {
                Option::<Allsky>::from(request).map(|allsky| Resource::Allsky(allsky))
            }
            RequestType::PixelMetadata(request) => {
                Option::<PixelMetadata>::from(request).map(|metadata| Resource::PixelMetadata(metadata))
            }
        }
    }
}

use allsky::Allsky;
use tile::Tile;
use blank::PixelMetadata;

pub enum Resource {
    Tile(Tile),
    Allsky(Allsky),
    PixelMetadata(PixelMetadata)
}
