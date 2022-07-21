use crate::math::{self, lonlat::LonLat};
use moclib::{
    moc::range::RangeMOC,
    qty::Hpx
};
use cgmath::{Vector3, Vector4};

pub type SMOC = RangeMOC<u64, Hpx<u64>>;

use crate::healpix::cell::HEALPixCell;
pub struct HEALPixCoverage(pub SMOC);

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

    pub fn allsky(depth_max: u8) -> Self {
        let moc = RangeMOC::new_full_domain(depth_max);
        HEALPixCoverage(moc)
    }

    pub fn contains_coo(&self, vertex: &Vector4<f64>) -> bool {
        let (lon, lat) = math::lonlat::xyzw_to_radec(vertex);
        self.0.is_in(lon.0, lat.0)
    }

    pub fn contains_tile(&self, cell: &HEALPixCell) -> bool {
        self.0.contains_depth_max_val(&cell.idx())
    }

    pub fn intersection(&self, other: &Self) -> bool {
        self.0.intersection(&other.0).is_empty()
    }
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = SMOC;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
