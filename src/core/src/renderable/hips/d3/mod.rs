pub mod cube;
pub mod texture;

use crate::browser_support::BrowserFeaturesSupport;
use crate::healpix::moc::FreqSpaceMoc;
use super::HpxTileBuffer;
use crate::math::quantity::frequency::Freq;
use crate::math::quantity::QtyType;

use crate::tile_fetcher::TileFetcherQueue;
use crate::renderable::hips::d3::texture::Hpx3DTex;
use al_api::hips::DataproductType;
use al_api::hips::ImageExt;
use al_api::hips::ImageMetadata;
use al_core::colormap::Colormaps;

use al_core::texture::format::PixelType;

use moclib::qty::Frequency;
use moclib::qty::MocQty;

use crate::healpix::cell::HEALPix3DCell;

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
    cells: Vec<HEALPix3DCell>,
    // flag to forcing the mesh to be rebuilt
    move_qty: bool,
    pub(crate) qty: QtyType,

    /// name of the layer
    layer: String,
}

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

            use crate::math::quantity::frequency::Wavelength;

        let qty = match cfg.dataproduct_type {
            DataproductType::SpectralCube | DataproductType::Cube => {
                let l = (cfg.em_min.unwrap_or(1.0) + cfg.em_max.unwrap_or(1.0)) * 0.5;
                QtyType::Freq(Wavelength(l).into())
            },
            DataproductType::TimeCube => {
                let mjd = (cfg.t_min.unwrap_or(1.0) + cfg.t_max.unwrap_or(1.0)) * 0.5;
                let jd = mjd + 2400000.5;
                let us = (jd * 86400000000.0) as u64;
                QtyType::Time(crate::math::quantity::time::Time(us))
            },
            _ => todo!()
        };

        let buffer = HiPS3DBuffer::new(gl, cfg)?;

        let cells = vec![];

        let gl = gl.clone();
        let moc = None;
        let hpx_cells_in_view = vec![];
        let move_qty = false;
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

            qty,
            cells,
            num_indices,
            move_qty,
            layer: layer.to_string(),
        })
    }

    pub(crate) fn get_layer(&self) -> &str {
        self.layer.as_str()
    }

    // From a space order corresponding to the current camera FoV,
    // this methods computes the quantity order following this schema:
    // https://www.ivoa.net/documents/HiPS/20260501/WD-HiPS-2.0-20260501.html#fig:fig21
    fn get_q_order(&self, s_order: u8, s_max_order: u8, q_spectra_order: u8, q_max_order: u8) -> u8 {
        if s_order == 0 {
            let q_order = q_max_order as i8 - (s_max_order as i8 - s_order as i8);
            (q_spectra_order as i8).min(q_order).max(0) as u8
        } else {
            (q_max_order as i8 - (s_max_order as i8 - s_order as i8)).max(0) as u8
        }
    }

    pub fn look_for_new_tiles(
        &mut self,
        tile_fetcher: &mut TileFetcherQueue,
        camera: &CameraViewPort,
        browser_features_support: &BrowserFeaturesSupport,
        q_order: u8
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
                let freq: f64 = self.qty.into();
                let em_min = cfg.em_min.unwrap_abort();
                let em_max = cfg.em_max.unwrap_abort();

                let channel_idx = (((freq - em_min)
                    / (em_max - em_min))
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
                                let cell = HEALPix3DCell::new(
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
            DataproductType::SpectralCube | DataproductType::TimeCube => {
                // Determination of the f_order from the s_order
                // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                let q_max_order = cfg.max_depth_axis2.unwrap_abort();
                let s_max_order = cfg.max_depth_tile;
                let s_order = depth_tile;

                let q_order = self.get_q_order(s_order, s_max_order, q_order, q_max_order);
                let q_hash = self.qty.hash(q_order);

                let cubic_tiles_iter = camera
                    .get_hpx_cells(depth_tile, survey_frame)
                    // query the tiles in the camera view
                    .map(|tile_cell| HEALPix3DCell::new(tile_cell, q_hash, q_order))
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

    pub(crate) fn get_cell_texture(&self, cell: &HEALPix3DCell) -> Option<&Hpx3DTex> {
        self.buffer.get(cell)
    }

    pub fn set_qty(&mut self, q: f64) {
        self.qty = match self.qty {
            QtyType::Time(_) => QtyType::Time(crate::math::quantity::time::Time(q as u64)),
            QtyType::Freq(_) => QtyType::Freq(Freq(q))
        };

        // Flag telling to recompute the mesh afterwards
        self.move_qty = true;
    }

    pub fn contains_tile(&self, cell: &HEALPix3DCell) -> bool {
        self.buffer.contains(cell)
    }

    pub fn draw(
        &mut self,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        camera: &mut CameraViewPort,
        cfg: &ImageMetadata,
        proj: &ProjectionType,
        q_order: u8
    ) -> Result<(), JsValue> {
        let available_tiles = self.reset_available_tiles();
        let new_cells_in_view = self.retrieve_cells_in_camera(camera);

        if camera.has_zoomed() {
            al_core::log("has zoomed");
            self.snap_to_nearest_qty(camera, q_order);
        }

        if new_cells_in_view | available_tiles | self.move_qty | camera.has_moved() {
            // TODO: append the vertices independently to the draw method
            self.recompute_vertices(camera, proj, q_order);

            self.move_qty = false;
        }

        self.draw_internal(shaders, colormaps, camera, cfg, proj)
    }

    pub fn get_qty(&self) -> QtyType {
        self.qty
    }

    pub fn snap_to_nearest_qty(&mut self, camera: &CameraViewPort, q_spectra_order: u8) {
        use crate::math::lonlat::LonLat;
        use crate::math::quantity::Qty;
        // Retrieve the central cell containing the cursor
        let cfg = self.get_config();
        let q_max_order = cfg.max_depth_axis2.unwrap_abort();
        let s_max_order = cfg.max_depth_tile;
        let s_order = camera
            .get_tile_depth()
            .min(cfg.get_max_depth_tile())
            .max(cfg.get_min_depth_tile());

        let q_order = self.get_q_order(s_order, s_max_order, q_spectra_order, q_max_order);
        let center = camera.get_center().lonlat();
        let hpx_cell = HEALPixCell::new(
            s_order,
            center.lon().to_radians(),
            center.lat().to_radians()
        );

        let q_hash = self.get_qty().hash(q_order);

        let q_hash_0 = (q_hash >> 1) << 1;
        let q_hash_1 = q_hash_0 + 1;

        let tile_depth = cfg.tile_depth.unwrap() as i64; 
        let dd = tile_depth.ilog2() as u8;
        let slice = self.get_qty().hash(q_order + dd);

        // off is between [ 0; 2 x tile_depth [
        let off = (slice - (q_hash_0 << dd)) as i64;
        assert!(off >= 0 && off < 2*tile_depth);
        // Then we have to seek the nearest occupied slice in these 2 cells
        let mut cell_in = HEALPix3DCell::new(hpx_cell, q_hash_0, q_order);
        let mut cell_near = HEALPix3DCell::new(hpx_cell, q_hash_1, q_order);

        al_core::log(&format!("off: {:?}", off));

        if off >= tile_depth {
            std::mem::swap(&mut cell_in, &mut cell_near);
        }

        al_core::log(&format!("cell in: {:?}", cell_in));

        let mut s: i64 = -1000;
        if self.contains_tile(&cell_in) {
            al_core::log(&format!("cellc: {:?}", cell_in));
        }
        match (self.get_cell_texture(&cell_in), self.get_cell_texture(&cell_near)) {
            (Some(t_in), Some(t_near)) => {
                al_core::log(&format!("both"));

                if off >= tile_depth {
                    // off is between [ tile_depth; 2 x tile_depth [
                    for i in off..(2*tile_depth) {
                        if t_in.contains_slice(&center, (i - tile_depth) as u16) {
                            s = i;
                            break;
                        }
                    }

                    for i in (tile_depth..off).rev() {
                        if t_in.contains_slice(&center, (i - tile_depth) as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }

                    for i in (0..tile_depth).rev() {
                        if t_near.contains_slice(&center, i as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                } else {
                    // off is between [ 0; tile_depth [
                    for i in off..tile_depth {
                        if t_in.contains_slice(&center, i as u16) {
                            s = i;
                            break;
                        }
                    }

                    for i in (0..off).rev() {
                        if t_in.contains_slice(&center, i as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }

                    for i in tile_depth..(2*tile_depth) {
                        if t_near.contains_slice(&center, (i - tile_depth) as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                }
            },
            (Some(t_in), None) => {
                al_core::log(&format!("one"));

                /*for i in 0..tile_depth {
                    if t_in.contains_slice(&center, i as u16) {
                        s = i;
                        break;
                    }
                }*/

                if off >= tile_depth {
                    // off is between [ tile_depth; 2 x tile_depth [
                    for i in off..(2*tile_depth) {
                        if t_in.contains_slice(&center, (i - tile_depth) as u16) {
                            s = i;
                            break;
                        }
                    }

                    for i in (tile_depth..off).rev() {
                        if t_in.contains_slice(&center, (i - tile_depth) as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                } else {
                    // off is between [ 0; tile_depth [
                    for i in off..tile_depth {
                        if t_in.contains_slice(&center, i as u16) {
                            s = i;
                            break;
                        }
                    }

                    for i in (0..off).rev() {
                        if t_in.contains_slice(&center, i as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                }

                al_core::log(&format!("ones: {:?}", s));

            },
            (None, Some(t_near)) => {
                al_core::log(&format!("one2"));

                if off >= tile_depth {
                    // t_near is [0; tile_depth[
                    for i in (0..tile_depth).rev() {
                        if t_near.contains_slice(&center, i as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                } else {
                    // t_near is [tile_depth; 2*tile_depth[
                    for i in tile_depth..(2*tile_depth) {
                        if t_near.contains_slice(&center, (i - tile_depth) as u16) && (s - off).abs() > (i - off).abs() {
                            s = i;
                            break;
                        }
                    }
                }
            },
            (None, None) => {
                al_core::log(&format!("none"));

                ()
            }
        }

        if s == -1000 {
            return;
            //unreachable!();
        }

        al_core::log(&format!("s {:?}", s));

        let pix_hash = (q_hash_0 << dd) + (s as u64);
        let qty = match self.qty {
            QtyType::Time(_) => {
                crate::math::quantity::time::Time::from_hash_with_order(pix_hash, q_order + dd).0 as f64
            },
            QtyType::Freq(_) => {
                Freq::from_hash_with_order(pix_hash, q_order + dd).0 as f64
            }
        };

        al_core::log(&format!("old, new: {:?}, {:?}", self.qty, qty));

        self.set_qty(qty);
    }

    fn recompute_vertices(&mut self, camera: &CameraViewPort, proj: &ProjectionType, q_order: u8) {
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
            .unwrap()
            .min(3);

        let cfg = self.get_config();
        let em_min = cfg.em_min;
        let em_max = cfg.em_max;

        let dataproduct_type = cfg.dataproduct_type;
        let max_depth_tile = cfg.max_depth_tile;

        let cube_depth = cfg.get_cube_depth();
        let max_depth_axis2 = cfg.max_depth_axis2;
        let min_depth_axis2 = cfg.min_depth_axis2;

        for cell in &self.hpx_cells_in_view {
            // filter textures that are not in the moc
            let cell = match dataproduct_type {
                DataproductType::SpectralCube | DataproductType::TimeCube => {
                    // Determination of the q_order from the s_order
                    // From https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf page 3
                    let q_max_order = max_depth_axis2.unwrap_abort();
                    let s_max_order = max_depth_tile;
                    let s_order = cell.depth();

                    let q_order = self.get_q_order(s_order, s_max_order, q_order, q_max_order);
                    let q_hash = self.get_qty().hash(q_order);

                    let hpx_f_cell = HEALPix3DCell::new(*cell, q_hash, q_order);

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
                    let freq: f64 = self.qty.into();
                    let em_min = em_min.unwrap_abort();
                    let em_max = em_max.unwrap_abort();
                    let channel_idx = (((freq - em_min)
                        / (em_max - em_min))
                        * (cube_depth.unwrap_abort() as f64))
                        as u64;

                    let tile_depth = 32;

                    let f_hash = channel_idx / tile_depth;

                    let hpx_f_cell =
                        HEALPix3DCell::new(*cell, f_hash, Frequency::<u64>::MAX_DEPTH);
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
                } else {
                    None
                };

                if let Some(texture) = hpx_cell_texture {
                    let texture_cell = texture.cell.clone();
                    // The slice is sure to be contained so we can unwrap
                    let slice_position = match dataproduct_type {
                        DataproductType::SpectralCube | DataproductType::TimeCube => {
                            // 1. hash of the frequency at max order
                            let f_hash = self.get_qty().hash_max_order();
                            // b. compute the hash range
                            let delta_f_order = self.qty.get_max_order() - texture_cell.q_depth;
                            let f_order_hash_0 = texture_cell.q_hash;
                            let f_order_hash_1 = f_order_hash_0 + 1;

                            // 3. hash range at max order
                            let f_hash_0 = f_order_hash_0 << delta_f_order;
                            let f_hash_1 = f_order_hash_1 << delta_f_order;

                            (f_hash - f_hash_0) as f32 / (f_hash_1 - f_hash_0) as f32
                        }
                        DataproductType::Cube => {
                            let freq: f64 = self.get_qty().into();
                            let channel_idx = (((freq - em_min.unwrap_abort())
                                / (em_max.unwrap_abort() - em_min.unwrap_abort()))
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

                        let mut sub_cell_crosses = false;

                        let mut clips = Vec::with_capacity(4);
                        let mut uvs = Vec::with_capacity(4);

                        for ((lon, lat), (di, dj)) in
                            sub_cell
                                .vertices()
                                .iter()
                                .zip([(0, 0), (1, 0), (1, 1), (0, 1)])
                        {
                            use crate::math::angle::ToAngle;
                            use crate::math::lonlat;
                            let xyz = crate::LonLatT::new(lon.to_angle(), lat.to_angle()).vector();
                            if let Some(clip) = proj.model_to_clip_space(&xyz, camera) {
                                clips.push([clip.x as f32, clip.y as f32]);

                                let hj0 = ((j + dj) as f32) / nside;
                                let hi0 = ((i + di) as f32) / nside;

                                let uv_end = [
                                    uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                                    uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                                    uv_1[TileCorner::BottomLeft].z,
                                ];
                                uvs.extend(uv_end);
                            } else {
                                sub_cell_crosses = true;
                                break;
                            }
                        }

                        use crate::renderable::utils::triangle::Triangle;
                        if sub_cell_crosses || Triangle::new(&clips[0], &clips[2], &clips[1]).is_elongated()
                            || Triangle::new(&clips[0], &clips[3], &clips[2]).is_elongated() {
                            continue;
                        }

                        pos.extend(clips);
                        self.uv.extend(uvs);

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
        cell: &HEALPix3DCell,
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
        cell: &HEALPix3DCell,
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
        cell: &HEALPix3DCell,
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
        cell: &HEALPix3DCell,
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
