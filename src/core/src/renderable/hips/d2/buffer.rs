use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use al_core::image::format::ChannelType;

use crate::renderable::hips::HpxTile;
use cgmath::Vector3;

use al_api::hips::ImageExt;
use al_core::webgl_ctx::WebGlRenderingCtx;

use crate::math::lonlat::LonLat;
use crate::CameraViewPort;
use crate::LonLatT;
use al_core::image::format::ImageFormat;
use al_core::image::format::{R16I, R32F, R32I, R64F, R8UI, RGB8U, RGBA8U};
use al_core::image::Image;
use al_core::shader::{SendUniforms, ShaderBound};
use al_core::Texture2DArray;
use al_core::WebGlContext;

use super::texture::{HpxTexture2D, HpxTexture2DUniforms};

use crate::downloader::request::allsky::Allsky;
use crate::healpix::cell::HEALPixCell;
use crate::healpix::cell::NUM_HPX_TILES_DEPTH_ZERO;
use crate::renderable::hips::config::HiPSConfig;
use crate::time::Time;
use crate::Abort;
use crate::JsValue;

#[derive(Clone, Debug)]
pub struct TextureCellItem {
    cell: HEALPixCell,
    time_request: Time,
}

impl TextureCellItem {
    fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}

impl PartialEq for TextureCellItem {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}
impl Eq for TextureCellItem {}

// Ordering based on the time the tile has been requested
impl PartialOrd for TextureCellItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.time_request.partial_cmp(&self.time_request)
    }
}
impl Ord for TextureCellItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

impl From<HpxTexture2D> for TextureCellItem {
    fn from(texture: HpxTexture2D) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}
impl From<&HpxTexture2D> for TextureCellItem {
    fn from(texture: &HpxTexture2D) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}
impl From<&mut HpxTexture2D> for TextureCellItem {
    fn from(texture: &mut HpxTexture2D) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}

struct HEALPixCellHeap(BinaryHeap<TextureCellItem>);

impl HEALPixCellHeap {
    fn with_capacity(cap: usize) -> Self {
        Self(BinaryHeap::with_capacity(cap))
    }

    fn push<E: Into<TextureCellItem>>(&mut self, item: E) {
        let item = item.into();
        self.0.push(item);
    }

