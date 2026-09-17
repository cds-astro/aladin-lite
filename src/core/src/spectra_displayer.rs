use crate::math::spectra::Freq;
use crate::math::spectra::SpectralUnit;
//use crate::math::spectra::FREQ_MAX;
//use crate::math::spectra::FREQ_MIN;
use crate::renderable::shape::Shape;
use crate::downloader::query;
use js_sys::Object;
use crate::renderable::hips::HiPS;
use crate::math::angle::ToAngle;
use crate::healpix::cell::HEALPixFreqCell;
use crate::LonLatT;
use moclib::qty::Frequency;
use moclib::qty::MocQty;
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

    freq: Freq,
    dfreq: Freq,

    /// The radius of the cone angle
    rad: Angle<f64>,
    location: LonLatT<f64>,

    pub tile_cells: Vec<HEALPixFreqCell>,
    pub hpx_tile_order: u8,
    pub f_tile_order: u8,

    pub s_max_order: u8,
    pub f_max_order: u8,

    tile_depth: u8,
    tile_size: u16,

    em_min: Freq,
    em_max: Freq,
    /// A layer ID is attached
    pub layer: Option<String>
}

pub(crate) struct FrequencyWindow {
    /// hash range at pixel order
    window_pixel_hash: Range<u64>,
    /// The pixel order
    pixel_depth: u8,
    /// em_min/em_max hash
    domain_pixel_hash: Range<u64>,
}

use std::ops::Range;
impl SpectraDisplayer {
    pub(crate) fn new() -> Self {
        let freq = Freq(1.0);
        let dfreq = Freq(1.0);
        let layer = None;
        let rad = 0.0.to_angle();
        let location = LonLatT::new(0.0.to_angle(), 0.0.to_angle());

        let shape = Shape::Circle { c: location, rad };
        let tile_cells = vec![];
        

        SpectraDisplayer {
            freq,
            dfreq,
            shape,
            rad,
            location,
            tile_cells,
            hpx_tile_order: 0,
            f_tile_order: 0,
            f_max_order: 0,
            s_max_order: 0,
            tile_depth: 0,
            tile_size: 1,
            em_min: Freq(1.0),
            em_max: Freq(1.0),
            layer
        }
    }

    /*fn get_dxdy_inside_cell(&self) -> (f64, f64) {
        let s_depth = self.cell.hpx.depth();
        let (lon, lat) = (
            self.location.lon().to_radians(),
            self.location.lat().to_radians(),
        );
        let (cell, dx, dy) = HEALPixCell::hash_with_dxdy(s_depth, lon, lat);

        debug_assert_eq!(cell, self.cell.hpx);

        (dx, dy)
    }*/

    pub(crate) fn f_order(&self) -> u8 {
        self.f_tile_order
    }

    pub(crate) fn hpx_pixel_order(&self) -> u8 {
        self.hpx_tile_order + self.tile_size.trailing_zeros() as u8
    }

