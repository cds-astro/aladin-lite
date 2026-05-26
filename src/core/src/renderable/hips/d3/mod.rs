pub mod cube;
pub mod texture;

use crate::browser_support::BrowserFeaturesSupport;
use crate::healpix::moc::FreqSpaceMoc;
use crate::math::angle::ToAngle;
use crate::math::lonlat;
use crate::math::lonlat::LonLatT;
use crate::math::spectra::SpectralUnit;
use crate::math::spectra::FREQ_MAX;
use crate::math::spectra::FREQ_MIN;

use crate::coosys;

use crate::CooSystem;

use crate::tile_fetcher::TileFetcherQueue;
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

use js_sys::Object;
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
    // The location of the cursor to extract the spectra
    cursor: Cursor,

    /// name of the layer
    layer: String,
}

struct Cursor {
    location: LonLatT<f64>,
    freq: Freq,

    cell: HEALPixFreqCell,
    s_max_order: u8,
    f_max_order: u8,
    tile_depth: u8,

    em_min: Freq,
    em_max: Freq,
}

struct FrequencyWindow {
    /// hash range at pixel order
    window_pixel_hash: Range<u64>,
    /// The pixel order
    pixel_depth: u8,
    /// em_min/em_max hash
    domain_pixel_hash: Range<u64>,
}

use std::ops::Range;
impl Cursor {
    fn new(cfg: &HiPSConfig) -> Self {
        let em_min = cfg.em_min.unwrap_abort();
        let em_max = cfg.em_max.unwrap_abort();

        let freq = Freq((em_min.0 + em_max.0) * 0.5);
        let location = LonLatT::new(0.0.to_angle(), 0.0.to_angle());

        let f_max_order = cfg.max_depth_freq.unwrap_or(Frequency::<u64>::MAX_DEPTH);
        let s_max_order = cfg.max_depth_tile;

        let tile_depth = cfg.tile_depth.unwrap_or(1);

        let cell = HEALPixFreqCell::from_lonlat(location, freq, 0, 0);

        Cursor {
            freq,
            location,
            cell,
            f_max_order,
            s_max_order,
            tile_depth,
            em_min,
            em_max,
        }
    }

    fn get_dxdy_inside_cell(&self) -> (f64, f64) {
        let s_depth = self.cell.hpx.depth();
        let (lon, lat) = (
            self.location.lon().to_radians(),
            self.location.lat().to_radians(),
        );
        let (cell, dx, dy) = HEALPixCell::hash_with_dxdy(s_depth, lon, lat);

        debug_assert_eq!(cell, self.cell.hpx);

        (dx, dy)
    }

    /// Get the window starting and ending hashed at the pixel order
    fn get_window_frequency_range(&self) -> FrequencyWindow {
        const NUM_VALUES: usize = 150;

        let delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.cell.f_depth + delta_depth as u8;
        let f_hash_val = self.freq.hash(pixel_depth);

        let f_hash_val_0 = (f_hash_val as i64 - NUM_VALUES as i64).max(0) as u64;
        let f_hash_val_1 =
            (f_hash_val + NUM_VALUES as u64).min(Freq::num_max_cells(pixel_depth) as u64);

        let min_hash = self.em_min.hash(pixel_depth);
        let max_hash = self.em_max.hash(pixel_depth);

        FrequencyWindow {
            window_pixel_hash: f_hash_val_0..f_hash_val_1,
            pixel_depth,
            domain_pixel_hash: min_hash..max_hash,
        }
    }

    /// Get hash at the pixel level
    fn get_freq_hash(&self, freq: Freq) -> u64 {
        let delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.cell.f_depth + delta_depth as u8;

        freq.hash(pixel_depth)
    }

    /// Get freq from hash given at the pixel level
    fn get_freq_from_hash(&self, hash: u64) -> Freq {
        let delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.cell.f_depth + delta_depth as u8;

        Freq::from_hash_with_order(hash, pixel_depth)
    }

    /// Get the frequency step at the cursor i.e. the jump in frequency
    /// between the cursor slice and the dx-th one
    fn get_frequency_step(&self, dx: i64) -> Freq {
        let delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.cell.f_depth + delta_depth as u8;
        let f_hash_val = self.freq.hash(pixel_depth);

        //let f_hash_val_0 = (f_hash_val as i64 - NUM_VALUES as i64).max(0) as u64;
        let h1 = (f_hash_val as i64 + dx)
            .min(
                (Frequency::<u64>::n_cells_max() >> (Frequency::<u64>::MAX_DEPTH - pixel_depth))
                    as i64,
            )
            .max(0) as u64;

        let h0 = (f_hash_val as i64 - dx)
            .min(
                (Frequency::<u64>::n_cells_max() >> (Frequency::<u64>::MAX_DEPTH - pixel_depth))
                    as i64,
            )
            .max(0) as u64;

        let f1 = Freq::from_hash_with_order(h1, pixel_depth);
        let f0 = Freq::from_hash_with_order(h0, pixel_depth);

        Freq((f1 - f0).0 * 0.5)
    }