    fn update_entry<E: Into<TextureCellItem>>(&mut self, item: E) {
        let item = item.into();
        self.0 = self
            .0
            .drain()
            // Remove the cell
            .filter(|texture_node| texture_node.cell != item.cell)
            // Collect to a new binary heap that does not have cell anymore
            .collect::<BinaryHeap<_>>();

        self.push(item);
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn pop(&mut self) -> Option<TextureCellItem> {
        self.0.pop()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

use crate::renderable::hips::HpxTileBuffer;
// Fixed sized binary heap
pub struct HiPS2DBuffer {
    // Some information about the HiPS
    config: HiPSConfig,
    heap: HEALPixCellHeap,

    num_root_textures_available: u8,
    size: usize,

    textures: HashMap<HEALPixCell, HpxTexture2D>,
    base_textures: [HpxTexture2D; NUM_HPX_TILES_DEPTH_ZERO],

    // Array of 2D textures
    texture_2d_array: Texture2DArray,

    available_tiles_during_frame: bool,
}

// Define a set of textures compatible with the HEALPix tile format and size
fn create_texture_array<F: ImageFormat>(
    gl: &WebGlContext,
    config: &HiPSConfig,
) -> Result<Texture2DArray, JsValue> {
    let texture_size = config.get_texture_size();
    Texture2DArray::create_empty::<F>(
        gl,
        texture_size,
        texture_size,
        // 256 is a consensus for targetting the maximum GPU architectures. We create a 128 slices to optimize performance
        128,
        &[
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
            (
                WebGlRenderingCtx::TEXTURE_WRAP_R,
                WebGlRenderingCtx::CLAMP_TO_EDGE,
            ),
        ],
    )
}

impl HiPS2DBuffer {
    pub fn push_allsky(&mut self, allsky: Allsky) -> Result<(), JsValue> {
        let Allsky {
            image,
            time_req,
            depth_tile,
            ..
        } = allsky;

        {
            let mutex_locked = image.borrow();
            let images = mutex_locked.as_ref().unwrap_abort();
            for (idx, image) in images.iter().enumerate() {
                self.push(&HEALPixCell(depth_tile, idx as u64), image, time_req)?;
            }
        }

        Ok(())
    }

    // Check whether the buffer has a tile
    // For that purpose, we first need to verify that its
    // texture ancestor exists and then, it it contains the tile
    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        let dd = self.config.delta_depth();

        let texture_cell = cell.get_texture_cell(dd);

        let tex_cell_is_root = texture_cell.is_root();
        if tex_cell_is_root {
            let HEALPixCell(_, idx) = texture_cell;
            self.base_textures[idx as usize].contains_tile(cell)
        } else {
            if let Some(texture) = self.get(&texture_cell) {
                // The texture is present in the buffer
                // We must check whether it contains the tile
                texture.contains_tile(cell)
            } else {
                // The texture in which cell should be is not present
                false
            }
        }
    }

    fn is_heap_full(&self) -> bool {
        // Check that there are no more than num_textures
        // textures in the buffer
        let num_textures_heap = self.heap.len();

        num_textures_heap == self.size
    }

    // Update the priority of the texture containing the tile
    // It must be ensured that the tile is already contained in the buffer
    pub fn update_priority(&mut self, cell: &HEALPixCell /*, new_fov_cell: bool*/) {
        debug_assert!(self.contains_tile(cell));

        let dd = self.config.delta_depth();

        // Get the texture cell in which the tile has to be
        let texture_cell = cell.get_texture_cell(dd);
        if texture_cell.is_root() {
            return;
        }

        let texture = self
            .textures
            .get_mut(&texture_cell)
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
        let mut tex_cell_item: TextureCellItem = texture.into();
        tex_cell_item.time_request = Time::now();

        self.heap.update_entry(tex_cell_item);
    }

    pub fn push<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        if !self.contains_tile(cell) {
            let dd = self.config.delta_depth();
            // Get the texture cell in which the tile has to be
            let tex_cell = cell.get_texture_cell(dd);

            let tex_cell_is_root = tex_cell.is_root();
            if !tex_cell_is_root && !self.textures.contains_key(&tex_cell) {
                // The texture is not among the essential ones
                // (i.e. is not a root texture)
                let texture = if self.is_heap_full() {
                    // Pop the oldest requested texture
                    let oldest_texture = self.heap.pop().unwrap_abort();
                    // Ensure this is not a base texture
                    debug_assert!(!oldest_texture.is_root());

                    // Remove it from the textures HashMap
                    let mut texture = self.textures.remove(&oldest_texture.cell).expect(
                        "Texture (oldest one) has not been found in the buffer of textures",
                    );
                    texture.replace(&tex_cell, time_request);

                    texture
                } else {
                    let idx = NUM_HPX_TILES_DEPTH_ZERO + self.heap.len();

                    HpxTexture2D::new(&tex_cell, idx as i32, time_request)
                };

                // Push it to the buffer
                self.heap.push(&texture);

                self.textures.insert(tex_cell, texture);
            }

            if tex_cell_is_root {
                self.num_root_textures_available += 1;
                if self.num_root_textures_available == 12 {
                    self.texture_2d_array.generate_mipmap()
                }
            }

            // At this point, the texture that should contain the tile
            // is in the buffer
            // and the tile is not already in any textures of the buffer
            // We can safely push it
            // First get the texture

            let texture = if !tex_cell_is_root {
                self.textures
                    .get_mut(&tex_cell)
                    .expect("the cell has to be in the tile buffer")
            } else {
                let HEALPixCell(_, idx) = tex_cell;
                &mut self.base_textures[idx as usize]
            };

            send_to_gpu(
                cell,
                texture,
                image,
                &self.texture_2d_array,
                &mut self.config,
            )?;

            texture.append(
                cell, // The tile cell
                &self.config,
            );

            self.available_tiles_during_frame = true;
        }

        Ok(())
    }
}

impl HpxTileBuffer for HiPS2DBuffer {
    type T = HpxTexture2D;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let size = 128 - NUM_HPX_TILES_DEPTH_ZERO;
        // Ensures there is at least space for the 12
        // root textures
        //debug_assert!(size >= NUM_HPX_TILES_DEPTH_ZERO);
        let heap = HEALPixCellHeap::with_capacity(size);
        let textures = HashMap::with_capacity(size);

