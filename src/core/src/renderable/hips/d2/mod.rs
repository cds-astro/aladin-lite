pub mod buffer;
pub mod texture;

use crate::app::BLENDING_ANIM_DURATION;
use crate::browser_support::BrowserFeaturesSupport;
use crate::downloader::query;
use crate::downloader::query::CellDesc;
use crate::downloader::request::allsky::AllskyRequest;
use crate::math::angle::ToAngle;
use crate::tile_fetcher::TileFetcherQueue;
use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
use al_core::colormap::Colormap;
use al_core::colormap::Colormaps;
use al_core::texture::format::PixelType;
use cgmath::Vector2;
use cgmath::Vector3;

use crate::renderable::hips::FitsParams;

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
use crate::utils;

use crate::healpix::{cell::HEALPixCell, moc::SpaceMoc};
use crate::time::Time;

use super::config::HiPSConfig;
use crate::math::lonlat::LonLat;
use std::collections::HashSet;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use buffer::HiPS2DBuffer;
use texture::HpxTex;

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
        starting_texture: &HpxTex,
        ending_texture: &HpxTex,
        cell: &'a HEALPixCell,
    ) -> Self {
        let uv_0 = TileUVW::new(
            cell,
            &Some(starting_texture.cell),
            starting_texture.idx() as f32,
        );
        let uv_1 = TileUVW::new(
            cell,
            &Some(ending_texture.cell),
            ending_texture.idx() as f32,
        );
        let start_time = ending_texture.start_time.unwrap_or(Time::now()).as_millis();

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
            cell,
            uv_0,
            uv_1,
            start_time,
        }
    }
}

pub fn get_raster_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    match config.get_format().get_pixel_format() {
        PixelType::R8U => crate::shader::get_shader(
            gl,
            shaders,
            "hips_rasterizer_raster.vert",
            "hips_rasterizer_u8.frag",
        ),
        PixelType::R16I => crate::shader::get_shader(
            gl,
            shaders,
            "hips_rasterizer_raster.vert",
            "hips_rasterizer_i16.frag",
        ),
        PixelType::R32I => crate::shader::get_shader(
            gl,
            shaders,
            "hips_rasterizer_raster.vert",
            "hips_rasterizer_i32.frag",
        ),
        PixelType::R32F => crate::shader::get_shader(
            gl,
            shaders,
            "hips_rasterizer_raster.vert",
            "hips_rasterizer_f32.frag",
        ),
        // color case
        _ => {
            if cmap.label() == "native" {
                crate::shader::get_shader(
                    gl,
                    shaders,
                    "hips_rasterizer_raster.vert",
                    "hips_rasterizer_rgba.frag",
                )
            } else {
                crate::shader::get_shader(
                    gl,
                    shaders,
                    "hips_rasterizer_raster.vert",
                    "hips_rasterizer_rgba2cmap.frag",
                )
            }
        }
    }
}

