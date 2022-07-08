use crate::math::{self, lonlat::LonLat};
use moclib::{
    moc::range::RangeMOC,
    qty::Hpx
};
use cgmath::{Vector3, Vector4};

pub type SMOC = RangeMOC<u64, Hpx<u64>>;

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
        let inside_lonlat = inside.lonlat();
        let inside_hpx = cdshealpix::nested::hash(depth, inside_lonlat.lon().0, inside_lonlat.lat().0);

        let mut moc = RangeMOC::from_polygon(&lonlat[..], false, depth);
        if !moc.contains_depth_max_val(&inside_hpx) {
            moc = moc.complement();
        }

        HEALPixCoverage(moc)
    }

    pub fn allsky() -> Self {
        let moc = RangeMOC::from_full_domain(0);
        HEALPixCoverage(moc)
    }

    pub fn contains(&self, idx: u64) -> bool {
        self.0.contains_depth_max_val(&idx)
    }
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = SMOC;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
