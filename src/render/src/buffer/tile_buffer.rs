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

impl Tile {
    fn new(cell: &HEALPixCell, config: &HiPSConfig) -> Self {
        Tile {
            cell: *cell,
            root_url: String::from(config.get_root_url()),
            format: config.get_format()
        }
    }
}

pub type Tiles = HashSet<Tile>;

pub struct TileBuffer {
    requested_tiles: Tiles,
    downloader: TileDownloader,

    //time_last_tile_written: Time,
}

use crate::{
    buffer::Texture,
    viewport::ViewPort,
    time::Time,
    async_task::TaskExecutor,
    image_fmt::FormatImageType
};
use super::tile_downloader::ResolvedTiles;
impl TileBuffer {
    pub fn new() -> TileBuffer {
        // Arbitrary number decided here
        let requested_tiles = HashSet::with_capacity(64);

        //let time_last_tile_written = Time::now();

        let downloader = TileDownloader::new();
        TileBuffer {
            requested_tiles,
            downloader,

            //time_last_tile_written,
        }

        //buffer.initialize(survey, viewport);
    }

    pub fn reset(&mut self) {
        //survey.clear(&gl, task_executor);

        self.requested_tiles.clear();
        self.downloader.reset();

        //self.initialize(survey, viewport);
    }

    /*pub fn get_cutoff(&self, tile_cell: &HEALPixCell) -> Option<(f32, f32)> {
        self.survey.get_cutoff(tile_cell)
    }*/

    /*pub fn request_tiles(&mut self, tiles: &Tiles) {
        // Loop over the tiles 
        for tile in tiles.iter() {
            self.request_tile(tile);
        }
    }*/

    pub fn request_tile(&mut self, tile: &Tile) {
        let already_requested = self.requested_tiles.contains(tile);
        // The cell is not already requested
        if !already_requested {
            // Add to the tiles requested
            self.requested_tiles.insert(*tile);
            self.downloader.add_tile_request(tile);
        }
    }

    pub fn try_sending_tile_requests(&mut self) {
        self.downloader.try_sending_tile_requests();
    }

    pub fn get_resolved_tiles(&mut self, available_tiles: &Tiles) -> ResolvedTiles {
        let tiles_resolved = self.downloader.retrieve_resolved_tiles(available_tiles);

        // Resolved tiles must be removed from the
        // "currently requested" tile set
        for tile in tiles_resolved {
            self.requested_tiles.remove(tile);
        }

        tiles_resolved
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
    /*pub fn time_last_tile_written(&self) -> Time {
        self.time_last_tile_written
    }

    pub fn set_time_last_tile_written(&mut self, time: Time) {
        self.time_last_tile_written = time;
    }*/

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