        let now = Time::now();
        let base_textures = [
            HpxTexture2D::new(&HEALPixCell(0, 0), 0, now),
            HpxTexture2D::new(&HEALPixCell(0, 1), 1, now),
            HpxTexture2D::new(&HEALPixCell(0, 2), 2, now),
            HpxTexture2D::new(&HEALPixCell(0, 3), 3, now),
            HpxTexture2D::new(&HEALPixCell(0, 4), 4, now),
            HpxTexture2D::new(&HEALPixCell(0, 5), 5, now),
            HpxTexture2D::new(&HEALPixCell(0, 6), 6, now),
            HpxTexture2D::new(&HEALPixCell(0, 7), 7, now),
            HpxTexture2D::new(&HEALPixCell(0, 8), 8, now),
            HpxTexture2D::new(&HEALPixCell(0, 9), 9, now),
            HpxTexture2D::new(&HEALPixCell(0, 10), 10, now),
            HpxTexture2D::new(&HEALPixCell(0, 11), 11, now),
        ];
        let channel = config.get_format().get_channel();

        let texture_2d_array = match channel {
            ChannelType::RGBA32F => unimplemented!(),
            ChannelType::RGB32F => unimplemented!(),
            ChannelType::RGBA8U => create_texture_array::<RGBA8U>(gl, &config)?,
            ChannelType::RGB8U => create_texture_array::<RGB8U>(gl, &config)?,
            ChannelType::R32F => create_texture_array::<R32F>(gl, &config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R8UI => create_texture_array::<R8UI>(gl, &config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R16I => create_texture_array::<R16I>(gl, &config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R32I => create_texture_array::<R32I>(gl, &config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R64F => create_texture_array::<R64F>(gl, &config)?,
        };
        // The root textures have not been loaded

        let num_root_textures_available = 0;
        let available_tiles_during_frame = false;

        Ok(HiPS2DBuffer {
            config,
            heap,

            size,
            num_root_textures_available,
            textures,
            base_textures,
            texture_2d_array,
            available_tiles_during_frame,
        })
    }

    fn set_image_ext(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue> {
        self.config.set_image_ext(ext)?;

        let channel = self.config.get_format().get_channel();

        self.texture_2d_array = match channel {
            ChannelType::RGBA32F => unimplemented!(),
            ChannelType::RGB32F => unimplemented!(),
            ChannelType::RGBA8U => create_texture_array::<RGBA8U>(gl, &self.config)?,
            ChannelType::RGB8U => create_texture_array::<RGB8U>(gl, &self.config)?,
            ChannelType::R32F => create_texture_array::<R32F>(gl, &self.config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R8UI => create_texture_array::<R8UI>(gl, &self.config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R16I => create_texture_array::<R16I>(gl, &self.config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R32I => create_texture_array::<R32I>(gl, &self.config)?,
            #[cfg(feature = "webgl2")]
            ChannelType::R64F => create_texture_array::<R64F>(gl, &self.config)?,
        };

        let now = Time::now();
        self.base_textures = [
            HpxTexture2D::new(&HEALPixCell(0, 0), 0, now),
            HpxTexture2D::new(&HEALPixCell(0, 1), 1, now),
            HpxTexture2D::new(&HEALPixCell(0, 2), 2, now),
            HpxTexture2D::new(&HEALPixCell(0, 3), 3, now),
            HpxTexture2D::new(&HEALPixCell(0, 4), 4, now),
            HpxTexture2D::new(&HEALPixCell(0, 5), 5, now),
            HpxTexture2D::new(&HEALPixCell(0, 6), 6, now),
            HpxTexture2D::new(&HEALPixCell(0, 7), 7, now),
            HpxTexture2D::new(&HEALPixCell(0, 8), 8, now),
            HpxTexture2D::new(&HEALPixCell(0, 9), 9, now),
            HpxTexture2D::new(&HEALPixCell(0, 10), 10, now),
            HpxTexture2D::new(&HEALPixCell(0, 11), 11, now),
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
    fn contains(&self, cell: &HEALPixCell) -> bool {
        if let Some(t) = self.get(cell) {
            t.is_full()
        } else {
            false
        }
    }

    /// Accessors
    fn get(&self, cell: &HEALPixCell) -> Option<&Self::T> {
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

    fn read_pixel(&self, pos: &LonLatT<f64>, camera: &CameraViewPort) -> Result<JsValue, JsValue> {
        // 1. Convert it to the hips frame system
        let cfg = self.config();
        let camera_frame = camera.get_coo_system();
        let hips_frame = cfg.get_frame();

        let pos: LonLatT<f64> =
            crate::coosys::apply_coo_system(camera_frame, hips_frame, &pos.vector()).lonlat();

        // Get the array of textures from that survey
        let depth = camera.get_texture_depth().min(cfg.get_max_depth_texture());

        // compute the tex
        let (pix, dx, dy) = crate::healpix::utils::hash_with_dxdy(depth, &pos);
        let texture_cell = HEALPixCell(depth, pix);

        if let Some(texture) = self.get(&texture_cell) {
            let cfg = self.config();

            // Index of the texture in the total set of textures
            let texture_idx = texture.idx();

            // The size of the global texture containing the tiles
            let texture_size = cfg.get_texture_size();

            // Offset in the slice in pixels
            let mut pos_tex = Vector3::new(
                (dy * (texture_size as f64)) as i32,
                (dx * (texture_size as f64)) as i32,
                texture_idx,
            );

            // Offset in the slice in pixels
            if cfg.tex_storing_fits {
                let texture_size = cfg.get_texture_size() as f32;
                let mut uvy = pos_tex.y as f32 / texture_size;
                uvy = cfg.size_tile_uv + 2.0 * cfg.size_tile_uv * (uvy / cfg.size_tile_uv).floor()
                    - uvy;

                pos_tex.y = (uvy * texture_size) as i32;
            }

            let mut value = self
                .texture_2d_array
                .read_pixel(pos_tex.x, pos_tex.y, pos_tex.z)?;

            if cfg.tex_storing_fits {
                // scale the value
                let f64_v = value
                    .as_f64()
                    .ok_or_else(|| JsValue::from_str("Error unwraping the pixel read value."))?;
                let scale = cfg.scale as f64;
                let offset = cfg.offset as f64;

                value = JsValue::from_f64(f64_v * scale + offset);
            }

            Ok(value)
        } else {
            Err(JsValue::from_str(&format!(
                "{:?} not loaded in the GPU, please wait before trying again.",
                texture_cell
            )))
        }
    }
}

fn send_to_gpu<I: Image>(
    cell: &HEALPixCell,
    texture: &HpxTexture2D,
    image: I,
    texture_array: &Texture2DArray,
    cfg: &mut HiPSConfig,
) -> Result<(), JsValue> {
    // Index of the texture in the total set of textures
    let texture_idx = texture.idx();
    // Index of the slice of textures
    let idx_slice = texture_idx;
    // Row and column indexes of the tile in its texture
    let delta_depth = cfg.delta_depth();
    let (idx_col_in_tex, idx_row_in_tex) = cell.get_offset_in_texture_cell(delta_depth);

    // The size of the global texture containing the tiles
    let texture_size = cfg.get_texture_size();
    // The size of a tile in its texture
    let tile_size = cfg.get_tile_size();

    // Offset in the slice in pixels
    let offset = Vector3::new(
        (idx_row_in_tex as i32) * tile_size,
        (idx_col_in_tex as i32) * tile_size,
        idx_slice,
    );

    image.insert_into_3d_texture(texture_array, &offset)?;

    Ok(())
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

        //if self.raytracing {
        for idx in 0..NUM_HPX_TILES_DEPTH_ZERO {
            let cell = HEALPixCell(0, idx as u64);

            let texture = self.get(&cell).unwrap();
            let texture_uniforms = HpxTexture2DUniforms::new(texture, idx as i32);
            shader.attach_uniforms_from(&texture_uniforms);
        }
        //}

        let shader = shader
            .attach_uniforms_from(&self.config)
            .attach_uniform("tex", &self.texture_2d_array)
            .attach_uniform("num_slices", &(self.texture_2d_array.num_slices as i32));

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
