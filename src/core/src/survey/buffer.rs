use crate::healpix::cell::HEALPixCell;

#[derive(Clone, Debug)]
pub struct TextureCellItem {
    cell: HEALPixCell,
    time_request: Time,
}

impl TextureCellItem {
    fn new(cell: HEALPixCell, time_request: Time) -> Self {
        Self { cell, time_request }
    }
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

use std::cmp::Ordering;
// Ordering based on the time the tile has been requested
impl PartialOrd for TextureCellItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.time_request.partial_cmp(&self.time_request)
    }
}
impl Ord for TextureCellItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

use super::texture::Texture;
impl From<Texture> for TextureCellItem {
    fn from(texture: Texture) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}
impl From<&Texture> for TextureCellItem {
    fn from(texture: &Texture) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}
impl From<&mut Texture> for TextureCellItem {
    fn from(texture: &mut Texture) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self { cell, time_request }
    }
}

use super::config::HiPSConfig;
use al_core::Texture2DArray;
use std::collections::HashMap;

use crate::time::Time;
use std::collections::BinaryHeap;
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
use std::cell::RefCell;
use std::rc::Rc;
// Fixed sized binary heap
pub struct ImageSurveyTextures {
    // Some information about the HiPS
    pub config: HiPSConfig,
    heap: HEALPixCellHeap,

    num_root_textures_available: usize,

    size: usize,

    pub textures: HashMap<HEALPixCell, Texture>,
    pub base_textures: [Texture; NUM_HPX_TILES_DEPTH_ZERO],
    //pub cutoff_values_tile: Rc<RefCell<HashMap<HEALPixCell, (f32, f32)>>>,

    // Array of 2D textures
    texture_2d_array: Rc<Texture2DArray>,

    // A boolean ensuring the root textures
    // have already been loaded
    ready: bool,

    available_tiles_during_frame: bool,

    //exec: Rc<RefCell<TaskExecutor>>,
}
use crate::async_task::{ImageTile2GpuTask, TaskExecutor, TaskResult, TaskType};
use crate::math::lonlat::LonLatT;
use crate::JsValue;
use al_core::WebGlContext;

use al_core::image::format::ImageFormat;
use al_core::image::Image;
use web_sys::WebGl2RenderingContext;
// Define a set of textures compatible with the HEALPix tile format and size
fn create_texture_array<F: ImageFormat>(
    gl: &WebGlContext,
    config: &HiPSConfig,
) -> Result<Texture2DArray, JsValue> {
    let texture_size = config.get_texture_size();
    let num_textures_by_side_slice = config.num_textures_by_side_slice();
    let num_slices = config.num_slices();
    Texture2DArray::create_empty::<F>(
        gl,
        texture_size * num_textures_by_side_slice,
        texture_size * num_textures_by_side_slice,
        num_slices,
        &[
            // The HiPS tiles sampling is NEAREST
            (
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST,
            ),
            (
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST,
            ),
            // Prevents s-coordinate wrapping (repeating)
            (
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE,
            ),
            // Prevents t-coordinate wrapping (repeating)
            (
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE,
            ),
        ],
    )
}

use crate::downloader::request::tile::Tile;
use al_core::image::format::{ImageFormatType, R32F, RGB8U, RGBA8U};

#[cfg(feature = "webgl2")]
use al_core::image::format::{R16I, R32I, R8UI};

