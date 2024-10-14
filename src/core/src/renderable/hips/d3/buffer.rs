use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;

use al_core::image::format::ChannelType;

use cgmath::Vector3;

use al_api::hips::ImageExt;
use al_core::webgl_ctx::WebGlRenderingCtx;

use al_core::image::format::ImageFormat;
use al_core::image::format::{R16I, R32F, R32I, R64F, R8UI, RGB8U, RGBA8U};
use al_core::image::Image;
use al_core::shader::{SendUniforms, ShaderBound};
use al_core::Texture2DArray;
use al_core::WebGlContext;

use super::texture::HEALPixTexturedCube;
use crate::downloader::request::allsky::Allsky;
use crate::healpix::cell::HEALPixCell;
use crate::healpix::cell::NUM_HPX_TILES_DEPTH_ZERO;
use crate::math::lonlat::LonLatT;
use crate::renderable::hips::config::HiPSConfig;
use crate::time::Time;
use crate::Abort;
use crate::JsValue;
// Fixed sized binary heap
pub struct HiPS3DBuffer {
    // Some information about the HiPS
    textures: HashMap<HEALPixCell, HEALPixTexturedCube>,

    config: HiPSConfig,
    num_root_textures_available: u8,

    available_tiles_during_frame: bool,

    gl: WebGlContext,
}

impl HiPS3DBuffer {
    pub fn new(gl: &WebGlContext, config: HiPSConfig) -> Result<Self, JsValue> {
        let textures = HashMap::new();

        let num_root_textures_available = 0;
        let available_tiles_during_frame = false;

        let gl = gl.clone();
        Ok(Self {
            config,

            num_root_textures_available,
            textures,
            available_tiles_during_frame,
            gl,
        })
    }

    /*
    pub fn set_format(&mut self, gl: &WebGlContext, ext: ImageExt) -> Result<(), JsValue> {
        self.config.set_image_fmt(ext)?;

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
            Texture::new(&HEALPixCell(0, 0), 0, now),
            Texture::new(&HEALPixCell(0, 1), 1, now),
            Texture::new(&HEALPixCell(0, 2), 2, now),
            Texture::new(&HEALPixCell(0, 3), 3, now),
            Texture::new(&HEALPixCell(0, 4), 4, now),
            Texture::new(&HEALPixCell(0, 5), 5, now),
            Texture::new(&HEALPixCell(0, 6), 6, now),
            Texture::new(&HEALPixCell(0, 7), 7, now),
            Texture::new(&HEALPixCell(0, 8), 8, now),
            Texture::new(&HEALPixCell(0, 9), 9, now),
            Texture::new(&HEALPixCell(0, 10), 10, now),
            Texture::new(&HEALPixCell(0, 11), 11, now),
        ];

        self.heap.clear();
        self.textures.clear();
        //self.ready = false;
        self.num_root_textures_available = 0;
        self.available_tiles_during_frame = false;

        Ok(())
    }*/

    pub fn push_allsky(&mut self, allsky: Allsky, slice_idx: u16) -> Result<(), JsValue> {
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
                self.push(
                    &HEALPixCell(depth_tile, idx as u64),
                    image,
                    time_req,
                    slice_idx,
                )?;
            }
        }

        Ok(())
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
                .insert(*cell, HEALPixTexturedCube::new(*cell, time_request));

            self.textures.get_mut(cell).unwrap()
        };

        // copy to the 3D textured block
        tex.append_slice(image, slice_idx, &self.config, &self.gl)?;

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
    pub fn contains(&self, texture_cell: &HEALPixCell) -> bool {
        self.get(texture_cell).is_some()
    }

    // lonlat is given in the
    /*pub fn get_pixel_position_in_texture(
        &self,
        lonlat: &LonLatT<f64>,
        depth: u8,
    ) -> Result<Vector3<i32>, JsValue> {
        let (pix, dx, dy) = crate::healpix::utils::hash_with_dxdy(depth, lonlat);
        let texture_cell = HEALPixCell(depth, pix);

        if let Some(texture) = self.get(&texture_cell) {
            let cfg = &self.config;

            // Index of the texture in the total set of textures
            let texture_idx = texture.idx();

            // The size of the global texture containing the tiles
            let texture_size = cfg.get_texture_size();

            // Offset in the slice in pixels
            let mut offset = Vector3::new(
                (dy * (texture_size as f64)) as i32,
                (dx * (texture_size as f64)) as i32,
                texture_idx,
            );

            // Offset in the slice in pixels
            if self.config.tex_storing_fits {
                let texture_size = self.config.get_texture_size() as f32;
                let mut uvy = offset.y as f32 / texture_size;
                uvy = self.config.size_tile_uv
                    + 2.0 * self.config.size_tile_uv * (uvy / self.config.size_tile_uv).floor()
                    - uvy;

                offset.y = (uvy * texture_size) as i32;
            }

            Ok(offset)
        } else {
            Err(JsValue::from_str(&format!(
                "{:?} not loaded in the GPU, please wait before trying again.",
                texture_cell
            )))
        }
    }*/

    /// Accessors
    pub fn get(&self, cell: &HEALPixCell) -> Option<&HEALPixTexturedCube> {
        self.textures.get(cell)
    }

    // Get the nearest parent tile found in the CPU buffer
    pub fn get_nearest_parent(&self, cell: &HEALPixCell) -> Option<HEALPixCell> {
        if cell.is_root() {
            // Root cells are in the buffer by definition
            Some(*cell)
        } else {
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

    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut HiPSConfig {
        &mut self.config
    }

    /*pub fn get_texture_array(&self) -> &Texture2DArray {
        &self.texture_2d_array
    }*/
}

/*
impl SendUniforms for HiPS3DBuffer {
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
            let texture_uniforms = TextureUniforms::new(texture, idx as i32);
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
*/

impl Drop for HiPS3DBuffer {
    fn drop(&mut self) {
        // drop all the 3D block textures
        self.textures.clear();
    }
}
