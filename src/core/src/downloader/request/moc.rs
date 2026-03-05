use crate::downloader::query;
use crate::renderable::CreatorDid;

use super::{Request, RequestType};

use crate::healpix::moc::Moc;
use crate::healpix::moc::{FreqSpaceMoc, SpaceMoc};
use al_api::hips::DataproductType;

pub struct MOCRequest {
    //pub id: QueryId,
    pub hips_cdid: CreatorDid,
    pub params: MOCOptions,
    pub request: Request<Moc>,
}

impl From<MOCRequest> for RequestType {
    fn from(request: MOCRequest) -> Self {
        RequestType::Moc(request)
    }
}

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Response};

use crate::Abort;
use al_api::moc::MOCOptions;

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
                DataproductType::Cube => {
                    let moc = SpaceMoc::from_fits_raw_bytes(&bytes)?;
                    Moc::FreqSpace(FreqSpaceMoc::from_space_moc(moc))
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
