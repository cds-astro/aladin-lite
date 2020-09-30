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
    fn draw<P: Projection>(gl: &WebGl2Context, shaders: &ShaderManager);
}

struct GrayscaleParameter {
    h: TransferFunction,
    min_value: f32,
    max_value: f32,

    scale: f32,
    offset: f32,
    blank: f32,
}

impl SendUniforms for GrayscaleParameter {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader.attach_uniforms_from(&self.h)
            .attach_uniform("min_value", &self.min_value)
            .attach_uniform("max_value", &self.max_value)
            .attach_uniform("scale", &self.scale)
            .attach_uniform("offset", &self.offset)
            .attach_uniform("blank", &self.blank);
    }
}

// Compute the size of the VBO in bytes
// We do want to draw maximum 768 tiles
const MAX_NUM_CELLS_TO_DRAW: usize = 768;
// Each cell has 4 vertices
const MAX_NUM_VERTICES_TO_DRAW: usize = MAX_NUM_CELLS_TO_DRAW * 4;
// There is 12 floats per vertices (lonlat, pos, uv_start, uv_end, time_start) = 2 + 3 + 3 + 3 + 1 = 12
const MAX_NUM_FLOATS_TO_DRAW: usize = MAX_NUM_VERTICES_TO_DRAW * 12;
const MAX_NUM_INDICES_TO_DRAW: usize = MAX_NUM_CELLS_TO_DRAW * 6;

struct ImageSurvey {
    id: String,
    color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: ViewHEALPixCells,

    num_idx: usize,

    sphere_sub: SphereSubdivided,
    vbo: WebGlBuffer,
    ebo: WebGlBuffer,

    gl: WebGl2Context,
}

impl ImageSurvey {
    fn new(gl: &WebGl2Context, config: HiPSConfig, color: Color, exec: Rc<RefCell<TaskExecutor>>) -> Self {
        let id = config.get_root_url().clone();

        let textures = ImageSurveyTextures::new(gl, config, exec);
        let view = ViewHEALPixCells::new();

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            MAX_NUM_FLOATS_TO_DRAW * std::mem::size_of::<f32>(),
            WebGl2RenderingContext::DYNAMIC_DRAW
        );
        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            MAX_NUM_FLOATS_TO_DRAW * std::mem::size_of::<u16>(),
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        let num_idx = 0;
        let sphere_sub = SphereSubdivided::new();
        let gl = gl.clone();
        let cells_depth_increased = false;
        ImageSurvey {
            id,
            color,
            // The image survey texture buffer
            textures,
            // Keep track of the cells in the FOV
            view,
            cells_depth_increased,
        
            num_idx,
        
            sphere_sub,
            vbo,
            ebo,
        
            gl,
        }
    }

    pub fn from<T: HiPS>(hips: T, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Self {
        hips.create()
    }

    pub fn set_positions<P: Projection>(&mut self, cells_to_draw: &HEALPixCells, last_user_action: UserAction) {
        match last_user_action {
            UserAction::Unzooming => {
                self.update_positions::<P, UnZoom>(&cells_to_draw);
            },
            UserAction::Zooming => {
                self.update_positions::<P, Zoom>(&cells_to_draw);
            },
            UserAction::Moving => {
                self.update_positions::<P, Move>(&cells_to_draw);
            },
            UserAction::Starting => {
                self.update_positions::<P, Move>(&cells_to_draw);
            }
        }
    }

    fn update_positions<P: Projection, T: RecomputeRasterizer>(&mut self, cells_to_draw: &HEALPixCells) {
        let mut lonlats = vec![];
        let mut positions = vec![];
        let mut idx_vertices = vec![];

        for cell in cells_to_draw {
            add_positions_grid::<P, T>(
                &mut lonlats,
                &mut positions,
                &mut idx_vertices,
                &cell,
                &self.sphere_sub,
            );
        }

        let mut coo = lonlats;
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 2 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);
        coo.extend(positions);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 5 - coo.len();
        coo.extend(vec![0.0; num_filling_floats]);

        let buf_positions = unsafe { js_sys::Float32Array::view(&coo) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0 as i32,
            &buf_positions
        );

        self.num_idx = idx_vertices.len();
        let buf_idx = unsafe { js_sys::Uint16Array::view(&idx_vertices) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0 as i32,
            &buf_idx
        );
    }

    pub fn set_UVs<P: Projection>(&mut self, cells_to_draw: &HEALPixCells, last_user_action: UserAction) {
        match last_user_action {
            UserAction::Unzooming => {
                let textures = UnZoom::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, UnZoom>(textures);
            },
            UserAction::Zooming => {
                let textures = Zoom::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Zoom>(textures);
            },
            UserAction::Moving => {
                let textures = Move::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Move>(textures);
            },
            UserAction::Starting => {
                let textures = Move::get_textures_from_survey(cells_to_draw, &self.textures);
                self.update_UVs::<P, Move>(textures);
            }
        }
    }


    fn update_UVs<P: Projection, T: RecomputeRasterizer>(&mut self, textures: &ImageSurveyTextures) {
        let mut uv_start = vec![];
        let mut uv_end = vec![];
        let mut start_times = vec![];

        for (cell, state) in textures.iter() {
            let uv_0 = TileUVW::new(cell, &state.starting_texture);
            let uv_1 = TileUVW::new(cell, &state.ending_texture);
            let start_time = state.ending_texture.start_time();

            add_uv_grid::<P, T>(
                &mut uv_start,
                &mut uv_end,
                &mut start_times,
                &cell,
                &self.sphere_sub,

                &uv_0, &uv_1,
                start_time.as_millis(),
            );
        }

        let mut uv = uv_start;
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 3 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(uv_end);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 6 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        uv.extend(start_time);
        let num_filling_floats = MAX_NUM_VERTICES_TO_DRAW * 7 - uv.len();
        uv.extend(vec![0.0; num_filling_floats]);

        let buf_uvs = unsafe { js_sys::Float32Array::view(&uv) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            MAX_NUM_VERTICES_TO_DRAW * 5 * std::mem::size_of::<f32>() as i32,
            &buf_uvs
        );
    }

    fn get_textures(&self) -> &ImageSurveyTextures {
        self.textures
    }
}

