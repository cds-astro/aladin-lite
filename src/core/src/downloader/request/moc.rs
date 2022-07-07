use crate::{healpix::cell::HEALPixCell};
use crate::downloader::query;

use super::{Request, RequestType};
use moclib::qty::Hpx;
use moclib::moc::range::RangeMOC;
use moclib::deser::fits::MocType;
pub struct MOCRequest {
    pub hips_url: Url,
    pub url: Url,

    request: Request<RangeMOC<u64, Hpx<u64>>>,
}

impl From<MOCRequest> for RequestType {
    fn from(request: MOCRequest) -> Self {
        RequestType::MOC(request)
    }
}
use al_core::image::bitmap::Bitmap;
use al_core::image::html::HTMLImage;
use num::iter::Range;
use crate::survey::Url;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast;
use moclib::deser::fits;

use moclib::moc::range::op::convert::convert_to_u64;

/// Convenient type for Space-MOCs
type SMOC = RangeMOC<u64, Hpx<u64>>;
fn from_fits_hpx<T: Idx>(
    moc: MocType<T, Hpx<T>, Cursor<&[u8]>>
) -> SMOC {
    match moc {
        MocType::Ranges(moc) => convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc).into_range_moc(),
        MocType::Cells(moc) => convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(
            moc.into_cell_moc_iter().ranges()
        ).into_range_moc(),
    }
}
use std::io::Cursor;
use moclib::idx::Idx;
use moclib::moc::{RangeMOCIterator, CellMOCIntoIterator, CellMOCIterator};
use moclib::deser::fits::MocIdxType;
use moclib::deser::fits::MocQtyType;
use wasm_bindgen::JsValue;
impl From<query::MOC> for MOCRequest {
    // Create a tile request associated to a HiPS
    fn from(query: query::MOC) -> Self {
        let query::MOC {
            url,
            hips_url,
        } = query;

        let url_clone = url.clone();

        let window = web_sys::window().unwrap();
        let request =  Request::new(async move {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let request = web_sys::Request::new_with_str_and_init(&url_clone, &opts).unwrap();
            let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
            // `resp_value` is a `Response` object.
            debug_assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into()?;
            let array_buffer = JsFuture::from(resp.array_buffer()?).await?;

            let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
            let smoc = match fits::from_fits_ivoa(Cursor::new(&bytes[..])).map_err(|e| JsValue::from_str(&e.to_string()))? {
                MocIdxType::U16(MocQtyType::<u16, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                MocIdxType::U32(MocQtyType::<u32, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                MocIdxType::U64(MocQtyType::<u64, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
                _ => Err(JsValue::from_str("MOC not supported. Must be a HPX MOC"))
            }?;

            Ok(smoc)
        });

        Self {
            hips_url,
            url,
            request,
        }
    }
}

use crate::time::Time;
use std::sync::{Arc, Mutex};
pub struct MOC {
    pub moc: Arc<Mutex<Option<RangeMOC<u64, Hpx<u64>>>>>,
    hips_url: Url,
    url: Url,
}

impl MOC {
    pub fn get_hips_url(&self) -> &Url {
        &self.hips_url
    }

    pub fn get_url(&self) -> &Url {
        &self.url
    }
}

impl<'a> From<&'a MOCRequest> for Option<MOC> {
    fn from(request: &'a MOCRequest) -> Self {
        let MOCRequest {
            request,
            hips_url,
            url,
        } = request;
        if request.is_resolved() {
            let Request::<RangeMOC<u64, Hpx<u64>>> {
                data, ..
            } = request;
            Some(MOC {
                // This is a clone on a Arc, it is supposed to be fast
                moc: data.clone(),
                hips_url: hips_url.clone(),
                url: url.clone(),
            })
        } else {
            None
        }
    }
}
