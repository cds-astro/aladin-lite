use crate::Vector3;
use crate::{healpix::cell::HEALPixCell, time::Time};
use al_core::image::Image;
use al_core::Texture2DArray;
use wasm_bindgen::JsValue;

pub struct HpxTexture2D {
    tile_cell: HEALPixCell,
    // Precomputed uniq number
    uniq: i32,
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
    copied_to_gpu: bool,
}

use crate::renderable::hips::HpxTile;

impl HpxTexture2D {
    pub fn new(cell: &HEALPixCell, idx: i32, time_request: Time) -> Self {
        let start_time = None;
        let copied_to_gpu = false;
        let tile_cell = *cell;
        let uniq = cell.uniq();

        Self {
            tile_cell,
            uniq,
            time_request,
            idx,
            start_time,
            copied_to_gpu,
        }
    }

    pub fn is_on_gpu(&self) -> bool {
        self.copied_to_gpu
    }

    pub fn idx(&self) -> i32 {
        self.idx
    }

    // Setter
    pub fn replace(&mut self, tile_cell: &HEALPixCell, time_request: Time) {
        // Cancel the tasks copying the tiles contained in the texture
        // which have not yet been completed.
        //self.clear_tasks_in_progress(config, exec);

        self.tile_cell = *tile_cell;
        self.uniq = tile_cell.uniq();
        self.copied_to_gpu = false;
        self.start_time = None;
        self.time_request = time_request;
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    pub fn copy_to_gpu<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: &I,
        gpu_texture: &Texture2DArray,
    ) -> Result<(), JsValue> {
        debug_assert!(*cell == self.tile_cell);
        debug_assert!(!self.copied_to_gpu);

        self.copied_to_gpu = true;
        self.start_time = Some(Time::now());

        image.insert_into_3d_texture(gpu_texture, &Vector3::new(0, 0, self.idx()))
    }
}

impl HpxTile for HpxTexture2D {
    // Getter
    // Returns the current time if the texture is not full
    fn start_time(&self) -> Time {
        if self.is_on_gpu() {
            self.start_time.unwrap_abort()
        } else {
            Time::now()
        }
    }

    fn time_request(&self) -> Time {
        self.time_request
    }

    fn cell(&self) -> &HEALPixCell {
        &self.tile_cell
    }
}

use std::cmp::Ordering;
impl PartialOrd for HpxTexture2D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
use crate::Abort;
impl Ord for HpxTexture2D {
    fn cmp(&self, other: &Self) -> Ordering {
        self.uniq.cmp(&other.uniq)
    }
}

impl PartialEq for HpxTexture2D {
    fn eq(&self, other: &Self) -> bool {
        self.uniq == other.uniq
    }
}
impl Eq for HpxTexture2D {}

pub struct HpxTexture2DUniforms<'a> {
    texture: &'a HpxTexture2D,
    name: String,
}

impl<'a> HpxTexture2DUniforms<'a> {
    pub fn new(texture: &'a HpxTexture2D, idx_texture: i32) -> Self {
        let name = format!("textures_tiles[{}].", idx_texture);
        HpxTexture2DUniforms { texture, name }
    }
}

use al_core::shader::{SendUniforms, ShaderBound};
impl SendUniforms for HpxTexture2DUniforms<'_> {
    // Info: These uniforms are used for raytracing drawing mode only
    fn attach_uniforms<'b>(&self, shader: &'b ShaderBound<'b>) -> &'b ShaderBound<'b> {
        shader
            .attach_uniform(&format!("{}{}", self.name, "uniq"), &self.texture.uniq)
            .attach_uniform(
                &format!("{}{}", self.name, "texture_idx"),
                &self.texture.idx,
            )
            .attach_uniform(
                &format!("{}{}", self.name, "empty"),
                // This is useful for FITS tiles only because:
                // - for JPEG, missing tiles are inserted in the buffer and black is drawn
                // - for PNG, tiles are not inserted but default color chosen is fully transparent (might be vec4(0.0, 0.0, 0.0, 0.0))
                //
                // Therefore for FITS files we must indicate GPU which base tiles are missing so that we draw fully transparent pixels
                &((!self.texture.copied_to_gpu as u8) as f32),
            )
            .attach_uniform(
                &format!("{}{}", self.name, "start_time"),
                &self.texture.start_time(),
            );

        shader
    }
}
