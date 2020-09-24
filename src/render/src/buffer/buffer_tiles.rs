use std::{
    rc::Rc,
    cell::RefCell,
    collections::{HashSet, HashMap, VecDeque},
};

use crate::{
    WebGl2Context,
    healpix_cell::HEALPixCell,
};
use super::TileDownloader;

use crate::buffer::{
 ImageSurvey,
 HiPSConfig,
};

// A tile is described by an image survey
// and an HEALPix cell
#[derive(PartialEq, Eq, Hash)]
#[derive(Clone, Debug)]
pub struct Tile {
    cell: HEALPixCell,
    root_url: String,
    format: FormatImageType,
}

pub type Tiles = HashSet<Tile>;

pub struct TileBuffer {
    requested_tiles: Tiles,
    request_system: TileDownloader,

    time_last_tile_written: Time,
}

use crate::{
    buffer::Texture,
    viewport::ViewPort,
    time::Time,
    async_task::AladinTaskExecutor,
    image_fmt::FormatImageType
};
impl TileBuffer {
    pub fn new() -> TileBuffer {
        // Arbitrary number decided here
        let requested_tiles = HashSet::with_capacity(64);

        let time_last_tile_written = Time::now();

        let request_system = TileDownloader::new();
        TileBuffer {
            requested_tiles,
            request_system,

            time_last_tile_written,
        }

        //buffer.initialize(survey, viewport);
    }

    pub fn reset(&mut self) {
        //survey.clear(&gl, task_executor);

        self.requested_tiles.clear();
        self.request_system.reset();

        //self.initialize(survey, viewport);
    }

    /*pub fn get_cutoff(&self, tile_cell: &HEALPixCell) -> Option<(f32, f32)> {
        self.survey.get_cutoff(tile_cell)
    }*/

    // Ask for the tiles until they are found in the buffer
    // TODO: API change, pass the viewport and the image survey. The viewport is storing views
    // on the different image surveys
    pub fn request_tiles(&mut self, survey: &ImageSurvey, cells: HEALPixCells) {
        // Update the views on the surveys and get the new tiles to request
        for texture_cell in cells.iter() {
            for tile_cell in texture_cell.get_tile_cells(survey.config()) {
                let tile = Tile::new(tile_cell, survey);

                self.request_tile(tile);
            }
        }
    }

    fn request_tile(&mut self, tile: &Tile) {
        let already_requested = self.requested_tiles.contains(tile);
        // The cell is not already requested
        if !already_requested {
            // Add to the tiles requested
            self.requested_tiles.insert(*tile);

            self.request_system.register_tile_request(tile);
        }
    }

    pub fn ack_tiles_sent_to_gpu(&mut self, survey: &mut ImageSurvey, copied_tiles: &HashSet<Tile>, task_executor: &mut AladinTaskExecutor) {
        survey.register_tiles_sent_to_gpu(copied_tiles);
        let is_tile_cells_copied = !copied_tiles.is_empty();

        // Process new requests
        self.request_system.run(
            copied_tiles,
            task_executor,
            survey,
            &mut self.requested_tiles
        );
        if is_tile_cells_copied {
            self.time_last_tile_written = Time::now();
        }
    }

    /*fn initialize(&mut self, survey: &mut ImageSurvey, viewport: &ViewPort) {
        // Request for the root texture cells
        let root_textures = HEALPixCell::root()
            .iter()
            .map(|&c| (c, true))
            .collect::<HashMap<_, _>>();

        self.ask_for_tiles(survey, &root_textures);

        // Request for the textures in the current fov
        let cell_textures = viewport.new_healpix_cells();
        self.ask_for_tiles(survey, &cell_textures);
    }*/

    /*fn load_tile(&mut self, tile: &Tile) {
        let already_loaded = survey.contains_tile(cell);
        if already_loaded {
            let start_time = Time::now();

            // Remove and append the texture with an updated
            // time_request
            survey.update_priority(cell, new, start_time);
            if new {
                self.time_last_tile_written = start_time;
            }
        } else {
            let already_requested = self.requested_tiles.contains(cell);
            // The cell is not already requested
            if !already_requested {
                // Add to the tiles requested
                self.requested_tiles.insert(*tile);

                self.request_system.register_tile_request(cell);
            }
        }
    }*/


    // Accessors
    pub fn time_last_tile_written(&self) -> Time {
        self.time_last_tile_written
    }
    pub fn set_time_last_tile_written(&mut self, time: Time) {
        self.time_last_tile_written = time;
    }

    /*pub fn is_ready(&self) -> bool {
        self.survey.is_ready()
    }*/
}
/*
use crate::shader::HasUniforms;
use crate::shader::ShaderBound;
impl HasUniforms for TileBuffer {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.survey);

        shader
    }
}
*/
