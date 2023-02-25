mod triangulation;
pub mod uv;
pub mod raytracing;

use al_api::hips::HiPSTileFormat;
use al_api::hips::ImageMetadata;

use al_core::colormap::Colormap;
use al_core::VertexArrayObject;
use al_core::VecData;
use al_core::shader::Shader;
use al_core::WebGlContext;
use al_core::image::Image;
use al_core::image::format::ImageFormatType;
use al_core::colormap::Colormaps;
use al_core::webgl_ctx::GlWrapper;

use crate::ProjectionType;
use crate::math::{vector::dist2, angle::Angle};
use crate::camera::CameraViewPort;
use crate::{shader::ShaderManager, survey::config::HiPSConfig};
use crate::{
    math::lonlat::LonLatT,
    utils,
};

use crate::math::lonlat::LonLat;
use crate::downloader::request::allsky::Allsky;
use crate::healpix::{cell::HEALPixCell, coverage::HEALPixCoverage};
use crate::time::Time;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use crate::survey::buffer::ImageSurveyTextures;
use crate::survey::texture::Texture;
use crate::survey::view::HEALPixCellsInView;
use raytracing::RayTracer;
use uv::{TileCorner, TileUVW};

use cgmath::{Matrix, Matrix4, Vector4, Vector2};
use web_sys::{WebGl2RenderingContext};
use std::fmt::Debug;
use wasm_bindgen::JsValue;

// Identity matrix
const ID: &Matrix4<f64> = &Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);
// Longitude reversed identity matrix
const ID_R: &Matrix4<f64> = &Matrix4::new(
    -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

const M: f64 = 280.0*280.0;
const N: f64 = 150.0*150.0;
const RAP: f64 = 0.7;

fn is_too_large(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> bool {
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

fn num_subdivision(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> u8 {
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

pub fn get_raster_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    let colored_hips = config.is_colored();

    if colored_hips && cmap.label() == "native" {
        crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerColorFS")
    } else {
        if config.tex_storing_unsigned_int {
            crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapUnsignedFS")
        } else if config.tex_storing_integers {
            crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapIntegerFS")
        } else {
            crate::shader::get_shader(gl, shaders, "RasterizerVS", "RasterizerGrayscale2ColormapFS")
        }
    }
}

pub fn get_raytracer_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    let colored_hips = config.is_colored();
    if colored_hips && cmap.label() == "native" {
        crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerColorFS")
    } else {
        if config.tex_storing_unsigned_int {
            crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapUnsignedFS")
        } else if config.tex_storing_integers {
            crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapIntegerFS")
        } else {
            crate::shader::get_shader(gl, shaders, "RayTracerVS", "RayTracerGrayscale2ColormapFS")
        }
    }
}

pub struct HiPS {
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

impl HiPS {
    pub fn new(
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
        Ok(HiPS {
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

    pub fn update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) {
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

    pub fn reset_frame(&mut self) {
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

    pub fn recompute_vertices(&mut self, camera: &CameraViewPort, projection: &ProjectionType) {
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

                            let ndc_pos = if let Some(ndc_pos) = projection.world_to_normalized_device_space(&world_pos, camera) {
                                ndc_pos
                            } else {
                                Vector2::new(0.0, 0.0)
                            };

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

    pub fn refresh_view(&mut self, camera: &CameraViewPort) {
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

    pub fn draw(
        &self,
        //switch_from_raytrace_to_raster: bool,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &CameraViewPort,
        raytracer: &RayTracer,
        cfg: &ImageMetadata,
    ) -> Result<(), JsValue> {
        // Get the coo system transformation matrix
        let selected_frame = camera.get_system();
        let hips_cfg = self.textures.config();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(&hips_frame);

        // Get whether the camera mode is longitude reversed
        //let longitude_reversed = hips_cfg.longitude_reversed;
        let rl = if camera.get_longitude_reversed() { ID_R } else { ID };

        // Retrieve the model and inverse model matrix
        let w2v = c * (*camera.get_w2m()) * rl;
        let v2w = w2v.transpose();

        let raytracing = raytracer.is_rendering(camera/* , depth_texture*/);
        let longitude_reversed = camera.get_longitude_reversed();
        let config = self.get_config();

        self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        // Add starting fading
        let fading = self.get_fading_factor();
        let opacity = opacity * fading;
        // Get the colormap from the color
        let cmap = colormaps.get(color.cmap_name.as_ref());

        blend_cfg.enable(&self.gl, || {
            if raytracing {
                // Triangle are defined in CCW
                self.gl.cull_face(WebGl2RenderingContext::BACK);
    
                let shader = get_raytracer_shader(
                    cmap,
                    &self.gl,
                    shaders,
                    &config,
                )?;
    
                let shader = shader.bind(&self.gl);
                shader
                    .attach_uniforms_from(camera)
                    .attach_uniforms_from(&self.textures)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cmap, colormaps)
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
                let shader = get_raster_shader(
                    cmap,
                    &self.gl,
                    shaders,
                    &config,
                )?
                .bind(&self.gl);
    
                shader
                    .attach_uniforms_from(camera)
                    .attach_uniforms_from(&self.textures)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cmap, colormaps)
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
        })?;

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}