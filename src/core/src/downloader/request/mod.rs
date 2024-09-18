// A request image should not be used outside this module
// but contained inside a more specific type of query (e.g. for a tile or allsky)
pub mod allsky;
pub mod blank;
pub mod moc;
pub mod tile;

/* ------------------------------------- */

use crate::time::Time;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
pub type Url = String;
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
    Failed,
    Found,
}
use crate::Abort;
use std::future::Future;
use wasm_bindgen::JsValue;
impl<R> Request<R>
where
    R: 'static,
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
                    *(data_cloned.lock().unwrap_abort()) = Some(resp);
                    resolved_cloned.set(ResolvedStatus::Found);
                } else {
                    resolved_cloned.set(ResolvedStatus::Failed);
                }

                Ok(JsValue::from_bool(true))
            };

            let _ = wasm_bindgen_futures::future_to_promise(fut);
        }

        Self {
            data,
            resolved,
            time_request,
        }
    }

    pub fn is_resolved(&self) -> bool {
        let resolved = self.resolve_status();
        resolved == ResolvedStatus::Found || resolved == ResolvedStatus::Failed
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }
}

use allsky::AllskyRequest;
use blank::PixelMetadataRequest;
use moc::MOCRequest;
use tile::TileRequest;
pub enum RequestType {
    Tile(TileRequest),
    Allsky(AllskyRequest),
    PixelMetadata(PixelMetadataRequest),
    Moc(MOCRequest), //..
}

use crate::downloader::QueryId;
impl RequestType {
    pub fn id(&self) -> &QueryId {
        match self {
            RequestType::Tile(request) => &request.id,
            RequestType::Allsky(request) => &request.id,
            RequestType::PixelMetadata(request) => &request.id,
            RequestType::Moc(request) => &request.hips_cdid,
        }
    }
}

impl<'a> From<&'a RequestType> for Option<Resource> {
    fn from(request: &'a RequestType) -> Self {
        match request {
            RequestType::Tile(request) => Option::<Tile>::from(request).map(Resource::Tile),
            RequestType::Allsky(request) => Option::<Allsky>::from(request).map(Resource::Allsky),
            RequestType::PixelMetadata(request) => {
                Option::<PixelMetadata>::from(request).map(Resource::PixelMetadata)
            }
            RequestType::Moc(request) => Option::<Moc>::from(request).map(Resource::Moc),
        }
    }
}

use allsky::Allsky;
use blank::PixelMetadata;
use moc::Moc;
use tile::Tile;
pub enum Resource {
    Tile(Tile),
    Allsky(Allsky),
    PixelMetadata(PixelMetadata),
    Moc(Moc),
}

impl Resource {
    pub fn id(&self) -> &String {
        match self {
            Resource::Tile(tile) => tile.get_hips_cdid(),
            Resource::Allsky(allsky) => allsky.get_hips_cdid(),
            Resource::PixelMetadata(PixelMetadata { hips_cdid, .. }) => hips_cdid,
            Resource::Moc(moc) => moc.get_hips_cdid(),
        }
    }
}
