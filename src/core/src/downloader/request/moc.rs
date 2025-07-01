use crate::downloader::query;
use crate::renderable::CreatorDid;

use super::{Request, RequestType};

use crate::healpix::moc::Moc;
use crate::healpix::moc::{FreqSpaceMoc, SpaceMoc};
use al_api::hips::DataproductType;
use moclib::deser::fits::MocType;
use moclib::qty::Hpx;

pub struct MOCRequest {
    //pub id: QueryId,
    pub hips_cdid: CreatorDid,
    pub params: MOCOptions,
    request: Request<Moc>,
}

impl From<MOCRequest> for RequestType {
    fn from(request: MOCRequest) -> Self {
        RequestType::Moc(request)
    }
}
use super::Url;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Response};

use crate::Abort;
use al_api::moc::MOCOptions;

use std::io::Cursor;
use wasm_bindgen::JsValue;
impl From<query::Moc> for MOCRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Moc) -> Self {
        let query::Moc {
            url,
            params,
            hips_cdid,
            credentials,
            mode,
            dataproduct_type,
        } = query;

        let url_clone = url.clone();

        let window = web_sys::window().unwrap_abort();
        let request = Request::new(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(mode);
            opts.credentials(credentials);

            let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
            let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
            // `resp_value` is a `Response` object.
            debug_assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into()?;
            let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

            let buf = js_sys::Uint8Array::new(&array_buffer);
            let bytes = buf.to_vec();

            // Coosys is permissive because we load a moc
            Ok(match dataproduct_type {
                DataproductType::SpectralCube => {
                    Moc::FreqSpace(FreqSpaceMoc::from_fits_raw_bytes(&bytes)?)
                }
                _ => Moc::Space(SpaceMoc::from_fits_raw_bytes(&bytes)?),
            })
        });

        Self {
            //id,
            //url,
            hips_cdid,
            request,
            params,
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;
pub struct FetchedMoc {
    pub moc: Rc<RefCell<Option<Moc>>>,
    pub params: MOCOptions,
    pub hips_cdid: Url,
}

impl FetchedMoc {
    pub fn get_hips_cdid(&self) -> &Url {
        &self.hips_cdid
    }
}

impl<'a> From<&'a MOCRequest> for Option<FetchedMoc> {
    fn from(request: &'a MOCRequest) -> Self {
        let MOCRequest {
            request,
            hips_cdid,
            params,
            ..
        } = request;
        if request.is_resolved() {
            let Request::<Moc> { data, .. } = request;
            Some(FetchedMoc {
                // This is a clone on a Arc, it is supposed to be fast
                moc: data.clone(),
                hips_cdid: hips_cdid.clone(),
                params: params.clone(),
            })
        } else {
            None
        }
    }
}
