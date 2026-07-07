use crate::math::quantity::frequency::Freq;
use crate::math::quantity::time::Time;

use crate::math::quantity::Qty;
use crate::math::quantity::QtyType;

use crate::renderable::shape::Shape;
use crate::downloader::query;
use js_sys::Object;
use crate::renderable::hips::HiPS;
use crate::math::angle::ToAngle;
use crate::healpix::cell::HEALPix3DCell;
use crate::LonLatT;
use js_sys::Reflect;
use crate::tile_fetcher::TileFetcherQueue;

use crate::renderable::hips::d3::HiPS3D;

use wasm_bindgen::JsValue;

use crate::Abort;

use crate::HEALPixCell;

use crate::renderable::Layers;
use crate::browser_support::BrowserFeaturesSupport;
use crate::math::angle::Angle;
pub(crate) struct SpectraDisplayer {
    shape: Shape,

    q: QtyType,
    dq: QtyType,

    /// The radius of the cone angle
    rad: Angle<f64>,
    location: LonLatT<f64>,

    pub tile_cells: Vec<HEALPix3DCell>,
    pub hpx_tile_order: u8,
    pub q_tile_order: u8,

    pub s_max_order: u8,
    pub q_max_order: u8,

    tile_depth: u8,
    tile_size: u16,

    min: QtyType,
    max: QtyType,
    /// A layer ID is attached
    pub layer: Option<String>,

    hidden: bool,
}

pub(crate) struct Window {
    /// hash range at pixel order
    hash: Range<u64>,
    /// The pixel order
    depth: u8,
    /// em_min/em_max hash
    domain_hash: Range<u64>,
}

use std::ops::Range;
impl SpectraDisplayer {
    pub(crate) fn new() -> Self {
        let q = QtyType::Freq(Freq::new(1.0));
        let dq = QtyType::Freq(Freq::new(1.0));
        let layer = None;
        let rad = 0.0.to_angle();
        let location = LonLatT::new(0.0.to_angle(), 0.0.to_angle());

        let shape = Shape::Circle { c: location, rad };
        let tile_cells = vec![];

        SpectraDisplayer {
            q,
            dq,
            shape,
            rad,
            location,
            tile_cells,
            hpx_tile_order: 0,
            q_tile_order: 0,
            q_max_order: 0,
            s_max_order: 0,
            tile_depth: 0,
            tile_size: 1,
            min: QtyType::Freq(Freq(1.0)),
            max: QtyType::Freq(Freq(1.0)),
            layer,
            hidden: false,
        }
    }

    pub(crate) fn q_order(&self) -> u8 {
        self.q_tile_order
    }

    pub(crate) fn hpx_pixel_order(&self) -> u8 {
        self.hpx_tile_order + self.tile_size.trailing_zeros() as u8
    }

    /// Get the window starting and ending hashed at the pixel order
    fn get_window(&self) -> Window {
        const NUM_VALUES: usize = 150;

        let q_delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.q_tile_order + q_delta_depth as u8;
        let q_hash_val = self.q.hash(pixel_depth);

        let max_cells_num = match self.q {
            QtyType::Freq(_) => Freq::num_max_cells(pixel_depth),
            QtyType::Time(_) => Time::num_max_cells(pixel_depth),
        };

        let q_hash_val_0 = (q_hash_val as i64 - NUM_VALUES as i64).max(0) as u64;
        let q_hash_val_1 =
            (q_hash_val + NUM_VALUES as u64).min(max_cells_num as u64);

        let min_hash = self.min.hash(pixel_depth);
        let max_hash = self.max.hash(pixel_depth);

        Window {
            hash: q_hash_val_0..q_hash_val_1,
            depth: pixel_depth,
            domain_hash: min_hash..max_hash,
        }
    }

