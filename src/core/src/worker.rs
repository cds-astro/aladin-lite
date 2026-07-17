use serde::Deserialize;

use crate::healpix::cell::HEALPixFreqCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::UnwrapThrowExt;

#[derive(Deserialize, Debug)]
pub struct Tile3DAttributes {
    #[serde(rename = "tileSize")]
    pub tile_size: u32,

    #[serde(rename = "tileDepth")]
    pub tile_depth: u32,

    #[serde(rename = "HiPSCDid")]
    pub hips_cdid: String,

    pub cell: HEALPixFreqCell,
}

use crate::HEALPixCell;

#[derive(Deserialize, Debug)]
pub struct TileAttributes {
    #[serde(rename = "tileSize")]
    pub tile_size: u32,

    pub hips: String,

    pub cell: HEALPixCell,
}

#[derive(Debug)]
pub enum WorkerResponse {
    Tile3D {
        attrs: Tile3DAttributes,
        bytes: Vec<u8>,
    },
    Tile {
        attrs: TileAttributes,
        bitmap: web_sys::ImageBitmap,
    },
}

use web_sys::{Worker, WorkerOptions};
pub fn create_worker<F>(
    src: &'static str,
    onmessage: F,
    sender: async_channel::Sender<WorkerResponse>,
) -> Result<Worker, JsValue>
where
    F: Fn(web_sys::MessageEvent) -> WorkerResponse + 'static,
{
    // Create Blob
    let parts = js_sys::Array::of1(&JsValue::from_str(src));
    let blob = web_sys::Blob::new_with_str_sequence(&parts)?;

    // Create object URL
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let opts = WorkerOptions::new();

    let worker = Worker::new_with_options(&url, &opts)?;

    // Attach the on message callback
    // Send the ack to the js promise so that she finished
    let onmessage = Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |e| {
        let response = onmessage(e);

        let sender_cloned = sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            sender_cloned.send(response).await.unwrap_throw();
        });
    });

    worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    // 🚨 VERY IMPORTANT: prevent the closure from being dropped
    onmessage.forget();

    Ok(worker)
}
