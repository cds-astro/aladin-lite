use crate::renderable::hips::d2::texture::HpxTex;
use std::collections::HashMap;

use al_core::texture::format::PixelType;

use crate::downloader::request::allsky::AllskyRequest;
use cgmath::Vector3;

use al_api::hips::ImageExt;
use al_core::webgl_ctx::WebGlRenderingCtx;

use al_core::image::Image;
use al_core::shader::{SendUniforms, ShaderBound};
use al_core::texture::format::{R16I, R32F, R32I, R8U, RGB8U, RGBA8U};
use al_core::Texture2DArray;
use al_core::WebGlContext;

use super::texture::HpxTexUniforms;

use crate::healpix::cell::HEALPixCell;
use crate::healpix::cell::NUM_HPX_TILES_DEPTH_ZERO;
use crate::renderable::hips::config::HiPSConfig;
use crate::time::Time;
use crate::Abort;
use crate::JsValue;

use super::super::tile_heap::{Tile, TileHeap};
use crate::renderable::hips::HpxTileBuffer;
// Fixed sized binary heap
pub struct HiPS2DBuffer {
    // Some information about the HiPS
    config: HiPSConfig,

    heap: TileHeap<HEALPixCell>,
    textures: HashMap<HEALPixCell, HpxTex>,

    num_root_textures_available: u8,

    base_textures: [HpxTex; NUM_HPX_TILES_DEPTH_ZERO],

    // Array of 2D textures
    tile_pixels: Texture2DArray,
    allsky_pixels: Texture2DArray,

    available_tiles_during_frame: bool,

    // allsky rendering mode <=> raytracing on the 12 base tiles
    allsky_rendering: bool,
}

