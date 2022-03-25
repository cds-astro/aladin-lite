use crate::healpix_cell::HEALPixCell;

use std::ops::Range;
pub struct SourceIndices {
    // Density at depth 7
    density: Box<[Range<u32>]>,
}

use super::source::Source;

impl SourceIndices {
    pub fn new(sources: &[Source]) -> Self {
        let mut healpix_idx: Box<[Option<Range<u32>>]> = vec![None; 196608].into_boxed_slice();

        for (idx_source, s) in sources.iter().enumerate() {
            let (lon, lat) = s.lonlat();
            let idx = healpix::nested::hash(7, lon as f64, lat as f64) as usize;

            if let Some(ref mut healpix_idx) = &mut healpix_idx[idx] {
                healpix_idx.end += 1;
            } else {
                healpix_idx[idx] = Some((idx_source as u32)..((idx_source + 1) as u32));
            }
        }
        let mut idx_source = 0;

        let healpix_idx = healpix_idx.iter()
            .map(|idx| {
                if let Some(r) = idx {
                    idx_source = r.end;

                    r.start..r.end
                } else {
                    idx_source..idx_source
                }
            })
            .collect::<Vec<_>>();

        SourceIndices {
            density: healpix_idx.into_boxed_slice(),
        }
    }

    pub fn get_source_indices(&self, cell: &HEALPixCell) -> Range<u32> {
        let HEALPixCell(depth, idx) = *cell;

        if depth <= 7 {
            let off = 2 * (7 - depth);

            let healpix_idx_start = (idx << off) as usize;
            let healpix_idx_end = ((idx + 1) << off) as usize;

            let idx_start_sources = self.density[healpix_idx_start].start;
            let idx_end_sources = self.density[healpix_idx_end - 1].end;

            idx_start_sources..idx_end_sources
        } else {
            // depth > 7
            // Get the sources that are contained in parent cell of depth 7
            let off = 2 * (depth - 7);
            let idx_start = (idx >> off) as usize;

            let idx_start_sources = self.density[idx_start].start;
            let idx_end_sources = self.density[idx_start].end;

            idx_start_sources..idx_end_sources
        }
    }

    // Returns k sources from a cell having depth <= 7
    pub fn get_k_sources<'a>(
        &self,
        sources: &'a [f32],
        cell: &HEALPixCell,
        k: usize,
        offset: usize,
    ) -> &'a [f32] {
        let HEALPixCell(depth, idx) = *cell;

        assert!(depth <= 7);
        let off = 2 * (7 - depth);

        let healpix_idx_start = (idx << off) as usize;
        let healpix_idx_end = ((idx + 1) << off) as usize;

        let idx_start_sources = self.density[healpix_idx_start].start as usize;
        let idx_end_sources = self.density[healpix_idx_end - 1].end as usize;

        let num_sources = idx_end_sources - idx_start_sources;

        let idx_sources = if (num_sources - offset) > k {
            (idx_start_sources + offset)..(idx_start_sources + offset + k)
        } else {
            idx_start_sources..idx_end_sources
        };

        let idx_f32 =
            (idx_sources.start * Source::num_f32())..(idx_sources.end * Source::num_f32());
        &sources[idx_f32]
    }
}
