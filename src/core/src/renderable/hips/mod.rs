pub mod raytracing;
mod triangulation;
pub mod uv;

use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
use al_core::colormap::Colormap;
use al_core::colormap::Colormaps;
use al_core::image::format::ChannelType;

use al_core::image::Image;

use al_core::shader::Shader;
use al_core::webgl_ctx::GlWrapper;

use al_core::VecData;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use crate::math::{angle::Angle, vector::dist2};
use crate::ProjectionType;

use crate::camera::CameraViewPort;

use crate::{math::lonlat::LonLatT, utils};
use crate::{shader::ShaderManager, survey::config::HiPSConfig};

use crate::downloader::request::allsky::Allsky;
use crate::healpix::{cell::HEALPixCell, coverage::HEALPixCoverage};
use crate::math::lonlat::LonLat;
use crate::renderable::utils::index_patch::DefaultPatchIndexIter;
use crate::time::Time;

use std::collections::HashSet;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use crate::survey::buffer::ImageSurveyTextures;
use crate::survey::texture::Texture;

use raytracing::RayTracer;
use uv::{TileCorner, TileUVW};

use cgmath::Matrix;
use std::fmt::Debug;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

const M: f64 = 280.0 * 280.0;
const N: f64 = 150.0 * 150.0;
const RAP: f64 = 0.7;

fn is_too_large(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> bool {
    let vertices = cell
        .vertices()
        .iter()
        .filter_map(|(lon, lat)| {
            let vertex = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            projection.icrs_celestial_to_screen_space(&vertex, camera)
        })
        .collect::<Vec<_>>();

    if vertices.len() < 4 {
        false
    } else {
        let d1 = dist2(vertices[0].as_ref(), &vertices[2].as_ref());
        let d2 = dist2(vertices[1].as_ref(), &vertices[3].as_ref());
        if d1 > M || d2 > M {
            true
        } else if d1 < N && d2 < N {
            false
        } else {
            let rap = if d2 > d1 { d1 / d2 } else { d2 / d1 };

            rap < RAP
        }
    }
}

fn num_subdivision(cell: &HEALPixCell, camera: &CameraViewPort, projection: &ProjectionType) -> u8 {
    let d = cell.depth();
    // Subdivide all cells at least one time.
    // TODO: use a single subdivision number computed from the current cells inside the view
    // i.e. subdivide all cells in the view with the cell that has to be the most subdivided
    let mut num_sub = 1;
    if d < 2 {
        num_sub = 2 - d;
    }

    // Largest deformation cell among the cells of a specific depth
    let largest_center_to_vertex_dist =
        healpix::largest_center_to_vertex_distance(d, 0.0, healpix::TRANSITION_LATITUDE);
    let smallest_center_to_vertex_dist =
        healpix::largest_center_to_vertex_distance(d, 0.0, healpix::LAT_OF_SQUARE_CELL);

    let (lon, lat) = cell.center();
    let center_to_vertex_dist = healpix::largest_center_to_vertex_distance(d, lon, lat);

    let skewed_factor = (center_to_vertex_dist - smallest_center_to_vertex_dist)
        / (largest_center_to_vertex_dist - smallest_center_to_vertex_dist);

    if skewed_factor > 0.25 || is_too_large(cell, camera, projection) || cell.is_on_pole() {
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

pub fn get_raster_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    if config.get_format().is_colored() && cmap.label() == "native" {
        crate::shader::get_shader(
            gl,
            shaders,
            "hips_rasterizer_raster.vert",
            "hips_rasterizer_color.frag",
        )
    } else {
        if config.tex_storing_unsigned_int {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_rasterizer_raster.vert",
                "hips_rasterizer_grayscale_to_colormap_u.frag",
            )
        } else if config.tex_storing_integers {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_rasterizer_raster.vert",
                "hips_rasterizer_grayscale_to_colormap_i.frag",
            )
        } else {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_rasterizer_raster.vert",
                "hips_rasterizer_grayscale_to_colormap.frag",
            )
        }
    }
}

pub fn get_raytracer_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    //let colored_hips = config.is_colored();
    if config.get_format().is_colored() && cmap.label() == "native" {
        crate::shader::get_shader(
            gl,
            shaders,
            "hips_raytracer_raytracer.vert",
            "hips_raytracer_color.frag",
        )
    } else {
        if config.tex_storing_unsigned_int {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_raytracer_raytracer.vert",
                "hips_raytracer_grayscale_to_colormap_u.frag",
            )
        } else if config.tex_storing_integers {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_raytracer_raytracer.vert",
                "hips_raytracer_grayscale_to_colormap_i.frag",
            )
        } else {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_raytracer_raytracer.vert",
                "hips_raytracer_grayscale_to_colormap.frag",
            )
        }
    }
}

