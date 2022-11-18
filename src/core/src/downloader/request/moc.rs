use crate::downloader::query;

use super::{Request, RequestType};
use moclib::qty::Hpx;
use moclib::deser::fits::MocType;
use crate::healpix::coverage::Smoc;
use crate::downloader::QueryId;

pub struct MOCRequest {
    pub id: QueryId,
    pub url: Url,
    pub params: al_api::moc::MOC,
    request: Request<HEALPixCoverage>,
}

impl From<MOCRequest> for RequestType {
    fn from(request: MOCRequest) -> Self {
        RequestType::Moc(request)
    }
}
use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast;
use moclib::deser::fits;

use moclib::moc::range::op::convert::convert_to_u64;

/// Convenient type for Space-MOCs
pub fn from_fits_hpx<T: Idx>(
    moc: MocType<T, Hpx<T>, Cursor<&[u8]>>
) -> Smoc {
    match moc {
        MocType::Ranges(moc) => convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc).into_range_moc(),
        MocType::Cells(moc) => convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(
            moc.into_cell_moc_iter().ranges()
        ).into_range_moc(),
    }
}
use crate::downloader::query::Query;
use std::io::Cursor;
use moclib::idx::Idx;
use moclib::moc::{RangeMOCIterator, CellMOCIntoIterator, CellMOCIterator};
use moclib::deser::fits::MocIdxType;
use moclib::deser::fits::MocQtyType;
use wasm_bindgen::JsValue;
use crate::healpix::coverage::HEALPixCoverage;
use crate::Abort;
impl From<query::Moc> for MOCRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::Moc) -> Self {
        let id = query.id();
        let query::Moc {
            url,
            params,
        } = query;

        let url_clone = url.clone();

        let window = web_sys::window().unwrap_abort();
        let request =  Request::new(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap_abort();
            let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
            // `resp_value` is a `Response` object.
            debug_assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into()?;
            let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

            let start_time = crate::utils::get_current_time();

            let bytes_buf = js_sys::Uint8Array::new(&array_buffer);
            let num_bytes = bytes_buf.length() as usize;
            let mut bytes = Vec::with_capacity(num_bytes);
            unsafe { bytes.set_len(num_bytes); }
            bytes_buf.copy_to(&mut bytes[..]);
    
            // Coosys is permissive because we load a moc
            let smoc = match fits::from_fits_ivoa_custom(Cursor::new(&bytes[..]), true).map_err(|e| JsValue::from_str(&e.to_string()))? {
                MocIdxType::U16(MocQtyType::<u16, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                MocIdxType::U32(MocQtyType::<u32, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                MocIdxType::U64(MocQtyType::<u64, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                _ => Err(JsValue::from_str("MOC not supported. Must be a HPX MOC"))
            }?;

            Ok(HEALPixCoverage(smoc))
        });

        Self {
            id,
            url,
            request,
            params,
        }
    }
}

use std::sync::{Arc, Mutex};
pub struct Moc {
    pub moc: Arc<Mutex<Option<HEALPixCoverage>>>,
    pub params: al_api::moc::MOC,
    pub url: Url,
}

impl Moc {
    pub fn get_url(&self) -> &Url {
        &self.url
    }
}

impl<'a> From<&'a MOCRequest> for Option<Moc> {
    fn from(request: &'a MOCRequest) -> Self {
        let MOCRequest {
            request,
            url,
            params,
            ..
        } = request;
        if request.is_resolved() {
            let Request::<HEALPixCoverage> {
                data, ..
            } = request;
            Some(Moc {
                // This is a clone on a Arc, it is supposed to be fast
                moc: data.clone(),
                url: url.clone(),
                params: params.clone()
            })
        } else {
            None
        }
    }
}
