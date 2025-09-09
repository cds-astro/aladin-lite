use crate::math::lonlat::LonLat;
use crate::math::lonlat::LonLatT;
use crate::math::PI;

use moclib::moc::RangeMOCIntoIterator;
use moclib::{
    moc::range::{CellSelection, RangeMOC},
    qty::Hpx,
    ranges::SNORanges,
};
pub type Smoc = RangeMOC<u64, Hpx<u64>>;

use crate::healpix::cell::HEALPixCell;
#[derive(Clone, Debug)]
pub struct SpaceMoc(pub Smoc);

use wasm_bindgen::JsValue;

use moclib::deser::fits;
use moclib::deser::fits::MocIdxType;
use moclib::deser::fits::MocQtyType;
use moclib::idx::Idx;
use moclib::moc::range::op::convert::convert_to_u64;
use moclib::moc::{CellMOCIntoIterator, CellMOCIterator, RangeMOCIterator};
/// Convenient type for Space-MOCs
pub fn from_fits_hpx<T: Idx>(moc: MocType<T, Hpx<T>, Cursor<&[u8]>>) -> Smoc {
    match moc {
        MocType::Ranges(moc) => convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc).into_range_moc(),
        MocType::Cells(moc) => {
            convert_to_u64::<T, Hpx<T>, _, Hpx<u64>>(moc.into_cell_moc_iter().ranges())
                .into_range_moc()
        }
    }
}

use moclib::deser::fits::MocType;
use std::io::Cursor;
impl SpaceMoc {
    pub fn from_fits_raw_bytes(bytes: &[u8]) -> Result<Self, JsValue> {
        let smoc = match fits::from_fits_ivoa_custom(Cursor::new(bytes), true)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
        {
            MocIdxType::U16(MocQtyType::<u16, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
            MocIdxType::U32(MocQtyType::<u32, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
            MocIdxType::U64(MocQtyType::<u64, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
            _ => Err(JsValue::from_str("MOC not supported. Must be a HPX MOC")),
        }?;

        Ok(Self(smoc))
    }

    pub fn from_json(s: &str) -> Result<Self, JsValue> {
        let moc = moclib::deser::json::from_json_aladin::<u64, Hpx<u64>>(s)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?
            .into_cell_moc_iter()
            .ranges()
            .into_range_moc();

        Ok(Self(moc))
    }

    pub fn serialize_to_json(&self) -> Result<String, JsValue> {
        let mut buf: Vec<u8> = Default::default();
        (&self.0)
            .into_range_moc_iter()
            .cells()
            .to_json_aladin(None, &mut buf)
            .map(|()| unsafe { String::from_utf8_unchecked(buf) })
            .map_err(|err| JsValue::from_str(&format!("{err:?}")))
    }

    pub fn from_3d_coos<T: LonLat<f64>>(
        // The depth of the smallest HEALPix cells contained in it
        depth: u8,
        // The vertices of the polygon delimiting the coverage
        vertices_iter: impl Iterator<Item = T>,
        // A vertex being inside the coverage,
        // typically the center of projection
        inside: &T,
    ) -> Self {
        let lonlat = vertices_iter
            .map(|vertex| {
                let LonLatT(lon, lat) = vertex.lonlat();
                (lon.to_radians(), lat.to_radians())
            })
            .collect::<Vec<_>>();

        let LonLatT(in_lon, in_lat) = inside.lonlat();
        let moc = RangeMOC::from_polygon_with_control_point(
            &lonlat[..],
            (in_lon.to_radians(), in_lat.to_radians()),
            depth,
            CellSelection::All,
        );
        SpaceMoc(moc)
    }

    pub fn from_fixed_hpx_cells(
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
    }

    pub fn from_cone(lonlat: &LonLatT<f64>, rad: f64, depth: u8) -> Self {
        if rad >= PI {
            Self::allsky(depth)
        } else {
            SpaceMoc(RangeMOC::from_cone(
                lonlat.lon().to_radians(),
                lonlat.lat().to_radians(),
                rad,
                depth,
                0,
                CellSelection::All,
            ))
        }
    }

    pub fn allsky(depth_max: u8) -> Self {
        let moc = RangeMOC::new_full_domain(depth_max);
        SpaceMoc(moc)
    }

    pub fn contains_lonlat(&self, lonlat: &LonLatT<f64>) -> bool {
        self.0
            .is_in(lonlat.lon().to_radians(), lonlat.lat().to_radians())
    }

    // O(log2(N))
    pub fn intersects_cell(&self, cell: &HEALPixCell) -> bool {
        let z29_rng = cell.z_29_rng();

        self.0.moc_ranges().intersects_range(&z29_rng)
    }

    /*pub fn is_intersecting(&self, other: &Self) -> bool {
        !self.0.intersection(&other.0).is_empty()
    }*/

    pub fn depth(&self) -> u8 {
        self.0.depth_max()
    }

    pub fn sky_fraction(&self) -> f64 {
        self.0.coverage_percentage()
    }

    pub fn not(&self) -> Self {
        SpaceMoc(self.0.not())
    }

    pub fn empty(depth: u8) -> Self {
        SpaceMoc(RangeMOC::new_empty(depth))
    }
}

use core::ops::Deref;
impl Deref for SpaceMoc {
    type Target = Smoc;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
