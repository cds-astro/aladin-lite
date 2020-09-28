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

        Self {
            cell,
            time_request,
        }
    }
}
impl From<&Texture> for TextureCellItem {
    fn from(texture: &Texture) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self {
            cell,
            time_request,
        }
    }
}
impl From<&mut Texture> for TextureCellItem {
    fn from(texture: &mut Texture) -> Self {
        let time_request = texture.time_request();
        let cell = *texture.cell();

        Self {
            cell,
            time_request,
        }
    }
}


use std::collections::HashMap;
use crate::{
    core::Texture2DArray,
    buffer::HiPSConfig
};

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
        self.0 = self.0.drain()
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
            .map(|res| res.into())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
use std::cell::RefCell;
use std::rc::Rc;
// Fixed sized binary heap
pub struct ImageSurvey {
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
    ready: bool
}
use crate::{
    WebGl2Context,
    buffer::Image,
    async_task::{AladinTaskExecutor, TaskType, TaskResult, SendTileToGPU}
};
use std::collections::HashSet;
use web_sys::WebGl2RenderingContext;
// Define a set of textures compatible with the HEALPix tile format and size
fn create_texture_array(gl: &WebGl2Context, config: &HiPSConfig) -> Texture2DArray {
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
            (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::NEAREST),
            (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST),
            
            // Prevents s-coordinate wrapping (repeating)
            (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE),
            // Prevents t-coordinate wrapping (repeating)
            (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE),
        ],
        config.format(),
    )
}

use std::cell::Cell;
use crate::image_fmt::FormatImageType;
impl ImageSurvey {
    pub fn new(gl: &WebGl2Context, config: HiPSConfig) -> ImageSurvey {
        let size = config.num_textures();
        // Ensures there is at least space for the 12
        // root textures
        assert!(size >= 12);
        let heap = HEALPixCellHeap::with_capacity(size - 12);

        let textures = HashMap::with_capacity(size);
        
        let texture_2d_array = Rc::new(create_texture_array(gl, &config));

        // The root textures have not been loaded
        let ready = false;
        let num_root_textures_available = 0;
        //let cutoff_values_tile = Rc::new(RefCell::new(HashMap::new()));
        // Push the 
        ImageSurvey {
            config,
            heap,

            size,
            num_root_textures_available,

            textures,
            //cutoff_values_tile,
            texture_2d_array,

            ready,
        }
    }

