use crate::{healpix_cell::HEALPixCell, time::Time};
use std::collections::HashSet;

pub struct Texture {
    texture_cell: HEALPixCell,
    // Precomputed uniq number
    uniq: i32,
    // The cells located in the Texture
    tiles: HashSet<HEALPixCell>,
    // Position of the texture in the buffer
    idx: i32,
    // The time the texture has been received
    // If the texture contains multiple tiles, then the receiving time
    // is set when all the tiles have been copied to the buffer
    start_time: Option<Time>,
    // The time request of the texture is the time request
    // of the first tile being inserted in it
    // It is then only given in the constructor of Texture
    // This is approximate, it should correspond to the minimum
    // of the time requests of the cells currenlty contained in the
    // texture. But this is too expensive because at each tile inserted
    // in the buffer, one should reevalute the priority of the texture
    // in the buffer's binary heap.
    time_request: Time,

    // Full flag telling the texture has been filled
    full: bool,

    // Num tiles written for the gpu
    num_tiles_written: usize,
    // Flag telling whether the texture is available
    // for drawing
    is_available: bool,
    missing: bool,
}

use super::Tile;
use crate::async_task::{TaskExecutor, TaskType};
use crate::buffer::HiPSConfig;
impl Texture {
    pub fn new(
        config: &HiPSConfig,
        texture_cell: &HEALPixCell,
        idx: i32,
        time_request: Time,
    ) -> Texture {
        let tiles = HashSet::with_capacity(config.num_tiles_per_texture());

        let start_time = None;
        let full = false;
        let texture_cell = *texture_cell;
        let uniq = texture_cell.uniq();
        let is_available = false;
        let missing = true;
        let num_tiles_written = 0;
        Texture {
            texture_cell,
            uniq,
            time_request,
            tiles,
            idx,
            start_time,
            full,
            is_available,
            num_tiles_written,
            missing,
        }
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    pub fn append(&mut self, cell: &HEALPixCell, config: &HiPSConfig, missing: bool) {
        let texture_cell = cell.get_texture_cell(config);
        assert!(texture_cell == self.texture_cell);
        assert!(!self.full);

        self.missing &= missing;

        // cell has the good ancestor for this texture
        let new_tile = self.tiles.insert(*cell);
        // Ensures the tile was not already present in the buffer
        // This is the case because already contained cells do not
        // lead to new requests
        assert!(new_tile);

        if self.tiles.len() == config.num_tiles_per_texture() {
            // The texture is full
            self.full = true;
        }
    }

    pub fn register_available_tile(&mut self, cell: &HEALPixCell, config: &HiPSConfig) {
        let texture_cell = cell.get_texture_cell(config);
        assert!(texture_cell == self.texture_cell);

        let num_tiles_per_texture = config.num_tiles_per_texture();
        self.num_tiles_written += 1;

        // The texture is available to be drawn if all its
        // sub tiles have been written to the texture array
        if self.num_tiles_written == num_tiles_per_texture {
            assert!(self.is_full());
            self.is_available = true;
            // The texture is available to be drawn, we set the start time
            self.start_time = Some(Time::now());
        }
    }

    pub fn contains(&self, cell: &HEALPixCell) -> bool {
        self.tiles.contains(cell)
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn is_available(&self) -> bool {
        self.is_available
    }

    // Getter
    // Returns the current time if the texture is not full
    pub fn start_time(&self) -> Time {
        if self.is_available {
            self.start_time.unwrap()
        } else {
            Time::now()
        }
    }

    pub fn time_request(&self) -> Time {
        self.time_request
    }

    pub fn cell(&self) -> &HEALPixCell {
        &self.texture_cell
    }

    pub fn idx(&self) -> i32 {
        self.idx
    }

    pub fn is_missing(&self) -> i32 {
        self.missing as i32
    }

    // Setter
    pub fn update_start_time(&mut self, start_time: Time) {
        self.start_time = Some(start_time);
    }

    pub fn replace(
        &mut self,
        texture_cell: &HEALPixCell,
        time_request: Time,
        config: &HiPSConfig,
        exec: &mut TaskExecutor,
    ) {
        // Cancel the tasks copying the tiles contained in the texture
        // which have not yet been completed.
        for tile_cell in self.texture_cell.get_tile_cells(config) {
            let tile = Tile::new(&tile_cell, config);
            exec.remove(&TaskType::SendTileToGPU(tile));
        }

        self.texture_cell = *texture_cell;
        self.uniq = texture_cell.uniq();
        self.full = false;
        self.start_time = None;
        self.time_request = time_request;
        self.tiles.clear();
        self.is_available = false;
        self.missing = true;
        self.num_tiles_written = 0;
    }

    pub fn clear_tasks_in_progress(&self, config: &HiPSConfig, exec: &mut TaskExecutor) {
        for tile_cell in self.texture_cell.get_tile_cells(config) {
            let tile = Tile::new(&tile_cell, config);
            exec.remove(&TaskType::SendTileToGPU(tile));
        }
    }
}

use std::cmp::Ordering;
impl PartialOrd for Texture {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.uniq.partial_cmp(&other.uniq)
    }
}
impl Ord for Texture {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        self.uniq == other.uniq
    }
}
impl Eq for Texture {}

pub struct TextureUniforms<'a> {
    texture: &'a Texture,
    name: String,
}

impl<'a> TextureUniforms<'a> {
    pub fn new(texture: &Texture, idx_texture: i32) -> TextureUniforms {
        let name = format!("textures_tiles[{}].", idx_texture);
        TextureUniforms { texture, name }
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
impl<'a> SendUniforms for TextureUniforms<'a> {
    fn attach_uniforms<'b>(&self, shader: &'b ShaderBound<'b>) -> &'b ShaderBound<'b> {
        shader
            .attach_uniform(&format!("{}{}", self.name, "uniq"), &self.texture.uniq)
            .attach_uniform(
                &format!("{}{}", self.name, "texture_idx"),
                &self.texture.idx,
            )
            .attach_uniform(
                &format!("{}{}", self.name, "empty"),
                &(self.texture.missing as i32),
            )
            .attach_uniform(
                &format!("{}{}", self.name, "start_time"),
                &self.texture.start_time(),
            );

        shader
    }
}
