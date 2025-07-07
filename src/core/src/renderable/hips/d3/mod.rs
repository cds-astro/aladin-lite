pub mod cube;
pub mod texture;

use crate::downloader::request::allsky::AllskyRequest;
use crate::healpix::moc::FreqSpaceMoc;
use crate::math::spectra::SpectralUnit;

use crate::tile_fetcher::TileFetcherQueue;
use al_api::hips::DataproductType;
use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
use al_core::al_print;
use al_core::colormap::Colormap;
use al_core::colormap::Colormaps;

use al_core::texture::format::PixelType;

use moclib::qty::Frequency;
use moclib::qty::MocQty;

use crate::healpix::cell::HEALPixFreqCell;

use crate::Abort;

use al_core::image::Image;

use al_core::shader::Shader;
use al_core::webgl_ctx::GlWrapper;

use al_core::VecData;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use crate::ProjectionType;

use crate::camera::CameraViewPort;

use crate::downloader::query;

use crate::shader::ShaderManager;

use crate::healpix::cell::HEALPixCell;
use crate::time::Time;

use self::cube::HiPS3DBuffer;

use super::config::HiPSConfig;
use super::FitsParams;
use std::collections::HashSet;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

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
    match config.get_format().get_pixel_format() {
        PixelType::R8U => {
            crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_u8.frag")
        }
        PixelType::R16I => {
            crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_i16.frag")
        }
        PixelType::R32I => {
            crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_i32.frag")
        }
        PixelType::R32F => {
            crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_f32.frag")
        }
        // color case
        _ => {
            if cmap.label() == "native" {
                crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_rgba.frag")
            } else {
                crate::shader::get_shader(
                    gl,
                    shaders,
                    "hips3d_raster.vert",
                    "hips3d_rgba2cmap.frag",
                )
            }
        }
    }
}

pub struct HiPS3D {
    // The image survey texture buffer
    buffer: HiPS3DBuffer,

    // The projected vertices data
    // For WebGL2 wasm, the data are interleaved
    // layout (location = 0) in vec3 position;
    position: Vec<f32>,
    // layout (location = 1) in vec3 uv_start;
    uv: Vec<f32>,
    idx_vertices: Vec<u16>,

    vao: VertexArrayObject,
    gl: WebGlContext,

    moc: Option<FreqSpaceMoc>,

    // A buffer storing the cells in the view
    hpx_cells_in_view: Vec<HEALPixCell>,

    pub(crate) fits_params: Option<FitsParams>,

    // The current slice index
    freq: Freq,

    num_indices: Vec<usize>,
    cells: Vec<HEALPixFreqCell>,
    // flag to forcing the mesh to be rebuilt
    move_freq: bool,
}

use super::HpxTileBuffer;
use crate::math::spectra::Freq;

impl HiPS3D {
    pub fn new(config: HiPSConfig, gl: &WebGlContext) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

        let freq = Freq(0.0);

        let num_indices = vec![];
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
                "position",
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

        let buffer = HiPS3DBuffer::new(gl, config)?;

        let cells = vec![];