    pub fn get_image_format(&self) -> &FormatImageType {
        &self.config.format()
    }

    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }

    pub fn get_root_url(&self) -> &str {
        &self.config.root_url
    }

    #[inline]
    pub fn get_blank_tile(&self) -> Rc<TileArrayBufferImage> {
        self.config.get_blank_tile()
    }

    /*pub fn get_cutoff(&self, tile_cell: &HEALPixCell) -> Option<(f32, f32)> {
        self.cutoff_values_tile.borrow().get(tile_cell).cloned()
    }*/

    // This method pushes a new downloaded tile into the buffer
    // It must be ensured that the tile is not already contained into the buffer
    pub fn push<I: Image + 'static>(&mut self, tile: &Tile, image: I, time_request: Time, exec: &mut AladinTaskExecutor) {
        let tile_cell = tile.cell;
        // Assert here to prevent pushing doublons
        assert!(!self.contains_tile(tile_cell));

        // Get the texture cell in which the tile has to be
        let texture_cell = tile_cell.get_texture_cell(&self.config);
        let mut old_texture_cell = None;
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
                    let oldest_texture = self.heap.pop()
                        .unwrap();
                    // Ensure this is not a base texture
                    assert!(!oldest_texture.is_root());

                    // Remove it from the textures HashMap
                    if let Some(mut texture) = self.textures.remove(&oldest_texture.cell) {
                        // Clear and assign it to texture_cell
                        texture.replace(&texture_cell, time_request, &self.config, exec);
                        old_texture_cell = Some(oldest_texture.cell);

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

                    let texture = Texture::new(&self.config, &texture_cell, idx as i32, time_request);
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
                tile_cell, // The tile cell
                &self.config
            );
            // Compute the cutoff of the received tile
            //let cutoff = image.get_cutoff_values();

            // Append new async task responsible for writing
            // the image into the texture 2d array for the GPU
            let spawner = exec.spawner();
            let task = SendTileToGPU::new(tile_cell, texture, image, self.texture_2d_array.clone(), &self.config);
            //let cutoff_values_tile = self.cutoff_values_tile.clone();
            let tile = *tile;
            spawner.spawn(TaskType::SendTileToGPU(tile_cell), async move {
                task.await;

                /*if let Some(cutoff) = cutoff {
                    // Remove the cutoff values of the image
                    if let Some(oldest_tex_cell) = old_texture_cell {
                        cutoff_values_tile.borrow_mut().remove(&oldest_tex_cell);
                    }

                    cutoff_values_tile.borrow_mut().insert(tile_cell, cutoff);
                }*/

                TaskResult::TileSentToGPU { tile }
            });
        } else {
            unreachable!()
        }
    }

    // Return true if at least one task has been processed
    pub fn register_available_tiles(&mut self, available_tile: &Tile) {
        let Tile {cell, ..} = available_tile;
        let texture_cell = cell.get_texture_cell(&self.config);

        if let Some(texture) = self.textures.get_mut(&texture_cell) {
            texture.register_available_tile(cell, &self.config);

            if texture_cell.is_root() && texture.is_available() {
                self.num_root_textures_available += 1;
                assert!(self.num_root_textures_available <= 12);
                //console::log_1(&format!("aass {:?}", self.num_root_textures_available).into());

                if self.num_root_textures_available == 12 {
                    self.ready = true;
                    crate::log("READYYYY");
                }
            }
        } else {
            // Textures written have to be in the textures collection
            unreachable!();
        }
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

    // This is called when the HiPS changes
    pub fn clear(&mut self, gl: &WebGl2Context, task_executor: &mut AladinTaskExecutor) {
        // Size i.e. the num of textures is the same
        // no matter the HiPS config
        self.heap.clear();

        for texture in self.textures.values() {
            texture.clear_tasks_in_progress(&self.config, task_executor);
        }
        self.textures.clear();

        self.ready = false;
        self.num_root_textures_available = 0;

        self.texture_2d_array = Rc::new(create_texture_array(gl, &self.config));
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    // Get the textures in the buffer
    // The resulting array is uniq sorted
    fn get_sorted_textures(&self) -> Vec<&Texture> {
        let mut textures = self.textures.values().collect::<Vec<_>>();
        textures.sort_unstable();
        textures
    }
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;
use crate::buffer::TextureUniforms;
impl HasUniforms for ImageSurvey {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // Send the textures
        let textures = self.get_sorted_textures();
        let mut num_textures = 0;
        for texture in textures {
            if texture.is_available() {
                let texture_uniforms = TextureUniforms::new(
                    texture,
                    num_textures as i32
                );

                shader.attach_uniforms_from(&texture_uniforms);
                num_textures += 1;
                // TODO: send more tiles to the ray tracer
                // As it is now, it only send the 64 min uniq tiles
                // from the texture buffer i.e. all the 0 and 1 depth tiles
                // + 4 tiles of depth 2: 12 + 48 + 4 = 64
                if num_textures == 63 {
                    break;
                }
            }
        }
        num_textures += 1;
        shader.attach_uniform("num_textures", &(num_textures as i32));

        // Texture 2d array
        if self.texture_2d_array.is_storing_integer() {
            shader.attach_uniform("texInt", &*self.texture_2d_array);
        } else {
            shader.attach_uniform("tex", &*self.texture_2d_array);
        }

        shader
    }
}
