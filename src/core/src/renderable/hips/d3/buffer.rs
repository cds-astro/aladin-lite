use std::collections::HashMap;

use crate::CameraViewPort;
use crate::LonLatT;
use al_core::image::Image;
use al_core::WebGlContext;

use super::texture::HpxTexture3D;
use crate::downloader::request::allsky::Allsky;
use crate::healpix::cell::HEALPixCell;
use crate::renderable::hips::config::HiPSConfig;
use crate::renderable::hips::HpxTileBuffer;
use crate::time::Time;
use crate::Abort;
use crate::JsValue;
use al_api::hips::ImageExt;
// Fixed sized binary heap
pub struct HiPS3DBuffer {
    // Some information about the HiPS
    textures: HashMap<HEALPixCell, HpxTexture3D>,

    config: HiPSConfig,

    available_tiles_during_frame: bool,

    gl: WebGlContext,
}

impl HiPS3DBuffer {
    pub fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let textures = HashMap::new();

        let available_tiles_during_frame = false;

        let gl = gl.clone();
        Ok(Self {
            config,

            textures,
            available_tiles_during_frame,
            gl,
        })
    }

    pub fn push_allsky(&mut self, allsky: Allsky) -> Result<(), JsValue> {
        let Allsky {
            image,
            time_req,
            depth_tile,
            channel,
            ..
        } = allsky;

        {
            let mutex_locked = image.borrow();
            let images = mutex_locked.as_ref().unwrap_abort();
            for (idx, image) in images.iter().enumerate() {
                self.push(
                    &HEALPixCell(depth_tile, idx as u64),
                    image,
                    time_req,
                    channel.map(|c| c as u16).unwrap_or(0),
                )?;
            }
        }

        Ok(())
    }

    pub fn find_nearest_slice(&self, cell: &HEALPixCell, slice: u16) -> Option<u16> {
        self.get(cell).and_then(|t| t.find_nearest_slice(slice))
    }

    // This method pushes a new downloaded tile into the buffer
    // It must be ensured that the tile is not already contained into the buffer
    pub fn push<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
        slice_idx: u16,
    ) -> Result<(), JsValue> {
        let tex = if let Some(tex) = self.textures.get_mut(cell) {
            tex
        } else {
            self.textures
                .insert(*cell, HpxTexture3D::new(*cell, time_request));

            self.textures.get_mut(cell).unwrap()
        };

        // copy to the 3D textured block
        tex.append(image, slice_idx, &self.config, &self.gl)?;

        self.available_tiles_during_frame = true;

        Ok(())
    }

    // Return if tiles did become available
    pub fn reset_available_tiles(&mut self) -> bool {
        let available_tiles_during_frame = self.available_tiles_during_frame;
        self.available_tiles_during_frame = false;

        available_tiles_during_frame
    }

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    pub fn contains_tile(&self, texture_cell: &HEALPixCell, slice: u16) -> bool {
        self.get(texture_cell)
            .map_or(false, |t| t.contains_slice(slice))
    }

    /// Accessors
    pub fn get(&self, cell: &HEALPixCell) -> Option<&HpxTexture3D> {
        self.textures.get(cell)
    }

    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }
}

impl HpxTileBuffer for HiPS3DBuffer {
    type T = HpxTexture3D;

    fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let textures = HashMap::new();

        let available_tiles_during_frame = false;

        let gl = gl.clone();
        Ok(Self {
            config,

            textures,
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

    fn set_image_ext(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue> {
        self.config.set_image_ext(ext)?;

        let channel = self.config.get_format().get_channel();

        self.textures.clear();
        //self.ready = false;
        self.available_tiles_during_frame = true;

        Ok(())
    }

    fn read_pixel(&self, pos: &LonLatT<f64>, camera: &CameraViewPort) -> Result<JsValue, JsValue> {
        todo!();
    }

    // Tell if a texture is available meaning all its sub tiles
    // must have been written for the GPU
    fn contains(&self, cell: &HEALPixCell) -> bool {
        self.get(cell).is_some()
    }

    /// Accessors
    fn get(&self, cell: &HEALPixCell) -> Option<&HpxTexture3D> {
        self.textures.get(cell)
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
    }
}
