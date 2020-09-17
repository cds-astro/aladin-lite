use super::{RequestTile, TileHTMLImage, TileArrayBuffer, ResolvedStatus, FITSImageRequest, CompressedImageRequest, RequestImage, ReceiveImage};
use crate::WebGl2Context;

use crate::buffer::{
 ImageSurvey,
 HiPSConfig,
};

use crate::async_task::AladinTaskExecutor;

use std::collections::{VecDeque, HashSet};
pub struct RequestSystem<T: RequestImage + ReceiveImage> {
    // Waiting cells to be loaded
    cells_to_be_requested: VecDeque<HEALPixCell>,

    // Collection
    requests: [RequestTile<T>; NUM_EVENT_LISTENERS],
}

const NUM_EVENT_LISTENERS: usize = 10;
const MAX_NUM_CELLS_MEMORY_REQUEST: usize = 100;
use crate::FormatImageType;
impl<T> RequestSystem<T> where T: ReceiveImage + RequestImage {
    fn new() -> RequestSystem<T> { 
        let requests: [RequestTile<T>; NUM_EVENT_LISTENERS] = Default::default();

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

    fn register_tile_request(&mut self, cell: &HEALPixCell) {
        self.cells_to_be_requested.push_back(*cell);
    }

    fn run(&mut self, config: &mut HiPSConfig, cells_copied: &HashSet<HEALPixCell>, task_executor: &mut AladinTaskExecutor, survey: &mut ImageSurvey, requested_tiles: &mut HashSet<HEALPixCell>) {
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
                            let image = config.get_blank_tile();
                            survey.push(&cell, time_request, image, task_executor, config);
                        },
                        ResolvedStatus::Found => {
                            let image = req.get_image(config);
                            survey.push(&cell, time_request, image, task_executor, config);
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
                    req.send(&cell, config);
                }
            }
        }
    }
}
use crate::healpix_cell::HEALPixCell;
pub enum RequestSystemType {
    CompressedRequestSystem(RequestSystem<CompressedImageRequest>),
    FITSRequestSystem(RequestSystem<FITSImageRequest>),
}

impl RequestSystemType {
    pub fn new(config: &HiPSConfig) -> Self {
        match config.format() {
            FormatImageType::JPG => RequestSystemType::CompressedRequestSystem(RequestSystem::<CompressedImageRequest>::new()),
            FormatImageType::PNG => RequestSystemType::CompressedRequestSystem(RequestSystem::<CompressedImageRequest>::new()),
            FormatImageType::FITS(_) => RequestSystemType::FITSRequestSystem(RequestSystem::<FITSImageRequest>::new())
        }
    }

    pub fn reset(&mut self, config: &HiPSConfig) {
        // First terminate the current requests
        match self {
            RequestSystemType::CompressedRequestSystem(rs) => rs.reset(),
            RequestSystemType::FITSRequestSystem(rs) => rs.reset(),
        }

        // Then, check the new format to change the requests
        // (HtmlImageElement for jpg/png vs XmlHttpRequest for fits files)
        *self = match config.format() {
            FormatImageType::JPG => RequestSystemType::CompressedRequestSystem(RequestSystem::new()),
            FormatImageType::PNG => RequestSystemType::CompressedRequestSystem(RequestSystem::new()),
            FormatImageType::FITS(_) => RequestSystemType::FITSRequestSystem(RequestSystem::new()),
        };
    }

    pub fn run(&mut self, config: &mut HiPSConfig, cells_copied: &HashSet<HEALPixCell>, task_executor: &mut AladinTaskExecutor, survey: &mut ImageSurvey, requested_tiles: &mut HashSet<HEALPixCell>) {
        match self {
            RequestSystemType::CompressedRequestSystem(rs) => rs.run(config, cells_copied, task_executor, survey, requested_tiles),
            RequestSystemType::FITSRequestSystem(rs) => rs.run(config, cells_copied, task_executor, survey, requested_tiles)
        }
    }

    pub fn register_tile_request(&mut self, cell: &HEALPixCell) {
        match self {
            RequestSystemType::CompressedRequestSystem(rs) => rs.register_tile_request(cell),
            RequestSystemType::FITSRequestSystem(rs) => rs.register_tile_request(cell)
        }
    }
}