fn create_hpx_texture_storage(
    gl: &WebGlContext,
    // The texture image channel definition
    channel: PixelType,
    // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
    num_tiles: i32,
    // The size of the tile
    tile_size: i32,
) -> Result<Texture2DArray, JsValue> {
    let tex_params = &[
        (
            WebGlRenderingCtx::TEXTURE_MIN_FILTER,
            // apply mipmapping
            WebGlRenderingCtx::NEAREST_MIPMAP_NEAREST,
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
    match channel {
        PixelType::RGBA8U => {
            Texture2DArray::create_empty::<RGBA8U>(
                gl, tile_size, tile_size,
                // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
                num_tiles, tex_params,
            )
        }
        PixelType::RGB8U => {
            Texture2DArray::create_empty::<RGB8U>(
                gl, tile_size, tile_size,
                // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
                num_tiles, tex_params,
            )
        }
        PixelType::R32F => Texture2DArray::create_empty::<R32F>(
            gl, tile_size, tile_size,
            // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
            num_tiles, tex_params,
        ),
        PixelType::R8U => Texture2DArray::create_empty::<R8U>(
            gl, tile_size, tile_size,
            // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
            num_tiles, tex_params,
        ),
        PixelType::R16I => Texture2DArray::create_empty::<R16I>(
            gl, tile_size, tile_size,
            // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
            num_tiles, tex_params,
        ),
        PixelType::R32I => Texture2DArray::create_empty::<R32I>(
            gl, tile_size, tile_size,
            // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
            num_tiles, tex_params,
        ),
    }
}

impl HiPS2DBuffer {
    pub fn push_allsky(&mut self, allsky: AllskyRequest) -> Result<(), JsValue> {
        let AllskyRequest { request, .. } = allsky;

        {
            let mutex_locked = request.data.borrow();
            let images = mutex_locked.as_ref().unwrap_abort();
            for (idx, image) in images.iter().enumerate() {
                self.push(&HEALPixCell(0, idx as u64), image, request.time_request)?;
            }
        }

        Ok(())
    }

    // Check whether the buffer has a tile
    // For that purpose, we first need to verify that its
    // texture ancestor exists and then, it it contains the tile
    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        let cell_is_root = cell.is_root();
        if cell_is_root {
            self.base_textures[cell.idx() as usize].is_on_gpu()
        } else if let Some(texture) = self.get(cell) {
            // The texture is present in the buffer
            // We must check whether it contains the tile
            texture.is_on_gpu()
        } else {
            false
        }
    }

    // Update the priority of the texture containing the tile
    // It must be ensured that the tile is already contained in the buffer
    pub fn update_priority(&mut self, cell: &HEALPixCell /*, new_fov_cell: bool*/) {
        debug_assert!(self.contains_tile(cell));

        // Get the texture cell in which the tile has to be
        if cell.is_root() {
            return;
        }

        let texture = self
            .textures
            .get(cell)
            .expect("Texture cell has not been found while the buffer contains one of its tile!");
        // Reset the time the tile has been received if it is a new cell present in the fov
        //if new_fov_cell {
        //    texture.update_start_time(Time::now());
        //}

        // MAYBE WE DO NOT NEED TO UPDATE THE TIME REQUEST IN THE BHEAP
        // BECAUSE IT INTRODUCES UNECESSARY CODE COMPLEXITY
        // Root textures are always in the buffer
        // But other textures can be removed thanks to the heap
        // data-structure. We have to update the time_request of the texture
        // and push it again in the heap to update its position.
        let mut tex_cell_item: Tile<HEALPixCell> = texture.into();
        tex_cell_item.reset_time();

        self.heap.update_entry(tex_cell_item);
    }

    pub fn push<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        let cell_is_root = cell.is_root();

        if cell_is_root {
            let tile_size = image.get_size().0;
            let is_tile = tile_size == self.config.get_tile_size() as u32;

            let texture = &mut self.base_textures[cell.idx() as usize];

            if is_tile {
                texture.copy_to_gpu(
                    cell, // The tile cell
                    &image,
                    &self.tile_pixels,
                )?;
            }

            let is_allsky_tile = tile_size == self.config.allsky_tile_size() as u32;
            if is_allsky_tile {
                texture.copy_to_gpu(
                    cell, // The tile cell
                    &image,
                    &self.allsky_pixels,
                )?;

                self.num_root_textures_available += 1;
                /*if self.num_root_textures_available == 12 {
                    self.allsky_pixels.generate_mipmap()
                }*/
            }

            self.available_tiles_during_frame = true;
        } else {
            // Not root cells
            if !self.contains_tile(cell) {
                // The texture is not among the essential ones
                // (i.e. is not a root texture)
                let mut texture = if self.heap.is_full() {
                    // Pop the oldest requested texture
                    let oldest_texture = self.heap.pop().unwrap_abort();
                    // Ensure this is not a base texture
                    debug_assert!(!oldest_texture.is_root());

                    // Remove it from the textures HashMap
                    let mut texture = self.textures.remove(oldest_texture.cell()).expect(
                        "Texture (oldest one) has not been found in the buffer of textures",
                    );
                    texture.replace(cell, time_request);

                    texture
                } else {
                    let idx = NUM_HPX_TILES_DEPTH_ZERO + self.heap.len();

                    HpxTex::new(cell, idx as i32, time_request)
                };

                texture.copy_to_gpu(
                    cell, // The tile cell
                    &image,
                    &self.tile_pixels,
                )?;

                // Push it to the buffer
                self.heap.push(&texture);
                self.textures.insert(*cell, texture);

                self.available_tiles_during_frame = true;
            }
        }

        Ok(())
    }

    pub(crate) fn read_pixel(
        &self,
        cell: &HEALPixCell,
        dx: f64,
        dy: f64,
        scale: f32,
        offset: f32,
    ) -> Result<JsValue, JsValue> {
        let value = if let Some(tile) = self.get(cell) {
            // Index of the texture in the total set of textures
            let tile_idx = tile.idx();

            // The size of the global texture containing the tiles
            let tile_size = self.config.get_tile_size() as f32;

            // Offset in the slice in pixels
            let mut pos_tex = Vector3::new(
                (dy * (tile_size as f64)) as i32,
                (dx * (tile_size as f64)) as i32,
                tile_idx,
            );

            match self.config.get_format().get_pixel_format() {
                PixelType::RGB8U | PixelType::RGBA8U => self
                    .tile_pixels
                    .read_pixel(pos_tex.x, pos_tex.y, pos_tex.z)?,
                _ => {
                    let uvy = 1.0 - (dx as f32);
                    pos_tex.y = (uvy * tile_size) as i32;

                    let f64_v = self
                        .tile_pixels
                        .read_pixel(pos_tex.x, pos_tex.y, pos_tex.z)?
                        .as_f64()
                        .ok_or("Error unwraping the pixel read value.")?;

                    // 1 channel
                    // scale the value
                    let scale = scale as f64;
                    let offset = offset as f64;

                    JsValue::from_f64(f64_v * scale + offset)
                }
            }
        } else {
            JsValue::null()
        };

        Ok(value)
    }

    pub fn render_allsky(&mut self, flag: bool) {
        self.allsky_rendering = flag;
    }

    // Get the nearest parent tile found in the CPU buffer
    pub fn get_nearest_parent(&self, cell: &HEALPixCell) -> Option<HEALPixCell> {
        let mut parent_cell = cell.parent();

        while !self.contains(&parent_cell) && !parent_cell.is_root() {
            parent_cell = parent_cell.parent();
        }

        if self.contains(&parent_cell) {
            Some(parent_cell)
        } else {
            None
        }
    }
}

impl HpxTileBuffer for HiPS2DBuffer {
    type T = HpxTex;
    type C = HEALPixCell;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let size = 128 - NUM_HPX_TILES_DEPTH_ZERO;
        //let size = 128;
        // Ensures there is at least space for the 12
        // root textures
        //debug_assert!(size >= NUM_HPX_TILES_DEPTH_ZERO);
        let heap = TileHeap::with_capacity(size);
        let textures = HashMap::with_capacity(size);

