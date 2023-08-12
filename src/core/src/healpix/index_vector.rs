use crate::{
    healpix::cell::HEALPixCell,
    math::sph_geom::great_circle_arc::{GreatCircleArc, HEALPixBBox},
};

use std::ops::Range;
#[derive(Debug)]
pub struct IdxVec(Box<[(u32, u32)]>);

use crate::math::lonlat::LonLat;
impl IdxVec {
    /// Build a coordinate index vector from a list of sky coordinates sorted by HEALPix value
    pub fn from_coo<T>(coos: &mut [T]) -> Self
    where
        T: LonLat<f32>,
    {
        coos.sort_unstable_by(|c1, c2| {
            let ll1 = c1.lonlat();
            let ll2 = c2.lonlat();

            let h1 = healpix::nested::hash(
                7,
                ll1.lon().to_radians() as f64,
                ll1.lat().to_radians() as f64,
            );
            let h2 = healpix::nested::hash(
                7,
                ll2.lon().to_radians() as f64,
                ll2.lat().to_radians() as f64,
            );

            h1.cmp(&h2)
        });

        let mut coo_idx_vector = vec![(u32::MAX, u32::MAX); 196608];

        for (idx, s) in coos.iter().enumerate() {
            let lonlat = s.lonlat();
            let hash = healpix::nested::hash(
                7,
                lonlat.lon().to_radians() as f64,
                lonlat.lat().to_radians() as f64,
            ) as usize;

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

        IdxVec(coo_idx_vector.into_boxed_slice())
    }

    // Create an index vector from a list of segments
    pub fn from_great_circle_arc(arcs: &mut [GreatCircleArc]) -> Self {
        arcs.sort_unstable_by(|a1, a2| {
            let bbox1 = a1.get_containing_hpx_cell();
            let bbox2 = a2.get_containing_hpx_cell();

            bbox1.cmp(&bbox2)
        });

        // At this point the arcs are sorted by the z-order curve of their
        // HEALPix cell bbox
        let zorder_hpx_cell_iter = arcs.iter().filter_map(|arc| {
            let hpx_bbox = arc.get_containing_hpx_cell();

            match hpx_bbox {
                HEALPixBBox::AllSky => None,
                HEALPixBBox::Cell(cell) => Some(cell),
            }
        });
        Self::from_hpx_cells(zorder_hpx_cell_iter)
    }

    // Create an index vector from a list of healpix cells sorted by z-order curve
    pub fn from_hpx_cells<'a>(zorder_hpx_cell_iter: impl Iterator<Item = &'a HEALPixCell>) -> Self {
        let mut hpx_idx_vector = vec![(u32::MAX, u32::MAX); 196608];

        for (idx, hpx_cell) in zorder_hpx_cell_iter.enumerate() {
            let HEALPixCell(hpx_cell_depth, hpx_cell_idx) = *hpx_cell;
            let hpx_cells_7 = if hpx_cell_depth >= 7 {
                let hpx_cell_7_start = hpx_cell_idx >> (2 * (hpx_cell_depth - 7));
                let hpx_cell_7_end = hpx_cell_7_start + 1;

                (hpx_cell_7_start as usize)..(hpx_cell_7_end as usize)
            } else {
                let shift = 2 * (7 - hpx_cell_depth);

                let hpx_cell_7_start = hpx_cell_idx << shift;
                let hpx_cell_7_end = (hpx_cell_idx + 1) << shift;

                (hpx_cell_7_start as usize)..(hpx_cell_7_end as usize)
            };

            for hash in hpx_cells_7 {
                if hpx_idx_vector[hash].0 == u32::MAX {
                    let idx_u32 = idx as u32;
                    hpx_idx_vector[hash] = (idx_u32, idx_u32 + 1);
                } else {
                    hpx_idx_vector[hash].1 += 1;
                }
            }
        }

        let mut idx_hash = 0;

        for item in hpx_idx_vector.iter_mut() {
            if item.0 == u32::MAX {
                *item = (idx_hash, idx_hash);
            } else {
                idx_hash = item.1;
            }
        }

        IdxVec(hpx_idx_vector.into_boxed_slice())
    }

    #[inline]
    pub fn get_item_indices_inside_hpx_cell(&self, cell: &HEALPixCell) -> Range<usize> {
        let HEALPixCell(depth, idx) = *cell;

        if depth <= 7 {
            let off = 2 * (7 - depth);

            let healpix_idx_start = (idx << off) as usize;
            let healpix_idx_end = ((idx + 1) << off) as usize;

            let idx_start_sources = self.0[healpix_idx_start].0;
            let idx_end_sources = self.0[healpix_idx_end - 1].1;

            (idx_start_sources as usize)..(idx_end_sources as usize)
        } else {
            // depth > 7
            // Get the sources that are contained in parent cell of depth 7
            let off = 2 * (depth - 7);
            let idx_start = (idx >> off) as usize;

            let idx_start_sources = self.0[idx_start].0;
            let idx_end_sources = self.0[idx_start].1;

            (idx_start_sources as usize)..(idx_end_sources as usize)
        }
    }
}
