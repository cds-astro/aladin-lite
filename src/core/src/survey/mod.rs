mod buffer;
pub mod config;
pub mod render;
pub mod texture;
pub mod view;


use texture::Texture;


use al_core::{VecData, SliceData};

const M: f64 = 280.0*280.0;
const N: f64 = 150.0*150.0;
const RAP: f64 = 0.7;

use crate::Abort;
use crate::ProjectionType;

use crate::math::{vector::dist2, angle::Angle};
fn is_too_large(cell: &HEALPixCell, camera: &CameraViewPort, projection: ProjectionType) -> bool {
    let vertices = cell.vertices()
        .iter()
        .filter_map(|(lon, lat)| {
            let vertex = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            projection.view_to_screen_space(&vertex, camera)
        })
        .collect::<Vec<_>>();

    if vertices.len() < 4 {
        false
    } else {
        let d1 = dist2(&vertices[0], &vertices[2]);
        let d2 = dist2(&vertices[1], &vertices[3]);
        if d1 > M || d2 > M {
            true
        } else if d1 < N && d2 < N {
            false
        } else {
            let rap = if d2 > d1 { 
                d1 / d2
            } else {
                d2 / d1
            };
        
            rap<RAP
        }
    }
}

fn num_subdivision(cell: &HEALPixCell, camera: &CameraViewPort, projection: ProjectionType) -> u8 {
    let d = cell.depth();
    let mut num_sub = 0;
    if d < 3 {
        num_sub = 3 - d;
    }

    // Largest deformation cell among the cells of a specific depth
    let largest_center_to_vertex_dist =
    cdshealpix::largest_center_to_vertex_distance(d, 0.0, cdshealpix::TRANSITION_LATITUDE);
    let smallest_center_to_vertex_dist =
    cdshealpix::largest_center_to_vertex_distance(d, 0.0, cdshealpix::LAT_OF_SQUARE_CELL);

    let (lon, lat) = cell.center();
    let center_to_vertex_dist = cdshealpix::largest_center_to_vertex_distance(d, lon, lat);

    let skewed_factor = (center_to_vertex_dist - smallest_center_to_vertex_dist)
    / (largest_center_to_vertex_dist - smallest_center_to_vertex_dist);

    if is_too_large(cell, camera, projection) || cell.is_on_pole() || skewed_factor > 0.25 {
        num_sub += 1;
    }

    num_sub
}

pub struct TextureToDraw<'a, 'b> {
    pub starting_texture: &'a Texture,
    pub ending_texture: &'a Texture,
    pub cell: &'b HEALPixCell,
}

impl<'a, 'b> TextureToDraw<'a, 'b> {
    fn new(
        starting_texture: &'a Texture,
        ending_texture: &'a Texture,
        cell: &'b HEALPixCell,
    ) -> TextureToDraw<'a, 'b> {
        TextureToDraw {
            starting_texture,
            ending_texture,
            cell,
        }
    }
}

use std::{
    collections::{HashMap, HashSet},
};

pub trait RecomputeRasterizer {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a, 'b>(
        view: &'b HEALPixCellsInView,
        // The survey from which we get the textures to plot
        // Usually it is the most refined survey
        survey: &'a ImageSurveyTextures,
    ) -> Vec<TextureToDraw<'a, 'b>>;
}

pub struct Move;
pub struct Zoom;
pub struct UnZoom;

impl RecomputeRasterizer for Move {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a, 'b>(
        view: &'b HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> Vec<TextureToDraw<'a, 'b>> {
    let cells_to_draw = view.get_cells();
        let mut textures = Vec::with_capacity(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                if let Some(ending_cell_in_tex) = survey.get(cell) {
                    if let Some(starting_cell_in_tex) = survey.get(&parent_cell) {
                        textures.push(TextureToDraw::new(
                            starting_cell_in_tex,
                            ending_cell_in_tex,
                            cell,
                        ));
                    }
                }
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                if let Some(ending_cell_in_tex) = survey.get(&parent_cell) {
                    if let Some(starting_cell_in_tex) = survey.get(&grand_parent_cell) {
                        textures.push(TextureToDraw::new(
                            starting_cell_in_tex,
                            ending_cell_in_tex,
                            cell,
                        ));
                    }
                }
            }
        }

        textures
    }
}

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed
use crate::healpix::{cell::HEALPixCell, coverage::HEALPixCoverage};

impl RecomputeRasterizer for Zoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a, 'b>(
        view: &'b HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> Vec<TextureToDraw<'a, 'b>> {
        let cells_to_draw = view.get_cells();
        let mut textures = Vec::with_capacity(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                if let Some(ending_cell_in_tex) = survey.get(cell) {
                    if let Some(starting_cell_in_tex) = survey.get(&parent_cell) {
                        textures.push(TextureToDraw::new(
                            starting_cell_in_tex,
                            ending_cell_in_tex,
                            cell,
                        ));
                    }
                }
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                if let Some(ending_cell_in_tex) = survey.get(&parent_cell) {
                    if let Some(starting_cell_in_tex) = survey.get(&grand_parent_cell) {
                        textures.push(TextureToDraw::new(
                            starting_cell_in_tex,
                            ending_cell_in_tex,
                            cell,
                        ));
                    }
                }
            }
        }

        textures
    }
}

