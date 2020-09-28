// Async task executor
// This executor can be used to spawn some async task that
// can be run concurrently on one thread under a time limit period
// When the time limit is reached, the executor stops polling the remaining
// futures and return the results of the finished ones
use task_async_executor::TaskExecutor;
pub type AladinTaskExecutor = TaskExecutor<TaskType, TaskResult>;

pub use crate::renderable::catalog::Source;
pub use crate::buffer::Tile;
pub enum TaskResult {
    TableParsed { name: String, sources: Vec<Source> },
    TileSentToGPU { tile: Tile }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum TaskType {
    SendTileToGPU(Tile),
    ParseTable
}

use futures::stream::Stream;

use wasm_bindgen::JsValue;

// Task that parse a table
pub struct ParseTable<T>
where T: DeserializeOwned + AsRef<[f32]> {
    table: js_sys::Array,
    idx: u32,
    next_val_ready: Option<T>,
}

use wasm_bindgen::JsCast;
impl<T> ParseTable<T>
where T: DeserializeOwned + AsRef<[f32]> {
    pub fn new(table: JsValue) -> Self {
        let table = table.dyn_into().unwrap();
        let idx = 0;
        let next_val_ready = None;
        Self {
            table,
            idx,
            next_val_ready
        }
    }
}

use std::pin::Pin;
use std::task::{Context, Poll};
use serde::de::DeserializeOwned;
impl<T> Stream for ParseTable<T>
where T: DeserializeOwned + AsRef<[f32]> + Unpin {
    type Item = T;

    /// Attempt to resolve the next item in the stream.
    /// Returns `Poll::Pending` if not ready, `Poll::Ready(Some(x))` if a value
    /// is ready, and `Poll::Ready(None)` if the stream has completed.
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>> {
        // Deserialize row by row.
        let len = self.table.length();
        if self.idx == len {
            Poll::Ready(None)
        } else {
            // Check whether the next value has been parsed
            if let Some(next_val) = self.next_val_ready.take() {
                self.idx += 1;
                Poll::Ready(Some(next_val))
            } else {
                // Parse the next value and pends the stream
                // if serde returns an error while parsing the row
                // it will be converted to a None and discarded
                self.next_val_ready = self.table.get(self.idx).into_serde::<Self::Item>().ok();
                if self.next_val_ready.is_none() {
                    // serde failed parsing the row
                    self.idx += 1;
                }
                Poll::Pending
            }
        }
    }
}

use cgmath::Vector3;

/// Task that send a tile to the GPU
pub struct SendTileToGPU {
    tile: Tile, // The tile cell that has been written
    //texture: HEALPixCell, // the texture cell that contains tile
    offset: Vector3<i32>,
    image: Box<dyn Image>,
    texture_array: Rc<Texture2DArray>
}

use crate::core::Texture2DArray;
use crate::buffer::{Image, Texture, HiPSConfig};

use std::rc::Rc;
impl SendTileToGPU {
    pub fn new<I: Image + 'static>(
        tile: &Tile, // The tile cell. It must lie in the texture
        texture: &Texture,
        image: I,
        texture_array: Rc<Texture2DArray>,
        conf: &HiPSConfig
    ) -> SendTileToGPU {
        let cell = tile.cell;
        // Index of the texture in the total set of textures
        let texture_idx = texture.idx();
        // Index of the slice of textures
        let idx_slice = texture_idx / conf.num_textures_by_slice();
        // Index of the texture in its slice
        let idx_in_slice = texture_idx % conf.num_textures_by_slice();

        // Index of the column of the texture in its slice
        let idx_col_in_slice = idx_in_slice / conf.num_textures_by_side_slice();
        // Index of the row of the texture in its slice
        let idx_row_in_slice = idx_in_slice % conf.num_textures_by_side_slice();

        // Row and column indexes of the tile in its texture
        let (idx_col_in_tex, idx_row_in_tex) = cell.get_offset_in_texture_cell(conf);

        // The size of the global texture containing the tiles
        let texture_size = conf.get_texture_size();
        // The size of a tile in its texture
        let tile_size = conf.get_tile_size();

        // Offset in the slice in pixels
        let offset = Vector3::new(
            (idx_row_in_slice as i32) * texture_size + (idx_row_in_tex as i32) * tile_size,
            (idx_col_in_slice as i32) * texture_size + (idx_col_in_tex as i32) * tile_size,
            idx_slice
        );

        let tile = *tile;
        let image = Box::new(image) as Box<dyn Image>;
        SendTileToGPU {
            tile,
            offset,
            image,
            texture_array
        }
    }
}

use futures::Future;
impl Future for SendTileToGPU {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        self.image.tex_sub_image_3d(&self.texture_array, &self.offset);

        Poll::Ready(())
    }
}