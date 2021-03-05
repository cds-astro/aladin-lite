use crate::healpix_cell::HEALPixCell;

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
        self.partial_cmp(&other).unwrap()
    }
}

use crate::buffer::Texture;
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

use crate::{buffer::HiPSConfig, core::Texture2DArray};
use std::collections::HashMap;

use crate::time::Time;
impl From<(HEALPixCell, Time)> for TextureCellItem {
    fn from(input: (HEALPixCell, Time)) -> Self {
        let (cell, time_request) = input;
        TextureCellItem::new(cell, time_request)
    }
}
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
            // Collect to a new binary heap that do not have cell anymore
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
    //pub cutoff_values_tile: Rc<RefCell<HashMap<HEALPixCell, (f32, f32)>>>,

    // Array of 2D textures
    texture_2d_array: Rc<Texture2DArray>,

    // A boolean ensuring the root textures
    // have already been loaded
    ready: bool,

    available_tiles_during_frame: bool,

    exec: Rc<RefCell<TaskExecutor>>,
}
use crate::JsValue;
use crate::{
    async_task::{SendTileToGPU, TaskExecutor, TaskResult, TaskType},
    buffer::Image,
    WebGl2Context,
};
use web_sys::WebGl2RenderingContext;
// Define a set of textures compatible with the HEALPix tile format and size
fn create_texture_array(
    gl: &WebGl2Context,
    config: &HiPSConfig,
) -> Result<Texture2DArray, JsValue> {
    let texture_size = config.get_texture_size();
    let num_textures_by_side_slice = config.num_textures_by_side_slice();
    let num_slices = config.num_slices();
    Texture2DArray::create_empty(
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
        config.format(),
    )
}

