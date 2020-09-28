use crate::buffer::TileBuffer;

use crate::buffer::Texture;
use crate::healpix_cell::HEALPixCell;
pub struct ImageSurveyTexture<'a> {
    pub starting_texture: &'a Texture,
    pub ending_texture: &'a Texture,
}

impl<'a> ImageSurveyTexture<'a> {
    fn new(starting_texture: &'a Texture, ending_texture: &'a Texture) -> ImageSurveyTexture<'a> {
        ImageSurveyTexture {
            starting_texture,
            ending_texture
        }
    }
}

use std::collections::{HashMap, HashSet};
pub struct ImageSurveyTextures<'a>(HashMap<HEALPixCell, ImageSurveyTexture<'a>>);

impl<'a> ImageSurveyTextures<'a> {
    fn new(cap: usize) -> ImageSurveyTextures<'a> {
        let states = HashMap::with_capacity(cap);

        ImageSurveyTextures(states)
    }
}

impl<'a> core::ops::Deref for ImageSurveyTextures<'a> {
    type Target = HashMap<HEALPixCell, TextureState<'a>>;

    fn deref (self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for SurveyTextures<'a> {
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
    fn get_textures_from_survey<'a, P: Projection>(
        // The survey from which we get the textures to plot
        // Usually it is the most refined survey
        survey: &'a ImageSurvey,
        // Its associated view
        view: &ViewHEALPixCells,
    ) -> ImageSurveyTextures<'a>;

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
        buffer: &'a TileBuffer,
        // The HEALPix cells located in the FOV
        viewport: &CameraViewPort,
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
        buffer: &'a TileBuffer,
        // The HEALPix cells located in the FOV
        viewport: &CameraViewPort,
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
        survey: &'a ImageSurvey,
        // The HEALPix cells located in the FOV
        camera: &CameraViewPort,
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

use crate::viewport::CameraViewPort;
use crate::WebGl2Context;

use crate::renderable::projection::Projection;

use crate::buffer::ImageSurvey;
use crate::renderable::RayTracer;
use crate::renderable::Rasterizer;

enum ImageSurveyType {
    FITSImageSurvey {
        survey: ImageSurvey,
        color: cgmath::Vector4<f32>,
    },
    ColoredImageSurvey {
        survey: ImageSurvey,
    }
}

impl ImageSurveyType {
    fn get_survey(&self) -> &ImageSurvey {
        match self {
            ImageSurveyType::FITSImageSurvey { survey, ..} => survey,
            ImageSurveyType::ColoredImageSurvey { survey } => survey,
        }
    }
}

use crate::camera::ViewHEALPixCells;
struct ImageSurveys {
    surveys: HashMap<String, ImageSurveyType>,
    views: HashMap<String, ViewHEALPixCells>,
    most_refined_survey: Option<String>,
}

impl ImageSurveys {
    pub fn new() -> Self {
        let surveys = HashMap::new();
        let views = HashMap::new();
        let most_refined_survey = None;

        ImageSurveys {
            surveys,
            views,
            most_refined_survey,
        }
    }

    pub fn add(&mut self, root_url: String, survey: ImageSurveyType, camera: &CameraViewPort) {
        if self.most_refined_survey.is_none() {
            // First survey added
            self.most_refined_survey = Some(root_url);
        } else {
            // Compare the new with the current
            let tex_size_cur = self.survey.get_survey()
                .config()
                .get_texture_size();
            let tex_size_new = survey.get_survey()
                .config()
                .get_texture_size();

            if tex_size_new < tex_size_cur {
                self.most_refined_survey = Some(root_url);
            }
        }

        self.surveys.insert(root_url, survey);
        // Instanciate a new view on this survey
        self.views.insert(root_url, ViewHEALPixCells::new());
    }

    pub fn get_view_survey(&self, root_url: &str) -> &ViewHEALPixCells {
        self.views.get(root_url).unwrap()
    }

    pub fn get_view_most_refined_survey(&self) -> &ViewHEALPixCells {
        self.views.get(&self.most_refined_survey).unwrap()
    }

    pub fn update_views(&mut self, camera: &CameraViewPort) {
        for (root_url, view) in self.views.iter_mut() {
            let survey = self.surveys.get(root_url).unwrap();
            view.update(survey, camera);
        }
    }

    // Update the surveys by telling which tiles are available
    pub fn set_available_tiles(&mut self, available_tiles: &Tiles) {
        for tile in available_tiles {
            let mut survey = &mut self.surveys.get_mut(&tile.root_url).unwrap();
            survey.register_available_tile(tile);
        }
    }