    fn get_surrounding_cell_hashes_along_spectra_axis(&self) -> Range<u64> {
        let FrequencyWindow {
            window_pixel_hash: f_hash_val,
            pixel_depth,
            ..
        } = self.get_window_frequency_range();

        let delta_depth = pixel_depth - self.cell.f_depth;

        // Get the tile hashes from the pixel hashes to load
        let f_hash_0 = f_hash_val.start >> delta_depth;
        let f_hash_1 = f_hash_val.end >> delta_depth;

        f_hash_0..(f_hash_1 + 1)
    }

    fn is_contained_in_spectral_view(&self, cell: &HEALPixFreqCell) -> bool {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .contains(&cell.f_hash)
    }

    fn set_location(&mut self, location: LonLatT<f64>, s_order: u8) {
        self.location = location;

        let f_order = self.f_max_order - (self.s_max_order - s_order);

        self.cell = HEALPixFreqCell::from_lonlat(self.location, self.freq, s_order, f_order);
    }

    fn set_freq(&mut self, freq: Freq) {
        if freq < FREQ_MAX && freq > FREQ_MIN {
            self.freq = freq;

            let s_order = self.cell.hpx.depth();
            let f_order = self.f_max_order - (self.s_max_order - s_order);

            self.cell = HEALPixFreqCell::from_lonlat(self.location, self.freq, s_order, f_order);
        }
    }

    fn get_surrounding_cells_along_spectra_axis(
        &self,
    ) -> impl Iterator<Item = HEALPixFreqCell> + '_ {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .map(move |f_hash| {
                // Do not include the cell containing the location AND containing the frequency because
                // it will be included when looking for new tiles in the view
                HEALPixFreqCell {
                    hpx: self.cell.hpx,
                    f_hash,
                    f_depth: self.cell.f_depth,
                }
            })
    }

    fn get_freq(&self) -> Freq {
        self.freq
    }

    fn get_freq_window(&self) -> [Freq; 2] {
        let FrequencyWindow {
            window_pixel_hash: f_hash_val,
            pixel_depth,
            ..
        } = self.get_window_frequency_range();

        let f0 = Freq::from_hash_with_order(f_hash_val.start, pixel_depth);
        let f1 = Freq::from_hash_with_order(f_hash_val.end, pixel_depth);

        [f0, f1]
    }
}

use super::HpxTileBuffer;
use crate::math::spectra::Freq;
use js_sys::Reflect;

