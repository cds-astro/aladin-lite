use crate::healpix_cell::HEALPixCell;

use std::ops::Range;
pub struct SourceIndices {
    // Density at depth 7
    density: Box<[Range<u32>]>,
    // Max density for each depth from 0 to 7 included
    max_density: [u32; 8],
}

use super::source::Source;

impl SourceIndices {
    pub fn new(sources: &mut [Source]) -> Self {
        // Sort the sources by HEALPix indices at depth 7
        //let mut rng = StdRng::seed_from_u64(0);

        /*sources.sort_unstable_by(|s1, s2| {
            let idx1 = healpix::nested::hash(7, s1.lon as f64, s1.lat as f64);
            let idx2 = healpix::nested::hash(7, s2.lon as f64, s2.lat as f64);

            let ordering = idx1.partial_cmp(&idx2).unwrap();
            match ordering {
                std::cmp::Ordering::Equal => {
                    rng.gen::<f64>().partial_cmp(&0.5).unwrap()
                    //s1.lon.partial_cmp(&s2.lon).unwrap()
                },
                _ => ordering
            }
            //ordering
        });*/

        let mut healpix_idx: Vec<Option<Range<u32>>> = vec![None; 196608];

        for (idx_source, s) in sources.iter().enumerate() {
            let idx = healpix::nested::hash(7, s.lon as f64, s.lat as f64) as usize;

            if let Some(ref mut healpix_idx) = &mut healpix_idx[idx] {
                healpix_idx.end += 1;
            } else {
                healpix_idx[idx] = Some((idx_source as u32)..((idx_source + 1) as u32));
            }
        }

        let mut idx_source = 0;
        let mut density = Vec::with_capacity(healpix_idx.len());
        for i in 0..healpix_idx.len() {
            if let Some(healpix_idx) = healpix_idx[i].clone() {
                idx_source = healpix_idx.end;
                density.push(healpix_idx);
            } else {
                density.push(idx_source..idx_source);
            };
        }

        let mut max_density = [0_u32; 8];

        let mut tmp = density
            .clone()
            .into_iter()
            .map(|r| r.end - r.start)
            .collect::<Vec<_>>();
        max_density[7] = *tmp.iter().max().unwrap();
        for depth in (0..7).rev() {
            let grouped_densities = unsafe { group_elements_by_4(tmp) };
            assert_eq!(grouped_densities.len(), 12 * (1 << (2 * depth)));
            tmp = grouped_densities
                .iter()
                .map(|e| {
                    let s = e.iter().sum::<u32>();
                    s
                })
                .collect();
            /*
            // Variance
            let mean = (tmp.iter().sum::<u32>() as f32)/(grouped_densities.len() as f32);
            max_density[depth] = tmp.iter()
                .map(|s| ((*s as f32) - mean).abs())
                .sum::<f32>() as u32;
            */
            /*
            // Variance on max
            let mean = (tmp.iter().sum::<u32>() as f32)/(grouped_densities.len() as f32);
            max_density[depth] = (mean - *tmp.iter().max().unwrap() as f32).abs() as u32;
            */
            // Max
            max_density[depth] = *tmp.iter().max().unwrap() as u32;
        }

        //crate::log(&format!("max densities {:?}", max_density));

        SourceIndices {
            density: density.into_boxed_slice(),
            max_density,
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

    pub fn max_density(&self, depth: usize) -> u32 {
        let max_depth_granularity = self.max_density.len();
        if depth >= max_depth_granularity {
            let max_density = self.max_density[max_depth_granularity - 1];
            let shift = 2 * (depth - max_depth_granularity + 1);
            if shift >= 32 {
                1
            } else {
                std::cmp::max(max_density >> shift, 1)
            }
        //self.max_density[max_depth_granularity - 1]
        } else {
            self.max_density[depth]
        }
    }
}

unsafe fn group_elements_by_4<T>(mut v: Vec<T>) -> Vec<[T; 4]> {
    let new_len = v.len() >> 2;
    v.set_len(new_len);
    std::mem::transmute(v)
}