impl RecomputeRasterizer for UnZoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a, 'b>(
        view: &'b HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> Vec<TextureToDraw<'a, 'b>> {
        let _depth = view.get_depth();
        let _max_depth = survey.config().get_max_depth();

        // We do not draw the parent cells if the depth has not decreased by at least one
        let cells_to_draw = view.get_cells();

        let mut textures = Vec::with_capacity(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                if let Some(starting_cell_in_tex) = survey.get(cell) {
                    textures.push(TextureToDraw::new(
                        starting_cell_in_tex,
                        starting_cell_in_tex,
                        cell,
                    ));
                }
            } else {
                let parent_cell = survey.get_nearest_parent(cell);

                if let Some(ending_cell_in_tex) = survey.get(&parent_cell) {
                    textures.push(TextureToDraw::new(
                        ending_cell_in_tex,
                        ending_cell_in_tex,
                        cell,
                    ));
                }
            }
        }

        textures
    }
}

use crate::camera::CameraViewPort;
use al_core::WebGlContext;

use crate::math::projection::Projection;

use buffer::ImageSurveyTextures;
use render::ray_tracer::RayTracer;
use al_api::hips::{GrayscaleColor, HiPSTileFormat};
use al_core::shader::Shader;

pub fn get_raster_shader<'a>(
    color: &HiPSColor,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    integer_tex: bool,
    unsigned_tex: bool,
) -> Result<&'a Shader, JsValue> {
    match color {
        HiPSColor::Color => crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerColorFS"),
        HiPSColor::Grayscale { color, .. } => match color {
            GrayscaleColor::Color(..) => {
                if unsigned_tex {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColorUnsignedFS")
                } else if integer_tex {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColorIntegerFS")
                } else {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColorFS")
                }
            }
            GrayscaleColor::Colormap { .. } => {
                if unsigned_tex {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapUnsignedFS")
                } else if integer_tex {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapIntegerFS")
                } else {
                    crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapFS")
                }
            }
        },
    }
}

pub fn get_raytracer_shader<'a>(
    color: &HiPSColor,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    integer_tex: bool,
    unsigned_tex: bool,
) -> Result<&'a Shader, JsValue> {
    match color {
        HiPSColor::Color => crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerColorFS"),
        HiPSColor::Grayscale { color, .. } => match color {
            GrayscaleColor::Color(..) => {
                if unsigned_tex {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColorUnsignedFS")
                } else if integer_tex {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColorIntegerFS")
                } else {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColorFS")
                }
            }
            GrayscaleColor::Colormap { .. } => {
                if unsigned_tex {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapUnsignedFS")
                } else if integer_tex {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapIntegerFS")
                } else {
                    crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapFS")
                }
            }
        },
    }
}

// Compute the size of the VBO in bytes
// We do want to draw maximum 768 tiles
//const MAX_NUM_CELLS_TO_DRAW: usize = 768;
use cgmath::{Vector4, Vector2};
use render::rasterizer::uv::{TileCorner, TileUVW};

// This method computes positions and UVs of a healpix cells
use al_core::VertexArrayObject;
pub struct ImageSurvey {
    //color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: HEALPixCellsInView,

    // The projected vertices data
    // For WebGL2 wasm, the data are interleaved
    //#[cfg(feature = "webgl2")]
    //vertices: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 0) in vec3 position;
    position: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 1) in vec3 uv_start;
    uv_start: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 2) in vec3 uv_end;
    uv_end: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 3) in float time_tile_received;
    time_tile_received: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 4) in float m0;
    m0: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 5) in float m1;
    m1: Vec<f32>,

    idx_vertices: Vec<u16>,

    num_idx: usize,

    vao: VertexArrayObject,
    gl: WebGlContext,

    min_depth_tile: u8,
    depth: u8,
    depth_tile: u8,

    footprint_moc: Option<HEALPixCoverage>,
}
use crate::{
    math::lonlat::LonLatT,
    utils,
};
use al_core::image::Image;

use web_sys::{WebGl2RenderingContext};
use crate::math::lonlat::LonLat;
use crate::downloader::request::allsky::Allsky;
use std::fmt::Debug;
impl ImageSurvey {
    fn new(
        config: HiPSConfig,
        gl: &WebGlContext,
        _camera: &CameraViewPort,
    ) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

        // layout (location = 0) in vec2 lonlat;
        // layout (location = 1) in vec3 position;
        // layout (location = 2) in vec3 uv_start;
        // layout (location = 3) in vec3 uv_end;
        // layout (location = 4) in float time_tile_received;
        // layout (location = 5) in float m0;
        // layout (location = 6) in float m1;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];

