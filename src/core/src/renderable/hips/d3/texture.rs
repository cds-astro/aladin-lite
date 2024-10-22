use crate::renderable::hips::d2::texture::HpxTexture2D;
use crate::{healpix::cell::HEALPixCell, time::Time};

use al_core::image::format::{
    ChannelType, R16I, R32F, R32I, R64F, R8UI, RGB32F, RGB8U, RGBA32F, RGBA8U,
};
use al_core::image::Image;
use al_core::texture::Texture3D;
use al_core::webgl_ctx::WebGlRenderingCtx;
use cgmath::Vector3;
use wasm_bindgen::JsValue;

pub struct HpxTexture3D {
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
    // slices inside a cubic tile is done with a u32 mask. Limited to 16384 slices
    blocks: [u32; 512],
    // sorted index list of 32-length blocks that are not empty
    block_indices: Vec<usize>,
}

use crate::renderable::hips::config::HiPSConfig;
use crate::WebGlContext;

use crate::renderable::hips::HpxTile;

impl HpxTexture3D {
    pub fn new(tile_cell: HEALPixCell, time_request: Time) -> Self {
        let start_time = None;
        let uniq = tile_cell.uniq();
        let textures = std::iter::repeat(None).take(512).collect();
        let blocks = [0; 512];
        let block_indices = Vec::new();
        Self {
            tile_cell,
            uniq,
            time_request,
            start_time,
            textures,
            blocks,
            block_indices,
        }
    }

    pub fn find_nearest_slice(&self, slice: u16) -> Option<u16> {
        let block_idx = (slice >> 5) as usize;

        match self.block_indices.binary_search(&block_idx) {
            Ok(_) => {
                if self.contains_slice(slice) {
                    Some(slice)
                } else {
                    // the slice is not present but we know there is one in the block
                    let block = self.blocks[block_idx];

                    let slice_idx = (slice & 0x1f) as u32;

                    let m2 = if slice_idx == 31 {
                        0
                    } else {
                        0xffffffff >> (slice_idx + 1)
                    };
                    let m1 = (!m2) & !(1 << (31 - slice_idx));

                    let lb = ((block & m1) >> (32 - slice_idx)) as u32;
                    let rb = (block & m2) as u32;

                    let lb_trailing_zeros = (lb.trailing_zeros() as u16).min(slice_idx as u16);
                    let rb_leading_zeros = (rb.leading_zeros() - slice_idx - 1) as u16;

                    let no_more_left_bits = slice_idx - (lb_trailing_zeros as u32) == 0;
                    let no_more_right_bits = slice_idx + (rb_leading_zeros as u32) == 31;

                    match (no_more_left_bits, no_more_right_bits) {
                        (false, false) => {
                            if lb_trailing_zeros <= rb_leading_zeros {
                                Some(slice - lb_trailing_zeros - 1)
                            } else {
                                Some(slice + rb_leading_zeros + 1)
                            }
                        }
                        (false, true) => {
                            if lb_trailing_zeros <= rb_leading_zeros {
                                Some(slice - lb_trailing_zeros - 1)
                            } else {
                                // explore next block
                                if block_idx == self.blocks.len() - 1 {
                                    // no after block
                                    Some(slice - lb_trailing_zeros - 1)
                                } else {
                                    // get the next block
                                    let next_block = self.blocks[block_idx + 1];

                                    let num_bits_to_next_block =
                                        next_block.leading_zeros() as u16 + rb_leading_zeros;

                                    if num_bits_to_next_block < lb_trailing_zeros {
                                        Some(slice + num_bits_to_next_block + 1)
                                    } else {
                                        Some(slice - lb_trailing_zeros - 1)
                                    }
                                }
                            }
                        }
                        (true, false) => {
                            if rb_leading_zeros <= lb_trailing_zeros {
                                Some(slice + rb_leading_zeros + 1)
                            } else {
                                // explore previous block
                                if block_idx == 0 {
                                    // no after block
                                    Some(slice + rb_leading_zeros + 1)
                                } else {
                                    // get the next block
                                    let prev_block = self.blocks[block_idx - 1];

                                    let num_bits_from_prev_block =
                                        prev_block.trailing_zeros() as u16 + lb_trailing_zeros;
                                    if num_bits_from_prev_block < rb_leading_zeros {
                                        Some(slice - num_bits_from_prev_block - 1)
                                    } else {
                                        Some(slice + rb_leading_zeros + 1)
                                    }
                                }
                            }
                        }
                        (true, true) => unreachable!(),
                    }
                }
            }
            Err(i) => {
                match (self.block_indices.get(i - 1), self.block_indices.get(i)) {
                    (Some(b_idx_1), Some(b_idx_2)) => {
                        let b1 = self.blocks[*b_idx_1];
                        let b2 = self.blocks[*b_idx_2];

                        let b1_tz = b1.trailing_zeros() as usize;
                        let b2_lz = b2.leading_zeros() as usize;

                        let slice_b1 = ((*b_idx_1 << 5) + 32 - b1_tz - 1) as u16;
                        let slice_b2 = ((*b_idx_2 << 5) + b2_lz) as u16;
                        if slice - slice_b1 <= slice_b2 - slice {
                            // the nearest slice is in b1
                            Some(slice_b1 as u16)
                        } else {
                            // the nearest slice is in b2
                            Some(slice_b2 as u16)
                        }
                    }
                    (None, Some(b_idx_2)) => {
                        let b2 = self.blocks[*b_idx_2];
                        let b2_lz = b2.leading_zeros() as usize;

                        Some(((*b_idx_2 << 5) + b2_lz) as u16)
                    }
                    (Some(b_idx_1), None) => {
                        let b1 = self.blocks[*b_idx_1];
                        let b1_tz = b1.trailing_zeros() as usize;

                        Some(((*b_idx_1 << 5) + 32 - b1_tz - 1) as u16)
                    }
                    (None, None) => None,
                }
            }
        }
    }

