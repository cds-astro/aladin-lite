use crate::math::lonlat::LonLatT;
use crate::math::PI;
use crate::math::{self, lonlat::LonLat};

use cgmath::Vector3;
use moclib::elemset::range::uniq::HpxUniqRanges;
use moclib::hpxranges2d::HpxRanges2D;
use moclib::moc::RangeMOCIntoIterator;
use moclib::moc2d::{HasTwoMaxDepth, RangeMOC2Iterator};
use moclib::ranges::ranges2d::Ranges2D;
use moclib::{
    moc::range::{CellSelection, RangeMOC},
    moc2d::range::RangeMOC2,
    qty::Hpx,
    ranges::SNORanges,
};

use moclib::qty::{Frequency, MocQty};

use crate::healpix::cell::HEALPixCell;
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
use moclib::deser::fits::RangeMoc2DIterFromFits;
use moclib::idx::Idx;
use moclib::moc::range::op::convert::convert_to_u64;
use moclib::moc::{CellMOCIntoIterator, CellMOCIterator, RangeMOCIterator};
use moclib::mocranges2d::Moc2DRanges;

use moclib::deser::fits::MocType;
use std::io::Cursor;

use crate::math::spectra::Freq;
use crate::math::spectra::SpectralUnit;
impl FreqSpaceMoc {
    /// Create a FreqSpaceMoc from a
    pub fn from_space_moc(moc: SpaceMoc) -> Self {
        let moc_2d = Moc2DRanges::new(vec![0..u64::MAX], vec![moc.0.into_moc_ranges().0]);
        FreqSpaceMoc(HpxRanges2D(moc_2d))
    }

    pub fn from_fits_raw_bytes(bytes: &[u8]) -> Result<Self, JsValue> {
        let sfmoc = match fits::from_fits_ivoa_custom(Cursor::new(bytes), true)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
        {
            //MocIdxType::U16(MocQtyType::<u16, _>::FreqHpx(moc)) => Ok(from_fits_hpx(moc)),
            //MocIdxType::U32(MocQtyType::<u32, _>::FreqHpx(moc)) => Ok(from_fits_hpx(moc)),
            MocIdxType::U64(MocQtyType::<u64, _>::FreqHpx(ranges_iter)) => {
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

    /// This methods builds a SFMOC made of:
    /// * the cells in the spatial viewport at a specific frequency f
    /// * the +/- f_window cells containing inside lonlat on the frequency axis
    /// This is the method to use when looking for new cube HiPS3D tiles
    pub fn from_coos_freq<L: LonLat<f64>, F: SpectralUnit>(
        // The depth of the smallest HEALPix cells contained in it
        depth: u8,
        // The vertices of the polygon delimiting the coverage
        vertices_iter: impl Iterator<Item = L>,
        // A vertex being inside the coverage,
        // typically the center of projection
        inside: &L,
        // The freq at which we want to compute the sfmoc
        f: F,
        // Frequency window i.e. the number of cells around f to query
        f_window: u8,
    ) -> Self {
        let freq: Freq = f.into();

        todo!();

        /*let lonlat = vertices_iter
            .map(|vertex| {
                let LonLatT(lon, lat) = vertex.lonlat();
                (lon.to_radians(), lat.to_radians())
            })
            .collect::<Vec<_>>();

        let LonLatT(in_lon, in_lat) = inside.lonlat();
        let moc = RangeMOC2::from_freqranges_in_hz_and_coos(
            &lonlat[..],
            (in_lon.to_radians(), in_lat.to_radians()),
            depth,
            CellSelection::All,
        );
        SpaceFreqMoc(moc)*/
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
        } = cell;
        let hpx_ranges_2d = HpxRanges2D::create_from_freq_positions(
            vec![*f_hash],
            vec![hpx.idx()],
            *f_depth,
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

/// A simple object describing a cubic tile of a HiPS3D
pub struct HEALPixFreqCell {
    pub hpx: HEALPixCell,
    pub f_hash: u64,
    pub f_depth: u8,
}

impl HEALPixFreqCell {
    pub fn new(hpx: HEALPixCell, f: Freq, f_depth: u8) -> Self {
        let f_hash = f.hash(f_depth);

        Self {
            hpx,
            f_hash,
            f_depth,
        }
    }

    pub fn from_f_hash(hpx: HEALPixCell, f_hash: u64) -> Self {
        Self {
            hpx,
            f_hash,
            f_depth: 16,
        }
    }
}