    // Update the surveys by adding to the surveys the tiles
    // that have been resolved
    pub fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles, exec: &mut AladinTaskExecutor) {
        for (tile, result) in resolved_tiles.iter() {
            let mut survey = &mut self.surveys.get_mut(&tile.root_url).unwrap();

            match result {
                TileResolved::Missing { time_req } => {
                    let missing_image = survey.get_blank_tile();
                    survey.push::<TileArrayBufferImage>(tile, missing_image, time_req, exec);
                },
                TileResolved::Found { image, time_req } => {
                    match image {
                        RetrievedImageType::FITSImage { image, metadata } => {
                            survey.push::<TileArrayBufferImage>(image, tile, time_req, exec);
                        },
                        RetrievedImageType::CompressedImage(image) => {
                            survey.push::<TileHTMLImage>(image, tile, time_req, exec);
                        }
                    }
                }
            }
        }
    }

    
    // Accessors
    fn get(&self, root_url: &str) -> Option<&ImageSurvey> {
        self.surveys.get(root_url)
    }

    fn len(&self) -> usize {
        self.surveys.len()
    }

    fn iter<'a>(&'a self) -> Iter<'a, String, ImageSurveyType> {
        self.surveys.iter()
    }
}

type ImageSurveyColor = cgmath::Vector4<f32>;

pub struct HiPSSphere {    
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

type IsNextFrameRendered = bool;
use crate::buffer::Tiles;

// This is specific to the rasterizer method of rendering
impl HEALPixSphere {
    pub fn new(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager) -> Self {

        crate::log(&format!("raytracer"));
        HEALPixSphere {
            buffer,
            surveys,

            gl,
        }
    }

    pub fn set_image_survey<P: Projection>(&mut self, hips_definition: HiPSDefinition, viewport: &mut CameraViewPort, task_executor: &mut AladinTaskExecutor) -> Result<(), JsValue> {        
        self.config.set_HiPS_definition(hips_definition)?;
        // Tell the viewport the config has changed
        viewport.set_image_survey::<P>(&self.config);

        // Clear the buffer
        self.buffer.reset(&self.gl, &self.config, viewport, task_executor);

        Ok(())
    }
    
    /*pub fn ask_for_tiles<P: Projection>(&mut self, cells: &HashMap<HEALPixCell, bool>) {
        // Ask for the real tiles being in the viewport
        self.buffer.ask_for_tiles(cells, &self.config);
    }*/

    pub fn request(&mut self, available_tiles: &Tiles, task_executor: &mut AladinTaskExecutor) {
        //survey.register_tiles_sent_to_gpu(copied_tiles);
        self.buffer.get_resolved_tiles(available_tiles);
    }

    pub fn set_projection<P: Projection>(&mut self, viewport: &CameraViewPort, shaders: &mut ShaderManager) {
        self.update::<P>(viewport);
        self.raytracer = RayTracer::new::<P>(&self.gl, viewport, shaders);
    }

    pub fn update<P: Projection>(&mut self, available_tiles: &Tiles, camera: &CameraViewPort, exec: &mut AladinTaskExecutor) -> IsNextFrameRendered {


        if self.survey.is_ready() {
            // Update the scene if:
            // - The viewport changed
            // - There are remaining tiles to write to the GPU
            // - The tiles blending in GPU must be done (500ms + the write time)
            let update =  |
                (Time::now() < self.time_last_tile_written + DeltaTime::from_millis(500_f32));

            if !update {
                false
            } else {
                let aperture = camera.get_aperture();
                let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();
                if aperture <= limit_aperture {
                    // Rasterizer mode
                    self.raster.update::<P>(&mut self.buffer, camera, &self.config);
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
        viewport: &CameraViewPort,
    ) {
        let aperture = viewport.get_aperture();
        let limit_aperture: Angle<f32> = ArcDeg(150_f32).into();

        if aperture <= limit_aperture {
            // Rasterization
            let shader = Rasterizer::get_shader::<P>(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(viewport)
                .attach_uniforms_from(&self.survey)
                //.attach_uniforms_from(&self.config)
                //.attach_uniforms_from(&self.buffer)
                .attach_uniform("inv_model", viewport.get_inverted_model_mat())
                .attach_uniform("current_time", &utils::get_current_time());

            self.raster.draw::<P>(gl, &shader_bound);
        } else {
            // Ray-tracing
            let shader = RayTracer::get_shader(gl, shaders, &self.buffer);
            let shader_bound = shader.bind(gl);
            shader_bound.attach_uniforms_from(viewport)
                .attach_uniforms_from(&self.survey)
                //.attach_uniforms_from(&self.config)
                //.attach_uniforms_from(&self.buffer)
                .attach_uniform("model", viewport.get_model_mat())
                .attach_uniform("current_depth", &(viewport.depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time());

            self.raytracer.draw(gl, &shader_bound);
        }   
    }

    /*#[inline]
    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }*/

    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) {
        crate::log(&format!("{:?} {:?}", min_cutout, max_cutout));
        self.survey.config_mut()
            .set_cutouts(min_cutout, max_cutout);
    }

    pub fn set_transfer_func(&mut self, h: TransferFunction) {
        self.survey.config_mut()
            .set_transfer_function(h);
    }

    pub fn set_fits_colormap(&mut self, colormap: Colormap) {
        self.survey.config_mut()
            .set_fits_colormap(colormap);
    }
}

use crate::utils;

use crate::renderable::DisableDrawing;
impl DisableDrawing for HiPSSphere {
    fn disable(&mut self, _: &CameraViewPort) {
    }
}