        //let vertices = vec![];
        let position = vec![];
        let uv_start = vec![];
        let uv_end = vec![];
        let time_tile_received = vec![];
        let m0 = vec![];
        let m1 = vec![];
        let idx_vertices = vec![];

        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer_single(
                2,
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            .add_array_buffer_single(
                3,
                "uv_start",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_start),
            )
            .add_array_buffer_single(
                3,
                "uv_end",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_end),
            )
            .add_array_buffer_single(
                1,
                "time_tile_received",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&time_tile_received),
            )
            .add_array_buffer_single(
                1,
                "m0",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m0),
            )
            .add_array_buffer_single(
                1,
                "m1",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m1),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            )
            .unbind();
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            .add_array_buffer(
                2,
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            .add_array_buffer(
                3,
                "uv_start",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_start),
            )
            .add_array_buffer(
                3,
                "uv_end",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_end),
            )
            .add_array_buffer(
                1,
                "time_tile_received",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&time_tile_received),
            )
            .add_array_buffer(
                1,
                "m0",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m0),
            )
            .add_array_buffer(
                1,
                "m1",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m1),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            )
            .unbind();

        let num_idx = 0;
        let min_depth_tile = config.get_min_depth_tile();

        let textures = ImageSurveyTextures::new(gl, config)?;
        let view = HEALPixCellsInView::new();

        let gl = gl.clone();
        let depth = 0;
        let depth_tile = 0;

        let footprint_moc = None;
        // request the allsky texture
        Ok(ImageSurvey {
            //color,
            // The image survey texture buffer
            textures,
            // Keep track of the cells in the FOV
            view,

            num_idx,

            vao,

            gl,

            position,
            uv_start,
            uv_end,
            time_tile_received,
            m0,
            m1,

            idx_vertices,
            min_depth_tile,
            
            depth,
            depth_tile,

            footprint_moc,
        })
    }

    fn update(&mut self, camera: &CameraViewPort, projection: ProjectionType) {
        let vertices_recomputation_needed = self.textures.reset_available_tiles() | camera.has_moved();
        if vertices_recomputation_needed {
            self.recompute_vertices(camera, projection);
        }
    }

    #[inline]
    pub fn set_moc(&mut self, moc: HEALPixCoverage) {
        self.footprint_moc = Some(moc);
    }

    #[inline]
    pub fn get_moc(&self) -> Option<&HEALPixCoverage> {
        self.footprint_moc.as_ref()
    }

    pub fn set_img_format(&mut self, fmt: HiPSTileFormat) -> Result<(), JsValue> {
        self.textures.set_format(&self.gl, fmt)
    }

    pub fn get_fading_factor(&self) -> f32 {
        self.textures
            .start_time
            .map(|start_time| {
                let fading = (Time::now().0 - start_time.0) / crate::app::BLENDING_ANIM_DURATION;
                fading.clamp(0.0, 1.0)
            })
            .unwrap_or(0.0)
    }

    pub fn is_allsky(&self) -> bool {
        self.textures.config().is_allsky
    } 

    fn reset_frame(&mut self) {
        self.view.reset_frame();
    }

    // Position given is in the camera space
    pub fn read_pixel(
        &self,
        pos: &LonLatT<f64>,
        camera: &CameraViewPort,
    ) -> Result<JsValue, JsValue> {
        // 1. Convert it to the hips frame system
        let cfg = self.textures.config();
        let camera_frame = camera.get_system();
        let hips_frame = &cfg.get_frame();

        let pos = crate::coosys::apply_coo_system(camera_frame, hips_frame, &pos.vector());

        // Get the array of textures from that survey
        let pos_tex = self
            .textures
            .get_pixel_position_in_texture(&pos.lonlat(), self.view.get_depth())?;

        let slice_idx = pos_tex.z as usize;
        let texture_array = self.textures.get_texture_array();

        let value = texture_array[slice_idx].read_pixel(pos_tex.x, pos_tex.y)?;

        if cfg.tex_storing_fits {
            let value = value
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Error unwraping the pixel read value."))?;
            let scale = cfg.scale as f64;
            let offset = cfg.offset as f64;

            Ok(JsValue::from_f64(value * scale + offset))
        } else {
            Ok(value)
        }
    }

    pub fn recompute_vertices(&mut self, camera: &CameraViewPort, projection: ProjectionType) {
        self.position.clear();
        self.uv_start.clear();
        self.uv_end.clear();
        self.time_tile_received.clear();
        self.m0.clear();
        self.m1.clear();
        self.idx_vertices.clear();

        let cfg = self.textures.config();
        // Get the coo system transformation matrix
        let selected_frame = camera.get_system();
        let hips_frame = cfg.get_frame();
        let c = selected_frame.to(&hips_frame);

        // Retrieve the model and inverse model matrix
        let w2v = c * (*camera.get_w2m());
        let v2w = w2v.transpose();

        let longitude_reversed = camera.get_longitude_reversed();
        for cell in self.view.get_cells() {
            // filter textures that are not in the moc
            let cell = if let Some(moc) = self.footprint_moc.as_ref() {
                if moc.contains(cell) {
                    Some(cell)
                } else {
                    if cfg.get_format() == ImageFormatType::RGB8U {
                        // Rasterizer does not render tiles that are not in the MOC
                        // This is not a problem for transparency rendered HiPses (FITS or PNG)
                        // but JPEG tiles do have black when no pixels data is found
                        // We therefore must draw in black for the tiles outside the HiPS MOC
                        Some(cell)
                    } else {
                        None
                    }
                }
            } else {
                Some(cell)
            };

            if let Some(cell) = cell {
                let texture_to_draw = if self.textures.contains(cell) {
                    let parent_cell = self.textures.get_nearest_parent(cell);

                    if let Some(ending_cell_in_tex) = self.textures.get(cell) {
                        if let Some(starting_cell_in_tex) = self.textures.get(&parent_cell) {
                            Some(TextureToDraw::new(
                                starting_cell_in_tex,
                                ending_cell_in_tex,
                                cell,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    let parent_cell = self.textures.get_nearest_parent(cell);
                    let grand_parent_cell = self.textures.get_nearest_parent(&parent_cell);

                    if let Some(ending_cell_in_tex) = self.textures.get(&parent_cell) {
                        if let Some(starting_cell_in_tex) = self.textures.get(&grand_parent_cell) {
                            Some(TextureToDraw::new(
                                starting_cell_in_tex,
                                ending_cell_in_tex,
                                cell,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(TextureToDraw {cell, starting_texture, ending_texture}) = texture_to_draw {
                    let uv_0 = TileUVW::new(cell, starting_texture, cfg);
                    let uv_1 = TileUVW::new(cell, ending_texture, cfg);
                    let start_time = ending_texture.start_time().as_millis();

                    let miss_0 = (starting_texture.is_missing()) as i32 as f32;
                    let miss_1 = (ending_texture.is_missing()) as i32 as f32;

                    let num_subdivision = num_subdivision(cell, camera, projection);

                    let n_segments_by_side: usize = 1 << (num_subdivision as usize);
                    let n_vertices_per_segment = n_segments_by_side + 1;

                    // Indices overwritten
                    let off_idx_vertices = (self.position.len() / 2) as u16;

                    let ll = crate::healpix::utils::grid_lonlat::<f64>(cell, n_segments_by_side as u16);
                    let n_segments_by_side_f32 = n_segments_by_side as f32;

                    for i in 0..n_vertices_per_segment {
                        for j in 0..n_vertices_per_segment {
                            let id_vertex_0 = (j + i * n_vertices_per_segment) as usize;
                            let world_pos: Vector4<f64> = v2w * ll[id_vertex_0].vector::<Vector4<f64>>();
        
                            let ndc_pos = projection.world_to_normalized_device_space_unchecked(&world_pos, camera);
                            self.position.push(ndc_pos.x as f32);
                            self.position.push(ndc_pos.y as f32);
        
                            let hj0 = (j as f32) / n_segments_by_side_f32;
                            let hi0 = (i as f32) / n_segments_by_side_f32;
        
                            let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
                            let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;
                            let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                            let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;
        
                            self.uv_start.push(uv_0[TileCorner::BottomLeft].x + hj0 * d01s);
                            self.uv_start.push(uv_0[TileCorner::BottomLeft].y + hi0 * d02s);
                            self.uv_start.push(uv_0[TileCorner::BottomLeft].z);

                            self.uv_end.push(uv_1[TileCorner::BottomLeft].x + hj0 * d01e);
                            self.uv_end.push(uv_1[TileCorner::BottomLeft].y + hi0 * d02e);
                            self.uv_end.push(uv_1[TileCorner::BottomLeft].z);

                            self.time_tile_received.push(start_time);
                            self.m0.push(miss_0);
                            self.m1.push(miss_1);

                            // push to idx_vertices
                            if i > 0 && j > 0 {
                                let idx_0 = (j - 1 + (i - 1) * n_vertices_per_segment) as u16;
                                let idx_1 = (j + (i - 1) * n_vertices_per_segment) as u16;
                                let idx_2 = (j - 1 + i * n_vertices_per_segment) as u16;
                                let idx_3 = (j + i * n_vertices_per_segment) as u16;
            
                                let i0 = 2*(idx_0 + off_idx_vertices) as usize;
                                let i1 = 2*(idx_1 + off_idx_vertices) as usize;
                                let i2 = 2*(idx_2 + off_idx_vertices) as usize;
                                let i3 = 2*(idx_3 + off_idx_vertices) as usize;
            
                                let c0 = Vector2::new(self.position[i0], self.position[i0 + 1]);
                                let c1 = Vector2::new(self.position[i1], self.position[i1 + 1]);
                                let c2 = Vector2::new(self.position[i2], self.position[i2 + 1]);
                                let c3 = Vector2::new(self.position[i3], self.position[i3 + 1]);
                        
                                let first_tri_ccw = crate::math::vector::ccw_tri(&c0, &c1, &c2);
                                let second_tri_ccw = crate::math::vector::ccw_tri(&c1, &c3, &c2);
            
                                if (!longitude_reversed && first_tri_ccw) || (longitude_reversed && !first_tri_ccw) {
                                    self.idx_vertices.push(off_idx_vertices + idx_0);
                                    self.idx_vertices.push(off_idx_vertices + idx_1);
                                    self.idx_vertices.push(off_idx_vertices + idx_2);
                                }
            
                                if (!longitude_reversed && second_tri_ccw) || (longitude_reversed && !second_tri_ccw) {
                                    self.idx_vertices.push(off_idx_vertices + idx_1);
                                    self.idx_vertices.push(off_idx_vertices + idx_3);
                                    self.idx_vertices.push(off_idx_vertices + idx_2);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.num_idx = self.idx_vertices.len();
        /*self.position.shrink_to_fit();
        self.uv_start.shrink_to_fit();
        self.uv_end.shrink_to_fit();
        self.time_tile_received.shrink_to_fit();
        self.m0.shrink_to_fit();
        self.m1.shrink_to_fit();
        self.idx_vertices.shrink_to_fit();*/

        let mut vao = self.vao.bind_for_update();
        vao.update_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.position),
        )
        .update_array(
            "uv_start",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.uv_start),
        )
        .update_array(
            "uv_end",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.uv_end),
        )
        .update_array(
            "time_tile_received",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.time_tile_received),
        )
        .update_array(
            "m0",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.m0),
        )
        .update_array(
            "m1",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.m1),
        )
        .update_element_array(
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.idx_vertices),
        );
    }

    fn refresh_view(&mut self, camera: &CameraViewPort) {
        let cfg = self.textures.config();
        let max_tile_depth = cfg.get_max_tile_depth();
        let delta_depth = cfg.delta_depth();

        let hips_frame = cfg.get_frame();
        // Compute that depth
        let camera_tile_depth = camera.get_tile_depth();
        self.depth_tile = camera_tile_depth.min(max_tile_depth);

        // Set the depth of the HiPS textures
        self.depth = if self.depth_tile > delta_depth {
            self.depth_tile - delta_depth
        } else {
            0
        };

        self.view.refresh(self.depth_tile, hips_frame, camera);
    }

    // Return a boolean to signal if the tile is present or not in the survey
    pub fn update_priority_tile(&mut self, cell: &HEALPixCell) -> bool {
        if self.textures.contains_tile(cell) {
            // The cell is present in the survey, we update its priority
            self.textures.update_priority(cell);
            true
        } else {
            false
        }
    }

    pub fn add_tile<I: Image + Debug>(
        &mut self,
        cell: &HEALPixCell,
        image: Option<I>,
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.textures.push(&cell, image, time_request)
    }

    pub fn add_allsky(
        &mut self,
        allsky: Allsky,
    ) -> Result<(), JsValue> {
        self.textures.push_allsky(allsky)
    }

    /* Accessors */
    #[inline]
    pub fn get_config(&self) -> &HiPSConfig {
        self.textures.config()
    }

    #[inline]
    pub fn get_config_mut(&mut self) -> &mut HiPSConfig {
        self.textures.config_mut()
    }

    #[inline]
    pub fn get_view(&self) -> &HEALPixCellsInView {
        &self.view
    }

    #[inline]
    pub fn get_min_depth_tile(&self) -> u8 {
        self.min_depth_tile
    }

    #[inline]
    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        self.textures.is_ready()
    }

    #[inline]
    pub fn get_ready_time(&self) -> &Option<Time> {
        &self.textures.start_time
    }

    fn draw(
        &self,
        raytracer: &RayTracer,
        screen_vao: &VertexArrayObject,
        //switch_from_raytrace_to_raster: bool,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &HiPSColor,
        mut opacity: f32,
        colormaps: &Colormaps,
    ) -> Result<(), JsValue> {
        // Get the coo system transformation matrix
        let selected_frame = camera.get_system();
        let hips_cfg = self.textures.config();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(&hips_frame);

        // Get whether the camera mode is longitude reversed
        //let longitude_reversed = hips_cfg.longitude_reversed;
        let rl = if camera.get_longitude_reversed() { ID_R } else { ID };

        // Add starting fading
        let fading = self.get_fading_factor();
        opacity *= fading;

        // Retrieve the model and inverse model matrix
        let w2v = c * (*camera.get_w2m()) * rl;
        let v2w = w2v.transpose();

        /*let depth_texture = if self.view.get_depth() > self.get_config().delta_depth() {
            self.view.get_depth() - self.get_config().delta_depth()
        } else {
            0
        };*/
        let raytracing = raytracer.is_rendering(camera/* , depth_texture*/);
        let longitude_reversed = camera.get_longitude_reversed();

        if raytracing {
            // Triangle are defined in CCW
            self.gl.cull_face(WebGl2RenderingContext::BACK);

            let shader = get_raytracer_shader(
                color,
                &self.gl,
                shaders,
                self.textures.config.tex_storing_integers,
                self.textures.config.tex_storing_unsigned_int,
            )?;

            let shader = shader.bind(&self.gl);
            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("model", &w2v)
                .attach_uniform("inv_model", &v2w)
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniforms_from(colormaps);

            raytracer.draw(&shader);
        } else {
            // Depending on if the longitude is reversed, triangles are either defined in:
            // - CCW for longitude_reversed = false
            // - CW for longitude_reversed = true
            // Get the reverse longitude flag
            if longitude_reversed {
                self.gl.cull_face(WebGl2RenderingContext::FRONT);
            } else {
                self.gl.cull_face(WebGl2RenderingContext::BACK);
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
            let config = self.get_config();
            let shader = get_raster_shader(
                color,
                &self.gl,
                shaders,
                config.tex_storing_integers,
                config.tex_storing_unsigned_int,
            )?
            .bind(&self.gl);

            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("model", &w2v)
                .attach_uniform("inv_model", &v2w)
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniforms_from(colormaps)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    Some(self.num_idx as i32),
                    WebGl2RenderingContext::UNSIGNED_SHORT,
                    0,
                );
        }

        // Depending on if the longitude is reversed, triangles are either defined in:
        // - CCW for longitude_reversed = false
        // - CW for longitude_reversed = true
        // Get the reverse longitude flag
        if longitude_reversed {
            self.gl.cull_face(WebGl2RenderingContext::FRONT);
        } else {
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        }
        Ok(())

        //self.gl.cull_face(WebGl2RenderingContext::BACK);
    }
}

use cgmath::Matrix4;
// Identity matrix
const ID: &Matrix4<f64> = &Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);
// Longitude reversed identity matrix
const ID_R: &Matrix4<f64> = &Matrix4::new(
    -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

use crate::time::Time;

use cgmath::Matrix;

use wasm_bindgen::JsValue;
use crate::{HiPSColor, SimpleHiPS};

use al_api::hips::ImageSurveyMeta;

use view::HEALPixCellsInView;
pub(crate) type Url = String;
type LayerId = String;
pub struct ImageSurveys {
    // Surveys to query
    pub surveys: HashMap<Url, ImageSurvey>,
    // The meta data associated with a layer
    meta: HashMap<LayerId, ImageSurveyMeta>,
    // Hashmap between urls and layers
    pub urls: HashMap<LayerId, Url>,
    // Layers given in a specific order to draw
    layers: Vec<LayerId>,

    most_precise_survey: Url,

    raytracer: RayTracer,
    // A vao that takes all the screen
    screen_vao: VertexArrayObject,

    background_color: ColorRGB,

    depth: u8,

    gl: WebGlContext,
}

//const BLACK_COLOR: ColorRGB = ColorRGB { r: 0.0, g: 0.0, b: 0.0 };
const DEFAULT_BACKGROUND_COLOR: ColorRGB = ColorRGB { r: 0.05, g: 0.05, b: 0.05 };
/*#[derive(PartialEq, Eq, Clone, Copy)]
enum RenderingMode {
    Raytrace,
    Rasterize,
}*/

use crate::colormap::Colormaps;
use std::borrow::Cow;
use crate::shader::ShaderId;
fn get_fontcolor_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
    shaders.get(
        gl,
        &ShaderId(
            Cow::Borrowed("RayTracerFontVS"),
            Cow::Borrowed("RayTracerFontFS"),
        ),
    )
    .unwrap_abort()
}

use al_core::image::format::ImageFormatType;
use al_api::color::ColorRGB;
use al_core::webgl_ctx::GlWrapper;
use crate::downloader::request::tile::Tile;

impl ImageSurveys {
    pub fn new(
        gl: &WebGlContext,
        projection: ProjectionType
    ) -> Self {
        let surveys = HashMap::new();
        let meta = HashMap::new();
        let urls = HashMap::new();
        let layers = Vec::new();

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new(gl, projection);
        let gl = gl.clone();
        let most_precise_survey = String::new();

        let mut screen_vao = VertexArrayObject::new(&gl);
        #[cfg(feature = "webgl2")]
        screen_vao.bind_for_update()
            .add_array_buffer_single(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                SliceData::<f32>(&[
                    -1.0, -1.0,
                    1.0, -1.0,
                    1.0, 1.0,
                    -1.0, 1.0,
                ]),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, SliceData::<u16>(&[0, 1, 2, 0, 2, 3]))
            // Unbind the buffer
            .unbind();

        #[cfg(feature = "webgl1")]
        screen_vao.bind_for_update()
            .add_array_buffer(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                SliceData::<f32>(&[
                    -1.0, -1.0,
                    1.0, -1.0,
                    1.0, 1.0,
                    -1.0, 1.0,
                ]),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, SliceData::<u16>(&[0, 1, 2, 0, 2, 3]))
            // Unbind the buffer
            .unbind();

        let depth = 0;
        let background_color = DEFAULT_BACKGROUND_COLOR;
        ImageSurveys {
            surveys,
            meta,
            urls,
            layers,

            most_precise_survey,

            raytracer,
            depth,

            background_color,
            screen_vao,

            gl,
        }
    }

    pub fn set_survey_url(&mut self, past_url: String, new_url: String) -> Result<(), JsValue> {
        if let Some(mut survey) = self.surveys.remove(&past_url) {
            // update the root_url
            survey.get_config_mut()
                .set_root_url(new_url.clone());
            
            self.surveys.insert(new_url.clone(), survey);

            // update all the layer urls
            for url in self.urls.values_mut() {
                if *url == past_url {
                    *url = new_url.clone(); 
                }
            }

            if self.most_precise_survey == past_url {
                self.most_precise_survey = new_url.clone();
            }

            Ok(())
        } else {
            Err(JsValue::from_str("Survey not found"))
        }
    }

    pub fn reset_frame(&mut self) {
        for survey in self.surveys.values_mut() {
            survey.reset_frame();
        }
    }

    pub fn set_projection(&mut self, projection: ProjectionType) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new(&self.gl, projection);
    }

    pub fn set_background_color(&mut self, color: ColorRGB) {
        self.background_color = color;
    }

    pub fn draw(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        projection: ProjectionType
    ) -> Result<(), JsValue> {
        let raytracer = &self.raytracer;
        let raytracing = raytracer.is_rendering(camera/* , depth_texture*/);

        // The first layer must be paint independently of its alpha channel
        self.gl.enable(WebGl2RenderingContext::BLEND);
        // Check whether a survey to plot is allsky
        // if neither are, we draw a font
        // if there are, we do not draw nothing
        if !self.surveys.is_empty() {
            let not_render_transparency_font = self.layers.iter()
                .any(|layer| {
                    let meta = self.meta.get(layer).unwrap_abort();
                    let url = self.urls.get(layer).unwrap_abort();
                    let survey = self.surveys.get(url).unwrap_abort();
                    let hips_cfg = survey.get_config();

                    (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0
                });

            // Need to render transparency font
            if !not_render_transparency_font {
                let opacity = self.surveys.values()
                    .fold(std::f32::MAX, |mut a, s| {
                        a = a.min(s.get_fading_factor()); a
                    });
                let background_color = &self.background_color * opacity;

                let vao = if raytracing {
                    raytracer.get_vao()
                } else {
                    // define a vao that consists of 2 triangles for the screen
                    &self.screen_vao
                };

                get_fontcolor_shader(&self.gl, shaders).bind(&self.gl).attach_uniforms_from(camera)
                    .attach_uniform("color", &background_color)
                    .attach_uniform("opacity", &opacity)
                    .bind_vertex_array_object_ref(vao)
                        .draw_elements_with_i32(
                            WebGl2RenderingContext::TRIANGLES,
                            None,
                            WebGl2RenderingContext::UNSIGNED_SHORT,
                            0,
                        );
            }
        }

        // Pre loop over the layers to see if a HiPS is entirely covering those behind
        // so that we do not have to render those
        let mut idx_start_layer = 0;
        for (idx_layer, layer) in self.layers.iter().enumerate().skip(1) {
            let meta = self.meta.get(layer).expect("Meta should be found");

            let url = self.urls.get(layer).expect("Url should be found");
            let survey = self.surveys.get_mut(url).unwrap_abort();
            let hips_cfg = survey.get_config();

            let fully_covering_survey = (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0;
            if fully_covering_survey {
                idx_start_layer = idx_layer;
            }
        }

        let rendered_layers = &self.layers[idx_start_layer..];
        for layer in rendered_layers {
            let meta = self.meta.get(layer).expect("Meta should be found");
            if meta.visible() {
                // 1. Update the survey if necessary
                let url = self.urls.get(layer).expect("Url should be found");
                let survey = self.surveys.get_mut(url).unwrap_abort();
                survey.update(camera, projection);

                let ImageSurveyMeta {
                    color,
                    opacity,
                    blend_cfg,
                    ..
                } = meta;

                let screen_vao = &self.screen_vao;

                // 2. Draw it if its opacity is not null
                blend_cfg.enable(&self.gl, || {
                    survey.draw(
                        raytracer,
                        screen_vao,
                        shaders,
                        camera,
                        color,
                        *opacity,
                        colormaps,
                    )?;

                    Ok(())
                })?;
            }
        }

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    pub fn set_image_surveys(
        &mut self,
        hipses: Vec<SimpleHiPS>,
        gl: &WebGlContext,
        camera: &mut CameraViewPort,
        projection: ProjectionType
    ) -> Result<(), JsValue> {
        // 1. Check if layer duplicated have been given
        for i in 0..hipses.len() {
            for j in 0..i {
                if hipses[i].get_layer() == hipses[j].get_layer() {
                    let layer = &hipses[i].get_layer();
                    return Err(JsValue::from_str(&format!(
                        "{:?} layer name are duplicates",
                        layer
                    )));
                }
            }
        }

        let mut current_needed_surveys = HashSet::new();
        for hips in hipses.iter() {
            let url = hips.get_properties().get_url();
            current_needed_surveys.insert(url);
        }

        // Remove surveys that are not needed anymore
        self.surveys = self
            .surveys
            .drain()
            .filter(|(_, m)| current_needed_surveys.contains(&m.textures.config().root_url))
            .collect();

        // Create the new surveys
        let mut max_depth_among_surveys = 0;

        self.meta.clear();
        self.layers.clear();
        self.urls.clear();

        let _num_surveys = hipses.len();
        let mut longitude_reversed = false;
        for SimpleHiPS {
            layer,
            properties,
            meta,
            img_format,
            ..
        } in hipses.into_iter()
        {
            let config = HiPSConfig::new(&properties, img_format)?;
            //camera.set_longitude_reversed(meta.longitude_reversed);

            // Get the most precise survey from all the ones given
            let url = properties.get_url();
            let max_order = properties.get_max_order();
            if max_order > max_depth_among_surveys {
                max_depth_among_surveys = max_order;
                self.most_precise_survey = url.clone();
            }

            // Add the new surveys
            if !self.surveys.contains_key(&url) {
                let survey = ImageSurvey::new(config, gl, camera)?;
                self.surveys.insert(url.clone(), survey);

                // A new survey has been added and it is lonely
                /*if num_surveys == 1 {
                    if let Some(initial_ra) = properties.get_initial_ra() {
                        if let Some(initial_dec) = properties.get_initial_dec() {
                            camera.set_center::<P>(&LonLatT(Angle((initial_ra).to_radians()), Angle((initial_dec).to_radians())), &properties.get_frame());
                        }
                    }

                    if let Some(initial_fov) = properties.get_initial_fov() {
                        camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
                    }
                }*/
            }

            longitude_reversed |= meta.longitude_reversed;

            self.meta.insert(layer.clone(), meta);
            self.urls.insert(layer.clone(), url);

            self.layers.push(layer);
        }

        camera.set_longitude_reversed(longitude_reversed, projection);

        Ok(())
    }

    pub fn get_image_survey_color_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue> {
        self.meta
            .get(layer)
            .cloned()
            .ok_or_else(|| JsValue::from(js_sys::Error::new("Survey not found")))
    }

    pub fn set_image_survey_color_cfg(
        &mut self,
        layer: String,
        meta: ImageSurveyMeta,
        camera: &CameraViewPort,
        projection: ProjectionType,
    ) -> Result<(), JsValue> {
        if let Some(meta_old) = self.meta.get(&layer) {
            if !meta_old.visible() && meta.visible() {
                if let Some(survey) = self.get_mut_from_layer(&layer) {
                    survey.recompute_vertices(camera, projection);
                }
            }
        }

        // Expect the image survey to be found in the hash map
        self.meta.insert(layer.clone(), meta).ok_or_else(|| {
            JsValue::from(js_sys::Error::new(&format!("{:?} layer not found", layer)))
        })?;

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        let ready = self
            .surveys
            .iter()
            .map(|(_, survey)| survey.is_ready())
            .fold(true, |acc, x| acc & x);

        ready
    }

    pub fn refresh_views(&mut self, camera: &mut CameraViewPort) {
        self.depth = 0;

        for survey in self.surveys.values_mut() {
            survey.refresh_view(camera);
            
            self.depth = self.depth.max(survey.get_depth());
        }
    }

    // Accessors
    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    pub fn get_from_layer(&self, id: &str) -> Option<&ImageSurvey> {
        self.urls.get(id).map(|url| self.surveys.get(url).unwrap_abort())
    }

    pub fn get_mut_from_layer(&mut self, id: &str) -> Option<&mut ImageSurvey> {
        let url = self.urls.get_mut(id);
        if let Some(url) = url {
            self.surveys.get_mut(url)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, root_url: &str) -> Option<&mut ImageSurvey> {
        self.surveys.get_mut(root_url)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, String, ImageSurvey> {
        self.surveys.iter_mut()
    }

    pub fn values(&self) -> impl Iterator<Item = &ImageSurvey> {
        self.surveys.values()
    }

    /*pub fn get_view(&self) -> Option<&HEALPixCellsInView> {
        if self.surveys.is_empty() {
            None
        } else {
            Some(
                self.surveys
                    .get(&self.most_precise_survey)
                    .unwrap_abort()
                    .get_view(),
            )
        }
    }*/
}

use crate::{shader::ShaderManager, survey::config::HiPSConfig};
use std::collections::hash_map::IterMut;