pub fn get_raytracer_shader<'a>(
    cmap: &Colormap,
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    config: &HiPSConfig,
) -> Result<&'a Shader, JsValue> {
    match config.get_format().get_pixel_format() {
        PixelType::R8U => crate::shader::get_shader(
            gl,
            shaders,
            "hips_raytracer_raytracer.vert",
            "hips_raytracer_u8.frag",
        ),
        PixelType::R16I => crate::shader::get_shader(
            gl,
            shaders,
            "hips_raytracer_raytracer.vert",
            "hips_raytracer_i16.frag",
        ),
        PixelType::R32I => crate::shader::get_shader(
            gl,
            shaders,
            "hips_raytracer_raytracer.vert",
            "hips_raytracer_i32.frag",
        ),
        PixelType::R32F => crate::shader::get_shader(
            gl,
            shaders,
            "hips_raytracer_raytracer.vert",
            "hips_raytracer_f32.frag",
        ),
        // color case
        _ => {
            if cmap.label() == "native" {
                crate::shader::get_shader(
                    gl,
                    shaders,
                    "hips_raytracer_raytracer.vert",
                    "hips_raytracer_rgba.frag",
                )
            } else {
                crate::shader::get_shader(
                    gl,
                    shaders,
                    "hips_raytracer_raytracer.vert",
                    "hips_raytracer_rgba2cmap.frag",
                )
            }
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
    //js_position: Float32Array,
    //cap: usize,
    //ptr: usize,
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
    moc: Option<SpaceMoc>,

    // A buffer storing the cells in the view
    hpx_cells_in_view: Vec<HEALPixCell>,

    pub(crate) fits_params: Option<FitsParams>,
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
        let moc = None;
        let hpx_cells_in_view = vec![];

        // request the allsky texture
        Ok(Self {
            // The image survey texture buffer
            buffer,
            num_idx,

            vao,

            gl,

            fits_params: None,

            position,
            uv_start,
            uv_end,
            time_tile_received,

            idx_vertices,

            moc,
            hpx_cells_in_view,
        })
    }

    pub fn look_for_new_tiles(
        &mut self,
        tile_fetcher: &mut TileFetcherQueue,
        camera: &CameraViewPort,
        browser_features_support: &BrowserFeaturesSupport,
    ) {
        // do not add tiles if the view is already at depth 0
        let cfg = self.get_config();
        let depth_tile = camera
            .get_tile_depth()
            .min(cfg.get_max_depth_tile())
            .max(cfg.get_min_depth_tile());

        let survey_frame = cfg.get_frame();
        let min_tile_depth = cfg.get_min_depth_tile();

        let tile_queries_iter = camera
            .get_hpx_cells(depth_tile, survey_frame)
            .into_iter()
            .filter_map(|tile_cell| {
                let make_query = if let Some(moc) = self.moc.as_ref() {
                    moc.intersects_cell(&tile_cell) && !self.update_priority_tile(&tile_cell)
                } else {
                    !self.update_priority_tile(&tile_cell)
                };

                if make_query {
                    Some(query::Tile::new(
                        &tile_cell,
                        self.get_config(),
                        browser_features_support,
                    ))
                } else {
                    None
                }
            });

        let mut ancestors = HashSet::new();

        for tile_query in tile_queries_iter {
            match tile_query.cell {
                CellDesc::HiPS2D { cell, .. } => {
                    let tile_cell = cell;
                    tile_fetcher.append(tile_query);

                    // check if we are starting aladin lite or not.
                    // If so we want to retrieve only the tiles in the view and access them
                    // directly i.e. without blending them with less precised tiles
                    if tile_fetcher.get_num_tile_fetched() > 0
                        && tile_cell.depth() >= min_tile_depth + 3
                    {
                        let ancestor_tile_cell = tile_cell.ancestor(3);
                        ancestors.insert(ancestor_tile_cell);
                    }
                }
                _ => unreachable!(),
            }
        }

        for ancestor in ancestors {
            if !self.update_priority_tile(&ancestor) {
                tile_fetcher.append(query::Tile::new(
                    &ancestor,
                    self.get_config(),
                    browser_features_support,
                ));
            }
        }
    }

    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        self.buffer.contains_tile(cell)
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
        let depth = camera.get_tile_depth().min(cfg.get_max_depth_tile());

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
    pub fn set_moc(&mut self, moc: SpaceMoc) {
        self.moc = Some(moc);
    }

    #[inline]
    pub fn get_moc(&self) -> Option<&SpaceMoc> {
        self.moc.as_ref()
    }

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        self.buffer.set_image_ext(&self.gl, ext)
    }

    pub fn is_allsky(&self) -> bool {
        self.buffer.config().is_allsky
    }

    pub fn read_pixel(
        &self,
        x: f64,
        y: f64,
        camera: &CameraViewPort,
        proj: &ProjectionType,
    ) -> Result<JsValue, JsValue> {
        if let Some(xyz) = proj.screen_to_model_space(&Vector2::new(x, y), camera) {
            // 1. Convert it to the hips frame system
            let cfg = self.buffer.config();
            let camera_frame = camera.get_coo_system();
            let hips_frame = cfg.get_frame();

            let lonlat = crate::coosys::apply_coo_system(camera_frame, hips_frame, &xyz).lonlat();

            // Get the array of textures from that survey
            let depth = camera.get_tile_depth().min(cfg.get_max_depth_tile());

            // compute the tex
            let (pix, dx, dy) = crate::healpix::utils::hash_with_dxdy(depth, &lonlat);
            let tile_cell = HEALPixCell(depth, pix);

            let (bscale, bzero) = if let Some(FitsParams { bscale, bzero, .. }) = self.fits_params {
                (bscale, bzero)
            } else {
                (1.0, 0.0)
            };

            self.buffer.read_pixel(&tile_cell, dx, dy, bscale, bzero)
        } else {
            Err(JsValue::from_str("Out of projection"))
        }
    }

    fn recompute_vertices(&mut self, camera: &mut CameraViewPort, projection: &ProjectionType) {
        //al_core::log(&format!("num position: {:?}", self.position.len()));
        self.position.clear();
        self.uv_start.clear();
        self.uv_end.clear();
        self.time_tile_received.clear();
        self.idx_vertices.clear();

        let cfg = self.buffer.config();
        // Get the coo system transformation matrix
        let channel = cfg.get_format().get_pixel_format();

        // Retrieve the model and inverse model matrix
        let mut off_indices = 0;
        // Define a global level of subdivisions for all the healpix tile cells in the view
        // This should prevent seeing many holes
        // We compute it from the first cell in the view but it might be an under/over estimate for the other cells in the view
        let num_sub = self
            .hpx_cells_in_view
            .iter()
            .map(|cell| super::subdivide::num_hpx_subdivision(cell, camera, projection))
            .max()
            .unwrap();

        //let num_sub =
        //    super::subdivide::num_hpx_subdivision(&self.hpx_cells_in_view[0], camera, projection);
        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell_in_cov = if let Some(moc) = self.moc.as_ref() {
                if moc.intersects_cell(cell) {
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
                } else if let Some(parent_cell) = self.buffer.get_nearest_parent(cell) {
                    if let Some(ending_cell_in_tex) = self.buffer.get(&parent_cell) {
                        if let Some(grand_parent_cell) =
                            self.buffer.get_nearest_parent(&parent_cell)
                        {
                            if let Some(starting_cell_in_tex) = self.buffer.get(&grand_parent_cell)
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
                    if channel == PixelType::RGB8U {
                        Some(HpxDrawData::new(cell))
                    } else {
                        None
                    }
                }
            } else {
                // No ancestor has been found in the buffer to draw.
                // We might want to check if the HiPS channel is JPEG to mock a cell that will be drawn in black
                if channel == PixelType::RGB8U {
                    Some(HpxDrawData::new(cell))
                } else {
                    None
                }
            };

            if let Some(HpxDrawData {
                cell,
                uv_0,
                uv_1,
                start_time,
            }) = hpx_cell
            {
                let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
                let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;
                let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;

                let sub_cells = super::subdivide::subdivide_hpx_cell(cell, num_sub, camera);

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

    pub fn push_tile<I: Image>(
        &mut self,
        cell: &HEALPixCell,
        image: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer.push(cell, image, time_request)
    }

    pub fn add_allsky(&mut self, allsky: AllskyRequest) -> Result<(), JsValue> {
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
        &mut self,
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

        let mut draw_allsky = camera.is_raytracing(proj);
        if !draw_allsky {
            let tile_size = self.get_config().get_tile_size();
            let pixel_p1 =
                camera.get_tile_depth() as u32 + crate::math::utils::log_2_unchecked(tile_size);

            let tile_size_order3_in_allsky = tile_size.min(64);
            let pixel_p2 = 3 + crate::math::utils::log_2_unchecked(tile_size_order3_in_allsky);

            draw_allsky = pixel_p1 <= pixel_p2;
        }

        self.buffer.render_allsky(draw_allsky);
        let config = self.get_config();

        let ImageMetadata {
            opacity,
            blending,
            colormap,
            ..
        } = cfg;

        // Get the colormap from the color
        let colormap = colormaps.get(colormap.as_ref());

        blending.enable(&self.gl, || {
            if draw_allsky {
                let w2v = c * (*camera.get_w2m());

                let shader = get_raytracer_shader(colormap, &self.gl, shaders, config)?;

                let shader = shader.bind(&self.gl);
                shader
                    .attach_uniforms_from(camera)
                    .attach_uniforms_from(&self.buffer)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(cfg, colormaps)
                    .attach_uniform("model", &w2v)
                    .attach_uniform("current_time", &utils::get_current_time())
                    .attach_uniform(
                        "no_tile_color",
                        &(if config.get_format().get_pixel_format() == PixelType::RGB8U {
                            Vector4::new(0.0, 0.0, 0.0, 1.0)
                        } else {
                            Vector4::new(0.0, 0.0, 0.0, 0.0)
                        }),
                    )
                    .attach_uniform("opacity", opacity)
                    .attach_uniforms_from(colormaps);

                if let Some(fits_params) = self.fits_params.as_ref() {
                    shader.attach_uniforms_from(fits_params);
                }

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
                let shader = get_raster_shader(colormap, &self.gl, shaders, config)?.bind(&self.gl);

                shader
                    .attach_uniforms_from(&self.buffer)
                    // send the cmap appart from the color config
                    .attach_uniforms_with_params_from(colormap, colormaps)
                    .attach_uniforms_from(cfg)
                    .attach_uniforms_from(camera)
                    .attach_uniform("inv_model", &v2w)
                    .attach_uniform("current_time", &utils::get_current_time())
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("u_proj", proj)
                    .attach_uniforms_from(colormaps);

                if let Some(fits_params) = self.fits_params.as_ref() {
                    shader.attach_uniforms_from(fits_params);
                }

                shader
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

    pub fn set_fits_params(&mut self, bscale: f32, bzero: f32, blank: Option<f32>) {
        self.fits_params = Some(FitsParams {
            bscale,
            bzero,
            blank,
        });
    }
}
