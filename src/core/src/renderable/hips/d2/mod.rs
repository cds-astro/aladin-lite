pub mod buffer;
pub mod texture;

use crate::app::BLENDING_ANIM_DURATION;
use crate::renderable::hips::HpxTile;
use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
use al_core::colormap::Colormap;
use al_core::colormap::Colormaps;
use al_core::image::format::ChannelType;
use cgmath::Vector3;

use crate::math::angle::ToAngle;
use crate::downloader::query;

use al_core::image::Image;

use al_core::shader::Shader;
use al_core::webgl_ctx::GlWrapper;
use cgmath::Vector4;

use al_core::VecData;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use crate::ProjectionType;

use crate::camera::CameraViewPort;

use crate::shader::ShaderManager;
use crate::{math::lonlat::LonLatT, utils};

use crate::downloader::request::allsky::Allsky;
use crate::healpix::{cell::HEALPixCell, coverage::HEALPixCoverage};
use crate::time::Time;

use super::config::HiPSConfig;
use std::collections::HashSet;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use buffer::HiPS2DBuffer;
use texture::HpxTexture2D;

use super::raytracing::RayTracer;
use super::uv::{TileCorner, TileUVW};

use cgmath::Matrix;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub struct HpxDrawData<'a> {
    pub uv_0: TileUVW,
    pub uv_1: TileUVW,
    pub start_time: f32,
    pub cell: &'a HEALPixCell,
}

impl<'a> HpxDrawData<'a> {
    fn from_texture(
        starting_texture: &HpxTexture2D,
        ending_texture: &HpxTexture2D,
        cell: &'a HEALPixCell,
    ) -> Self {
        let uv_0 = TileUVW::new(cell, starting_texture);
        let uv_1 = TileUVW::new(cell, ending_texture);
        let start_time = ending_texture.start_time().as_millis();

        Self {
            uv_0,
            uv_1,
            start_time,
            cell,
        }
    }

    fn new(cell: &'a HEALPixCell) -> Self {
        let uv_0 = TileUVW([Vector3::new(-1.0, -1.0, -1.0); 4]);
        let uv_1 = TileUVW([Vector3::new(-1.0, -1.0, -1.0); 4]);
        let start_time = BLENDING_ANIM_DURATION.as_millis();

        Self {
            cell, uv_0, uv_1, start_time
        }
    }
}

pub fn get_raster_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    if config.get_format().is_colored() {
        if cmap.label() == "native" {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_rasterizer_raster.vert",
                "hips_rasterizer_color.frag",
            )
        } else {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_rasterizer_raster.vert",
                "hips_rasterizer_color_to_colormap.frag",
            )
        }
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
    if config.get_format().is_colored() {
        if cmap.label() == "native" {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_raytracer_raytracer.vert",
                "hips_raytracer_color.frag",
            )
        } else {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips_raytracer_raytracer.vert",
                "hips_raytracer_color_to_colormap.frag",
            )
        }
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

pub struct HiPS2D {
    //color: Color,
    // The image survey texture buffer
    buffer: HiPS2DBuffer,

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

    idx_vertices: Vec<u16>,

    num_idx: usize,

    vao: VertexArrayObject,
    gl: WebGlContext,

    footprint_moc: Option<HEALPixCoverage>,

    // A buffer storing the cells in the view
    hpx_cells_in_view: Vec<HEALPixCell>,
}

use super::HpxTileBuffer;

impl HiPS2D {
    pub fn new(config: HiPSConfig, gl: &WebGlContext) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

        // layout (location = 0) in vec2 lonlat;
        // layout (location = 1) in vec3 position;
        // layout (location = 2) in vec3 uv_start;
        // layout (location = 3) in vec3 uv_end;
        // layout (location = 4) in float time_tile_received;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];

        //let vertices = vec![];
        let position = vec![];
        let uv_start = vec![];
        let uv_end = vec![];
        let time_tile_received = vec![];
        let idx_vertices = vec![];

        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer_single(
                3,
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
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            )
            .unbind();

        let num_idx = 0;
        let buffer = HiPS2DBuffer::new(gl, config)?;

        let gl = gl.clone();
        let footprint_moc = None;
        let hpx_cells_in_view = vec![];
        // request the allsky texture
        Ok(Self {
            // The image survey texture buffer
            buffer,
            num_idx,

            vao,

            gl,

            position,
            uv_start,
            uv_end,
            time_tile_received,

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
        let cfg = self.get_config();
        let mut depth_tile = (camera.get_texture_depth() + cfg.delta_depth())
            .min(cfg.get_max_depth_tile())
            .max(cfg.get_min_depth_tile());
        let dd = cfg.delta_depth();

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
        let survey_frame = cfg.get_frame();
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

        Some(tile_cells_iter)
    }

    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        self.buffer.contains_tile(cell)
    }