    fn get_surrounding_cell_hashes_along_spectra_axis(&self) -> Range<u64> {
        let Window {
            hash,
            depth,
            ..
        } = self.get_window();

        let q_delta_depth = depth - self.q_tile_order;

        // Get the tile hashes from the pixel hashes to load
        let q_hash_0 = hash.start >> q_delta_depth;
        let q_hash_1 = hash.end >> q_delta_depth;

        q_hash_0..(q_hash_1 + 1)
    }

    fn is_contained_in_spectral_view(&self, cell: &HEALPix3DCell) -> bool {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .contains(&cell.q_hash)
    }

    fn get_surrounding_cells_along_spectra_axis(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = HEALPix3DCell> + use<'_>> + '_ {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .map(move |q_hash| {
                // Do not include the cell containing the location AND containing the frequency because
                // it will be included when looking for new tiles in the view
                self.tile_cells.iter()
                    .map(move |cell| {
                        HEALPix3DCell::new(
                            cell.hpx,
                            q_hash,
                            cell.q_depth
                        )
                    })
            })
    }
}

impl SpectraDisplayer {
    pub(crate) fn set_location(&mut self, location: &LonLatT<f64>) {
        self.location = *location;
        self.shape = Shape::Circle { c: self.location, rad: self.rad };

        //let s_order = self.cell.hpx.depth();
        //let f_order = self.f_max_order - (self.s_max_order - s_order);
        self.recompute_tile_cells();
    }

    pub(crate) fn set_radius(&mut self, rad: Angle<f64>) {
        self.rad = rad;
        self.shape = Shape::Circle { c: self.location, rad: self.rad };

        //let s_order = self.cell.hpx.depth();
        //let f_order = self.f_max_order - (self.s_max_order - s_order);
        self.recompute_tile_cells();
    }

    /// This method recomputes the tile cells needed by the spectra displayer
    fn recompute_tile_cells(&mut self) {
        self.tile_cells = self.shape
            .to_flattened_hpx_cells(self.hpx_tile_order)
            .into_iter()
            .map(|hpx_cell| {
                match self.q {
                    QtyType::Freq(q) => HEALPix3DCell::from_hpx_cell_and_qty::<Freq>(&hpx_cell, q, self.q_tile_order),
                    QtyType::Time(q) => HEALPix3DCell::from_hpx_cell_and_qty::<Time>(&hpx_cell, q, self.q_tile_order)
                }
            })
            .collect();
    }

    pub(crate) fn get_dq(&self) -> &QtyType {
        &self.dq
    }

    pub(crate) fn set_qty(&mut self, q: f64) {
        match self.q {
            QtyType::Freq(_) => {
                self.q = QtyType::Freq(Freq::new(q));
            },
            QtyType::Time(_) => {
                self.q = QtyType::Time(Time::new(q as u64));
            },
        }

        let q_order_pixel = self.q_tile_order + (self.tile_depth.ilog2() as u8);
        self.dq = self.q.dq(q_order_pixel);

        self.recompute_tile_cells();
    }

    pub(crate) fn get_qty(&self) -> QtyType {
        self.q
    }

    pub(crate) fn attach_hips3d(&mut self, hips: &HiPS3D) {
        let cfg = hips.get_config();

        self.layer = Some(hips.get_layer().to_string());

        self.s_max_order = cfg.max_depth_tile;

        self.tile_depth = cfg.tile_depth.unwrap_or(1);
        self.tile_size = cfg.tile_size as u16;

        match hips.qty {
            QtyType::Freq(_) => {
                use crate::math::quantity::frequency::Wavelength;
                self.min = QtyType::Freq(Wavelength(cfg.em_min.unwrap_abort()).into());
                self.max = QtyType::Freq(Wavelength(cfg.em_max.unwrap_abort()).into());

                self.q_max_order = cfg.max_depth_axis2.unwrap_or(Freq::MAX_DEPTH);
            },
            QtyType::Time(_) => {
                self.min = QtyType::Time(Time::from_mjd(cfg.t_min.unwrap_abort()));
                self.max = QtyType::Time(Time::from_mjd(cfg.t_max.unwrap_abort()));

                self.q_max_order = cfg.max_depth_axis2.unwrap_or(Time::MAX_DEPTH);
            }
        }

        self.q = hips.qty;

        self.hpx_tile_order = self.hpx_tile_order.min(self.s_max_order);
        let q_tile_order = ((self.hpx_tile_order as i8 + self.q_max_order as i8 - self.s_max_order as i8).max(0)) as u8;

        self.set_q_order(q_tile_order);
    }

