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
        cells_to_draw: &HEALPixCells,
        // The survey from which we get the textures to plot
        // Usually it is the most refined survey
        survey: &'a ImageSurvey,
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
    fn get_textures_from_survey<'a, P: Projection>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurvey) -> ImageSurveyTextures<'a> {
        let mut textures = ImageSurveyTextures::new(cells_to_draw.len());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(*cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(*cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
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
    fn get_textures_from_survey<'a, P: Projection>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurvey) -> ImageSurveyTextures<'a> {
        let mut textures = ImageSurveyTextures::new(cells_to_draw.len());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.insert(*cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.insert(*cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
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
    fn get_textures_from_survey<'a, P: Projection>(cells_to_draw: &HEALPixCells, survey: &'a ImageSurvey) -> ImageSurveyTextures<'a> {
        let mut textures = ImageSurveyTextures::new(cells_to_draw.len());

        for cell in cells_to_draw {
            let parent_cell = cell.parent();

            if survey.contains(&parent_cell) {
                let starting_cell = if survey.contains(&cell) {
                    cell
                } else {
                    survey.get_nearest_parent(&parent_cell)
                };
                let starting_cell_in_tex = survey.get(&starting_cell);
                let ending_cell_in_tex = survey.get(&parent_cell);

                textures.insert(cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
            } else {
                let starting_cell = if survey.contains(&cell) {
                    cell
                } else {
                    survey.get_nearest_parent(&parent_cell)
                };

                let ending_cell = starting_cell;

                let starting_cell_in_tex = survey.get(&starting_cell);
                let ending_cell_in_tex = survey.get(&ending_cell);

                textures.insert(cell, ImageSurveyTexture::new(starting_cell_in_tex, ending_cell_in_tex));
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
use crate::shaders::Colormap;

trait Draw {
    fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a ShaderManager) -> &'a Shader;
    fn set_uniforms<'a>(shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a>;
}

impl Draw for ColoredImageSurvey {
    fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a ShaderManager) -> &'a Shader {
        P::get_rasterizer_shader_jpg(gl, shaders)
    }
    fn set_uniforms<'a>(shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
    }
}

impl Draw for FITSImageSurveyColormap {
    fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a ShaderManager) -> &'a Shader {
        P::get_rasterizer_shader_fits_colormap(gl, shaders)
    }
    fn set_uniforms<'a>(shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // store the colormap, transfer function, cutouts here
        // not in the HiPSConfig
        shader
    }
}

impl Draw for FITSImageSurveyColor {

}

struct GrayscaleParameter {
    h: TransferFunction,
    min_value: f32,
    max_value: f32,

    scale: f32,
    offset: f32,
    blank: f32,
}

impl GrayscaleParameter {
    fn set_uniforms<'a>(shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.h)
            .attach_uniform("min_value", &self.min_value)
            .attach_uniform("max_value", &self.max_value)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);
    }
}

/// List of the different type of surveys
enum ImageSurveyType {
    ColoredSimple {
        // The image survey texture buffer
        pub survey: ImageSurvey,
    },
    GrayscaleSimple {
        // The image survey texture buffer
        pub survey: ImageSurvey,
        colormap: String,

        param: GrayscaleParameter,
    },
    GrayscaleComponent {
        // The image survey texture buffer
        pub survey: ImageSurvey,
        // A color associated to the component
        color: cgmath::Vector3<f32>,

        param: GrayscaleParameter,
    }
}

use crate::SimpleHiPS;
impl From<SimpleHiPS> for ImageSurveyType {
    fn from(hips: SimpleHiPS) -> ImageSurveyType {
        let SimpleHiPS { properties, colormap } = hips;
        let config = HiPSConfig::new(gl, &properties);
        let survey = ImageSurvey::new();

        if properties.isColor {
            ImageSurveyType::ColoredSimple {
                survey
            }
        } else {
            // Use the colormap
        }
    }
}

impl Draw for ImageSurveyType {
    fn get_shader<'a, P: Projection>(gl: &WebGl2Context, shaders: &'a ShaderManager) -> &'a Shader {
        P::get_rasterizer_shader_fits_color(gl, shaders)
    }

    fn set_uniforms<'a>(shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        // store the colormap, transfer function, cutouts here
        // not in the HiPSConfig
        shader.attach_uniform("C", &self.color)
            .attach_uniform("K", &self.k);

        shader
    }
}

