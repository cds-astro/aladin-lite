use std::collections::HashMap;

use al_core::image::Image;
use al_core::WebGlContext;

use super::super::tile_heap::TileHeap;
use super::texture::HpxFreqTex;
use crate::healpix::cell::HEALPixFreqCell;
use crate::renderable::hips::config::HiPSConfig;
use crate::renderable::hips::HpxTileBuffer;
use crate::time::Time;
use crate::Abort;
use crate::JsValue;
use al_api::hips::ImageExt;
// Fixed sized binary heap
pub struct HiPS3DBuffer {
    // Some information about the HiPS
    textures: HashMap<HEALPixFreqCell, HpxFreqTex>,
    heap: TileHeap<HEALPixFreqCell>,

    config: HiPSConfig,

    available_tiles_during_frame: bool,

    gl: WebGlContext,
}

impl HiPS3DBuffer {
    /*pub fn push_allsky(&mut self, allsky: AllskyRequest) -> Result<(), JsValue> {
        let AllskyRequest {
            request,
            //depth_tile,
            channel,
            ..
        } = allsky;

        {
            let mutex_locked = request.data.borrow();
            let images = mutex_locked.as_ref().unwrap_abort();
            for (idx, image) in images.iter().enumerate() {
                self.push(
                    &HEALPixCell(0, idx as u64),
                    image,
                    request.time_request,
                    channel.map(|c| c as u16).unwrap_or(0),
                )?;
            }
        }

        Ok(())
    }*/

    /*pub fn find_nearest_slice(&self, cell: &HEALPixCell, slice: u16) -> Option<u16> {
        self.get(cell).and_then(|t| t.find_nearest_slice(slice))
    }*/

    fn push_cell(&mut self, cell: &HEALPixFreqCell, time_request: Time) -> Result<(), JsValue> {
        // Check if the cell is not yet contain in the buffer
        if !self.contains(cell) {
            // If not, add create it and add it to the buffer
            if self.heap.is_full() {
                // Pop the oldest requested texture
                let oldest_texture = self.heap.pop().unwrap_abort();

                // Remove it from the textures HashMap
                self.textures
                    .remove(oldest_texture.cell())
                    .expect("Texture (oldest one) has not been found in the buffer of textures");
            }

            let texture = HpxFreqTex::new(
                cell.clone(),
                time_request,
                self.config.tile_size as u16,
                self.config.tile_depth.unwrap_or(32) as u16,
                self.config.get_format().get_pixel_format(),
                &self.gl,
            )?;

            // Push it to the buffer
            self.heap.push(&texture);
            self.textures.insert(cell.clone(), texture);
        };

        Ok(())
    }

    // Push a image slice into the buffer
    pub fn push_tile_slice<I: Image>(
        &mut self,
        cell: &HEALPixFreqCell,
        image: I,
        time_request: Time,
        // this slice index inside the cubic cell
        slice_idx: u16,
    ) -> Result<(), JsValue> {
        self.push_cell(cell, time_request)?;

        let texture = self.textures.get_mut(cell).unwrap_abort();

        // And copy the image in that cubic tile
        texture.append_tile_slice(image, slice_idx)?;
        self.available_tiles_during_frame = true;

        Ok(())
    }

    pub fn push_tile_from_fits(
        &mut self,
        cell: &HEALPixFreqCell,
        raw_bytes: js_sys::Uint8Array,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.push_cell(cell, time_request)?;

        let texture = self.textures.get_mut(cell).unwrap_abort();

        // And copy the image in that cubic tile
        texture.set_data_from_fits(raw_bytes, size)?;
        self.available_tiles_during_frame = true;

        Ok(())
    }

    pub fn push_tile_from_jpeg(
        &mut self,
        cell: &HEALPixFreqCell,
        decoded_bytes: Box<[u8]>,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.push_cell(cell, time_request)?;

        let texture = self.textures.get_mut(cell).unwrap_abort();

        // And copy the image in that cubic tile
        texture.set_data_from_jpeg(decoded_bytes, size)?;
        self.available_tiles_during_frame = true;

        Ok(())
    }

    pub fn push_tile_from_png(
        &mut self,
        cell: &HEALPixFreqCell,
        decoded_bytes: Box<[u8]>,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.push_cell(cell, time_request)?;

        let texture = self.textures.get_mut(cell).unwrap_abort();

        // And copy the image in that cubic tile
        texture.set_data_from_png(decoded_bytes, size)?;
        self.available_tiles_during_frame = true;

        Ok(())
    }

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    pub fn contains_slice(
        &self,
        // the cell to check
        cell: &HEALPixFreqCell,
        // the idx of one slice inside the cube, has to be in [0; 2^(f_order) - 1]
        idx_slice: u16,
    ) -> bool {
        self.get(cell).is_some_and(|t| t.contains_slice(idx_slice))
    }

    // Get the nearest spatial parent found in the buffer
    pub fn get_nearest_parent(&self, cell: &HEALPixFreqCell) -> Option<HEALPixFreqCell> {
        let mut parent_cell = cell.parent();

        while !self.contains(&parent_cell) && !parent_cell.is_hpx_root() {
            parent_cell = parent_cell.parent();
        }

        if self.contains(&parent_cell) {
            Some(parent_cell)
        } else {
            None
        }
    }
}

impl HpxTileBuffer for HiPS3DBuffer {
    type T = HpxFreqTex;
    type C = HEALPixFreqCell;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let textures = HashMap::new();
        // Limit the number of cached cubes to 256 so approx 256 MB
        let heap = TileHeap::with_capacity(1024);

        let available_tiles_during_frame = false;

        let gl = gl.clone();
        Ok(Self {
            config,

            textures,
            heap,
            available_tiles_during_frame,
            gl,
        })
    }

    // Return if tiles did become available
    fn reset_available_tiles(&mut self) -> bool {
        let available_tiles_during_frame = self.available_tiles_during_frame;
        self.available_tiles_during_frame = false;

        available_tiles_during_frame
    }

    fn set_image_ext(&mut self, _gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue> {
        self.config.set_image_ext(ext)?;

        self.textures.clear();
        self.heap.clear();

        self.available_tiles_during_frame = true;

        Ok(())
    }

    /// Accessors
    fn get(&self, cell: &Self::C) -> Option<&HpxFreqTex> {
        self.textures.get(cell)
    }

    fn contains(&self, cell: &Self::C) -> bool {
        self.get(cell).is_some()
    }

    fn config(&self) -> &HiPSConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }
}

use al_core::shader::SendUniforms;
use al_core::shader::ShaderBound;
impl SendUniforms for HiPS3DBuffer {
    // Send only the allsky textures
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.config)
    }
}

impl Drop for HiPS3DBuffer {
    fn drop(&mut self) {
        // drop all the 3D block textures
        self.textures.clear();
        self.heap.clear();
    }
}