    /// Set freq order of the tile
    pub(crate) fn set_q_order(&mut self, q_order: u8) {
        self.q_tile_order = q_order.min(self.q_max_order).max(0);
        self.hpx_tile_order = ((self.q_tile_order as i8 - self.q_max_order as i8 + self.s_max_order as i8).max(0)) as u8;

        let q_order_pixel = self.q_tile_order + (self.tile_depth.ilog2() as u8);
        self.dq = self.q.dq(q_order_pixel);

        self.recompute_tile_cells();
    }

    /// Set the dfreq resolution between 2 freqs (at pixel level, not tile level)
    pub(crate) fn set_dq(&mut self, v: f64) {
        // The appropriate order required to map a ∆ftarget frequency resolution is approximated by this formulae:
        // f_order ~= log2 (56 * ln(10) * freq / ∆ftarget )

        let dq = match self.dq {
            QtyType::Freq(_) => {
                QtyType::Freq(Freq::new(v))
            },
            QtyType::Time(_) => {
                QtyType::Time(Time::new(v as u64))
            }
        };

        let q_order_pixel = self.q.order(dq);
        let q_order = (q_order_pixel as i8 - self.tile_depth.ilog2() as i8).max(0) as u8;
        self.set_q_order(q_order);
        // Check if f_order has been correctly applied
        if self.q_tile_order == q_order {
            // set the true resolution
            self.dq = dq;
        }
    }

    pub(crate) fn launch_new_tile_requests(&mut self, tile_fetcher: &mut TileFetcherQueue, browser_features_support: &BrowserFeaturesSupport, layers: &Layers) {
        if let Some(id) = &self.layer {
            if let Some(HiPS::D3(hips)) = layers.get_hips_from_layer(id) {
                // Query the tiles under the spectra_displayer as well
                let cfg = hips.get_config();

                let tiles_to_fetch_iter = self.get_surrounding_cells_along_spectra_axis()
                    .flatten()
                    // filter the cubic tiles by the sfmoc
                    .filter(|cell| {
                        if hips.contains_tile(&cell) {
                            false
                        } else if let Some(moc) = hips.get_moc() {
                            moc.intersects_cell(&cell)
                        } else {
                            true
                        }
                    });

                for tile in tiles_to_fetch_iter {
                    tile_fetcher.append(query::Tile::new_cubic(
                        &tile,
                        cfg,
                        browser_features_support,
                    ));
                }
            }
        }
    }

    /// Get the hpx cells of the region where the spectra is evaluated
    pub(crate) fn get_hpx_region(&self) -> Vec<HEALPixCell> {
        let hpx_pixel_order = self.hpx_tile_order + self.tile_size.ilog2() as u8;

        self.shape
            .to_flattened_hpx_cells(hpx_pixel_order)            
    }

    pub(crate) fn hide(&mut self) {
        self.hidden = true;
    }

