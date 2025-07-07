use crate::renderable::hips::d2::texture::HpxTex;
use crate::{healpix::cell::HEALPixCell, time::Time};

use crate::renderable::hips::config::HiPSConfig;
use crate::Abort;
use crate::WebGlContext;
use al_core::image::Image;
use al_core::texture::format::{PixelType, R16I, R32F, R32I, R8U, RGB8U, RGBA8U};
use al_core::texture::Texture3D;
use al_core::webgl_ctx::WebGlRenderingCtx;
use cgmath::Vector3;
use std::cmp::Ordering;
use wasm_bindgen::JsValue;

pub struct HpxFreqTex {
    pub cell: HEALPixFreqCell,
    // The time the texture has been received
    // If the texture contains multiple tiles, then the receiving time
    // is set when all the tiles have been copied to the buffer
    pub start_time: Option<Time>,
    // The time request of the texture is the time request
    // of the first tile being inserted in it
    // It is then only given in the constructor of Texture
    // This is approximate, it should correspond to the minimum
    // of the time requests of the cells currenlty contained in the
    // texture. But this is too expensive because at each tile inserted
    // in the buffer, one should reevalute the priority of the texture
    // in the buffer's binary heap.
    pub time_request: Time,

    // OLD CODE
    // We autorize 512 cubic tiles of size 32 each which allows to store max 16384 slices
    //textures: Vec<Option<Texture3D>>,
    // A set of already inserted slices. Each cubic tiles can have 32 slices. The occupancy of the
    // slices inside a cubic tile is done with a u32 mask. Limited to 16384 slices
    //blocks: [u32; 512],
    // sorted index list of 32-length blocks that are not empty
    //block_indices: Vec<usize>,
    /// The webgl2 3D texture of the cubic tile
    pub texture: Texture3D,

    /// A bitvector keeping track of the slices that have been inserted into the 3D texture
    /// It is limited to a cube depth of 256 (~ to the max texture size).
    slice_idx: [u32; 8],

    /// Depth of the tile
    num_slices: u16,
    /// Number of slices copied (concerns only HiPSCube)
    num_stored_slices: u16,
}

/*
impl HpxFreqTex {
    pub fn new(cell: HEALPixCell, time_request: Time) -> Self {
        let start_time = None;
        let uniq = cell.uniq();
        let textures = std::iter::repeat_n(None, 512).collect();
        let blocks = [0; 512];
        let block_indices = Vec::new();
        Self {
            cell,
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

                    let lb = (block & m1) >> (32 - slice_idx);
                    let rb = block & m2;

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
                let prev_block = if i > 0 {
                    self.block_indices.get(i - 1)
                } else {
                    None
                };

                let cur_block = self.block_indices.get(i);
                match (prev_block, cur_block) {
                    (Some(b_idx_1), Some(b_idx_2)) => {
                        let b1 = self.blocks[*b_idx_1];
                        let b2 = self.blocks[*b_idx_2];

                        let b1_tz = b1.trailing_zeros() as usize;
                        let b2_lz = b2.leading_zeros() as usize;

                        let slice_b1 = ((*b_idx_1 << 5) + 32 - b1_tz - 1) as u16;
                        let slice_b2 = ((*b_idx_2 << 5) + b2_lz) as u16;
                        if slice - slice_b1 <= slice_b2 - slice {
                            // the nearest slice is in b1
                            Some(slice_b1)
                        } else {
                            // the nearest slice is in b2
                            Some(slice_b2)
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
        let block_idx = (slice >> 5) as usize;

        self.textures[block_idx].as_ref()
    }

    pub fn extract_2d_slice_texture(&self, slice: u16) -> Option<HpxTex> {
        // Find the good sub cube containing the slice
        let block_idx = (slice >> 5) as usize;
        let slice_idx = (slice & 0x1f) as u8;

        // check the texture is there
        if self.blocks[block_idx] & (1 << (31 - slice_idx)) != 0 {
            Some(HpxTex::new(&self.cell, slice_idx as i32, self.time_request))
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

        let texture = if let Some(texture) = self.textures[block_idx].as_ref() {
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

            let texture = match cfg.get_format().get_pixel_format() {
                PixelType::RGBA8U => {
                    Texture3D::create_empty::<RGBA8U>(gl, tile_size, tile_size, 32, params)
                }
                PixelType::RGB8U => {
                    Texture3D::create_empty::<RGB8U>(gl, tile_size, tile_size, 32, params)
                }
                PixelType::R32F => {
                    Texture3D::create_empty::<R32F>(gl, tile_size, tile_size, 32, params)
                }
                PixelType::R8U => {
                    Texture3D::create_empty::<R8U>(gl, tile_size, tile_size, 32, params)
                }
                PixelType::R16I => {
                    Texture3D::create_empty::<R16I>(gl, tile_size, tile_size, 32, params)
                }
                PixelType::R32I => {
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
                Ok(_) => {} // element already in vector @ `pos`
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
*/

