use cgmath::BaseFloat;

use crate::healpix::cell::HEALPixCell;

use std::ops::Range;
pub struct CooIdxVec([(u32, u32); 196608]);

use crate::math::lonlat::LonLat;
impl CooIdxVec {
    /// Build a coordinate index vector from a list of sky coordinates sorted by HEALPix value
    pub fn new<T>(coo: &[T]) -> Self
    where
        T: LonLat<f32>
    {
        let mut coo_idx_vector: [(u32, u32); 196608] = [(u32::MAX, u32::MAX); 196608];

        for (idx, s) in coo.iter().enumerate() {
            let lonlat = s.lonlat();
            let hash = cdshealpix::nested::hash(7, lonlat.lon().to_radians() as f64, lonlat.lat().to_radians() as f64) as usize;

            if coo_idx_vector[hash].0 == u32::MAX {
                let idx_u32 = idx as u32;
                coo_idx_vector[hash] = (idx_u32, idx_u32 + 1);
            } else {
                coo_idx_vector[hash].1 += 1;
            }
        }

        let mut idx_source = 0;

        for coo_idx in coo_idx_vector.iter_mut() {
            if coo_idx.0 == u32::MAX {
                *coo_idx = (idx_source, idx_source);
            } else {
                idx_source = coo_idx.1;
            }
        }

        CooIdxVec(coo_idx_vector)
    }

    pub fn get_source_indices(&self, cell: &HEALPixCell) -> Range<u32> {
        let HEALPixCell(depth, idx) = *cell;

        if depth <= 7 {
            let off = 2 * (7 - depth);

            let healpix_idx_start = (idx << off) as usize;
            let healpix_idx_end = ((idx + 1) << off) as usize;

            let idx_start_sources = self.0[healpix_idx_start].0;
            let idx_end_sources = self.0[healpix_idx_end - 1].1;

            idx_start_sources..idx_end_sources
        } else {
            // depth > 7
            // Get the sources that are contained in parent cell of depth 7
            let off = 2 * (depth - 7);
            let idx_start = (idx >> off) as usize;

            let idx_start_sources = self.0[idx_start].0;
            let idx_end_sources = self.0[idx_start].1;

            idx_start_sources..idx_end_sources
        }
    }

    // Returns k sources from a cell having depth <= 7
    pub fn get_k_sources<'a, S, T>(
        &self,
        sources: &'a [T],
        cell: &HEALPixCell,
        k: usize,
        offset: usize,
    ) -> &'a [T]
    where
        S: BaseFloat,
        T: LonLat<S>
    {
        let HEALPixCell(depth, idx) = *cell;

        debug_assert!(depth <= 7);
        let off = 2 * (7 - depth);

        let healpix_idx_start = (idx << off) as usize;
        let healpix_idx_end = ((idx + 1) << off) as usize;

        let idx_start_sources = self.0[healpix_idx_start].0 as usize;
        let idx_end_sources = self.0[healpix_idx_end - 1].1 as usize;

        let num_sources = idx_end_sources - idx_start_sources;

        let idx_sources = if (num_sources - offset) > k {
            (idx_start_sources + offset)..(idx_start_sources + offset + k)
        } else {
            idx_start_sources..idx_end_sources
        };

        &sources[idx_sources]
    }
}