impl HiPS3D {
    pub fn new(config: HiPSConfig, gl: &WebGlContext, layer: &str) -> Result<Self, JsValue> {
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

        let cursor = Cursor::new(&config);
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

            cells,
            num_indices,
            move_freq,
            cursor,
            layer: layer.to_string(),
        })
    }

    /// Get hash at the pixel level
    pub fn get_freq_hash(&self, freq: Freq) -> u64 {
        self.cursor.get_freq_hash(freq)
    }

    /// Get freq from hash given at the pixel level
    pub fn get_freq_from_hash(&self, hash: u64) -> Freq {
        self.cursor.get_freq_from_hash(hash)
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
                let channel_idx = (((self.cursor.get_freq().0 - cfg.em_min.unwrap_abort().0)
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

                let cubic_tiles_iter = camera
                    .get_hpx_cells(depth_tile, survey_frame)
                    .into_iter()
                    // query the tiles in the camera view
                    .filter_map(|tile_cell| {
                        let f_hash = self.cursor.get_freq().hash(f_order);

                        let cell = HEALPixFreqCell::new(tile_cell, f_hash, f_order);

                        if self.cursor.cell == cell {
                            None
                        } else {
                            Some(cell)
                        }
                    })
                    // query the tiles under the cursor as well
                    .chain(self.cursor.get_surrounding_cells_along_spectra_axis())
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

    /// Read the spectra under the cursor location
    fn compute_spectra_on_cursor(&self) {
        // Determine the slices window
        let tile_depth = self.get_config().tile_depth.unwrap_abort();
        let cell_hash_f = self.cursor.get_surrounding_cell_hashes_along_spectra_axis();
        let delta_depth = tile_depth.trailing_zeros();
        let pixel_hash_0 = cell_hash_f.start << delta_depth;
        let FrequencyWindow {
            window_pixel_hash,
            domain_pixel_hash,
            pixel_depth,
        } = self.cursor.get_window_frequency_range();

        // Determine the frequencies the borders of the window
        let f0 = Freq::from_hash_with_order(window_pixel_hash.start, pixel_depth);
        let f1 = Freq::from_hash_with_order(window_pixel_hash.end, pixel_depth);

        // Determine the spectral step at the cursor position
        let f_step = self.cursor.get_frequency_step(1);

        // Determine the spectral values in the window
        let (dy, dx) = self.cursor.get_dxdy_inside_cell();

        let tile_size = self.get_config().tile_size as f64;
        let x = (dx * tile_size) as u32;
        let y = (dy * tile_size) as u32;

        let indices =
            (window_pixel_hash.start - pixel_hash_0)..(window_pixel_hash.end - pixel_hash_0);

        // create the js object containing:
        // * spectra values
        // * min and max frequency values
        let spectra_js_obj = Object::new();

        // Set properties using Reflect::set
        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("freqMin"),
            &JsValue::from_f64(f0.0),
        )
        .unwrap_abort();
        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("freqMax"),
            &JsValue::from_f64(f1.0),
        )
        .unwrap_abort();
        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("freq"),
            &JsValue::from_f64(self.cursor.freq.0),
        )
        .unwrap_abort();
        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("freqStep"),
            &JsValue::from_f64(f_step.0),
        )
        .unwrap_abort();

        let mut start = window_pixel_hash.start.max(domain_pixel_hash.start);
        let mut end = window_pixel_hash.end.min(domain_pixel_hash.end);

        if start <= end {
            start = start - pixel_hash_0 - indices.start;
            end = end - pixel_hash_0 - indices.start;

            Reflect::set(
                &spectra_js_obj,
                &JsValue::from_str("freqIdxStart"),
                &JsValue::from_f64(start as f64),
            )
            .unwrap_abort();

            Reflect::set(
                &spectra_js_obj,
                &JsValue::from_str("freqIdxEnd"),
                &JsValue::from_f64(end as f64),
            )
            .unwrap_abort();

            Reflect::set(
                &spectra_js_obj,
                &JsValue::from_str("layer"),
                &JsValue::from_str(&self.layer),
            )
            .unwrap_abort();
        }

        let mut freqs = vec![];
        let spectra = self
            .cursor
            .get_surrounding_cells_along_spectra_axis()
            .flat_map(|c| {
                freqs.extend(c.pixel_frequencies(tile_depth as usize));
                if let Some(cubic_tex) = self.buffer.get(&c) {
                    (0..(tile_depth as u32))
                        .map(|z| cubic_tex.read_pixel(x, y, z).unwrap_or(f32::NAN))
                        .collect::<Vec<_>>()
                } else {
                    vec![f32::NAN; tile_depth as usize]
                }
            })
            .enumerate()
            .filter_map(|(i, value)| {
                if indices.contains(&(i as u64)) {
                    Some(value)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("values"),
            &js_sys::Float32Array::from(&spectra[..]),
        )
        .unwrap_abort();

        Reflect::set(
            &spectra_js_obj,
            &JsValue::from_str("freqs"),
            &js_sys::Float32Array::from(&freqs[..]),
        )
        .unwrap_abort();

        crate::event::send_custom_event("spectra", JsValue::from(spectra_js_obj));
    }

    pub fn set_cursor_location(&mut self, camera: &CameraViewPort) {
        let (lon, lat) = lonlat::xyz_to_radec(&coosys::apply_coo_system(
            camera.get_coo_system(),
            CooSystem::ICRS,
            camera.get_center(),
        ));

        let lonlat = LonLatT(lon, lat);

        let cfg = self.get_config();
        let s_order = camera
            .get_tile_depth()
            .min(cfg.get_max_depth_tile())
            .max(cfg.get_min_depth_tile());
        let dataproduct_type = cfg.dataproduct_type;

        self.cursor.set_location(lonlat, s_order);

        // update the spectra
        if dataproduct_type == DataproductType::SpectralCube {
            self.compute_spectra_on_cursor();
        }
    }

    pub fn set_freq(&mut self, f: Freq) {
        self.cursor.set_freq(f);

        // update the spectra
        if self.get_config().dataproduct_type == DataproductType::SpectralCube {
            self.compute_spectra_on_cursor();
        }

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
        self.cursor.get_freq()
    }

    pub fn get_freq_window(&self) -> [Freq; 2] {
        self.cursor.get_freq_window()
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
            .map(|()| {
                if self.cursor.is_contained_in_spectral_view(cell) {
                    // compute the spectra in case the cell is contained into the current spectral view
                    self.compute_spectra_on_cursor();
                }
            })
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
            .map(|()| {
                if self.cursor.is_contained_in_spectral_view(cell) {
                    // compute the spectra in case the cell is contained into the current spectral view
                    self.compute_spectra_on_cursor();
                }
            })
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
            .map(|()| {
                if self.cursor.is_contained_in_spectral_view(cell) {
                    // compute the spectra in case the cell is contained into the current spectral view
                    self.compute_spectra_on_cursor();
                }
            })
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