impl Draw for ImageSurvey {
    fn draw<P: Projection>(&self, raster: &Rasterizer, raytracer: &RayTracer, shaders: &mut ShaderManager, camera: &CameraViewPort) {
        if camera.get_aperture() > 150.0 {
            // Raytracer
            let shader = self.color.get_raytracer_shader(&self.gl, shaders).bind();
            shader
                .attach_uniforms_from(&self.camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(&self.color)
                .attach_uniform("model", camera.get_model_mat())
                .attach_uniform("current_depth", &(cells_to_draw.get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time());

            // The raytracer vao is bound at the lib.rs level
            raytracer.draw();
            return;
        }

        // The rasterizer has a buffer containing:
        // - The vertices of the HEALPix cells for the most refined survey
        // - The starting and ending uv for the blending animation
        // - The time for each HEALPix cell at which the animation begins
        //
        // Each of these data can be changed at different circumstances:
        // - The vertices are changed if:
        //     * new cells are added/removed (because new cells are added)
        //       to the previous frame.
        // - The UVs are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU 
        // - The starting blending animation times are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU

        let last_user_action = camera.last_user_action();
        // Get the cells to draw
        let cells_to_draw = if last_user_action == UserAction::UnZooming {
            if self.view.has_depth_decreased() || self.cells_depth_increased {
                self.cells_depth_increased = true;
                let new_depth = self.view.get_depth();

                super::get_cells_in_fov(new_depth + 1, &camera)
            } else {
                self.view.get_cells()
            }
        } else {
            // no more unzooming
            self.cells_depth_increased = false;
            self.view.get_cells()
        };

        let new_cells_added = self.view.is_there_new_cells_added();
        let recompute_vertex_positions = new_cells_added;
        if recompute_vertex_positions {
            self.set_positions(cells_to_draw, last_user_action);
        }

        let recompute_uvs = new_cells_added | self.textures.is_there_available_tiles();
        if recompute_uvs {
            self.set_UVs(cells_to_draw, last_user_action);
        }

        {
            let shader = self.color.get_raster_shader::<P>(&self.gl, shaders).bind();
            shader
                .attach_uniforms_from(&self.camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(&self.color)
                .attach_uniform("model", camera.get_model_mat())
                .attach_uniform("current_depth", &(cells_to_draw.get_depth() as i32))
                .attach_uniform("current_time", &utils::get_current_time());

            // The raster vao is bound at the lib.rs level
            self.raster.draw(self.num_idx);
        }
    }
}

trait HiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue>;
}

impl HiPS for SimpleHiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue> {
        let SimpleHiPS { properties, colormap, transfer } = self;

        let config = HiPSConfig::new(gl, &properties)?;

        if properties.isColor {
            ImageSurvey::new(gl, config, Color::Colored, exec)
        } else {
            ImageSurvey::new(
                gl,
                config,
                Color::Grayscale2Colormap {
                    colormap: colormap.into(),
                    param: GrayscaleParameter {
                        h: transfer.into(),
                        min_value: properties.minCutout,
                        max_value: properties.maxCutout,
                        
                        // These Parameters are not in the properties
                        // They will be retrieved by looking inside a tile
                        scale: 1.0,
                        offset: 0.0,
                        blank: 0.0,
                    }
                },
                exec
            )
        }
    }
}
use crate::{SimpleHiPS, ComponentHiPS};
impl HiPS for ComponentHiPS {
    fn create(self, gl: &WebGl2Context, exec: Rc<RefCell<TaskExecutor>>) -> Result<ImageSurvey, JsValue> {
        let ComponenHiPS { properties, color, transfer, k } = self;

        let config = HiPSConfig::new(gl, &properties)?;

        if properties.isColor {
            Err(format!("{} tiles does not contain grayscale data!", config.get_root_url()).into())
        } else {
            ImageSurvey::new(
                gl,
                config,
                Color::Grayscale2Color {
                    color,
                    k,
                    param: GrayscaleParameter {
                        h: transfer.into(),
                        min_value: properties.minCutout,
                        max_value: properties.maxCutout,
                        
                        // These Parameters are not in the properties
                        // They will be retrieved by looking inside a tile
                        scale: 1.0,
                        offset: 0.0,
                        blank: 0.0,
                    }
                },
                exec
            )
        }
    }
}

/// List of the different type of surveys
enum Color {
    Colored,
    Grayscale2Colormap {
        colormap: Colormap,
        param: GrayscaleParameter,
    },
    Grayscale2Color {
        // A color associated to the component
        color: cgmath::Vector3<f32>,
        k: f32, // factor controlling the amount of this HiPS
        param: GrayscaleParameter,
    }
}

impl Color {
    pub fn get_raster_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raster_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                P::get_raster_shader_grayscale2colormap(gl, shaders)
            },
            Color::Grayscale2Color { .. } => {
                P::get_raster_shader_grayscale2color(gl, shaders)
            },
        }
    }

    pub fn get_raytracer_shader<'a, P: Projection>(&self, gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        match self {
            Color::Colored => {
                P::get_raytracer_shader_color(gl, shaders)
            },
            Color::Grayscale2Colormap { .. } => {
                P::get_raytracer_shader_grayscale2colormap(gl, shaders)
            },
            Color::Grayscale2Color { .. } => {
                P::get_raytracer_shader_grayscale2color(gl, shaders)
            },
        }
    }
}