use super::Tile;
impl ImageSurveyTextures {
    pub fn new(
        gl: &WebGl2Context,
        config: HiPSConfig,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<ImageSurveyTextures, JsValue> {
        let size = config.num_textures();
        // Ensures there is at least space for the 12
        // root textures
        assert!(size >= 12);
        let heap = HEALPixCellHeap::with_capacity(size - 12);

        let textures = HashMap::with_capacity(size);

        let texture_2d_array = Rc::new(create_texture_array(gl, &config)?);

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
            texture_2d_array,
            available_tiles_during_frame,

            ready,
            exec,
        })
    }

    // This method pushes a new downloaded tile into the buffer
    // It must be ensured that the tile is not already contained into the buffer
    pub fn push<I: Image + 'static>(
        &mut self,
        tile: Tile,
        image: I,
        time_request: Time,
        missing: bool,
    ) {
        let tile_cell = tile.cell;
        // Assert here to prevent pushing doublons
        if self.contains_tile(&tile_cell) {
            return;
        }

        // Get the texture cell in which the tile has to be
        let texture_cell = tile_cell.get_texture_cell(&self.config);
        // Check whether the texture is a new texture
        if !self.textures.contains_key(&texture_cell) {
            let HEALPixCell(_, idx) = texture_cell;
            let texture = if texture_cell.is_root() {
                let texture = Texture::new(&self.config, &texture_cell, idx as i32, time_request);

                texture
            } else {
                // The texture is not among the essential ones
                // (i.e. is not a root texture)
                let texture = if self.is_heap_full() {
                    // Pop the oldest requested texture
                    let oldest_texture = self.heap.pop().unwrap();
                    // Ensure this is not a base texture
                    assert!(!oldest_texture.is_root());

                    // Remove it from the textures HashMap
                    if let Some(mut texture) = self.textures.remove(&oldest_texture.cell) {
                        // Clear and assign it to texture_cell
                        texture.replace(
                            &texture_cell,
                            time_request,
                            &self.config,
                            &mut self.exec.borrow_mut(),
                        );

                        texture
                    } else {
                        // The hashmap must contain the texture by construction
                        unreachable!()
                    }
                } else {
                    // The heap buffer is not full, let's create a new
                    // texture with an unique idx
                    // The idx is computed based on the current size of the buffer
                    let root_texture_off_idx = 12;
                    let idx = root_texture_off_idx + self.heap.len();

                    let texture =
                        Texture::new(&self.config, &texture_cell, idx as i32, time_request);
                    texture
                };
                // Push it to the buffer
                self.heap.push(&texture);

                texture
            };
            // Insert it the texture
            self.textures.insert(texture_cell, texture);
        }

        // At this point, the texture that should contain the tile
        // is in the buffer
        // and the tile is not already in any textures of the buffer
        // We can safely push it
        // First get the texture
        if let Some(texture) = self.textures.get_mut(&texture_cell) {
            texture.append(
                &tile_cell, // The tile cell
                &self.config,
                missing,
            );

            // Append new async task responsible for writing
            // the image into the texture 2d array for the GPU
            let mut exec_ref = self.exec.borrow_mut();
            let task = SendTileToGPU::new(
                &tile,
                texture,
                image,
                self.texture_2d_array.clone(),
                &self.config,
            );
            //task.tex_sub();

            let tile = tile;
            exec_ref
                .spawner()
                .spawn(TaskType::SendTileToGPU(tile.clone()), async move {
                    task.await;

                    TaskResult::TileSentToGPU { tile }
                });
            
        } else {
            unreachable!()
        }
    }

    // Return true if at least one task has been processed
    pub fn register_available_tile(&mut self, available_tile: &Tile) {
        let Tile { cell, .. } = available_tile;
        let texture_cell = cell.get_texture_cell(&self.config);
        self.available_tiles_during_frame = true;

        if let Some(texture) = self.textures.get_mut(&texture_cell) {
            texture.register_available_tile(cell, &self.config);

            if texture_cell.is_root() && texture.is_available() {
                self.num_root_textures_available += 1;
                assert!(self.num_root_textures_available <= 12);
                //console::log_1(&format!("aass {:?}", self.num_root_textures_available).into());

                if self.num_root_textures_available == 12 {
                    self.ready = true;
                }
            }
        } else {
            // Textures written have to be in the textures collection
            unreachable!();
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
        let root_texture_off_idx = 12;
        let num_textures_heap = self.heap.len();
        let full_heap = num_textures_heap == (self.size - root_texture_off_idx);
        full_heap
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
    pub fn update_priority(&mut self, cell: &HEALPixCell, new_fov_cell: bool) {
        assert!(self.contains_tile(cell));

        // Get the texture cell in which the tile has to be
        let texture_cell = cell.get_texture_cell(&self.config);
        if texture_cell.is_root() {
            return;
        }

        if let Some(texture) = self.textures.get_mut(&texture_cell) {
            // Reset the time the tile has been received if it is a new cell present in the fov
            if new_fov_cell {
                texture.update_start_time(Time::now());
            }

            // MAYBE WE DO NOT NEED TO UPDATE THE TIME REQUEST IN THE BHEAP
            // BECAUSE IT INTRODUCES UNECESSARY CODE COMPLEXITY
            // Root textures are always in the buffer
            // But other textures can be removed thanks to the heap
            // data-structure. We have to update the time_request of the texture
            // and push it again in the heap to update its position.
            let mut tex_cell_item: TextureCellItem = texture.into();
            tex_cell_item.time_request = Time::now();

            self.heap.update_entry(tex_cell_item);
        } else {
            unreachable!();
        }
    }

    /*// This is called when the HiPS changes
    pub fn clear(&mut self) -> Result<(), JsValue> {
        // Size i.e. the num of textures is the same
        // no matter the HiPS config
        self.heap.clear();

        for texture in self.textures.values() {
            texture.clear_tasks_in_progress(&self.config, &mut self.exec.borrow_mut());
        }
        self.textures.clear();

        self.ready = false;
        self.num_root_textures_available = 0;

        let texture_2d_array = create_texture_array(&self.gl, &self.config)?;
        self.texture_2d_array = Rc::new(texture_2d_array);

        Ok(())
    }*/

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

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    // Get the textures in the buffer
    // The resulting array is uniq sorted
    fn get_allsky_textures(&self) -> [&Texture; 12] {
        assert!(self.is_ready());
        /*let mut textures = self.textures.values().collect::<Vec<_>>();
        textures.sort_unstable();
        textures*/
        [
            self.textures.get(&HEALPixCell(0, 0)).unwrap(),
            self.textures.get(&HEALPixCell(0, 1)).unwrap(),
            self.textures.get(&HEALPixCell(0, 2)).unwrap(),
            self.textures.get(&HEALPixCell(0, 3)).unwrap(),
            self.textures.get(&HEALPixCell(0, 4)).unwrap(),
            self.textures.get(&HEALPixCell(0, 5)).unwrap(),
            self.textures.get(&HEALPixCell(0, 6)).unwrap(),
            self.textures.get(&HEALPixCell(0, 7)).unwrap(),
            self.textures.get(&HEALPixCell(0, 8)).unwrap(),
            self.textures.get(&HEALPixCell(0, 9)).unwrap(),
            self.textures.get(&HEALPixCell(0, 10)).unwrap(),
            self.textures.get(&HEALPixCell(0, 11)).unwrap(),
        ]
    }

    pub fn get_texture_array(&self) -> Rc<Texture2DArray> {
        self.texture_2d_array.clone()
    }
}

use crate::buffer::TextureUniforms;
use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
impl SendUniforms for ImageSurveyTextures {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        if self.is_ready() {
            // Send the textures
            let textures = self.get_allsky_textures();
            let mut num_textures = 0;
            for texture in textures.iter() {
                if texture.is_available() {
                    let texture_uniforms = TextureUniforms::new(texture, num_textures as i32);

                    shader.attach_uniforms_from(&texture_uniforms);
                    num_textures += 1;
                }
            }
            shader.attach_uniforms_from(&self.config);
        }

        shader
    }
}

impl Drop for ImageSurveyTextures {
    fn drop(&mut self) {
        // Cleanup the heap
        self.heap.clear();

        // Cancel the tasks that have not been finished
        // by the exec
        for texture in self.textures.values() {
            texture.clear_tasks_in_progress(&self.config, &mut self.exec.borrow_mut());
        }
        self.textures.clear();
    }
}