    /// Get the window starting and ending hashed at the pixel order
    fn get_window_frequency_range(&self) -> FrequencyWindow {
        const NUM_VALUES: usize = 150;

        let f_delta_depth = self.tile_depth.trailing_zeros();
        let pixel_depth = self.f_tile_order + f_delta_depth as u8;
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

    /*
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
    */

    /// Get the frequency step at the cursor i.e. the jump in frequency
    /// between the cursor slice and the dx-th one
    fn get_frequency_step(&self, dx: i64) -> Freq {
        let f_delta_depth = self.tile_depth.trailing_zeros();
        let f_pixel_depth = self.f_tile_order + f_delta_depth as u8;
        let f_hash_val = self.freq.hash(f_pixel_depth);

        let h1 = (f_hash_val as i64 + dx)
            .min(
                (Frequency::<u64>::n_cells_max() >> (Frequency::<u64>::MAX_DEPTH - f_pixel_depth))
                    as i64,
            )
            .max(0) as u64;

        let h0 = (f_hash_val as i64 - dx)
            .min(
                (Frequency::<u64>::n_cells_max() >> (Frequency::<u64>::MAX_DEPTH - f_pixel_depth))
                    as i64,
            )
            .max(0) as u64;

        let f1 = Freq::from_hash_with_order(h1, f_pixel_depth);
        let f0 = Freq::from_hash_with_order(h0, f_pixel_depth);

        Freq((f1 - f0).0 * 0.5)
    }

    fn get_surrounding_cell_hashes_along_spectra_axis(&self) -> Range<u64> {
        let FrequencyWindow {
            window_pixel_hash: f_hash_val,
            pixel_depth,
            ..
        } = self.get_window_frequency_range();

        let f_delta_depth = pixel_depth - self.f_tile_order;

        // Get the tile hashes from the pixel hashes to load
        let f_hash_0 = f_hash_val.start >> f_delta_depth;
        let f_hash_1 = f_hash_val.end >> f_delta_depth;

        f_hash_0..(f_hash_1 + 1)
    }

    fn is_contained_in_spectral_view(&self, cell: &HEALPixFreqCell) -> bool {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .contains(&cell.f_hash)
    }

    fn get_surrounding_cells_along_spectra_axis(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = HEALPixFreqCell> + use<'_>> + '_ {
        self.get_surrounding_cell_hashes_along_spectra_axis()
            .map(move |f_hash| {
                // Do not include the cell containing the location AND containing the frequency because
                // it will be included when looking for new tiles in the view
                self.tile_cells.iter()
                    .map(move |cell| {
                        HEALPixFreqCell {
                            hpx: cell.hpx,
                            f_hash,
                            f_depth: cell.f_depth,
                        }
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
                HEALPixFreqCell::from_hpx_cell_and_freq(&hpx_cell, self.freq, self.f_tile_order)
            })
            .collect();
    }

    pub(crate) fn get_dfreq(&self) -> &Freq {
        &self.dfreq
    }

    pub(crate) fn set_freq(&mut self, freq: Freq) {   
        const FREQ_MIN: f64 = 1e-18;
        const FREQ_MAX: f64 = 1e+38;
        
        self.freq = Freq(freq.0.min(FREQ_MAX).max(FREQ_MIN));

        let f_order_pixel = self.f_tile_order + (self.tile_depth.ilog2() as u8);
        self.dfreq = self.freq * (10.0_f64.powf(56.0 / ((1 << f_order_pixel) as f64)) - 1.0);

        self.recompute_tile_cells();
    }

    pub(crate) fn get_freq(&self) -> Freq {
        self.freq
    }

    pub(crate) fn get_freq_window(&self) -> [Freq; 2] {
        let FrequencyWindow {
            window_pixel_hash: f_hash_val,
            pixel_depth,
            ..
        } = self.get_window_frequency_range();

        let f0 = Freq::from_hash_with_order(f_hash_val.start, pixel_depth);
        let f1 = Freq::from_hash_with_order(f_hash_val.end, pixel_depth);

        [f0, f1]
    }

    pub(crate) fn attach_hips3d(&mut self, hips: &HiPS3D) {
        let cfg = hips.get_config();

        self.layer = Some(hips.get_layer().to_string());

        self.em_min = cfg.em_min.unwrap_abort();
        self.em_max = cfg.em_max.unwrap_abort();

        self.f_max_order = cfg.max_depth_freq.unwrap_or(Frequency::<u64>::MAX_DEPTH);
        self.s_max_order = cfg.max_depth_tile;

        self.tile_depth = cfg.tile_depth.unwrap_or(1);
        self.tile_size = cfg.tile_size as u16;

        self.set_freq(hips.freq);
    }

    /// Set freq order of the tile
    pub(crate) fn set_freq_order(&mut self, mut f_order: u8) {
        self.f_tile_order = f_order.min(self.f_max_order).max(self.f_max_order - self.s_max_order);
        self.hpx_tile_order = (self.f_tile_order as i8 - self.f_max_order as i8 + self.s_max_order as i8) as u8;

        let f_order_pixel = self.f_tile_order + (self.tile_depth.ilog2() as u8);
        self.dfreq = self.freq * (10.0_f64.powf(56.0 / ((1 << f_order_pixel) as f64)) - 1.0);

        self.recompute_tile_cells();
    }

    /// Set the dfreq resolution between 2 freqs (at pixel level, not tile level)
    pub(crate) fn set_dfreq(&mut self, dfreq: Freq) {
        // The appropriate order required to map a ∆ftarget frequency resolution is approximated by this formulae:
        // f_order ~= log2 (56 * ln(10) * freq / ∆ftarget )

        let f_order_pixel = (56.0 / (1.0 + dfreq.0 / self.freq.0).log10()).log2().floor() as u8;
        let f_order = f_order_pixel - (self.tile_depth.ilog2() as u8);
        self.set_freq_order(f_order);
        // Check if f_order has been correctly applied
        if self.f_tile_order == f_order {
            // set the true resolution
            self.dfreq = dfreq;
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

    pub(crate) fn redraw(&self, layers: &Layers) {
        if let Some(id) = &self.layer {
            if let Some(HiPS::D3(hips)) = layers.get_hips_from_layer(id) {

                // Determine the slices window
                let cell_hash_f = self.get_surrounding_cell_hashes_along_spectra_axis();

                let delta_depth = self.tile_depth.trailing_zeros();
                let pixel_hash_0 = cell_hash_f.start << delta_depth;
                let FrequencyWindow {
                    window_pixel_hash,
                    domain_pixel_hash,
                    pixel_depth,
                } = self.get_window_frequency_range();

                // Determine the frequencies the borders of the window
                let f0 = Freq::from_hash_with_order(window_pixel_hash.start, pixel_depth);
                let f1 = Freq::from_hash_with_order(window_pixel_hash.end, pixel_depth);

                // Determine the spectral step at the spectra_displayer position

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
                    &JsValue::from_f64(self.freq.0),
                )
                .unwrap_abort();
                Reflect::set(
                    &spectra_js_obj,
                    &JsValue::from_str("dfreq"),
                    &JsValue::from_f64(self.dfreq.0),
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

                    /*Reflect::set(
                        &spectra_js_obj,
                        &JsValue::from_str("layer"),
                        &JsValue::from_str(&self.layer),
                    )
                    .unwrap_abort();*/
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

                        freqs.extend(
                            tile_cell.pixel_frequencies(self.tile_depth as usize)
                        );

                        // Now loop over the region
                        let mut i = vec![f32::NAN; f_size];
                        let mut num_pixels = 0;

                        for hpx_pixel_cell in self.get_hpx_region() {
                            let (pixel_lon, pixel_lat) = hpx_pixel_cell.center();
                            let hpx_tile_cell = hpx_pixel_cell.ancestor(dd);

                            if let Some(tile_tex) = hips.get_cell_texture(&HEALPixFreqCell { hpx: hpx_tile_cell, f_hash: tile_cell.f_hash, f_depth: tile_cell.f_depth }) {
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
