use crate::buffer::BufferTextures;

use crate::buffer::Texture;
use crate::healpix_cell::HEALPixCell;
pub struct TextureState<'a> {
    pub starting_texture: &'a Texture,
    pub ending_texture: &'a Texture,
}

impl<'a> TextureState<'a> {
    fn new(starting_texture: &'a Texture, ending_texture: &'a Texture) -> TextureState<'a> {
        TextureState {
            starting_texture,
            ending_texture
        }
    }
}

use std::collections::{HashMap, HashSet};
pub struct TextureStates<'a>(HashMap<HEALPixCell, TextureState<'a>>);

impl<'a> TextureStates<'a> {
    fn new(cap: usize) -> TextureStates<'a> {
        let states = HashMap::with_capacity(cap);

        TextureStates(states)
    }
}

impl<'a> core::ops::Deref for TextureStates<'a> {
    type Target = HashMap<HEALPixCell, TextureState<'a>>;

    fn deref (self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for TextureStates<'a> {
    fn deref_mut (self: &'_  mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

use crate::healpix_cell::SphereSubdivided;
pub trait RecomputeRasterizer {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn compute_texture_buffer<'a, P: Projection>(
        // The buffer that will be modified due to the need of specific tile textures by the GPU
        buffer: &'a BufferTextures,
        // The HEALPix cells located in the FOV
        viewport: &ViewPort,
    ) -> TextureStates<'a>;

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8;
}

pub struct Move;
pub struct Zoom;
pub struct UnZoom;

impl RecomputeRasterizer for Move  {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn compute_texture_buffer<'a, P: Projection>(
        // The buffer that will be modified due to the need of specific tile textures by the GPU
        buffer: &'a BufferTextures,
        // The HEALPix cells located in the FOV
        viewport: &ViewPort,
    ) -> TextureStates<'a> {
        let cells_fov = viewport.cells();

        let mut textures = TextureStates::new(cells_fov.len());

        for &cell in cells_fov {
            if buffer.contains(&cell) {
                let parent_cell = buffer.get_nearest_parent(&cell);

                let ending_cell_in_tex = buffer.get_texture(&cell);
                let starting_cell_in_tex = buffer.get_texture(&parent_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = buffer.get_nearest_parent(&cell);
                let grand_parent_cell = buffer.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = buffer.get_texture(&parent_cell);
                let starting_cell_in_tex = buffer.get_texture(&grand_parent_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }
    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        sphere_sub.get_num_subdivide::<P>(cell)
    }
}

impl RecomputeRasterizer for Zoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn compute_texture_buffer<'a, P: Projection>(
        // The buffer that will be modified due to the need of specific tile textures by the GPU
        buffer: &'a BufferTextures,
        // The HEALPix cells located in the FOV
        viewport: &ViewPort,
    ) -> TextureStates<'a> {
        let cells_fov = viewport.cells();

        let mut textures = TextureStates::new(cells_fov.len());

        for &cell in cells_fov {
            if buffer.contains(&cell) {
                let parent_cell = buffer.get_nearest_parent(&cell);

                let ending_cell_in_tex = buffer.get_texture(&cell);
                let starting_cell_in_tex = buffer.get_texture(&parent_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = buffer.get_nearest_parent(&cell);
                let grand_parent_cell = buffer.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = buffer.get_texture(&parent_cell);
                let starting_cell_in_tex = buffer.get_texture(&grand_parent_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        sphere_sub.get_num_subdivide::<P>(cell)
    }
}

impl RecomputeRasterizer for UnZoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn compute_texture_buffer<'a, P: Projection>(
        // The buffer that will be modified due to the need of specific tile textures by the GPU
        buffer: &'a BufferTextures,
        // The HEALPix cells located in the FOV
        viewport: &ViewPort,
    ) -> TextureStates<'a> {
        let depth_plus_one = viewport.depth() + 1;
        let cells_fov = viewport.get_cells_in_fov::<P>(depth_plus_one);

        let mut textures = TextureStates::new(cells_fov.len());

        for cell in cells_fov {
            let parent_cell = cell.parent();

            if buffer.contains(&parent_cell) {
                let starting_cell = if buffer.contains(&cell) {
                    cell
                } else {
                    buffer.get_nearest_parent(&parent_cell)
                };
                let starting_cell_in_tex = buffer.get_texture(&starting_cell);
                let ending_cell_in_tex = buffer.get_texture(&parent_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let starting_cell = if buffer.contains(&cell) {
                    cell
                } else {
                    buffer.get_nearest_parent(&parent_cell)
                };

                let ending_cell = starting_cell;

                let starting_cell_in_tex = buffer.get_texture(&starting_cell);
                let ending_cell_in_tex = buffer.get_texture(&ending_cell);

                textures.insert(cell, TextureState::new(starting_cell_in_tex, ending_cell_in_tex));
            }
        }

        textures
    }

    fn num_subdivision<P: Projection>(cell: &HEALPixCell, sphere_sub: &SphereSubdivided) -> u8 {
        let num_subdivision = sphere_sub.get_num_subdivide::<P>(cell);
        if num_subdivision <= 1 {
            0
        } else {
            num_subdivision - 1
        }
    }
}

use crate::viewport::ViewPort;
use crate::WebGl2Context;

use crate::renderable::projection::Projection;


use crate::renderable::RayTracer;
use crate::renderable::Rasterizer;
pub struct HiPSSphere {
    // Some information about the HiPS
    pub config: HiPSConfig,
    
    // The buffer responsible for: 
    // * Performing the async request of tiles
    // * Storing the most recently asked texture tiles
    // * Sending them to the GPU
    // TODO: Move this field to the main App struct
    buffer: BufferTextures,

    raster: Rasterizer,
    raytracer: RayTracer,

    gl: WebGl2Context,
}

use crate::{
    renderable::{Angle, ArcDeg},
    buffer::HiPSConfig,
    shader::ShaderManager,
    time::{Time, DeltaTime},
    async_task::AladinTaskExecutor,
};

use crate::TransferFunction;
use crate::shaders::Colormap;
use crate::HiPSDefinition;
use wasm_bindgen::JsValue;

impl HiPSSphere {
    pub fn new<P: Projection>(gl: &WebGl2Context, viewport: &ViewPort, config: HiPSConfig, shaders: &mut ShaderManager) -> Self {
        let buffer = BufferTextures::new(gl, &config, viewport);
        crate::log(&format!("config: {:?}", config));

        let gl = gl.clone();

        let raster = Rasterizer::new(&gl, shaders);
        crate::log(&format!("raster"));

        let raytracer = RayTracer::new::<P>(&gl, viewport, shaders);
        crate::log(&format!("raytracer"));
        HiPSSphere {
            config,
            buffer,

            raster,
            raytracer,

            gl,
        }
    }
    pub fn set_image_survey<P: Projection>(&mut self, hips_definition: HiPSDefinition, viewport: &mut ViewPort, task_executor: &mut AladinTaskExecutor) -> Result<(), JsValue> {        
        self.config.set_HiPS_definition(hips_definition)?;
        // Tell the viewport the config has changed
        viewport.set_image_survey::<P>(&self.config);

        // Clear the buffer
        self.buffer.reset(&self.gl, &self.config, viewport, task_executor);

        Ok(())
    }
    
    pub fn ask_for_tiles<P: Projection>(&mut self, cells: &HashMap<HEALPixCell, bool>) {
        // Ask for the real tiles being in the viewport
        self.buffer.ask_for_tiles(cells, &self.config);
    }

    pub fn ack_tiles_sent_to_gpu(&mut self, copied_tiles: &HashSet<HEALPixCell>, task_executor: &mut AladinTaskExecutor) {
        self.buffer.ack_tiles_sent_to_gpu(copied_tiles, task_executor, &mut self.config);
    }

    pub fn set_projection<P: Projection>(&mut self, viewport: &ViewPort, shaders: &mut ShaderManager) {
        self.update::<P>(viewport);
        self.raytracer = RayTracer::new::<P>(&self.gl, viewport, shaders);
    }

    pub fn update<P: Projection>(&mut self, viewport: &ViewPort) -> bool {
        // This call handles:
        // - The request of new asked tiles
        // - The copy of the tiles to the GPU
        //self.buffer.update(viewport, task_executor);

        /*// Debug code
        for cell in cells {
            if !self.buffer.contains(cell) {
                crate::log(&format!("{:?} not contained", cell));
            }
        }*/

        if self.buffer.is_ready() {
            // Update the scene if:
            // - The viewport changed
            // - There are remaining tiles to write to the GPU
            // - The tiles blending in GPU must be done (500ms + the write time)
            let update = viewport.is_viewport_updated() |
                (Time::now() < self.buffer.time_last_tile_written() + DeltaTime::from_millis(500_f32));

            if !update {
                false
            } else {
                let aperture = viewport.get_aperture();
                let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();
                if aperture <= limit_aperture {
                    // Rasterizer mode
                    self.raster.update::<P>(&mut self.buffer, viewport, &self.config);
                }

                true
            }
        } else {
            // Do not render the scene while the buffer is not ready
            true
        }
    }

    pub fn draw<P: Projection>(
        &mut self,
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        viewport: &ViewPort,
    ) {
        let aperture = viewport.get_aperture();
        let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();
        let delta_depth = self.config.delta_depth();

        if aperture <= limit_aperture {
            // Rasterization
            let shader = Rasterizer::get_shader::<P>(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(viewport)
                .attach_uniforms_from(&self.config)
                .attach_uniforms_from(&self.buffer)
                .attach_uniform("inv_model", viewport.get_inverted_model_mat())
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("size_tile_uv", &(1_f32 / ((8 << delta_depth) as f32)));

            self.raster.draw::<P>(gl, &shader_bound);
        } else {
            // Ray-tracing
            let shader = RayTracer::get_shader(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(viewport)
                .attach_uniforms_from(&self.config)
                .attach_uniforms_from(&self.buffer)
                .attach_uniform("model", viewport.get_model_mat())
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("current_depth", &(viewport.depth() as i32))
                .attach_uniform("size_tile_uv", &(1_f32 / ((8 << delta_depth) as f32)));

            self.raytracer.draw(gl, &shader_bound);
        }   
    }

    #[inline]
    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }

    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) {
        crate::log(&format!("{:?} {:?}", min_cutout, max_cutout));
        self.config.set_cutouts(min_cutout, max_cutout);
    }

    pub fn set_transfer_func(&mut self, h: TransferFunction) {
        self.config.set_transfer_function(h);
    }

    pub fn set_fits_colormap(&mut self, colormap: Colormap) {
        self.config.set_fits_colormap(colormap);
    }
}

use crate::utils;

use crate::renderable::DisableDrawing;
impl DisableDrawing for HiPSSphere {
    fn disable(&mut self, _: &ViewPort) {
    }
}