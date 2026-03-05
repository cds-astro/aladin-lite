use crate::healpix::cell::HEALPixFreqCell;
use moclib::hpxranges2d::HpxRanges2D;
use moclib::ranges::ranges2d::Ranges2D;

use moclib::qty::{Frequency, MocQty};

#[derive(Debug)]
pub struct FreqSpaceMoc(pub moclib::hpxranges2d::FreqSpaceMoc<u64, u64>);

impl Clone for FreqSpaceMoc {
    fn clone(&self) -> Self {
        let HpxRanges2D(Moc2DRanges {
            ranges2d: Ranges2D { x, y },
            ..
        }) = &**self;

        Self(HpxRanges2D(Moc2DRanges::new(x.clone(), y.clone())))
    }
}

use wasm_bindgen::JsValue;

use moclib::deser::fits;
use moclib::deser::fits::MocIdxType;
use moclib::deser::fits::MocQtyType;
use moclib::mocranges2d::Moc2DRanges;

use std::io::Cursor;

impl FreqSpaceMoc {
    /// Create a FreqSpaceMoc from a
    pub fn from_space_moc(moc: SpaceMoc) -> Self {
        let moc_2d = Moc2DRanges::new(vec![0..u64::MAX; 1], vec![moc.0.into_moc_ranges().0]);
        FreqSpaceMoc(HpxRanges2D(moc_2d))
    }

    pub fn from_fits_raw_bytes(bytes: &[u8]) -> Result<Self, JsValue> {
        let sfmoc = match fits::from_fits_ivoa_custom(Cursor::new(bytes), true)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
        {
            //MocIdxType::U16(MocQtyType::<u16, _>::FreqHpx(moc)) => Ok(from_fits_hpx(moc)),
            //MocIdxType::U32(MocQtyType::<u32, _>::FreqHpx(moc)) => Ok(from_fits_hpx(moc)),
            MocIdxType::U64(MocQtyType::<u64, _>::FreqHpx(ranges_iter)) => {
                /*al_core::log(&format!(
                    "ranges moc 2D iter from fits {:?}",

                ));*/
                let moc_2d_ranges = Moc2DRanges::from_ranges_it(ranges_iter);
                let inner = moclib::hpxranges2d::HpxRanges2D(moc_2d_ranges);
                Ok(inner)
            }
            _ => Err(JsValue::from_str(
                "MOC not supported. Must be a FREQ|HPX 2DMOC coded on U64 only",
            )),
        }?;

        Ok(Self(sfmoc))
    }

    /*pub fn from_fixed_hpx_cells(
        depth: u8,
        hpx_idx: impl Iterator<Item = u64>,
        cap: Option<usize>,
    ) -> Self {
        let moc = RangeMOC::from_fixed_depth_cells(depth, hpx_idx, cap);
        SpaceMoc(moc)
    }

    pub fn from_hpx_cells<'a>(
        depth: u8,
        hpx_cell_it: impl Iterator<Item = &'a HEALPixCell>,
        cap: Option<usize>,
    ) -> Self {
        let cells_it = hpx_cell_it.map(|HEALPixCell(depth, idx)| (*depth, *idx));

        let moc = RangeMOC::from_cells(depth, cells_it, cap);
        SpaceMoc(moc)
    }*/

    pub fn f_max_depth(&self) -> u8 {
        self.0.compute_min_depth().0
    }

    pub fn s_max_depth(&self) -> u8 {
        self.0.compute_min_depth().1
    }

    pub fn sky_fraction(&self) -> f64 {
        todo!()
    }

    pub fn intersects_cell(&self, cell: &HEALPixFreqCell) -> bool {
        let HEALPixFreqCell {
            hpx,
            f_hash,
            f_depth,
        } = *cell;

        let f_hash_0 = f_hash << (Frequency::<u64>::MAX_DEPTH - f_depth);
        let f_hash_1 = (f_hash + 1) << (Frequency::<u64>::MAX_DEPTH - f_depth);

        let hpx_ranges_2d = HpxRanges2D::create_from_freq_ranges_positions(
            vec![f_hash_0..f_hash_1; 1],
            vec![hpx.idx()],
            Frequency::<u64>::MAX_DEPTH,
            hpx.depth(),
        );

        !self.0.intersection(&hpx_ranges_2d).is_empty()
    }

    /*/// provide the list of (hash hpx, hash freq) of the cells contained in the sfmoc
    pub fn cells(&self) -> impl Iterator<Item = (u64, u64)> {
        todo!()
    }*/
}

use core::ops::Deref;

use super::SpaceMoc;
impl Deref for FreqSpaceMoc {
    type Target = moclib::hpxranges2d::FreqSpaceMoc<u64, u64>;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
