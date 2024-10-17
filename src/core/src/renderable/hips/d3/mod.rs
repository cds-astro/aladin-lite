pub mod buffer;
pub mod texture;

use crate::renderable::hips::HpxTile;
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

use crate::shader::ShaderManager;
use crate::{math::lonlat::LonLatT, utils};

use crate::downloader::request::allsky::Allsky;
use crate::healpix::{cell::HEALPixCell, coverage::HEALPixCoverage};
use crate::renderable::utils::index_patch::DefaultPatchIndexIter;
use crate::time::Time;

use super::config::HiPSConfig;
use std::collections::HashSet;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use super::d2::texture::HpxTexture2D;
use buffer::HiPS3DBuffer;

use super::raytracing::RayTracer;
use super::uv::{TileCorner, TileUVW};

use cgmath::Matrix;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

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
            "hips3d_rasterizer_raster.vert",
            "hips3d_rasterizer_color.frag",
        )
    } else {
        if config.tex_storing_unsigned_int {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips3d_rasterizer_raster.vert",
                "hips3d_rasterizer_grayscale_to_colormap_u.frag",
            )
        } else if config.tex_storing_integers {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips3d_rasterizer_raster.vert",
                "hips3d_rasterizer_grayscale_to_colormap_i.frag",
            )
        } else {
            crate::shader::get_shader(
                gl,
                shaders,
                "hips3d_rasterizer_raster.vert",
                "hips3d_rasterizer_grayscale_to_colormap.frag",
            )
        }
    }
}

/*
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
}*/

pub struct HiPS3D {
    //color: Color,
    // The image survey texture buffer
    buffer: HiPS3DBuffer,

    // The projected vertices data
    // For WebGL2 wasm, the data are interleaved
    //#[cfg(feature = "webgl2")]
    //vertices: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 0) in vec3 position;
    position: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 1) in vec3 uv_start;
    uv: Vec<f32>,
    idx_vertices: Vec<u16>,

    num_idx: usize,

    vao: VertexArrayObject,
    gl: WebGlContext,

    footprint_moc: Option<HEALPixCoverage>,

    // A buffer storing the cells in the view
    hpx_cells_in_view: Vec<HEALPixCell>,

    // The current slice index
    slice: u16,
}

use super::HpxTileBuffer;

impl HiPS3D {
    pub fn new(config: HiPSConfig, gl: &WebGlContext) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

        let slice = 0;

        // layout (location = 0) in vec2 lonlat;
        // layout (location = 1) in vec3 position;
        // layout (location = 2) in vec3 uv_start;
        // layout (location = 3) in vec3 uv_end;
        // layout (location = 4) in float time_tile_received;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];