const TEX_PARAMS: &[(u32, u32)] = &[
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

use crate::healpix::cell::HEALPixFreqCell;
impl HpxFreqTex {
    pub fn new(
        // The cubic tile definition to locate the cube in the sky + spectral axis
        cell: HEALPixFreqCell,
        // The time the request has been made, i.e. when the tile was needed
        time_request: Time,
        // The size of the cubis tile
        tile_size: u16,
        // The depth of the cubic tile. Must be a power of two
        num_slices: u16,
        // pixel format
        pixel_format: PixelType,
        // The Gl context
        gl: &WebGlContext,
    ) -> Result<Self, JsValue> {
        let start_time = None;

        let texture = match pixel_format {
            PixelType::RGBA8U => Texture3D::create_empty::<RGBA8U>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::RGB8U => Texture3D::create_empty::<RGB8U>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R32F => Texture3D::create_empty::<R32F>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R8U => Texture3D::create_empty::<R8U>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R16I => Texture3D::create_empty::<R16I>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R32I => Texture3D::create_empty::<R32I>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
        }?;

        let num_stored_slices = 0;
        let slice_idx = [0x0; 8];
        Ok(Self {
            cell,
            slice_idx,
            time_request,
            start_time,
            texture,
            num_slices,
            num_stored_slices,
        })
    }

    /*pub fn find_nearest_slice(&self, slice: u16) -> Option<u16> {
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

                    let lb = (block & m1) >> (32 - slice_idx);
                    let rb = block & m2;

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
                let prev_block = if i > 0 {
                    self.block_indices.get(i - 1)
                } else {
                    None
                };

                let cur_block = self.block_indices.get(i);
                match (prev_block, cur_block) {
                    (Some(b_idx_1), Some(b_idx_2)) => {
                        let b1 = self.blocks[*b_idx_1];
                        let b2 = self.blocks[*b_idx_2];

                        let b1_tz = b1.trailing_zeros() as usize;
                        let b2_lz = b2.leading_zeros() as usize;

                        let slice_b1 = ((*b_idx_1 << 5) + 32 - b1_tz - 1) as u16;
                        let slice_b2 = ((*b_idx_2 << 5) + b2_lz) as u16;
                        if slice - slice_b1 <= slice_b2 - slice {
                            // the nearest slice is in b1
                            Some(slice_b1)
                        } else {
                            // the nearest slice is in b2
                            Some(slice_b2)
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
    }*/

    pub fn append_tile<I: Image>(
        &mut self,
        // the tile image
        image: I,
        cfg: &HiPSConfig,
        gl: &WebGlContext,
    ) -> Result<(), JsValue> {
        image.insert_into_3d_texture(&self.texture, &Vector3::<i32>::new(0, 0, 0))?;

        self.num_stored_slices = self.num_slices;
        self.start_time = Some(Time::now());

        Ok(())
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    pub fn append_tile_slice<I: Image>(
        &mut self,
        // the tile image
        image: I,
        // the slice offset in the cubic tile
        offset: u16,
        cfg: &HiPSConfig,
        gl: &WebGlContext,
    ) -> Result<(), JsValue> {
        // If there is already something, do not tex sub
        let block_idx = (offset >> 5) as usize;
        let slice_idx = (offset & 0x1f) as u8;

        if self.slice_idx[block_idx] & (1 << (31 - slice_idx)) == 0 {
            image.insert_into_3d_texture(
                &self.texture,
                &Vector3::<i32>::new(0, 0, slice_idx as i32),
            )?;

            self.slice_idx[block_idx] |= (1 << (31 - slice_idx));
            self.num_stored_slices += 1;
        }

        self.start_time = Some(Time::now());

        Ok(())
    }

    // Cell must be contained in the texture
    pub fn contains_slice(&self, offset: u16) -> bool {
        let block_idx = (offset >> 5) as usize;
        let slice_idx = offset & 0x1f;

        (self.slice_idx[block_idx] >> (31 - slice_idx)) & 0x1 == 1
    }
}

impl PartialOrd for HpxFreqTex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HpxFreqTex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

impl PartialEq for HpxFreqTex {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}
impl Eq for HpxFreqTex {}