pub struct HiPS {
    //color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,

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

    //min_depth_tile: u8,
    footprint_moc: Option<HEALPixCoverage>,

    // A buffer storing the cells in the view
    hpx_cells_in_view: Vec<HEALPixCell>,
}

impl HiPS {
    pub fn new(config: HiPSConfig, gl: &WebGlContext) -> Result<Self, JsValue> {
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
        let textures = ImageSurveyTextures::new(gl, config)?;

        let gl = gl.clone();
        let footprint_moc = None;
        let hpx_cells_in_view = vec![];
        // request the allsky texture
        Ok(HiPS {
            // The image survey texture buffer
            textures,
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

            footprint_moc,
            hpx_cells_in_view,
        })
    }

    pub fn look_for_new_tiles<'a>(
        &'a mut self,
        camera: &'a CameraViewPort,
        proj: &ProjectionType,
    ) -> Option<impl Iterator<Item = HEALPixCell> + 'a> {
        // do not add tiles if the view is already at depth 0
        let mut depth_tile = (camera.get_texture_depth() + self.get_config().delta_depth())
            .min(self.get_config().get_max_depth_tile())
            .max(self.get_config().get_min_depth_tile());
        let dd = self.get_config().delta_depth();

        //let min_depth_tile = self.get_min_depth_tile();
        //let delta_depth = self.get_config().delta_depth();

        //let min_bound_depth = min_depth_tile.max(delta_depth);
        // do not ask to query tiles that:
        // * either do not exist because < to min_depth_tile
        // * either are part of a base tile already handled i.e. tiles < delta_depth
        //console_log(depth_tile);
        //console_log(min_bound_depth);

        //if depth_tile >= min_bound_depth {
        //let depth_tile = depth_tile.max(min_bound_depth);
        let survey_frame = self.get_config().get_frame();
        let mut already_considered_tiles = HashSet::new();

        // raytracer is rendering and the shader only renders HPX texture cells of depth 0
        if camera.is_raytracing(proj) {
            depth_tile = 0;
        }

        let tile_cells_iter = camera
            .get_hpx_cells(depth_tile, survey_frame)
            //.flat_map(move |cell| {
            //    let texture_cell = cell.get_texture_cell(delta_depth);
            //    texture_cell.get_tile_cells(delta_depth)
            //})
            .into_iter()
            .flat_map(move |tile_cell| {
                let tex_cell = tile_cell.get_texture_cell(dd);
                tex_cell.get_tile_cells(dd)
            })
            .filter(move |tile_cell| {
                if already_considered_tiles.contains(tile_cell) {
                    return false;
                }

                already_considered_tiles.insert(*tile_cell);

                if let Some(moc) = self.footprint_moc.as_ref() {
                    moc.intersects_cell(tile_cell) && !self.update_priority_tile(tile_cell)
                } else {
                    !self.update_priority_tile(tile_cell)
                }
            });

        /*if depth_tile >= min_depth_tile + 3 {
            // Retrieve the grand-grand parent cells but not if it is root ones as it may interfere with already done requests
            let tile_cells_ancestor_iter =
                (&tile_cells_iter).map(|tile_cell| tile_cell.ancestor(3));

            tile_cells_iter.chain(tile_cells_ancestor_iter);
        }*/

        /*let tile_cells: HashSet<_> = if let Some(moc) = survey.get_moc() {
            tile_cells_iter
                .filter(|tile_cell| moc.intersects_cell(tile_cell))
                .collect()
        } else {
            tile_cells_iter.collect()
        };*/

        Some(tile_cells_iter)
        //} else {
        //    None
        //}
    }

    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        self.textures.contains_tile(cell)
    }

    pub fn update(&mut self, camera: &mut CameraViewPort, projection: &ProjectionType) {
        let raytracing = camera.is_raytracing(projection);

        if raytracing {
            return;
        }

        // rasterizer mode
        let available_tiles = self.textures.reset_available_tiles();
        let new_cells_in_view = self.retrieve_cells_in_camera(camera);

        if new_cells_in_view || available_tiles {
            self.recompute_vertices(camera, projection);
        }
    }

    // returns a boolean if the view cells has changed with respect to the last frame
    pub fn retrieve_cells_in_camera(&mut self, camera: &CameraViewPort) -> bool {
        let cfg = self.textures.config();
        // Get the coo system transformation matrix
        let hips_frame = cfg.get_frame();
        let depth = camera.get_texture_depth().min(cfg.get_max_depth_texture());

        let hpx_cells_in_view = camera.get_hpx_cells(depth, hips_frame);
        let new_cells = if hpx_cells_in_view.len() != self.hpx_cells_in_view.len() {
            true
        } else {
            !self
                .hpx_cells_in_view
                .iter()
                .zip(hpx_cells_in_view.iter())
                .all(|(&a, &b)| a == b)
        };

        self.hpx_cells_in_view = hpx_cells_in_view;

        new_cells
    }

    #[inline]
    pub fn set_moc(&mut self, moc: HEALPixCoverage) {
        self.footprint_moc = Some(moc);
    }

    #[inline]
    pub fn get_moc(&self) -> Option<&HEALPixCoverage> {
        self.footprint_moc.as_ref()
    }

    pub fn set_img_format(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        self.textures.set_format(&self.gl, ext)
    }

    pub fn is_allsky(&self) -> bool {
        self.textures.config().is_allsky
    }

    // Position given is in the camera space
    pub fn read_pixel(
        &self,
        pos: &LonLatT<f64>,
        camera: &CameraViewPort,
    ) -> Result<JsValue, JsValue> {
        // 1. Convert it to the hips frame system
        let cfg = self.textures.config();
        let camera_frame = camera.get_coo_system();
        let hips_frame = cfg.get_frame();

        let pos = crate::coosys::apply_coo_system(camera_frame, hips_frame, &pos.vector());

        // Get the array of textures from that survey
        let tile_depth = camera.get_texture_depth().min(cfg.get_max_depth_texture());

        let pos_tex = self
            .textures
            .get_pixel_position_in_texture(&pos.lonlat(), tile_depth)?;

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

    pub fn recompute_vertices(&mut self, camera: &mut CameraViewPort, projection: &ProjectionType) {
        self.position.clear();
        self.uv_start.clear();
        self.uv_end.clear();
        self.time_tile_received.clear();
        self.m0.clear();
        self.m1.clear();
        self.idx_vertices.clear();

        let cfg = self.textures.config();
        // Get the coo system transformation matrix
        let channel = cfg.get_format().get_channel();

        // Retrieve the model and inverse model matrix
        let mut off_indices = 0;

        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell = if let Some(moc) = self.footprint_moc.as_ref() {
                if moc.intersects_cell(&cell) {
                    Some(&cell)
                } else {
                    if channel == ChannelType::RGB8U {
                        // Rasterizer does not render tiles that are not in the MOC
                        // This is not a problem for transparency rendered HiPses (FITS or PNG)
                        // but JPEG tiles do have black when no pixels data is found
                        // We therefore must draw in black for the tiles outside the HiPS MOC
                        Some(&cell)
                    } else {
                        None
                    }
                }
            } else {
                Some(&cell)
            };

            if let Some(cell) = cell {
                let texture_to_draw = if self.textures.contains(cell) {
                    if let Some(ending_cell_in_tex) = self.textures.get(cell) {
                        if let Some(parent_cell) = self.textures.get_nearest_parent(cell) {
                            if let Some(starting_cell_in_tex) = self.textures.get(&parent_cell) {
                                Some(TextureToDraw::new(
                                    starting_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            } else {
                                // no blending here
                                Some(TextureToDraw::new(
                                    ending_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            }
                        } else {
                            Some(TextureToDraw::new(
                                ending_cell_in_tex,
                                ending_cell_in_tex,
                                cell,
                            ))
                        }
                    } else {
                        None
                    }
                } else {
                    if let Some(parent_cell) = self.textures.get_nearest_parent(cell) {
                        if let Some(ending_cell_in_tex) = self.textures.get(&parent_cell) {
                            if let Some(grand_parent_cell) =
                                self.textures.get_nearest_parent(&parent_cell)
                            {
                                if let Some(starting_cell_in_tex) =
                                    self.textures.get(&grand_parent_cell)
                                {
                                    Some(TextureToDraw::new(
                                        starting_cell_in_tex,
                                        ending_cell_in_tex,
                                        cell,
                                    ))
                                } else {
                                    // no blending
                                    Some(TextureToDraw::new(
                                        ending_cell_in_tex,
                                        ending_cell_in_tex,
                                        cell,
                                    ))
                                }
                            } else {
                                Some(TextureToDraw::new(
                                    ending_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            }
                        } else {
                            unreachable!();
                        }
                    } else {
                        None
                    }
                };

                if let Some(TextureToDraw {
                    cell,
                    starting_texture,
                    ending_texture,
                }) = texture_to_draw
                {
                    let uv_0 = TileUVW::new(cell, starting_texture, cfg);
                    let uv_1 = TileUVW::new(cell, ending_texture, cfg);
                    let start_time = ending_texture.start_time().as_millis();

                    let miss_0 = (false) as i32 as f32;
                    let miss_1 = (false) as i32 as f32;

                    let num_subdivision = num_subdivision(cell, camera, projection);

                    let n_segments_by_side: usize = 1 << (num_subdivision as usize);
                    let n_segments_by_side_f32 = n_segments_by_side as f32;

                    let n_vertices_per_segment = n_segments_by_side + 1;

                    let mut pos = Vec::with_capacity((n_segments_by_side + 1) * 4);

                    let grid_lonlat =
                        healpix::nested::grid(cell.depth(), cell.idx(), n_segments_by_side as u16);
                    let grid_lonlat_iter = grid_lonlat.iter();

                    for (idx, &(lon, lat)) in grid_lonlat_iter.enumerate() {
                        //let xyzw = crate::math::lonlat::radec_to_xyzw(lon, lat);
                        //let xyzw =
                        //    crate::coosys::apply_coo_system(hips_frame, selected_frame, &xyzw);

                        //let ndc = projection
                        //    .model_to_normalized_device_space(&xyzw, camera)
                        //    .map(|v| [v.x as f32, v.y as f32]);

                        let i: usize = idx / n_vertices_per_segment;
                        let j: usize = idx % n_vertices_per_segment;

                        let hj0 = (j as f32) / n_segments_by_side_f32;
                        let hi0 = (i as f32) / n_segments_by_side_f32;

                        let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
                        let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;
                        let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                        let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;

                        let uv_start = [
                            uv_0[TileCorner::BottomLeft].x + hj0 * d01s,
                            uv_0[TileCorner::BottomLeft].y + hi0 * d02s,
                            uv_0[TileCorner::BottomLeft].z,
                        ];

                        let uv_end = [
                            uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                            uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                            uv_1[TileCorner::BottomLeft].z,
                        ];

                        self.uv_start.extend(uv_start);
                        self.uv_end.extend(uv_end);
                        self.m0.push(miss_0);
                        self.m1.push(miss_1);
                        self.time_tile_received.push(start_time);

                        pos.push([lon as f32, lat as f32]);
                    }

                    let patch_indices_iter = DefaultPatchIndexIter::new(
                        &(0..=n_segments_by_side),
                        &(0..=n_segments_by_side),
                        n_vertices_per_segment,
                    )
                    .flatten()
                    .map(|indices| {
                        [
                            indices.0 + off_indices,
                            indices.1 + off_indices,
                            indices.2 + off_indices,
                        ]
                    })
                    .flatten();
                    self.idx_vertices.extend(patch_indices_iter);

                    off_indices += pos.len() as u16;

                    // Replace options with an arbitrary vertex
                    let position_iter = pos
                        .into_iter()
                        //.map(|ndc| ndc.unwrap_or([0.0, 0.0]))
                        .flatten();
                    self.position.extend(position_iter);
                }
            }
        }

        self.num_idx = self.idx_vertices.len();

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
        image: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.textures.push(&cell, image, time_request)
    }

    pub fn add_allsky(&mut self, allsky: Allsky) -> Result<(), JsValue> {
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
    pub fn get_ready_time(&self) -> &Option<Time> {
        &self.textures.start_time
    }

    pub fn draw(
        &self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &CameraViewPort,
        raytracer: &RayTracer,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        // Get the coo system transformation matrix
        let selected_frame = camera.get_coo_system();
        let hips_cfg = self.textures.config();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(hips_frame);

        let raytracing = camera.is_raytracing(proj);
        let config = self.get_config();

        self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        // Add starting fading
        //let fading = self.get_fading_factor();
        //let opacity = opacity * fading;
        // Get the colormap from the color
        let cmap = colormaps.get(color.cmap_name.as_ref());

        blend_cfg.enable(&self.gl, || {
            if raytracing {
                let w2v = c * (*camera.get_w2m());

                let shader = get_raytracer_shader(cmap, &self.gl, shaders, &config)?;

                let shader = shader.bind(&self.gl);
                shader
                    .attach_uniforms_from(camera)
                    .attach_uniforms_from(&self.textures)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cmap, colormaps)
                    .attach_uniforms_from(color)
                    .attach_uniform("model", &w2v)
                    .attach_uniform("current_time", &utils::get_current_time())
                    .attach_uniform("opacity", opacity)
                    .attach_uniforms_from(colormaps);

                raytracer.draw(&shader);
            } else {
                let v2w = (*camera.get_m2w()) * c.transpose();

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
                let shader = get_raster_shader(cmap, &self.gl, shaders, &config)?.bind(&self.gl);

                shader
                    .attach_uniforms_from(&self.textures)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cmap, colormaps)
                    .attach_uniforms_from(color)
                    .attach_uniforms_from(camera)
                    .attach_uniform("inv_model", &v2w)
                    .attach_uniform("current_time", &utils::get_current_time())
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("u_proj", proj)
                    .attach_uniforms_from(colormaps)
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(self.num_idx as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        0,
                    );
            }

            Ok(())
        })?;

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