        //let vertices = vec![];
        let position = vec![];
        let uv = vec![];
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
                "uv",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            )
            .unbind();

        let num_idx = 0;
        let buffer = HiPS3DBuffer::new(gl, config)?;

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
            uv,
            idx_vertices,

            footprint_moc,
            hpx_cells_in_view,

            slice,
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
                    moc.intersects_cell(tile_cell)
                } else {
                    true
                }
            });

        Some(tile_cells_iter)
    }

    pub fn set_slice(&mut self, slice: u16) {
        self.slice = slice;
    }

    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        self.buffer.contains(cell)
    }

    pub fn draw(
        &mut self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &mut CameraViewPort,
        raytracer: &RayTracer,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        let raytracing = camera.is_raytracing(proj);

        if raytracing {
            self.draw_internal(shaders, colormaps, camera, raytracer, cfg, proj)
        } else {
            // rasterizer mode
            let available_tiles = self.reset_available_tiles();
            let new_cells_in_view = self.retrieve_cells_in_camera(camera);

            if new_cells_in_view || available_tiles {
                self.draw_internal(shaders, colormaps, camera, raytracer, cfg, proj)
            } else {
                Ok(())
            }
        }
    }

    fn reset_available_tiles(&mut self) -> bool {
        self.buffer.reset_available_tiles()
    }

    // returns a boolean if the view cells has changed with respect to the last frame
    fn retrieve_cells_in_camera(&mut self, camera: &CameraViewPort) -> bool {
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

    fn draw_internal(
        &mut self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &mut CameraViewPort,
        raytracer: &RayTracer,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        let hips_cfg = self.buffer.config();
        // Get the coo system transformation matrix
        let channel = hips_cfg.get_format().get_channel();

        // Get the coo system transformation matrix
        let selected_frame = camera.get_coo_system();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(hips_frame);

        let raytracing = camera.is_raytracing(proj);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        let cmap = colormaps.get(color.cmap_name.as_ref());

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

            let mut slice_contained = 0;
            if let Some(cell) = cell {
                let hpx_cell_texture = if self.buffer.contains_slice(cell, self.slice) {
                    slice_contained = self.slice;
                    if let Some(texture) = self.buffer.get(cell) {
                        Some(texture)
                    } else {
                        unreachable!();
                    }
                } else {
                    if let Some(parent_cell) = self.buffer.get_nearest_parent(cell) {
                        // find the slice of the parent available, if possible near slice
                        slice_contained = self.buffer.find_nearest_slice(cell, self.slice).unwrap();

                        if let Some(texture) = self.buffer.get(&parent_cell) {
                            Some(texture)
                        } else {
                            unreachable!();
                        }
                    } else {
                        None
                    }
                };

                if let Some(texture) = hpx_cell_texture {
                    self.position.clear();
                    self.uv.clear();
                    self.idx_vertices.clear();
                    let mut off_indices = 0;

                    // The slice is sure to be contained so we can unwrap
                    let hpx_slice_tex = texture.extract_2d_slice_texture(slice_contained).unwrap();

                    let uv_1 = TileUVW::new(cell, &hpx_slice_tex, hips_cfg);
                    let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                    let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;

                    let num_subdivision =
                        super::subdivide::num_hpxcell_subdivision(cell, camera, proj);

                    let n_segments_by_side: usize = 1 << (num_subdivision as usize);
                    let n_segments_by_side_f32 = n_segments_by_side as f32;

                    let n_vertices_per_segment = n_segments_by_side + 1;

                    let mut pos = Vec::with_capacity((n_segments_by_side + 1) * 4);

                    let grid_lonlat =
                        healpix::nested::grid(cell.depth(), cell.idx(), n_segments_by_side as u16);
                    let grid_lonlat_iter = grid_lonlat.iter();

                    for (idx, &(lon, lat)) in grid_lonlat_iter.enumerate() {
                        let i: usize = idx / n_vertices_per_segment;
                        let j: usize = idx % n_vertices_per_segment;

                        let hj0 = (j as f32) / n_segments_by_side_f32;
                        let hi0 = (i as f32) / n_segments_by_side_f32;

                        let uv_end = [
                            uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                            uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                            uv_1[TileCorner::BottomLeft].z,
                        ];

                        self.uv.extend(uv_end);

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

                    {
                        let mut vao = self.vao.bind_for_update();
                        vao.update_array(
                            "position",
                            WebGl2RenderingContext::DYNAMIC_DRAW,
                            VecData(&self.position),
                        )
                        .update_array(
                            "uv",
                            WebGl2RenderingContext::DYNAMIC_DRAW,
                            VecData(&self.uv),
                        )
                        .update_element_array(
                            WebGl2RenderingContext::DYNAMIC_DRAW,
                            VecData(&self.idx_vertices),
                        );
                    }

                    let num_idx = self.idx_vertices.len();

                    blend_cfg.enable(&self.gl, || {
                        if raytracing {
                            /*let w2v = c * (*camera.get_w2m());

                            let shader = get_raytracer_shader(cmap, &self.gl, shaders, &cfg)?;

                            let shader = shader.bind(&self.gl);
                            shader
                                .attach_uniforms_from(camera)
                                .attach_uniforms_from(&self.buffer)
                                // send the cmap appart from the color config
                                .attach_uniforms_with_params_from(cmap, colormaps)
                                .attach_uniforms_from(color)
                                .attach_uniform("model", &w2v)
                                .attach_uniform("current_time", &utils::get_current_time())
                                .attach_uniform("opacity", opacity)
                                .attach_uniforms_from(colormaps);

                            raytracer.draw(&shader);*/
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
                            let shader = get_raster_shader(cmap, &self.gl, shaders, &hips_cfg)?
                                .bind(&self.gl);

                            shader
                                //.attach_uniforms_from(self.buffer)
                                // send the cmap appart from the color config
                                .attach_uniform(
                                    "tex",
                                    texture.get_3d_block_from_slice(self.slice).unwrap(),
                                )
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
                                    Some(num_idx as i32),
                                    WebGl2RenderingContext::UNSIGNED_SHORT,
                                    0,
                                );
                        }

                        Ok(())
                    })?;
                }
            }
        }

        Ok(())
    }

    pub fn add_tile<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
        slice_idx: u16,
    ) -> Result<(), JsValue> {
        self.buffer.push(&cell, image, time_request, slice_idx)
    }

    pub fn add_allsky(&mut self, allsky: Allsky, slice_idx: u16) -> Result<(), JsValue> {
        self.buffer.push_allsky(allsky, slice_idx)
    }

    #[inline]
    pub fn get_slice(&self) -> u16 {
        self.slice
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
}
