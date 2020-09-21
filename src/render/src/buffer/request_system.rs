use super::{TileRequest, TileHTMLImage, TileArrayBuffer, ResolvedStatus, FITSImageRequest, CompressedImageRequest, ImageRequest};
use crate::WebGl2Context;

use crate::buffer::{
    ImageSurvey,
    HiPSConfig,
};

use crate::async_task::AladinTaskExecutor;

use std::collections::{VecDeque, HashSet};
pub struct RequestSystem {
    // Waiting cells to be loaded
    cells_to_be_requested: VecDeque<HEALPixCell>,

    // Collection
    requests: [TileRequest; NUM_EVENT_LISTENERS],
}

// A power of two maximum simultaneous tile requests
const NUM_EVENT_LISTENERS: usize = 16;
const MAX_NUM_CELLS_MEMORY_REQUEST: usize = 100;
use crate::FormatImageType;
use crate::healpix_cell::HEALPixCell;
impl RequestSystem {
    pub fn new() -> RequestSystem {
        let requests: [TileRequest; NUM_EVENT_LISTENERS] = Default::default();

        let cells_to_be_requested = VecDeque::with_capacity(MAX_NUM_CELLS_MEMORY_REQUEST);
        Self {
            cells_to_be_requested,
            requests,
        }
    }

    pub fn reset(&mut self) {
        self.cells_to_be_requested.clear();

        for req in self.requests.iter_mut() {
            req.clear();
        }
    }

    pub fn register_tile_request(&mut self, cell: &HEALPixCell) {
        self.cells_to_be_requested.push_back(*cell);
    }

    pub fn run(&mut self,
        cells_copied: &HashSet<HEALPixCell>,
        task_executor: &mut AladinTaskExecutor,
        survey: &mut ImageSurvey,
        requested_tiles: &mut HashSet<HEALPixCell>
    ) {
        for req in self.requests.iter_mut() {
            // First, tag the tile requests as ready if they just have been
            // given to the GPU
            if req.is_resolved() {
                let cell = *req.get_cell();

                // A tile request can be reused if its cell texture is available/readable
                // by the GPU
                let available_req = cells_copied.contains(&cell);
                if available_req {
                    req.set_ready();
                } else if requested_tiles.contains(&cell) {
                    //Tile received
                    let time_request = req.get_time_request();
                    requested_tiles.remove(&cell);
    
                    match req.resolve_status() {
                        ResolvedStatus::Missing => {
                            let image = survey.get_blank_tile();
                            survey.push(&cell, time_request, image, task_executor);
                        },
                        ResolvedStatus::Found => {
                            let image = req.get_image(config);
                            survey.push(&cell, time_request, image, task_executor);
                        },
                        _ => unreachable!()
                    }
                }
            }

            // Then, send new requests for available ones
            if !self.cells_to_be_requested.is_empty() {
                if req.is_ready() {
                    // Launch requests if the tile has yet not been used (start)
                    let cell = self.cells_to_be_requested.pop_front().unwrap();
                    req.send(&cell, survey);
                }
            }
        }
    }
}