    pub fn get_3d_block_from_slice(&self, slice: u16) -> Option<&Texture3D> {
        let block_idx = slice >> 5;

        self.textures[block_idx as usize].as_ref()
    }

    pub fn extract_2d_slice_texture(&self, slice: u16) -> Option<HpxTexture2D> {
        // Find the good sub cube containing the slice
        let block_idx = (slice >> 5) as usize;
        let slice_idx = (slice & 0x1f) as u8;

        // check the texture is there
        if self.blocks[block_idx] & (1 << (31 - slice_idx)) != 0 {
            Some(HpxTexture2D::new(
                &self.tile_cell,
                slice_idx as i32,
                self.time_request,
            ))
        } else {
            None
        }
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    pub fn append<I: Image>(
        &mut self,
        image: I,
        slice: u16,
        cfg: &HiPSConfig,
        gl: &WebGlContext,
    ) -> Result<(), JsValue> {
        let block_idx = (slice >> 5) as usize;

        let texture = if let Some(texture) = self.textures[block_idx as usize].as_ref() {
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
            self.textures[block_idx] = Some(texture?);

            self.textures[block_idx].as_ref().unwrap()
        };

        let slice_idx = slice & 0x1f;

        // if there is already something, do not tex sub
        if self.blocks[block_idx] & (1 << (31 - slice_idx)) == 0 {
            image.insert_into_3d_texture(texture, &Vector3::<i32>::new(0, 0, slice_idx as i32))?;

            match self.block_indices.binary_search(&block_idx) {
                Ok(i) => {} // element already in vector @ `pos`
                Err(i) => self.block_indices.insert(i, block_idx),
            }

            self.blocks[block_idx] |= 1 << (31 - slice_idx);
        }

        self.start_time = Some(Time::now());

        Ok(())
    }

    // Cell must be contained in the texture
    pub fn contains_slice(&self, slice: u16) -> bool {
        let block_idx = (slice >> 5) as usize;
        let idx_in_block = slice & 0x1f;

        (self.blocks[block_idx] >> (31 - idx_in_block)) & 0x1 == 1
    }
}

impl HpxTile for HpxTexture3D {
    // Getter
    // Returns the current time if the texture is not full
    fn start_time(&self) -> Time {
        if let Some(t) = self.start_time {
            t
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
impl PartialOrd for HpxTexture3D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.uniq.partial_cmp(&other.uniq)
    }
}
use crate::Abort;
impl Ord for HpxTexture3D {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

impl PartialEq for HpxTexture3D {
    fn eq(&self, other: &Self) -> bool {
        self.uniq == other.uniq
    }
}
impl Eq for HpxTexture3D {}

/*
pub struct TextureUniforms<'a> {
    texture: &'a HpxTexture3D,
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
