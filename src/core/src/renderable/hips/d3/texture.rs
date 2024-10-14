use crate::{healpix::cell::HEALPixCell, time::Time};

use al_core::image::format::{
    ChannelType, ImageFormatType, R16I, R32F, R32I, R64F, R8UI, RGB32F, RGB8U, RGBA32F, RGBA8U,
};
use al_core::image::Image;
use al_core::texture::Texture3D;
use al_core::webgl_ctx::WebGlRenderingCtx;
use cgmath::Vector3;
use std::collections::HashSet;
use wasm_bindgen::JsValue;

pub struct HEALPixTexturedCube {
    tile_cell: HEALPixCell,
    // Precomputed uniq number
    uniq: i32,
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

    // We autorize 512 cubic tiles of size 32 each which allows to store max 16384 slices
    textures: Vec<Option<Texture3D>>,
    // A set of already inserted slices. Each cubic tiles can have 32 slices. The occupancy of the
    // slices inside a cubic tile is done with a u32 mask
    slices: [u32; 512],
}

use crate::renderable::hips::config::HiPSConfig;
use crate::WebGlContext;

impl HEALPixTexturedCube {
    pub fn new(tile_cell: HEALPixCell, time_request: Time) -> Self {
        let start_time = None;
        let uniq = tile_cell.uniq();
        let textures = std::iter::repeat(None).take(512).collect();
        let slices = [0; 512];

        Self {
            tile_cell,
            uniq,
            time_request,
            start_time,
            textures,
            slices,
        }
    }

    // Get the good cubic texture and the slice idx inside it
    pub fn get_cubic_texture_from_slice(&self, slice: u16) -> (Option<&Texture3D>, u8) {
        let cube_idx = slice >> 5;
        let slice_idx = (slice & 0x1f) as u8;
        (self.textures[cube_idx as usize].as_ref(), slice_idx)
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    pub fn append_slice<I: Image>(
        &mut self,
        image: I,
        slice: u16,
        cfg: &HiPSConfig,
        gl: &WebGlContext,
    ) -> Result<(), JsValue> {
        let cube_idx = (slice >> 5) as usize;

        let texture = if let Some(texture) = self.textures[cube_idx as usize].as_ref() {
            texture
        } else {
            let tile_size = cfg.get_tile_size();
            let params = &[
                (
                    WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                    WebGlRenderingCtx::NEAREST,
                ),
                (
                    WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                    WebGlRenderingCtx::NEAREST,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_S,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_T,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
                // Prevents r-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_R,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
            ];

            let texture = match cfg.get_format().get_channel() {
                ChannelType::RGBA32F => {
                    Texture3D::create_empty::<RGBA32F>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::RGB32F => {
                    Texture3D::create_empty::<RGB32F>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::RGBA8U => {
                    Texture3D::create_empty::<RGBA8U>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::RGB8U => {
                    Texture3D::create_empty::<RGB8U>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::R32F => {
                    Texture3D::create_empty::<R32F>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::R64F => {
                    Texture3D::create_empty::<R64F>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::R8UI => {
                    Texture3D::create_empty::<R8UI>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::R16I => {
                    Texture3D::create_empty::<R16I>(gl, tile_size, tile_size, 32, params)
                }
                ChannelType::R32I => {
                    Texture3D::create_empty::<R32I>(gl, tile_size, tile_size, 32, params)
                }
            };
            self.textures[cube_idx] = Some(texture?);

            self.textures[cube_idx].as_ref().unwrap()
        };

        let slice_idx = slice & 0x1f;

        // if there is already something, do not tex sub
        if self.slices[cube_idx] & (1 << slice_idx) == 0 {
            image.insert_into_3d_texture(texture, &Vector3::<i32>::new(0, 0, slice_idx as i32))?
        }

        self.start_time = Some(Time::now());

        Ok(())
    }

    // Cell must be contained in the texture
    pub fn contains_slice(&self, slice: u16) -> bool {
        let cube_idx = (slice >> 5) as usize;
        let slice_idx = slice & 0x1f;

        self.slices[cube_idx] & (1 << slice_idx) == 1
    }

    // Getter
    // Returns the current time if the texture is not full
    pub fn start_time(&self) -> Time {
        if let Some(t) = self.start_time {
            t
        } else {
            Time::now()
        }
    }

    pub fn time_request(&self) -> Time {
        self.time_request
    }

    pub fn cell(&self) -> &HEALPixCell {
        &self.tile_cell
    }

    // Setter
    /*pub fn replace(&mut self, texture_cell: &HEALPixCell, time_request: Time) {
        // Cancel the tasks copying the tiles contained in the texture
        // which have not yet been completed.
        //self.clear_tasks_in_progress(config, exec);

        self.texture_cell = *texture_cell;
        self.uniq = texture_cell.uniq();
        self.full = false;
        self.start_time = None;
        self.time_request = time_request;
        self.tiles.clear();
        //self.missing = true;
        self.num_tiles_written = 0;
    }*/
}

use std::cmp::Ordering;
impl PartialOrd for HEALPixTexturedCube {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.uniq.partial_cmp(&other.uniq)
    }
}
use crate::Abort;
impl Ord for HEALPixTexturedCube {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

impl PartialEq for HEALPixTexturedCube {
    fn eq(&self, other: &Self) -> bool {
        self.uniq == other.uniq
    }
}
impl Eq for HEALPixTexturedCube {}

/*
pub struct TextureUniforms<'a> {
    texture: &'a HEALPixTexturedCube,
    name: String,
}

impl<'a> TextureUniforms<'a> {
    pub fn new(texture: &Texture, idx_texture: i32) -> TextureUniforms {
        let name = format!("textures_tiles[{}].", idx_texture);
        TextureUniforms { texture, name }
    }
}

use al_core::shader::{SendUniforms, ShaderBound};
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
                //&((self.texture.full as u8) as f32),
                &0.0,
            )
            .attach_uniform(
                &format!("{}{}", self.name, "start_time"),
                &self.texture.start_time(),
            );

        shader
    }
}
*/