    pub(crate) fn show(&mut self) {
        self.hidden = false;
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub(crate) fn redraw(&self, layers: &Layers) {
        if self.hidden {
            return;
        }

        if let Some(id) = &self.layer {
            if let Some(HiPS::D3(hips)) = layers.get_hips_from_layer(id) {
                // Determine the slices window
                let cell_hash_f = self.get_surrounding_cell_hashes_along_spectra_axis();

                let delta_depth = self.tile_depth.trailing_zeros();
                let pixel_hash_0 = cell_hash_f.start << delta_depth;
                let Window {
                    hash,
                    domain_hash,
                    depth,
                } = self.get_window();

                let (q0, q1) = match self.q {
                    QtyType::Freq(_) => (
                        Freq::from_hash_with_order(hash.start, depth).0 as f64,
                        Freq::from_hash_with_order(hash.end, depth).0 as f64
                    ),
                    QtyType::Time(_) => (
                        Time::from_hash_with_order(hash.start, depth).0 as f64,
                        Time::from_hash_with_order(hash.end, depth).0 as f64
                    )
                };
                // Determine the spectral step at the spectra_displayer position
                let indices =
                    (hash.start - pixel_hash_0)..(hash.end - pixel_hash_0);

                // create the js object containing:
                // * spectra values
                // * min and max frequency values
                let spectra_js_obj = Object::new();

                // Set properties using Reflect::set
                Reflect::set(
                    &spectra_js_obj,
                    &JsValue::from_str("freqMin"),
                    &JsValue::from_f64(q0),
                )
                .unwrap_abort();
                Reflect::set(
                    &spectra_js_obj,
                    &JsValue::from_str("freqMax"),
                    &JsValue::from_f64(q1),
                )
                .unwrap_abort();
                Reflect::set(
                    &spectra_js_obj,
                    &JsValue::from_str("freq"),
                    &JsValue::from_f64(self.q.into()),
                )
                .unwrap_abort();
                Reflect::set(
                    &spectra_js_obj,
                    &JsValue::from_str("dfreq"),
                    &JsValue::from_f64(self.dq.into()),
                )
                .unwrap_abort();

                let mut start = hash.start.max(domain_hash.start);
                let mut end = hash.end.min(domain_hash.end);

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
                }

                // Determine the spectral values in the window
                let mut freqs = vec![];
                let dd = self.tile_size.ilog2() as u8;
                let f_size = self.tile_depth as usize;

                let spectra = self
                    .get_surrounding_cells_along_spectra_axis()
                    .flat_map(|mut cells_iter| {                       
                        let tile_cell = cells_iter
                            .next()
                            .unwrap();

                        match self.q {
                            QtyType::Freq(_) => {
                                let pixels = tile_cell.pixels::<Freq>(self.tile_depth as usize).map(|f| f.0 as f32);
                                freqs.extend(pixels);
                            },
                            QtyType::Time(_) => {
                                let pixels = tile_cell.pixels::<Time>(self.tile_depth as usize).map(|f| f.0 as f32);
                                freqs.extend(pixels);
                            },
                        };

                        // Now loop over the region
                        let mut i = vec![f32::NAN; f_size];
                        let mut num_pixels = 0;

                        for hpx_pixel_cell in self.get_hpx_region() {
                            let (pixel_lon, pixel_lat) = hpx_pixel_cell.center();
                            let hpx_tile_cell = hpx_pixel_cell.ancestor(dd);

                            if let Some(tile_tex) = hips.get_cell_texture(&HEALPix3DCell::new(hpx_tile_cell, tile_cell.q_hash, tile_cell.q_depth)) {
                                let (_, dy, dx) = HEALPixCell::hash_with_dxdy(hpx_tile_cell.depth(), pixel_lon, pixel_lat);

                                let x = (dx * (self.tile_size as f64)) as u32;
                                let y = (dy * (self.tile_size as f64)) as u32;

                                for z in 0..f_size {
                                    let intensity = tile_tex.read_pixel(x, y, z as u32).unwrap_or(f32::NAN);
                                    if i[z].is_nan() && !intensity.is_nan() {
                                        i[z] = intensity;
                                    } else {
                                        i[z] += intensity;
                                    }
                                }

                                num_pixels += 1;
                            }
                        }

                        for intensity in &mut i {
                            *intensity /= num_pixels as f32;
                        }
                        
                        i
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
        }
    }
}
