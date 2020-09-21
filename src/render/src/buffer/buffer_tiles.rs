use std::{
    rc::Rc,
    cell::RefCell,
    collections::{HashSet, HashMap, VecDeque},
};

use crate::{
    WebGl2Context,
    healpix_cell::HEALPixCell,
};
use super::RequestSystem;

use crate::buffer::{
 ImageSurvey,
 HiPSConfig,
};

pub struct BufferTextures {
    // The cells that are currently in the buffer.
    // The buffer is composed of two parts:
    // * A fixed part that will never change. The 12 base tiles are always
    //   stored
    // * A binary heap storing the most recent requested cells.
    // A set of the cells that have been requested but
    // not yet received
    requested_tiles: HashSet<HEALPixCell>,
    request_system: RequestSystem,

    time_last_tile_written: Time,
}

use crate::{
    buffer::Texture,
    viewport::ViewPort,
    time::Time,
    async_task::AladinTaskExecutor,
    image_fmt::FormatImageType
};
impl BufferTextures {
    pub fn new(gl: &WebGl2Context, survey: &mut ImageSurvey, viewport: &ViewPort) -> BufferTextures {
        // Arbitrary number decided here
        let requested_tiles = HashSet::with_capacity(64);

        let time_last_tile_written = Time::now();

        let request_system = RequestSystem::new();
        let mut buffer = BufferTextures {
            requested_tiles,
            request_system,

            time_last_tile_written,
        };

        buffer.initialize(survey, viewport);

        buffer
    }

    pub fn reset(&mut self, gl: &WebGl2Context, survey: &mut ImageSurvey, viewport: &ViewPort, task_executor: &mut AladinTaskExecutor) {
        survey.clear(&gl, task_executor);
        self.requested_tiles.clear();

        self.request_system.reset();

        self.initialize(survey, viewport);
    }

    /*pub fn get_cutoff(&self, tile_cell: &HEALPixCell) -> Option<(f32, f32)> {
        self.survey.get_cutoff(tile_cell)
    }*/

    // Ask for the tiles until they are found in the buffer
    // TODO: API change, pass the viewport and the image survey. The viewport is storing views
    // on the different image surveys
    pub fn ask_for_tiles(&mut self, survey: &mut ImageSurvey, cells: &HashMap<HEALPixCell, bool>) {
        for (texture_cell, new) in cells.iter() {
            for tile_cell in texture_cell.get_tile_cells(survey.config()) {
                self.load_tile(survey, &tile_cell, *new);
            }
        }
    }

    pub fn ack_tiles_sent_to_gpu(&mut self, survey: &mut ImageSurvey, copied_tiles: &HashSet<HEALPixCell>, task_executor: &mut AladinTaskExecutor) {
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

    pub fn time_last_tile_written(&self) -> Time {
        self.time_last_tile_written
    }

    fn initialize(&mut self, survey: &mut ImageSurvey, viewport: &ViewPort) {
        // Request for the root texture cells
        let root_textures = HEALPixCell::root()
            .iter()
            .map(|&c| (c, true))
            .collect::<HashMap<_, _>>();

        self.ask_for_tiles(survey, &root_textures);

        // Request for the textures in the current fov
        let cell_textures = viewport.new_healpix_cells();
        self.ask_for_tiles(survey, &cell_textures);
    }

    fn load_tile(&mut self,
        survey: &mut ImageSurvey,
        // The HEALPix cell to load. First check whether it is already in the buffer
        cell: &HEALPixCell,
        // A flag telling whether the HEALPix cell to load is new (i.e. not contained in the previous
        // field of view).
        new: bool,
    ) {
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
                self.requested_tiles.insert(*cell);

                self.request_system.register_tile_request(cell);
            }
        }
    }

    /*pub fn is_ready(&self) -> bool {
        self.survey.is_ready()
    }*/
}
/*
use crate::shader::HasUniforms;
use crate::shader::ShaderBound;
impl HasUniforms for BufferTextures {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.survey);

        shader
    }
}
*/
