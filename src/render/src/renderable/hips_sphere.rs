use crate::buffer::TileBuffer;

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
        buffer: &'a TileBuffer,
        // The HEALPix cells located in the FOV
        viewport: &CameraViewPort,
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
        buffer: &'a TileBuffer,
        // The HEALPix cells located in the FOV
        viewport: &CameraViewPort,
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

use crate::camera::ViewOnImageSurvey;
struct ImageSurveys {
    surveys: HashMap<String, ImageSurveyType>,
    view: ViewOnImageSurvey,
}

std::collections::hash_map::Iter
impl ImageSurveys {
    fn new() -> Self {
        
    }

    fn add(&mut self, root_url: String, survey: ImageSurveyType) {
        // check if the new survey needs a more refined view
        // if so, update it
    }

    fn get(&self, root_url: &str) -> Option<&ImageSurvey> {
        self.surveys.get(root_url)
    }

    fn iter<'a>(&'a self) -> Iter<'a, String, ImageSurveyType> {
        self.surveys.iter()
    }
}

const NUM_MAX_FITS_SURVEYS: usize = 3;
type ImageSurveyColor = cgmath::Vector4<f32>;
struct FITSImageSurveys {
    // The images surveys
    surveys: ImageSurveys,
    // Each survey has a view
    views: [Option<ViewOnImageSurvey>; NUM_MAX_FITS_SURVEYS],
    num_surveys: usize,

    time_last_tile_written: Time,
}

trait ImageSurveysOverlaying {
    fn update(surveys: &[Option<ImageSurvey>]) {

    }

    fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles, exec: &mut AladinTaskExecutor) {
        for (tile, result) in resolved_tiles.iter() {
            let idx_survey = self.idx_surveys.get(tile.root_url).unwrap();
            let mut survey = &mut self.surveys[idx_survey];

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
}

impl FITSImageSurveys {
    fn new() -> Self {
        let num_surveys = 0;
        let surveys = [None, None, None];
        let views = [None, None, None];

        let colors = [
            Vector4::new(1.0, 0.0, 0.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 1.0),
            Vector4::new(0.0, 0.0, 1.0, 1.0),
        ];

        let idx_surveys = HashMap::new();
        let time_last_tile_written = Time::now();

        FITSImageSurveys {
            num_surveys,

            surveys,
            views,
            colors,

            idx_surveys,
            time_last_tile_written
        }
    }

    fn set_image_survey(&mut self, idx: usize, survey: ImageSurvey, color: ImageSurveyColor) {
        assert!(idx < NUM_MAX_FITS_SURVEYS);

        if self.surveys[idx].is_none() {
            self.num_surveys += 1;
        }

        self.surveys[idx] = Some(survey);
        self.colors[idx] = Some(color);
        self.views[idx] = Some(ViewOnImageSurvey::new());


        let root_url = survey.get_root_url();
        self.idx_surveys.insert(root_url, idx);
    }

    fn set_color_image_survey(&mut self, idx: usize, color: ImageSurveyColor) {
        assert!(idx < NUM_MAX_FITS_SURVEYS);

        self.colors[idx] = color;
    }

    fn set_available_tiles(&mut self, available_tiles: &Tiles) {
        for tile in available_tiles {
            let idx_survey = self.idx_surveys.get(tile.root_url).unwrap();
            let mut survey = &mut self.surveys[idx_survey];
            survey.register_available_tile(tile);
        }
    }

    fn move(&mut self, camera: &CameraViewPort) {
        for (idx, view) in self.views.iter_mut().enumerate() {
            if let Some(view) = view {
                let survey = &self.surveys[idx];

                // Move the view
                view.update(survey, camera);

                // Request for the tiles not found in the buffer
                let cells_in_fov = view.get_cells();
                for cell in cells_in_fov.iter() {
                    let already_available = survey.contains_tile(cell);
                    if already_available {
                        let start_time = Time::now();
            
                        // Remove and append the texture with an updated
                        // time_request
                        let is_cell_new = view.is_new(cell);
                        survey.update_priority(cell, is_cell_new, start_time);
                        if is_cell_new {
                            self.time_last_tile_written = start_time;
                        }
                    } else {
                        // Submit the request to the buffer
                        let root_url = survey.get_root_url();
                        let format = survey.get_image_format();
                        let tile = Tile {
                            root_url,
                            format,
                            cell
                        };
                        self.buffer.request_tile(&tile);
                    }
                }
            }
        }
    }
}

pub struct HiPSSphere {    
    // The buffer responsible for: 
    // * Performing the async request of tiles
    // * Storing the most recently asked texture tiles
    // * Sending them to the GPU
    // TODO: Move this field to the main App struct
    buffer: TileBuffer,
    scheme: FITSImageSurveys,

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

impl HiPSSphere {
    pub fn new<P: Projection>(gl: &WebGl2Context, viewport: &CameraViewPort, shaders: &mut ShaderManager) -> Self {
        let buffer = TileBuffer::new(gl, viewport);
        let scheme = FITSImageSurveys::new();
        crate::log(&format!("config: {:?}", config));

        let gl = gl.clone();

        let raster = Rasterizer::new(&gl, shaders);
        crate::log(&format!("raster"));

        let raytracer = RayTracer::new::<P>(&gl, viewport, shaders);
        crate::log(&format!("raytracer"));
        HiPSSphere {
            buffer,
            survey,

            raster,
            raytracer,

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
    
    pub fn ask_for_tiles<P: Projection>(&mut self, cells: &HashMap<HEALPixCell, bool>) {
        // Ask for the real tiles being in the viewport
        self.buffer.ask_for_tiles(cells, &self.config);
    }

    pub fn request(&mut self, available_tiles: &Tiles, task_executor: &mut AladinTaskExecutor) {
        //survey.register_tiles_sent_to_gpu(copied_tiles);
        self.buffer.get_resolved_tiles(available_tiles);
    }

    pub fn set_projection<P: Projection>(&mut self, viewport: &CameraViewPort, shaders: &mut ShaderManager) {
        self.update::<P>(viewport);
        self.raytracer = RayTracer::new::<P>(&self.gl, viewport, shaders);
    }

    pub fn update<P: Projection>(&mut self, available_tiles: &Tiles, camera: &CameraViewPort, exec: &mut AladinTaskExecutor) -> IsNextFrameRendered {
        // Newly available tiles must lead to
        let is_there_new_available_tiles = !available_tiles.is_empty();
        if is_there_new_available_tiles {
            self.time_last_tile_written = Time::now();
        }

        // 1. Surveys must be aware of the new available tiles
        self.scheme.set_available_tiles(available_tiles);
        // 2. Get the resolved tiles and push them to the image surveys
        let resolved_tiles = self.buffer.get_resolved_tiles(available_tiles);
        self.sheme.add_resolved_tiles(resolved_tiles, exec);
        // 3. Try sending new tile requests after 
        self.buffer.try_sending_tile_requests();

        if self.survey.is_ready() {
            // Update the scene if:
            // - The viewport changed
            // - There are remaining tiles to write to the GPU
            // - The tiles blending in GPU must be done (500ms + the write time)
            let update = camera.has_camera_moved() |
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
                .attach_uniform("current_time", &utils::get_current_time())

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
                .attach_uniform("current_time", &utils::get_current_time())

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