impl SendUniforms for Color {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        match self {
            Color::Colored => (),
            Color::Grayscale2Colormap { colormap, param } => {
                shader
                    .attach_uniforms_from(&colormap)
                    .attach_uniforms_from(&param);
            },
            Color::Grayscale2Color { color, k, param } => {
                shader
                    .attach_uniforms_from(&param)
                    .attach_uniform("C", &self.color)
                    .attach_uniform("K", &self.k);
            }
        }
    }
}

enum ImageSurveyIdx<'a> {
    Composite(Vec<&'a str>),
    Simple(&'a str),
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
    surveys: HashMap<String, ImageSurvey>,
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

    pub fn add(&mut self, survey: ImageSurveyType, camera: &CameraViewPort) {
        let root_url = survey.get_root_url();
        match survey {
            ImageSurveyType::FITSImageSurveyColor(_) => {
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
        /*// Instanciate a new view on this survey
        self.views.insert(root_url, ViewHEALPixCells::new());

        // Check for what is the most refined survey
        self.most_refined_survey = Some(self.get_most_refined_survey_id());*/
    }

   /* pub fn get_view_survey(&self, root_url: &str) -> &ViewHEALPixCells {
        self.views.get(root_url).unwrap()
    }

    pub fn get_view_most_refined_survey(&self) -> &ViewHEALPixCells {
        self.views.get(&self.most_refined_survey).unwrap()
    }*/

    pub fn update_views(&mut self, camera: &CameraViewPort) {
        for (root_url, view) in self.views.iter_mut() {
            let survey = self.surveys.get(root_url).unwrap();
            view.update(survey, camera);
        }
    }

    // Update the surveys by telling which tiles are available
    pub fn set_available_tiles(&mut self, available_tiles: &Tiles) {
        for tile in available_tiles {
            let mut textures = &mut self.surveys.get_mut(&tile.root_url)
                .unwrap()
                .get_textures();
            textures.register_available_tile(tile);
        }
    }

    // Update the surveys by adding to the surveys the tiles
    // that have been resolved
    pub fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles) {
        for (tile, result) in resolved_tiles.iter() {
            let mut textures = &mut self.surveys.get_mut(&tile.root_url)
                .unwrap()
                .get_textures();

            match result {
                TileResolved::Missing { time_req } => {
                    let default_image = textures.config().get_black_tile();
                    textures.push::<TileArrayBufferImage>(tile, default_image, time_req);
                },
                TileResolved::Found { image, time_req } => {
                    match image {
                        RetrievedImageType::FITSImage { image, metadata } => {
                            textures.push::<TileArrayBufferImage>(image, tile, time_req);
                        },
                        RetrievedImageType::CompressedImage { image } => {
                            textures.push::<TileHTMLImage>(image, tile, time_req);
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

    fn iter<'a>(&'a self) -> Iter<'a, String, ImageSurvey> {
        self.surveys.iter()
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