impl ImageSurveyType {
    fn get_survey(&self) -> &ImageSurvey {
        match self {
            ImageSurveyType::FITSImageSurveyColor(FITSImageSurveyColor {survey, ..}) => survey,
            ImageSurveyType::FITSImageSurveyColormap(FITSImageSurveyColormap { survey, ..}) => survey,
            ImageSurveyType::ColoredImageSurvey(ColoredImageSurvey { survey }) => survey,
        }
    }
}

enum ImageSurveyPrimaryType<'a> {
    FITSImageSurveyColor(Vec<&'a str>),
    FITSImageSurveyColormap(&'a str),
    ColoredImageSurvey(&'a str)
}

impl ImageSurveyPrimaryType {
    fn append_fits(&self, root_url: &str) -> Result<(), JsValue> {
        match self {
            ImageSurveyPrimaryType::FITSImageSurveyColor(surveys) => {
                if let Some(s) = surveys.get(root_url) {
                    Some(s.survey)
                } else {
                    None
                }
            },
            ImageSurveyPrimaryType::FITSImageSurveyColormap(FITSImageSurveyColormap { survey, ..}) => {
                if survey.get_root_url() == root_url {
                    Some(survey)
                } else {
                    None
                }
            },
            ImageSurveyPrimaryType::ColoredImageSurvey(ColoredImageSurvey { survey }) => {
                if survey.get_root_url() == root_url {
                    Some(survey)
                } else {
                    None
                }
            },
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

        colored_image_survey = None,
        fits_image_survey_colormap = None,
        fits_image_survey_colors = None,

        ImageSurveys {
            surveys,
            views,
            most_refined_survey,

            colored_image_survey,
            fits_image_survey_colormap,
            fits_image_survey_colors
        }
    }

    fn get_most_refined_survey_id(&self) -> String {
        let mut tex_size_min = std::i32::MAX;

        let mut most_refined_survey = String::new();
        for (id, survey) in self.surveys.iter() {
            let tex_size_cur = survey.get_survey()
                .config()
                .get_texture_size();

            if tex_size_cur < tex_size_min {
                most_refined_survey = id;
                tex_size_min = tex_size_cur;
            }
        }

        most_refined_survey
    }

    pub fn set_primary_hips(&mut self, ) {

    }

    pub fn add(&mut self, survey: ImageSurveyType, camera: &CameraViewPort) {
        let root_url = survey.get_root_url();
        match survey {
            ImageSurveyType::FITSImageSurveyColor(_) {
                if self.surveys.contains_key(root_url) {
                    self.surveys.remove(root_url);
                    self.views.remove(root_url);

                    let index = self.fits_image_survey_colors
                        .iter()
                        .position(|x| *x == root_url)
                        .unwrap();
                    self.fits_image_survey_colors.remove(index);
                }

                if let Some(surveys_id) = &mut self.fits_image_survey_colors {
                    surveys_id.push(root_url);
                } else {
                    self.fits_image_survey_colors = Some(vec![root_url.clone()]);
                }
            },
            ImageSurveyType::FITSImageSurveyColormap(_) {
                if let Some(survey_id) = &mut self.fits_image_survey_colormap {
                    self.surveys.remove(survey_id);
                    self.views.remove(survey_id);

                    survey_id = root_url.clone();
                } else {
                    self.fits_image_survey_colormap = Some(root_url.clone());
                }
            },
            ImageSurveyType::ColoredImageSurvey(_) {
                if self.surveys.contains_key(root_url) {
                    // If its contains the same JPG/PNG survey, we quit
                    return;
                }

                if let Some(survey_id) = &mut self.colored_image_survey {
                    self.surveys.remove(survey_id);
                    self.views.remove(survey_id);

                    survey_id = root_url.clone();
                } else {
                    self.colored_image_survey = Some(root_url.clone());
                }
            },
        }

        self.surveys.insert(root_url, survey);
        // Instanciate a new view on this survey
        self.views.insert(root_url, ViewHEALPixCells::new());

        // Check for what is the most refined survey
        self.most_refined_survey = Some(self.get_most_refined_survey_id());
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
    pub fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles, exec: &mut TaskExecutor) {
        for (tile, result) in resolved_tiles.iter() {
            let mut survey = &mut self.surveys.get_mut(&tile.root_url)
                .unwrap()
                .get_survey();

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

    pub fn get_colored_survey(&self) -> Option<&ColoredImageSurvey> {
        if let Some(colored_image_survey) = self.colored_image_survey {
            let survey = self.surveys.get(colored_image_survey).unwrap();

            match survey {
                ImageSurveyType::ColoredImageSurvey(survey) => {
                    Some(survey)
                },
                _ => unreachable!()
            }
        } else {
            None
        }
    }
    pub fn get_fits_colormap_survey(&self) -> Option<&FITSImageSurveyColormap> {
        if let Some(fits_survey_colormap) = self.fits_image_survey_colormap {
            let survey = self.surveys.get(fits_survey_colormap).unwrap();

            match survey {
                ImageSurveyType::FITSImageSurveyColormap(survey) => {
                    Some(survey)
                },
                _ => unreachable!()
            }
        } else {
            None
        }
    }
    pub fn get_fits_color_surveys(&self) -> Option<Vec<&FITSImageSurveyColor>> {
        if let Some(fits_survey_colors) = self.fits_image_survey_colors {
            let surveys = fits_survey_colors.iter()
                .map(|survey_color_id| {
                    let survey = self.surveys.get(fits_survey_colormap).unwrap();
                    match survey {
                        ImageSurveyType::FITSImageSurveyColor(survey) => {
                            survey
                        },
                        _ => unreachable!()
                    }
                })
                .collect();

            Some(surveys)
        } else {
            None
        }
    }
}
/*
pub struct HiPSSphere {    
    raster: Rasterizer,
    raytracer: RayTracer,

    gl: WebGl2Context,
}
*/
use crate::{
    renderable::{Angle, ArcDeg},
    buffer::HiPSConfig,
    shader::ShaderManager,
    time::{Time, DeltaTime},
    async_task::TaskExecutor,
};

use crate::TransferFunction;
use crate::shaders::Colormap;
use crate::HiPSDefinition;
use wasm_bindgen::JsValue;

type IsNextFrameRendered = bool;
use crate::buffer::Tiles;

// This is specific to the rasterizer method of rendering
/*impl HEALPixSphere {
    pub fn new(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager) -> Self {

        crate::log(&format!("raytracer"));
        HEALPixSphere {
            buffer,
            surveys,

            gl,
        }
    }

    pub fn set_image_survey<P: Projection>(&mut self, hips_definition: HiPSDefinition, viewport: &mut CameraViewPort, task_executor: &mut TaskExecutor) -> Result<(), JsValue> {        
        self.config.set_HiPS_definition(hips_definition)?;
        // Tell the viewport the config has changed
        viewport.set_image_survey::<P>(&self.config);

        // Clear the buffer
        self.buffer.reset(&self.gl, &self.config, viewport, task_executor);

        Ok(())
    }*/
    
    /*pub fn ask_for_tiles<P: Projection>(&mut self, cells: &HashMap<HEALPixCell, bool>) {
        // Ask for the real tiles being in the viewport
        self.buffer.ask_for_tiles(cells, &self.config);
    }*/

    /*pub fn request(&mut self, available_tiles: &Tiles, task_executor: &mut TaskExecutor) {
        //survey.register_tiles_sent_to_gpu(copied_tiles);
        self.buffer.get_resolved_tiles(available_tiles);
    }

    pub fn set_projection<P: Projection>(&mut self, viewport: &CameraViewPort, shaders: &mut ShaderManager) {
        self.update::<P>(viewport);
        self.raytracer = RayTracer::new::<P>(&self.gl, viewport, shaders);
    }

    pub fn update<P: Projection>(&mut self, available_tiles: &Tiles, camera: &CameraViewPort, exec: &mut TaskExecutor) -> IsNextFrameRendered {


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

    #[inline]
    pub fn config(&self) -> &HiPSConfig {
        &self.config
    }
}*/

use crate::utils;

use crate::renderable::DisableDrawing;
impl DisableDrawing for HiPSSphere {
    fn disable(&mut self, _: &CameraViewPort) {
    }
}