    pub fn get_tile_query(&self, cell: &HEALPixCell) -> query::Tile {
        let cfg = self.get_config();
        query::Tile::new(cell, None, cfg)
    }

    pub fn update(&mut self, camera: &mut CameraViewPort, projection: &ProjectionType) {
        let raytracing = camera.is_raytracing(projection);

        if raytracing {
            return;
        }

        // rasterizer mode
        let available_tiles = self.buffer.reset_available_tiles();
        let new_cells_in_view = self.retrieve_cells_in_camera(camera);

        if new_cells_in_view || available_tiles {
            self.recompute_vertices(camera, projection);
        }
    }

    // returns a boolean if the view cells has changed with respect to the last frame
    pub fn retrieve_cells_in_camera(&mut self, camera: &CameraViewPort) -> bool {
        let cfg = self.get_config();
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

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        self.buffer.set_image_ext(&self.gl, ext)
    }

    pub fn is_allsky(&self) -> bool {
        self.buffer.config().is_allsky
    }

    // Position given is in the camera space
    pub fn read_pixel(
        &self,
        p: &LonLatT<f64>,
        camera: &CameraViewPort,
    ) -> Result<JsValue, JsValue> {
        self.buffer.read_pixel(p, camera)
    }

    fn recompute_vertices(&mut self, camera: &mut CameraViewPort, projection: &ProjectionType) {
        self.position.clear();
        self.uv_start.clear();
        self.uv_end.clear();
        self.time_tile_received.clear();
        self.idx_vertices.clear();

        let cfg = self.buffer.config();
        // Get the coo system transformation matrix
        let channel = cfg.get_format().get_channel();

        // Retrieve the model and inverse model matrix
        let mut off_indices = 0;
        // Define a global level of subdivisions for all the healpix tile cells in the view
        // This should prevent seeing many holes
        // We compute it from the first cell in the view but it might be an under/over estimate for the other cells in the view
        let num_sub = self.hpx_cells_in_view.iter()
            .map(|cell| super::subdivide::num_hpx_subdivision(cell, camera, projection))
            .max().unwrap();

        //let num_sub =
        //    super::subdivide::num_hpx_subdivision(&self.hpx_cells_in_view[0], camera, projection);
        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell_in_cov = if let Some(moc) = self.footprint_moc.as_ref() {
                if moc.intersects_cell(&cell) {
                    // Rasterizer does not render tiles that are not in the MOC
                    // This is not a problem for transparency rendered HiPses (FITS or PNG)
                    // but JPEG tiles do have black when no pixels data is found
                    // We therefore must draw in black for the tiles outside the HiPS MOC
                    Some(&cell)
                } else {
                    None
                }
            } else {
                Some(&cell)
            };

            let hpx_cell = if let Some(cell) = cell_in_cov {
                if self.buffer.contains(cell) {
                    if let Some(ending_cell_in_tex) = self.buffer.get(cell) {
                        if let Some(parent_cell) = self.buffer.get_nearest_parent(cell) {
                            if let Some(starting_cell_in_tex) = self.buffer.get(&parent_cell) {
                                Some(HpxDrawData::from_texture(
                                    starting_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            } else {
                                // no blending here
                                Some(HpxDrawData::from_texture(
                                    ending_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            }
                        } else {
                            Some(HpxDrawData::from_texture(
                                ending_cell_in_tex,
                                ending_cell_in_tex,
                                cell,
                            ))
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    if let Some(parent_cell) = self.buffer.get_nearest_parent(cell) {
                        if let Some(ending_cell_in_tex) = self.buffer.get(&parent_cell) {
                            if let Some(grand_parent_cell) =
                                self.buffer.get_nearest_parent(&parent_cell)
                            {
                                if let Some(starting_cell_in_tex) =
                                    self.buffer.get(&grand_parent_cell)
                                {
                                    Some(HpxDrawData::from_texture(
                                        starting_cell_in_tex,
                                        ending_cell_in_tex,
                                        cell,
                                    ))
                                } else {
                                    // no blending
                                    Some(HpxDrawData::from_texture(
                                        ending_cell_in_tex,
                                        ending_cell_in_tex,
                                        cell,
                                    ))
                                }
                            } else {
                                Some(HpxDrawData::from_texture(
                                    ending_cell_in_tex,
                                    ending_cell_in_tex,
                                    cell,
                                ))
                            }
                        } else {
                            unreachable!()
                        }
                    } else {
                        // No ancestor has been found in the buffer to draw.
                        // We might want to check if the HiPS channel is JPEG to mock a cell that will be drawn in black
                        if channel == ChannelType::RGB8U {
                            Some(HpxDrawData::new(cell))
                        } else {
                            None
                        }
                    }
                }
            } else {
                // No ancestor has been found in the buffer to draw.
                // We might want to check if the HiPS channel is JPEG to mock a cell that will be drawn in black
                if channel == ChannelType::RGB8U {
                    Some(HpxDrawData::new(cell))
                } else {
                    None
                }
            };

            if let Some(HpxDrawData {
                cell,
                uv_0,
                uv_1,
                start_time
            }) = hpx_cell {
                let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
                let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;
                let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;

                let sub_cells =
                    super::subdivide::subdivide_hpx_cell(cell, num_sub, camera);

                let mut pos = Vec::with_capacity(sub_cells.len() * 4);

                let mut idx = 0;

                for sub_cell in sub_cells {
                    let (i, j) = sub_cell.offset_in_parent(cell);
                    let nside = (1 << (sub_cell.depth() - cell.depth())) as f32;

                    for ((lon, lat), (di, dj)) in
                        sub_cell
                            .vertices()
                            .iter()
                            .zip([(0, 0), (1, 0), (1, 1), (0, 1)])
                    {
                        let hj0 = ((j + dj) as f32) / nside;
                        let hi0 = ((i + di) as f32) / nside;

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
                        self.time_tile_received.push(start_time);

                        let xyz = crate::math::lonlat::radec_to_xyz(lon.to_angle(), lat.to_angle());
                        pos.push([xyz.x as f32, xyz.y as f32, xyz.z as f32]);
                    }

                    // GL TRIANGLES
                    self.idx_vertices.extend([
                        idx + off_indices,
                        idx + 1 + off_indices,
                        idx + 2 + off_indices,
                        idx + off_indices,
                        idx + 2 + off_indices,
                        idx + 3 + off_indices,
                    ]);
                    // GL LINES
                    /*self.idx_vertices.extend([
                        idx + off_indices,
                        idx + 1 + off_indices,
                        idx + 1 + off_indices,
                        idx + 2 + off_indices,
                        idx + 2 + off_indices,
                        idx + 3 + off_indices,
                        idx + 3 + off_indices,
                        idx + off_indices,
                    ]);*/

                    idx += 4;
                }

                off_indices += pos.len() as u16;

                // Replace options with an arbitrary vertex
                let position_iter = pos
                    .into_iter()
                    //.map(|ndc| ndc.unwrap_or([0.0, 0.0]))
                    .flatten();
                self.position.extend(position_iter);
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
        .update_element_array(
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.idx_vertices),
        );
    }

    // Return a boolean to signal if the tile is present or not in the survey
    pub fn update_priority_tile(&mut self, cell: &HEALPixCell) -> bool {
        if self.buffer.contains_tile(cell) {
            // The cell is present in the survey, we update its priority
            self.buffer.update_priority(cell);
            true
        } else {
            false
        }
    }

    pub fn add_tile<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer.push(&cell, image, time_request)
    }

    pub fn add_allsky(&mut self, allsky: Allsky) -> Result<(), JsValue> {
        self.buffer.push_allsky(allsky)
    }

    /* Accessors */
    #[inline]
    pub fn get_config(&self) -> &HiPSConfig {
        self.buffer.config()
    }

    #[inline]
    pub fn get_config_mut(&mut self) -> &mut HiPSConfig {
        self.buffer.config_mut()
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
        let hips_cfg = self.buffer.config();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(hips_frame);

        let raytracing = camera.is_raytracing(proj);
        let config = self.get_config();

        //self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        // Get the colormap from the color
        let cmap = colormaps.get(color.cmap_name.as_ref());

        blend_cfg.enable(&self.gl, || {
            if raytracing {
                let w2v = c * (*camera.get_w2m());

                let shader = get_raytracer_shader(cmap, &self.gl, shaders, &config)?;

                let shader = shader.bind(&self.gl);
                shader
                    .attach_uniforms_from(camera)
                    .attach_uniforms_from(&self.buffer)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cmap, colormaps)
                    .attach_uniforms_from(color)
                    .attach_uniform("model", &w2v)
                    .attach_uniform("current_time", &utils::get_current_time())
                    .attach_uniform("no_tile_color",  &(if config.get_format().get_channel() == ChannelType::RGB8U { Vector4::new(0.0, 0.0, 0.0, 1.0) } else { Vector4::new(0.0, 0.0, 0.0, 0.0) }))
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
                    .attach_uniforms_from(&self.buffer)
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
                        //WebGl2RenderingContext::LINES,
                        Some(self.num_idx as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        0,
                    );
            }

            Ok(())
        })?;

        //self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