        let now = Time::now();
        let base_textures = [
            HpxTex::new(&HEALPixCell(0, 0), 0, now),
            HpxTex::new(&HEALPixCell(0, 1), 1, now),
            HpxTex::new(&HEALPixCell(0, 2), 2, now),
            HpxTex::new(&HEALPixCell(0, 3), 3, now),
            HpxTex::new(&HEALPixCell(0, 4), 4, now),
            HpxTex::new(&HEALPixCell(0, 5), 5, now),
            HpxTex::new(&HEALPixCell(0, 6), 6, now),
            HpxTex::new(&HEALPixCell(0, 7), 7, now),
            HpxTex::new(&HEALPixCell(0, 8), 8, now),
            HpxTex::new(&HEALPixCell(0, 9), 9, now),
            HpxTex::new(&HEALPixCell(0, 10), 10, now),
            HpxTex::new(&HEALPixCell(0, 11), 11, now),
        ];

        let channel = config.get_format().get_pixel_format();
        let tile_size = config.get_tile_size();
        let tile_pixels = create_hpx_texture_storage(gl, channel, 128, tile_size)?;

        let allsky_tile_size = config.allsky_tile_size();
        let allsky_pixels = create_hpx_texture_storage(gl, channel, 12, allsky_tile_size)?;
        // The root textures have not been loaded

        let num_root_textures_available = 0;
        let available_tiles_during_frame = false;

        let allsky_rendering = false;

        Ok(HiPS2DBuffer {
            config,
            heap,

            num_root_textures_available,
            textures,
            base_textures,

            tile_pixels,
            allsky_pixels,

            available_tiles_during_frame,
            allsky_rendering,
        })
    }

    fn set_image_ext(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue> {
        self.config.set_image_ext(ext)?;

        let channel = self.config.get_format().get_pixel_format();

        let tile_size = self.config.get_tile_size();
        self.tile_pixels = create_hpx_texture_storage(gl, channel, 128, tile_size)?;

        let allsky_tile_size = self.config.allsky_tile_size();
        self.allsky_pixels = create_hpx_texture_storage(gl, channel, 12, allsky_tile_size)?;

        let now = Time::now();
        self.base_textures = [
            HpxTex::new(&HEALPixCell(0, 0), 0, now),
            HpxTex::new(&HEALPixCell(0, 1), 1, now),
            HpxTex::new(&HEALPixCell(0, 2), 2, now),
            HpxTex::new(&HEALPixCell(0, 3), 3, now),
            HpxTex::new(&HEALPixCell(0, 4), 4, now),
            HpxTex::new(&HEALPixCell(0, 5), 5, now),
            HpxTex::new(&HEALPixCell(0, 6), 6, now),
            HpxTex::new(&HEALPixCell(0, 7), 7, now),
            HpxTex::new(&HEALPixCell(0, 8), 8, now),
            HpxTex::new(&HEALPixCell(0, 9), 9, now),
            HpxTex::new(&HEALPixCell(0, 10), 10, now),
            HpxTex::new(&HEALPixCell(0, 11), 11, now),
        ];

        self.heap.clear();
        self.textures.clear();
        //self.ready = false;
        self.num_root_textures_available = 0;
        self.available_tiles_during_frame = false;

        Ok(())
    }

    // This method pushes a new downloaded tile into the buffer
    // It must be ensured that the tile is not already contained into the buffer
    // Return if tiles did become available
    fn reset_available_tiles(&mut self) -> bool {
        let available_tiles_during_frame = self.available_tiles_during_frame;
        self.available_tiles_during_frame = false;

        available_tiles_during_frame
    }

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    fn contains(&self, cell: &Self::C) -> bool {
        if let Some(t) = self.get(cell) {
            t.is_on_gpu()
        } else {
            false
        }
    }

    /// Accessors
    fn get(&self, cell: &Self::C) -> Option<&Self::T> {
        if cell.is_root() {
            let HEALPixCell(_, idx) = cell;
            Some(&self.base_textures[*idx as usize])
        } else {
            self.textures.get(cell)
        }
    }

    fn config(&self) -> &HiPSConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }
}

impl SendUniforms for HiPS2DBuffer {
    // Send only the allsky textures
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send the textures
        /*let textures = &self.base_textures;
        for (idx, texture) in textures.iter().enumerate() {
            let texture_uniforms = TextureUniforms::new(texture, idx as i32);
            shader.attach_uniforms_from(&texture_uniforms);
        }*/
        let shader = shader.attach_uniforms_from(&self.config);

        if self.allsky_rendering {
            for idx in 0..NUM_HPX_TILES_DEPTH_ZERO {
                let cell = HEALPixCell(0, idx as u64);

                let texture = self.get(&cell).unwrap();
                let texture_uniforms = HpxTexUniforms::new(texture, idx as i32);
                shader.attach_uniforms_from(&texture_uniforms);
            }

            shader
                .attach_uniform("tex", &self.allsky_pixels)
                .attach_uniform("num_slices", &12);
        } else {
            shader
                .attach_uniform("tex", &self.tile_pixels)
                .attach_uniform("num_slices", &self.tile_pixels.num_slices);
        }

        shader
    }
}

impl Drop for HiPS2DBuffer {
    fn drop(&mut self) {
        // Cleanup the heap
        self.heap.clear();

        // Cancel the tasks that have not been finished by the exec
        self.textures.clear();
    }
}