use crate::healpix::cell::NUM_HPX_TILES_DEPTH_ZERO;
use cgmath::Vector3;
impl ImageSurveyTextures {
    pub fn new(
        gl: &WebGlContext,
        config: HiPSConfig,
        //exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<ImageSurveyTextures, JsValue> {
        let size = config.num_textures();
        // Ensures there is at least space for the 12
        // root textures
        debug_assert!(size >= NUM_HPX_TILES_DEPTH_ZERO);
        let heap = HEALPixCellHeap::with_capacity(size - NUM_HPX_TILES_DEPTH_ZERO);
        let textures = HashMap::with_capacity(size);
        let base_textures = [
            Texture::new(&config, &HEALPixCell(0, 0), 0, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 1), 1, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 2), 2, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 3), 3, Time::now()),

            Texture::new(&config, &HEALPixCell(0, 4), 4, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 5), 5, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 6), 6, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 7), 7, Time::now()),

            Texture::new(&config, &HEALPixCell(0, 8), 8, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 9), 9, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 10), 10, Time::now()),
            Texture::new(&config, &HEALPixCell(0, 11), 11, Time::now()),
        ];

        #[cfg(feature = "webgl2")]
        let texture_2d_array = match config.get_format() {
            ImageFormatType::RGBA32F => unimplemented!(),
            ImageFormatType::RGB32F => unimplemented!(),
            ImageFormatType::RGBA8U => Rc::new(create_texture_array::<RGBA8U>(gl, &config)?),
            ImageFormatType::RGB8U => Rc::new(create_texture_array::<RGB8U>(gl, &config)?),
            ImageFormatType::R8UI => Rc::new(create_texture_array::<R8UI>(gl, &config)?),
            ImageFormatType::R16I => Rc::new(create_texture_array::<R16I>(gl, &config)?),
            ImageFormatType::R32I => Rc::new(create_texture_array::<R32I>(gl, &config)?),
            ImageFormatType::R32F => Rc::new(create_texture_array::<R32F>(gl, &config)?),
        };
        #[cfg(feature = "webgl1")]
        let texture_2d_array = match config.get_format() {
            ImageFormatType::RGBA32F => unimplemented!(),
            ImageFormatType::RGB32F => unimplemented!(),
            ImageFormatType::RGBA8U => Rc::new(create_texture_array::<RGBA8U>(gl, &config)?),
            ImageFormatType::RGB8U => Rc::new(create_texture_array::<RGB8U>(gl, &config)?),
            ImageFormatType::R32F => Rc::new(create_texture_array::<R32F>(gl, &config)?),
        };
        // The root textures have not been loaded
        let ready = false;
        let num_root_textures_available = 0;
        let available_tiles_during_frame = false;
        Ok(ImageSurveyTextures {
            config,
            heap,

            size,
            num_root_textures_available,

            textures,
            base_textures,

            texture_2d_array,
            available_tiles_during_frame,

            ready,
            //exec,
        })
    }

    // This method pushes a new downloaded tile into the buffer
    // It must be ensured that the tile is not already contained into the buffer
    pub fn push<I: Image + std::fmt::Debug>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
        missing: bool,
    ) {
        // Assert here to prevent pushing doublons
        if self.contains_tile(cell) {
            return;
        }

        // Get the texture cell in which the tile has to be
        let tex_cell = cell.get_texture_cell(&self.config);
        // Check whether the texture is a new texture
        /*if texture_cell.is_root() {
            if !self.base_textures.contains_key(&texture_cell) {
                let HEALPixCell(_, idx) = texture_cell;
                self.base_textures[idx] = Texture::new(&self.config, &texture_cell, idx as i32, time_request);
            }
        } else {
            // The texture is not among the essential ones
            // (i.e. is not a root texture)
            let texture = if self.is_heap_full() {
                // Pop the oldest requested texture
                let oldest_texture = self.heap.pop().unwrap();
                // Ensure this is not a base texture
                debug_assert!(!oldest_texture.is_root());

                // Remove it from the textures HashMap
                let mut texture = self.textures.remove(&oldest_texture.cell)
                    .expect("Texture (oldest one) has not been found in the buffer of textures");
                // Clear and assign it to texture_cell
                texture.replace(
                    &texture_cell,
                    time_request,
                    &self.config,
                    &mut self.exec.borrow_mut(),
                );

                texture
            } else {
                // The heap buffer is not full, let's create a new
                // texture with an unique idx
                // The idx is computed based on the current size of the buffer
                let idx = NUM_HPX_TILES_DEPTH_ZERO + self.heap.len();

                Texture::new(&self.config, &texture_cell, idx as i32, time_request)
            };
            // Push it to the buffer
            self.heap.push(&texture);
            
            // Insert it the texture
            self.textures.insert(texture_cell, texture);
        }*/

        if !self.textures.contains_key(&tex_cell) {
            let HEALPixCell(_, idx) = tex_cell;
            let texture = if tex_cell.is_root() {
                Texture::new(&self.config, &tex_cell, idx as i32, time_request)
            } else {
                // The texture is not among the essential ones
                // (i.e. is not a root texture)
                let texture = if self.is_heap_full() {
                    // Pop the oldest requested texture
                    let oldest_texture = self.heap.pop().unwrap();
                    // Ensure this is not a base texture
                    debug_assert!(!oldest_texture.is_root());

                    // Remove it from the textures HashMap
                    let mut texture = self.textures.remove(&oldest_texture.cell)
                        .expect("Texture (oldest one) has not been found in the buffer of textures");
                    // Clear and assign it to tex_cell
                    texture.replace(
                        &tex_cell,
                        time_request,
                        &self.config,
                        //&mut self.exec.borrow_mut(),
                    );

                    texture
                } else {
                    // The heap buffer is not full, let's create a new
                    // texture with an unique idx
                    // The idx is computed based on the current size of the buffer
                    let idx = NUM_HPX_TILES_DEPTH_ZERO + self.heap.len();

                    Texture::new(&self.config, &tex_cell, idx as i32, time_request)
                };
                // Push it to the buffer
                self.heap.push(&texture);

                texture
            };
            // Insert it the texture
            self.textures.insert(tex_cell, texture);
        }

        // At this point, the texture that should contain the tile
        // is in the buffer
        // and the tile is not already in any textures of the buffer
        // We can safely push it
        // First get the texture
        let texture = self.textures.get_mut(&tex_cell).expect("the cell has to be in the tile buffer");
        /*
        // Append new async task responsible for writing
        // the image into the texture 2d array for the GPU
        let mut exec_ref = self.exec.borrow_mut();
        let task = ImageTile2GpuTask::<I>::new(
            &tile,
            texture,
            image,
            self.texture_2d_array.clone(),
            &self.config,
        );

        let tile = tile;
        exec_ref
            .spawner()
            .spawn(TaskType::ImageTile2GpuTask(tile.clone()), async move {
                task.await;

                TaskResult::TileSentToGPU { tile }
            });
        */

        // Direct sub
        /*let task = ImageTile2GpuTask::<I>::new(
            tile,
            texture,
            image,
            self.texture_2d_array.clone(),
            &self.config,
        );

        task.tex_sub();*/
        if missing {
            send_to_gpu(
                cell,
                texture,
                self.config.get_default_image(),
                self.texture_2d_array.clone(),
                &self.config,
            );
        } else {
            send_to_gpu(
                cell,
                texture,
                image,
                self.texture_2d_array.clone(),
                &self.config,
            );
        };



        //al_core::log(&format!("{:?}", tex_cell));
        // Once the texture has been received in the GPU
        texture.append(
            cell, // The tile cell
            &self.config,
            missing,
        );

        let tex_cell = cell.get_texture_cell(&self.config);
        self.available_tiles_during_frame = true;

        //texture.register_available_tile(cell, &self.config);

        if tex_cell.is_root() && texture.is_available() {
            self.num_root_textures_available += 1;
            debug_assert!(self.num_root_textures_available <= NUM_HPX_TILES_DEPTH_ZERO);
            //console::log_1(&format!("aass {:?}", self.num_root_textures_available).into());

            if self.num_root_textures_available == NUM_HPX_TILES_DEPTH_ZERO {
                self.ready = true;
            }
        }
    }

    pub fn is_there_available_tiles(&mut self) -> bool {
        let available_tiles_during_frame = self.available_tiles_during_frame;
        self.available_tiles_during_frame = false;

        available_tiles_during_frame
    }

    fn is_heap_full(&self) -> bool {
        // Check that there are no more than num_textures
        // textures in the buffer
        let num_textures_heap = self.heap.len();
        
        num_textures_heap == (self.size - NUM_HPX_TILES_DEPTH_ZERO)
    }

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    pub fn contains(&self, texture_cell: &HEALPixCell) -> bool {
        if let Some(texture) = self.textures.get(texture_cell) {
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

    // Check whether the buffer has a tile
    // For that purpose, we first need to verify that its
    // texture ancestor exists and then, it it contains the tile
    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        let texture_cell = cell.get_texture_cell(&self.config);
        if let Some(texture) = self.textures.get(&texture_cell) {
            // The texture is present in the buffer
            // We must check whether it contains the tile
            texture.contains(cell)
        } else {
            // The texture in which cell should be is not present
            false
        }
    }

    // Update the priority of the texture containing the tile
    // It must be ensured that the tile is already contained in the buffer
    pub fn update_priority(&mut self, cell: &HEALPixCell/*, new_fov_cell: bool*/) {
        debug_assert!(self.contains_tile(cell));

        // Get the texture cell in which the tile has to be
        let texture_cell = cell.get_texture_cell(&self.config);
        if texture_cell.is_root() {
            return;
        }

        let texture = self.textures.get_mut(&texture_cell)
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

    pub fn get_pixel_position_in_texture(
        &self,
        lonlat: &LonLatT<f64>,
        depth: u8,
    ) -> Result<Vector3<i32>, JsValue> {
        let (pix, dx, dy) = healpix::nested::hash_with_dxdy(depth, lonlat.lon().0, lonlat.lat().0);

        let cell = HEALPixCell(depth, pix);
        // Index of the texture in the total set of textures
        if let Some(texture) = self.textures.get(&cell) {
            let texture_idx = texture.idx();
            // Index of the slice of textures
            let idx_slice = texture_idx / self.config.num_textures_by_slice();
            // Index of the texture in its slice
            let idx_in_slice = texture_idx % self.config.num_textures_by_slice();

            // Index of the column of the texture in its slice
            let idx_col_in_slice = idx_in_slice / self.config.num_textures_by_side_slice();
            // Index of the row of the texture in its slice
            let idx_row_in_slice = idx_in_slice % self.config.num_textures_by_side_slice();

            // Row and column indexes of the tile in its texture
            let (idx_col_in_tex, idx_row_in_tex) = cell.get_offset_in_texture_cell(&self.config);

            // The size of the global texture containing the tiles
            let texture_size = self.config.get_texture_size();
            // The size of a tile in its texture
            let tile_size = self.config.get_tile_size();

            // Offset in the slice in pixels
            let mut offset = Vector3::new(
                (idx_row_in_slice as i32) * texture_size
                    + (idx_row_in_tex as i32) * tile_size
                    + ((dy * (tile_size as f64)) as i32),
                (idx_col_in_slice as i32) * texture_size
                    + (idx_col_in_tex as i32) * tile_size
                    + ((dx * (tile_size as f64)) as i32),
                idx_slice,
            );

            if self.config.tex_storing_fits {
                let mut uvy = offset.y as f32 / 4096.0;
                uvy = self.config.size_tile_uv
                    + 2.0 * self.config.size_tile_uv * (uvy / self.config.size_tile_uv).floor()
                    - uvy;

                offset.y = (uvy * 4096.0) as i32;
            }

            Ok(offset)
        } else {
            Err(JsValue::from_str(&format!(
                "{:?} not loaded in the GPU, please wait before trying again.",
                cell
            )))
        }
    }

    /// Accessors
    pub fn get(&self, texture_cell: &HEALPixCell) -> Option<&Texture> {
        self.textures.get(texture_cell)
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

    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }


    pub fn is_ready(&self) -> bool {
        self.ready
    }

    // Get the textures in the buffer
    // The resulting array is uniq sorted
    fn get_allsky_textures(&self) -> [Option<&Texture>; NUM_HPX_TILES_DEPTH_ZERO] {
        //debug_assert!(self.is_ready());
        /*let mut textures = self.textures.values().collect::<Vec<_>>();
        textures.sort_unstable();
        textures*/
        [
            self.textures.get(&HEALPixCell(0, 0)),
            self.textures.get(&HEALPixCell(0, 1)),
            self.textures.get(&HEALPixCell(0, 2)),
            self.textures.get(&HEALPixCell(0, 3)),
            self.textures.get(&HEALPixCell(0, 4)),
            self.textures.get(&HEALPixCell(0, 5)),
            self.textures.get(&HEALPixCell(0, 6)),
            self.textures.get(&HEALPixCell(0, 7)),
            self.textures.get(&HEALPixCell(0, 8)),
            self.textures.get(&HEALPixCell(0, 9)),
            self.textures.get(&HEALPixCell(0, 10)),
            self.textures.get(&HEALPixCell(0, 11)),
        ]
    }

    // Get the textures in the buffer
    // The resulting array is uniq sorted
    fn get_textures(&self) -> Vec<&Texture> {
        debug_assert!(self.is_ready());
        let mut textures = self.textures.values().collect::<Vec<_>>();
        textures.sort_unstable();
        textures
    }

    pub fn get_texture_array(&self) -> Rc<Texture2DArray> {
        self.texture_2d_array.clone()
    }
}

fn send_to_gpu<I: Image>(
    cell: &HEALPixCell,
    texture: &Texture,
    image: I,
    texture_array: Rc<Texture2DArray>,
    cfg: &HiPSConfig,
) {
    // Index of the texture in the total set of textures
    let texture_idx = texture.idx();
    // Index of the slice of textures
    let num_textures_by_slice = cfg.num_textures_by_slice();
    let idx_slice = texture_idx / num_textures_by_slice;
    // Index of the texture in its slice
    let idx_in_slice = texture_idx % num_textures_by_slice;

    // Index of the column of the texture in its slice
    let num_textures_by_side_slice = cfg.num_textures_by_side_slice();
    let idx_col_in_slice = idx_in_slice / num_textures_by_side_slice;
    // Index of the row of the texture in its slice
    let idx_row_in_slice = idx_in_slice % num_textures_by_side_slice;

    // Row and column indexes of the tile in its texture
    let (idx_col_in_tex, idx_row_in_tex) = cell.get_offset_in_texture_cell(cfg);

    // The size of the global texture containing the tiles
    let texture_size = cfg.get_texture_size();
    // The size of a tile in its texture
    let tile_size = cfg.get_tile_size();

    // Offset in the slice in pixels
    let offset = Vector3::new(
        (idx_row_in_slice as i32) * texture_size + (idx_row_in_tex as i32) * tile_size,
        (idx_col_in_slice as i32) * texture_size + (idx_col_in_tex as i32) * tile_size,
        idx_slice,
    );

    image.tex_sub_image_3d(&texture_array, &offset);
}
use super::texture::TextureUniforms;

use al_core::shader::{SendUniforms, ShaderBound};
impl SendUniforms for ImageSurveyTextures {
    /*
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        if self.is_ready() {
            // Send the textures
            let textures = self.get_textures();
            let mut num_textures = 0;
            for texture in textures.iter() {
                if texture.is_available() {
                    let texture_uniforms = TextureUniforms::new(texture, num_textures as i32);

                    shader.attach_uniforms_from(&texture_uniforms);
                    num_textures += 1;
                }
            }
            let num_tiles = textures.len() as i32;
            shader
                .attach_uniform("num_tiles", &num_tiles)
                .attach_uniforms_from(&self.config)
                .attach_uniforms_from(&*self.texture_2d_array);
        }

        shader
    }
    */
    // Send only the allsky textures
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        //if self.is_ready() {
            // Send the textures
            let textures = self.get_allsky_textures();
            for (idx, texture) in textures.iter().enumerate() {
                if let Some(texture) = texture {
                    let texture_uniforms = TextureUniforms::new(texture, idx as i32);
                    shader.attach_uniforms_from(&texture_uniforms);
                }
            }
            let num_tiles = textures.len() as i32;
            shader
                .attach_uniform("num_tiles", &num_tiles)
                .attach_uniforms_from(&self.config)
                .attach_uniforms_from(&*self.texture_2d_array);
        //}

        shader
    }
}

impl Drop for ImageSurveyTextures {
    fn drop(&mut self) {
        //al_core::log(&format!("Drop image surveys"));
        // Cleanup the heap
        self.heap.clear();

        // Cancel the tasks that have not been finished
        // by the exec
        /*for texture in self.textures.values() {
            texture.clear_tasks_in_progress(&self.config, &mut self.exec.borrow_mut());
        }*/
        self.textures.clear();
    }
}