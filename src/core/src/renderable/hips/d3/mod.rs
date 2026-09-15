pub mod cube;
pub mod texture;

use crate::browser_support::BrowserFeaturesSupport;
use crate::healpix::moc::FreqSpaceMoc;
use crate::math::spectra::SpectralUnit;

use crate::tile_fetcher::TileFetcherQueue;
use crate::renderable::hips::d3::texture::HpxFreqTex;
use al_api::hips::DataproductType;
use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
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
        _ => crate::shader::get_shader(gl, shaders, "hips3d_raster.vert", "hips3d_red.frag"),
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

    num_indices: Vec<usize>,
    cells: Vec<HEALPixFreqCell>,
    // flag to forcing the mesh to be rebuilt
    move_freq: bool,
    pub(crate) freq: Freq,

    /// name of the layer
    layer: String,
}

use super::HpxTileBuffer;
use crate::math::spectra::Freq;

impl HiPS3D {
    pub fn new(cfg: HiPSConfig, gl: &WebGlContext, layer: &str) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

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

        let freq = (cfg.em_min.unwrap_or(Freq(1.0)) + cfg.em_max.unwrap_or(Freq(1.0))) * 0.5;
        let buffer = HiPS3DBuffer::new(gl, cfg)?;

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
            layer: layer.to_string(),
        })
    }

    pub(crate) fn get_layer(&self) -> &str {
        self.layer.as_str()
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

        match cfg.dataproduct_type {
            DataproductType::Cube => {
                // Usual tile fetching heuristic similar to HiPS2D but with a channel
                let channel_idx = (((self.freq.0 - cfg.em_min.unwrap_abort().0)
                    / (cfg.em_max.unwrap_abort().0 - cfg.em_min.unwrap_abort().0))
                    * (cfg.get_cube_depth().unwrap_abort() as f64))
                    as u64;
                let tile_depth = 32;

                let tiles_iter =
                    camera
                        .get_hpx_cells(depth_tile, survey_frame)
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
                        browser_features_support,
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
                        browser_features_support,
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
                let f_hash = self.freq.hash(f_order);

                let cubic_tiles_iter = camera
                    .get_hpx_cells(depth_tile, survey_frame)
                    // query the tiles in the camera view
                    .map(|tile_cell| HEALPixFreqCell::new(tile_cell, f_hash, f_order))
                    // filter the cubic tiles by the sfmoc
                    .filter_map(|cell| {
                        if self.contains_tile(&cell) {
                            None
                        } else if let Some(moc) = self.moc.as_ref() {
                            if moc.intersects_cell(&cell) {
                                Some(cell)
                            } else {
                                None
                                // FIXME ME READ THE MOC
                                //Some(cell)
                            }
                        } else {
                            Some(cell)
                        }
                    });

                for cubic_tile in cubic_tiles_iter {
                    tile_fetcher.append(query::Tile::new_cubic(
                        &cubic_tile,
                        cfg,
                        browser_features_support,
                    ));
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_cell_texture(&self, cell: &HEALPixFreqCell) -> Option<&HpxFreqTex> {
        self.buffer.get(cell)
    }

    /*pub fn set_spectra_displayer_location(&mut self, camera: &CameraViewPort) {
        let (lon, lat) = lonlat::xyz_to_radec(&coosys::apply_coo_system(
            camera.get_coo_system(),
            CooSystem::ICRS,
            camera.get_center(),
        ));

        let lonlat = LonLatT(lon, lat);

        crate::log!("set spectra_displayer location");
        self.spectra_displayer.set_location(lonlat);

        // update the spectra
        let cfg = self.get_config();
        let dataproduct_type = cfg.dataproduct_type;
        if dataproduct_type == DataproductType::SpectralCube {
            self.compute_spectra_on_spectra_displayer();
        }
    }*/

    pub fn set_freq(&mut self, f: Freq) {
        self.freq = f;

        // Flag telling to recompute the mesh afterwards
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

    /*pub fn get_freq_window(&self) -> [Freq; 2] {
        self.spectra_displayer.get_freq_window()
    }*/

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

        let cfg = self.get_config();
        let dataproduct_type = cfg.dataproduct_type;
        let max_depth_tile = cfg.max_depth_tile;

        let em_min = cfg.em_min;
        let em_max = cfg.em_max;
        let cube_depth = cfg.get_cube_depth();
        let max_depth_freq = cfg.max_depth_freq;

        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell = match dataproduct_type {
                DataproductType::SpectralCube => {
                    // Determination of the f_order from the s_order
                    // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                    let f_max_order = max_depth_freq.unwrap_abort();
                    let s_max_order = max_depth_tile;
                    let s_order = cell.depth();

                    let f_order = f_max_order - (s_max_order - s_order);
                    let f_hash = self.get_freq().hash(f_order);

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
                            //None
                            // FIXME SFMOC parsing
                            //Some(hpx_f_cell)
                            None
                        }
                    } else {
                        Some(hpx_f_cell)
                    }
                }
                DataproductType::Cube => {
                    let channel_idx = (((self.get_freq().0 - em_min.unwrap_abort().0)
                        / (em_max.unwrap_abort().0 - em_min.unwrap_abort().0))
                        * (cube_depth.unwrap_abort() as f64))
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
                    let texture_cell = texture.cell.clone();
                    // The slice is sure to be contained so we can unwrap
                    let slice_position = match dataproduct_type {
                        DataproductType::SpectralCube => {
                            // 1. hash of the frequency at max order
                            let f_hash = Frequency::<u64>::freq2hash(self.get_freq().0);
                            // b. compute the hash range
                            let delta_f_order = Frequency::<u64>::MAX_DEPTH - texture_cell.f_depth;
                            let f_order_hash_0 = texture_cell.f_hash;
                            let f_order_hash_1 = f_order_hash_0 + 1;

                            // 3. hash range at max order
                            let f_hash_0 = f_order_hash_0 << delta_f_order;
                            let f_hash_1 = f_order_hash_1 << delta_f_order;

                            (f_hash - f_hash_0) as f32 / (f_hash_1 - f_hash_0) as f32
                        }
                        DataproductType::Cube => {
                            let channel_idx = (((self.get_freq().0 - em_min.unwrap_abort().0)
                                / (em_max.unwrap_abort().0 - em_min.unwrap_abort().0))
                                * (cube_depth.unwrap_abort() as f64))
                                as u64;
                            let tile_depth = 32;

                            ((channel_idx % tile_depth) as f32) / (tile_depth as f32 - 1.0)
                        }
                        _ => unreachable!(),
                    };

                    let uv_1 = TileUVW::new(&cell.hpx, &Some(texture_cell.hpx), slice_position);
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
                            idx + 1 + off_indices,
                            idx + 3 + off_indices,
                            idx + 2 + off_indices,
                            idx + 1 + off_indices,
                            idx + off_indices,
                            idx + 3 + off_indices,
                        ]);

                        idx += 4;

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
                    }

                    off_indices += pos.len() as u16;

                    self.num_indices.push(self.idx_vertices.len() - tmp);

                    // Replace options with an arbitrary vertex
                    let position_iter = pos.into_iter().flatten();
                    self.position.extend(position_iter);

                    self.cells.push(texture_cell);
                }
            }
        }

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

    fn reset_available_tiles(&mut self) -> bool {
        self.buffer.reset_available_tiles()
    }

    // returns a boolean if the view cells has changed with respect to the last frame
    fn retrieve_cells_in_camera(&mut self, camera: &CameraViewPort) -> bool {
        let cfg = self.get_config();
        // Get the coo system transformation matrix
        let hips_frame = cfg.get_frame();
        let depth = camera.get_tile_depth().min(cfg.get_max_depth_tile());

        let hpx_cells_in_view = camera.get_hpx_cells(depth, hips_frame).collect::<Vec<_>>();
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
            colormap,
            opacity,
            blending,
            ..
        } = cfg;

        let colormap = colormaps.get(colormap.as_ref());

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

        let shader = get_raster_shader(&self.gl, shaders, hips_cfg)?;
        for (cell, num_indices) in self.cells.iter().zip(self.num_indices.iter()) {
            blending.enable(&self.gl, || {
                // Bind the shader at each draw of a cell to not exceed the max number of tex image units bindable
                // to a shader. It is 32 in my case
                let shaderbound = shader.bind(&self.gl);

                shaderbound
                    .attach_uniform("tex", &self.buffer.get(cell).unwrap_abort().texture)
                    .attach_uniforms_from(&self.buffer)
                    .attach_uniforms_with_params_from(colormap, colormaps)
                    .attach_uniforms_from(cfg)
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

    pub fn push_tile_from_fits(
        &mut self,
        cell: &HEALPixFreqCell,
        // the image slice
        data: js_sys::Uint8Array,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer
            .push_tile_from_fits(cell, data, size, time_request)
    }

    pub fn push_tile_from_jpeg(
        &mut self,
        cell: &HEALPixFreqCell,
        // the image slice
        data: Box<[u8]>,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer
            .push_tile_from_jpeg(cell, data, size, time_request)
    }

    pub fn push_tile_from_png(
        &mut self,
        cell: &HEALPixFreqCell,
        // the image slice
        data: Box<[u8]>,
        size: (u32, u32, u32),
        time_request: Time,
    ) -> Result<(), JsValue> {
        self.buffer
            .push_tile_from_png(cell, data, size, time_request)
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
