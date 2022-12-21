use crate::math;
use moclib::{
    moc::range::RangeMOC,
    qty::Hpx
};
use cgmath::{Vector3, Vector4};

pub type Smoc = RangeMOC<u64, Hpx<u64>>;

use crate::healpix::cell::HEALPixCell;
#[derive(Clone)]
pub struct HEALPixCoverage(pub Smoc);

use moclib::elemset::range::MocRanges;
impl HEALPixCoverage {
    pub fn new(
        // The depth of the smallest HEALPix cells contained in it
        depth: u8,
        // The vertices of the polygon delimiting the coverage
        vertices: &[Vector4<f64>],
        // A vertex being inside the coverage,
        // typically the center of projection
        inside: &Vector3<f64>,
    ) -> Self {

        let lonlat = vertices
            .iter()
            .map(|vertex| {
                let (lon, lat) = math::lonlat::xyzw_to_radec(vertex);
                (lon.0, lat.0)
            })
            .collect::<Vec<_>>();
        let (inside_lon, inside_lat) = math::lonlat::xyz_to_radec(inside);
        
        let moc = RangeMOC::from_polygon_with_control_point(&lonlat[..], (inside_lon.0, inside_lat.0), depth);
        HEALPixCoverage(moc)
    }

    pub fn from_hpx_cells(depth: u8, hpx_idx: impl Iterator<Item = u64>, cap: Option<usize>) -> Self {
        let moc = RangeMOC::from_fixed_depth_cells(depth, hpx_idx, cap);
        HEALPixCoverage(moc)
    }

    pub fn allsky(depth_max: u8) -> Self {
        let moc = RangeMOC::new_full_domain(depth_max);
        HEALPixCoverage(moc)
    }

    pub fn contains_coo(&self, vertex: &Vector4<f64>) -> bool {
        let (lon, lat) = math::lonlat::xyzw_to_radec(vertex);
        self.0.is_in(lon.0, lat.0)
    }

    pub fn contains(&self, cell: &HEALPixCell) -> bool {
        let HEALPixCell(depth, idx) = *cell;

        let start_idx = idx << (2*(29 - depth));
        let end_idx = (idx + 1) << (2*(29 - depth));

        let moc = RangeMOC::new(
            29,
            MocRanges::<u64, moclib::qty::Hpx<u64>>::new_unchecked(
                vec![start_idx..end_idx],
            )
        );

        self.is_intersecting(&HEALPixCoverage(moc))
    }

    pub fn is_intersecting(&self, other: &Self) -> bool {
        !self.0.intersection(&other.0).is_empty()
    }

    pub fn depth(&self) -> u8 {
        self.0.depth_max()
    }
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = Smoc;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
