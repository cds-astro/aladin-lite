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

    // Flag telling if FITS tiles are being downloaded
    fits: bool,
    i_internal_format: bool,
}

use crate::{
    buffer::Texture,
    viewport::ViewPort,
    time::Time,
    async_task::AladinTaskExecutor,
    image_fmt::FormatImageType
};
impl BufferTextures {
    pub fn new(gl: &WebGl2Context, config: &HiPSConfig, viewport: &ViewPort) -> BufferTextures {
        // Arbitrary number decided here
        let requested_tiles = HashSet::with_capacity(64);

        let time_last_tile_written = Time::now();

        let request_system = RequestSystem::new();
        let fits = false;
        let i_internal_format = false;
        let mut buffer = BufferTextures {
            requested_tiles,
            i_internal_format,

            request_system,

            time_last_tile_written,
            fits,
        };

        buffer.initialize(viewport, config);

        buffer
    }

    pub fn reset(&mut self, gl: &WebGl2Context, config: &HiPSConfig, viewport: &ViewPort, task_executor: &mut AladinTaskExecutor) {
        self.survey.clear(&gl, config, task_executor);
        self.requested_tiles.clear();

        self.request_system.reset(config);

        self.initialize(viewport, config);
    }

    /*pub fn get_cutoff(&self, tile_cell: &HEALPixCell) -> Option<(f32, f32)> {
        self.survey.get_cutoff(tile_cell)
    }*/

    // Ask for the tiles until they are found in the buffer
    pub fn ask_for_tiles(&mut self, cells: &HashMap<HEALPixCell, bool>, config: &HiPSConfig) {
        for (texture_cell, new) in cells.iter() {
            for tile_cell in texture_cell.get_tile_cells(config) {
                self.load_tile(&tile_cell, *new, config);
            }
        }
    }

    pub fn ack_tiles_sent_to_gpu(&mut self, copied_tiles: &HashSet<HEALPixCell>, task_executor: &mut AladinTaskExecutor, config: &mut HiPSConfig) {
        self.survey.register_tiles_sent_to_gpu(copied_tiles, config);
        let is_tile_cells_copied = !copied_tiles.is_empty();

        // Process new requests
        self.request_system.run(
            config,
            copied_tiles,
            task_executor,
            &mut self.survey,
            &mut self.requested_tiles
        );
        if is_tile_cells_copied {
            self.time_last_tile_written = Time::now();
        }
    }

    pub fn time_last_tile_written(&self) -> Time {
        self.time_last_tile_written
    }

    // cell is contained into the buffer
    pub fn get_texture(&self, cell: &HEALPixCell) -> &Texture {
        self.survey.get(cell)
            .unwrap()
    }

    // Get the nearest parent tile found in the CPU buffer
    pub fn get_nearest_parent(&self, cell: &HEALPixCell) -> HEALPixCell {
        if cell.is_root() {
            // Root cells are in the buffer by definition
            *cell
        } else {
            let mut parent_cell = cell.parent();

            while !self.contains(&parent_cell) && !parent_cell.is_root() {
                parent_cell = parent_cell.parent();
            }

            parent_cell
        }
    }

    fn initialize(&mut self, viewport: &ViewPort, config: &HiPSConfig) {
        // Request for the root texture cells
        let root_textures = HEALPixCell::root()
            .iter()
            .map(|&c| (c, true))
            .collect::<HashMap<_, _>>();

        self.ask_for_tiles(&root_textures, config);

        // Request for the textures in the current fov
        let cell_textures = viewport.new_healpix_cells();
        self.ask_for_tiles(&cell_textures, config);

        // Keep a flag if fits tiles are requested or not
        let fmt = &config.format();
        match config.format() {
            FormatImageType::FITS(fits) => {
                self.fits = true;
            },
            _ => {
                self.fits = false;
            }
        };
        self.i_internal_format = fmt.is_i_internal_format();
    }

    fn load_tile(&mut self,
        // The HEALPix cell to load. First check whether it is already in the buffer
        cell: &HEALPixCell,
        // A flag telling whether the HEALPix cell to load is new (i.e. not contained in the previous
        // field of view).
        new: bool,
        config: &HiPSConfig
    ) {
        let already_loaded = self.survey.contains_tile(cell, config);
        if already_loaded {
            let start_time = Time::now();

            // Remove and append the texture with an updated
            // time_request
            self.survey.update_priority(cell, new, start_time, config);
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

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    pub fn contains(&self, texture_cell: &HEALPixCell) -> bool {
        if let Some(texture) = self.survey.get(texture_cell) {
            // The texture is in the buffer i.e. there is at least one
            // sub tile received

            // It is possible that it is not available. Available means
            // all its sub tiles have been received and written to the
            // textures array!
            texture.is_available()
        } else {
            // The texture is not contained in the buffer i.e.
            // even not one sub tile that has been received
            false
        }
    }

    pub fn is_ready(&self) -> bool {
        self.survey.is_ready()
    }

    pub fn fits_tiles_requested(&self) -> bool {
        self.fits
    }
    pub fn fits_i_format(&self) -> bool {
        self.i_internal_format
    }
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;
impl HasUniforms for BufferTextures {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.survey);

        shader
    }
}

