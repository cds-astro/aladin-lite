// A request image should not be used outside this module
// but contained inside a more specific type of query (e.g. for a tile or allsky)
pub mod allsky;
pub mod moc;
pub mod tile;

use wasm_bindgen_futures::JsFuture;

use crate::time::Time;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
pub type Url = String;
pub struct Request<R> {
    data: Rc<RefCell<Option<R>>>,
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

        let data = Rc::new(RefCell::new(None));

        {
            let data_cloned = data.clone();
            let resolved_cloned = resolved.clone();

            let fut = async move {
                let resp = f.await;
                if let Ok(resp) = resp {
                    data_cloned.replace(Some(resp));
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
use moc::MOCRequest;
use tile::TileRequest;
pub enum RequestType {
    Tile(TileRequest),
    Allsky(AllskyRequest),
    Moc(MOCRequest), //..
}

use crate::downloader::QueryId;
impl RequestType {
    pub fn id(&self) -> &QueryId {
        match self {
            RequestType::Tile(request) => &request.id,
            RequestType::Allsky(request) => &request.id,
            RequestType::Moc(request) => &request.hips_cdid,
        }
    }
}

impl<'a> From<&'a RequestType> for Option<Resource> {
    fn from(request: &'a RequestType) -> Self {
        match request {
            RequestType::Tile(request) => Option::<Tile>::from(request).map(Resource::Tile),
            RequestType::Allsky(request) => Option::<Allsky>::from(request).map(Resource::Allsky),
            RequestType::Moc(request) => Option::<Moc>::from(request).map(Resource::Moc),
        }
    }
}

use crate::Abort;
use allsky::Allsky;
use moc::Moc;
use tile::Tile;
pub enum Resource {
    Tile(Tile),
    Allsky(Allsky),
    Moc(Moc),
}

use web_sys::RequestCredentials;
async fn query_html_image(
    url: &str,
    credentials: RequestCredentials,
) -> Result<web_sys::HtmlImageElement, JsValue> {
    let image = web_sys::HtmlImageElement::new().unwrap_abort();
    let image_cloned = image.clone();

    // Set the CORS and credentials options for the image
    let cors_value = match credentials {
        RequestCredentials::Include => Some("use-credentials"),
        RequestCredentials::SameOrigin => Some("anonymous"),
        _ => Some(""),
    };

    let promise = js_sys::Promise::new(
        &mut (Box::new(move |resolve, reject| {
            // Ask for CORS permissions
            image_cloned.set_cross_origin(cors_value);
            image_cloned.set_onload(Some(&resolve));
            image_cloned.set_onerror(Some(&reject));
            image_cloned.set_src(url);
        }) as Box<dyn FnMut(js_sys::Function, js_sys::Function)>),
    );

    let _ = JsFuture::from(promise).await?;

    Ok(image)
}