        let gl = gl.clone();
        let moc = None;
        let hpx_cells_in_view = vec![];
        let move_freq = false;
        // request the allsky texture
        Ok(Self {
            // The image survey texture buffer
            buffer,

            vao,

            gl,

            position,
            uv,
            idx_vertices,

            fits_params: None,

            moc,
            hpx_cells_in_view,

            freq,
            cells,
            num_indices,
            move_freq,
        })
    }

    pub fn build_tile_query(&self, cell: &HEALPixCell) -> query::Tile {
        let cfg = self.get_config();
        match cfg.dataproduct_type {
            DataproductType::SpectralCube => {
                // Determination of the f_order from the s_order
                // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                let f_max_order = cfg.max_depth_freq.unwrap_abort();
                let s_max_order = cfg.max_depth_tile;
                let s_order = cell.depth();

                let f_order = f_max_order - (s_max_order - s_order);
                let f_hash = self.freq.hash(f_order);
                let cell = HEALPixFreqCell::new(*cell, f_hash, f_order);

                query::Tile::new_cubic(&cell, cfg)
            }
            DataproductType::Cube => {
                let channel_idx = (((self.freq.0 - cfg.em_min.unwrap_abort().0)
                    / (cfg.em_max.unwrap_abort().0 - cfg.em_min.unwrap_abort().0))
                    * (cfg.get_cube_depth().unwrap_abort() as f64))
                    as u32;

                query::Tile::new_with_channel(&cell, channel_idx, cfg)
            }
            _ => unreachable!(),
        }
    }

    pub fn look_for_new_tiles(
        &mut self,
        tile_fetcher: &mut TileFetcherQueue,
        camera: &CameraViewPort,
    ) {
        // do not add tiles if the view is already at depth 0
        let cfg = self.get_config();
        let depth_tile = camera
            .get_tile_depth()
            .min(cfg.get_max_depth_tile())
            .max(cfg.get_min_depth_tile());

        let survey_frame = cfg.get_frame();

        match cfg.dataproduct_type {
            DataproductType::Cube => {
                // Usual tile fetching heuristic similar to HiPS2D but with a channel
                let channel_idx = (((self.freq.0 - cfg.em_min.unwrap_abort().0)
                    / (cfg.em_max.unwrap_abort().0 - cfg.em_min.unwrap_abort().0))
                    * (cfg.get_cube_depth().unwrap_abort() as f64))
                    as u64;
                let tile_depth = 32;

                let tiles_iter = camera
                    .get_hpx_cells(depth_tile, survey_frame)
                    .into_iter()
                    .filter(|tile_cell| {
                        if let Some(moc) = self.moc.as_ref() {
                            // TODO: Check this part of code, the moc is only spatial so it should intersect whatever f hash you give
                            let f_hash = channel_idx / tile_depth;
                            let cell = HEALPixFreqCell::new(
                                *tile_cell,
                                f_hash,
                                Frequency::<u64>::MAX_DEPTH,
                            );
                            moc.intersects_cell(&cell)
                        } else {
                            true
                        }
                    });

                let min_tile_depth = cfg.get_min_depth_tile();
                let mut ancestors = HashSet::new();

                for tile_cell in tiles_iter {
                    tile_fetcher.append(query::Tile::new_with_channel(
                        &tile_cell,
                        channel_idx as u32,
                        cfg,
                    ));

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

                for ancestor in ancestors {
                    tile_fetcher.append(query::Tile::new_with_channel(
                        &ancestor,
                        channel_idx as u32,
                        cfg,
                    ));
                }
            }
            DataproductType::SpectralCube => {
                // Determination of the f_order from the s_order
                // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                let f_max_order = cfg.max_depth_freq.unwrap_abort();
                let s_max_order = cfg.max_depth_tile;
                let s_order = depth_tile;

                let f_order = f_max_order - (s_max_order - s_order);

                let cubic_tiles_iter = camera
                    .get_hpx_cells(depth_tile, survey_frame)
                    .into_iter()
                    .filter_map(|tile_cell| {
                        let f_hash = self.freq.hash(f_order);
                        let cell = HEALPixFreqCell::new(tile_cell, f_hash, f_order);

                        if let Some(moc) = self.moc.as_ref() {
                            if moc.intersects_cell(&cell) {
                                Some(cell)
                            } else {
                                None
                            }
                        } else {
                            Some(cell)
                        }
                    });

                // TODO: construct the cubic tile queries along the lonlat(position) to get the spectra
                // We take +/- 4 cells around the freq

                for cubic_tile in cubic_tiles_iter {
                    tile_fetcher.append(query::Tile::new_cubic(&cubic_tile, cfg));
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn set_freq(&mut self, f: Freq) {
        self.freq = f;

        self.move_freq = true;
    }

    pub fn contains_tile(&self, cell: &HEALPixFreqCell) -> bool {
        self.buffer.contains(cell)
    }

    pub fn draw(
        &mut self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &mut CameraViewPort,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        let available_tiles = self.reset_available_tiles();
        let new_cells_in_view = self.retrieve_cells_in_camera(camera);

        if new_cells_in_view | available_tiles | self.move_freq {
            // TODO: append the vertices independently to the draw method
            self.recompute_vertices(camera, proj);

            self.move_freq = false;
        }

        self.draw_internal(shaders, colormaps, camera, cfg, proj)
    }

    pub fn get_freq(&self) -> Freq {
        self.freq
    }

    fn recompute_vertices(&mut self, camera: &CameraViewPort, proj: &ProjectionType) {
        self.cells.clear();

        self.position.clear();
        self.uv.clear();
        self.idx_vertices.clear();

        self.num_indices.clear();

        let mut off_indices = 0;

        let channel = self.get_config().get_format().get_pixel_format();

        // Define a global level of subdivisions for all the healpix tile cells in the view
        // This should prevent seeing many holes
        // We compute it from the first cell in the view but it might be an under/over estimate for the other cells in the view
        //let num_sub = super::subdivide::num_hpx_subdivision(&self.hpx_cells_in_view[0], camera, proj);

        let num_sub = self
            .hpx_cells_in_view
            .iter()
            .map(|cell| super::subdivide::num_hpx_subdivision(cell, camera, proj))
            .max()
            .unwrap();

        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell = match self.get_config().dataproduct_type {
                DataproductType::SpectralCube => {
                    // Determination of the f_order from the s_order
                    // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                    let f_max_order = self.get_config().max_depth_freq.unwrap_abort();
                    let s_max_order = self.get_config().max_depth_tile;
                    let s_order = cell.depth();

                    let f_order = f_max_order - (s_max_order - s_order);
                    let f_hash = self.freq.hash(f_order);

                    let hpx_f_cell = HEALPixFreqCell::new(*cell, f_hash, f_order);

                    if let Some(moc) = self.moc.as_ref() {
                        if moc.intersects_cell(&hpx_f_cell) {
                            Some(hpx_f_cell)
                        } else if channel == PixelType::RGB8U {
                            // Rasterizer does not render tiles that are not in the MOC
                            // This is not a problem for transparency rendered HiPses (FITS or PNG)
                            // but JPEG tiles do have black when no pixels data is found
                            // We therefore must draw in black for the tiles outside the HiPS MOC
                            Some(hpx_f_cell)
                        } else {
                            None
                        }
                    } else {
                        Some(hpx_f_cell)
                    }
                }
                DataproductType::Cube => {
                    /*al_core::log(&format!(
                        "{:?}, {:?} {:?} {:?}",
                        self.freq.0,
                        self.get_config().em_min,
                        self.get_config().em_max,
                        self.get_config().get_cube_depth(),
                    ));*/

                    let channel_idx = (((self.freq.0 - self.get_config().em_min.unwrap_abort().0)
                        / (self.get_config().em_max.unwrap_abort().0
                            - self.get_config().em_min.unwrap_abort().0))
                        * (self.get_config().get_cube_depth().unwrap_abort() as f64))
                        as u64;

                    let tile_depth = 32;

                    let f_hash = channel_idx / tile_depth;

                    let hpx_f_cell =
                        HEALPixFreqCell::new(*cell, f_hash, Frequency::<u64>::MAX_DEPTH);
                    if let Some(moc) = self.moc.as_ref() {
                        if moc.intersects_cell(&hpx_f_cell) {
                            Some(hpx_f_cell)
                        } else if channel == PixelType::RGB8U {
                            // Rasterizer does not render tiles that are not in the MOC
                            // This is not a problem for transparency rendered HiPses (FITS or PNG)
                            // but JPEG tiles do have black when no pixels data is found
                            // We therefore must draw in black for the tiles outside the HiPS MOC
                            Some(hpx_f_cell)
                        } else {
                            None
                        }
                    } else {
                        Some(hpx_f_cell)
                    }
                }
                _ => unreachable!(),
            };

            if let Some(cell) = cell {
                let hpx_cell_texture = if self.contains_tile(&cell) {
                    self.buffer.get(&cell)
                } else if let Some(parent_cell) = self.buffer.get_nearest_parent(&cell) {
                    // Check in the spatial parent if the freq data is present
                    if self.contains_tile(&parent_cell) {
                        self.buffer.get(&parent_cell)
                    } else {
                        None
                    }
                /*
                } else if let Some(next_slice) = self.buffer.find_nearest_slice(cell, self.slice) {
                    slice_contained = next_slice;
                    self.buffer.get(cell)
                } else if let Some(parent_cell) = self.buffer.get_nearest_parent(cell) {
                    // find the slice of the parent available, if possible near slice
                    slice_contained = self
                        .buffer
                        .find_nearest_slice(&parent_cell, self.slice)
                        .unwrap();
                    self.buffer.get(&parent_cell)
                */
                } else {
                    None
                };

                if let Some(texture) = hpx_cell_texture {
                    self.cells.push(texture.cell.clone());
                    // The slice is sure to be contained so we can unwrap
                    let slice_position = match self.get_config().dataproduct_type {
                        DataproductType::SpectralCube => {
                            let f_hash_0 = texture.cell.f_hash
                                << (Frequency::<u64>::MAX_DEPTH - texture.cell.f_depth);
                            let f_hash_1 = (texture.cell.f_hash + 1)
                                << (Frequency::<u64>::MAX_DEPTH - texture.cell.f_depth);

                            let f_hash = Frequency::<u64>::freq2hash(self.freq.0);

                            (f_hash - f_hash_0) as f32 / (f_hash_1 - f_hash_0) as f32
                        }
                        DataproductType::Cube => {
                            let channel_idx = (((self.freq.0
                                - self.get_config().em_min.unwrap_abort().0)
                                / (self.get_config().em_max.unwrap_abort().0
                                    - self.get_config().em_min.unwrap_abort().0))
                                * (self.get_config().get_cube_depth().unwrap_abort() as f64))
                                as u64;
                            let tile_depth = 32;

                            ((channel_idx % tile_depth) as f32) / (tile_depth as f32 - 1.0)
                        }
                        _ => unreachable!(),
                    };

                    al_core::log(&format!("{:?}, {:?}", slice_position, texture.cell));

                    let uv_1 = TileUVW::new(&cell.hpx, &Some(texture.cell.hpx), slice_position);
                    let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
                    let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;

                    let sub_cells =
                        super::subdivide::subdivide_hpx_cell(&cell.hpx, num_sub, camera);

                    let mut pos = Vec::with_capacity(sub_cells.len() * 4);

                    let mut idx = 0;

                    let tmp = self.idx_vertices.len();

                    for sub_cell in sub_cells {
                        let (i, j) = sub_cell.offset_in_parent(&cell.hpx);
                        let nside = (1 << (sub_cell.depth() - cell.hpx.depth())) as f32;

                        for ((lon, lat), (di, dj)) in
                            sub_cell
                                .vertices()
                                .iter()
                                .zip([(0, 0), (1, 0), (1, 1), (0, 1)])
                        {
                            let hj0 = ((j + dj) as f32) / nside;
                            let hi0 = ((i + di) as f32) / nside;

                            let uv_end = [
                                uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                                uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                                uv_1[TileCorner::BottomLeft].z,
                            ];

                            self.uv.extend(uv_end);

                            pos.push([*lon as f32, *lat as f32]);
                        }

                        // GL TRIANGLES
                        self.idx_vertices.extend([
                            idx + off_indices,
                            idx + 2 + off_indices,
                            idx + 1 + off_indices,
                            idx + off_indices,
                            idx + 3 + off_indices,
                            idx + 2 + off_indices,
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

                    self.num_indices.push(self.idx_vertices.len() - tmp);

                    // Replace options with an arbitrary vertex
                    let position_iter = pos.into_iter().flatten();
                    self.position.extend(position_iter);
                }
            }
        }

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
    }

    fn reset_available_tiles(&mut self) -> bool {
        self.buffer.reset_available_tiles()
    }

    // returns a boolean if the view cells has changed with respect to the last frame
    fn retrieve_cells_in_camera(&mut self, camera: &CameraViewPort) -> bool {
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
    pub fn set_moc(&mut self, moc: FreqSpaceMoc) {
        self.moc = Some(moc);
    }

    pub fn set_fits_params(&mut self, bscale: f32, bzero: f32, blank: Option<f32>) {
        self.fits_params = Some(FitsParams {
            bscale,
            bzero,
            blank,
        });
    }

    #[inline]
    pub fn get_moc(&self) -> Option<&FreqSpaceMoc> {
        self.moc.as_ref()
    }

    pub fn set_image_ext(&mut self, ext: ImageExt) -> Result<(), JsValue> {
        self.buffer.set_image_ext(&self.gl, ext)
    }

    pub fn is_allsky(&self) -> bool {
        self.buffer.config().is_allsky
    }

    // Position given is in the camera space
    /*pub fn read_pixel(
        &self,
        p: &LonLatT<f64>,
        camera: &CameraViewPort,
    ) -> Result<JsValue, JsValue> {
        self.buffer.read_pixel(p, camera)
    }*/

    fn draw_internal(
        &self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &mut CameraViewPort,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        let hips_cfg = self.buffer.config();
        // Get the coo system transformation matrix
        let selected_frame = camera.get_coo_system();
        let hips_frame = hips_cfg.get_frame();
        let c = selected_frame.to(hips_frame);

        let big_fov = camera.is_raytracing(proj);
        if big_fov {
            self.gl.enable(WebGl2RenderingContext::CULL_FACE);
        }

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        let cmap = colormaps.get(color.cmap_name.as_ref());

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
        let mut off_idx = 0;

        let shader = get_raster_shader(cmap, &self.gl, shaders, hips_cfg)?;
        for (cell, num_indices) in self.cells.iter().zip(self.num_indices.iter()) {
            blend_cfg.enable(&self.gl, || {
                // Bind the shader at each draw of a cell to not exceed the max number of tex image units bindable
                // to a shader. It is 32 in my case
                let shaderbound = shader.bind(&self.gl);

                shaderbound
                    .attach_uniform("tex", &self.buffer.get(cell).unwrap_abort().texture)
                    .attach_uniforms_from(&self.buffer)
                    .attach_uniforms_with_params_from(cmap, colormaps)
                    .attach_uniforms_from(color)
                    .attach_uniforms_from(camera)
                    .attach_uniform("inv_model", &v2w)
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("u_proj", proj)
                    .attach_uniforms_from(colormaps);

                if let Some(fits_params) = self.fits_params.as_ref() {
                    shaderbound.attach_uniforms_from(fits_params);
                }

                shaderbound
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(*num_indices as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        (off_idx * std::mem::size_of::<u16>()) as i32,
                    );

                off_idx += *num_indices;

                Ok(())
            })?;
        }

        if big_fov {
            self.gl.disable(WebGl2RenderingContext::CULL_FACE);
        }

        Ok(())
    }

    pub fn push_tile_slice<I: Image>(
        &mut self,
        cell: &HEALPixFreqCell,
        // the image slice
        image: I,
        time_request: Time,
        // this slice index inside the cubic cell
        slice_idx: u16,
    ) -> Result<(), JsValue> {
        self.buffer
            .push_tile_slice(cell, image, time_request, slice_idx)
    }

    pub fn push_tile<I: Image>(
        &mut self,
        cell: &HEALPixFreqCell,
        // the image slice
        cube: I,
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer.push_tile(cell, cube, time_request)
    }

    /*pub fn add_allsky(&mut self, allsky: AllskyRequest) -> Result<(), JsValue> {
        self.buffer.push_allsky(allsky)
    }*/

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
