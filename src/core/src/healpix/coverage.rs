use crate::math::{self, lonlat::LonLat};
use cdshealpix::nested::bmoc::{Status, BMOC};
use cgmath::{Vector3, Vector4};
pub struct HEALPixCoverage(pub BMOC);

use cdshealpix::nested::bmoc::BMOCBuilderFixedDepth;
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
        let moc = cdshealpix::nested::polygon_coverage(depth, &lonlat[..], false);
        let inside_lonlat = inside.lonlat();
        let result = moc.test_coo(inside_lonlat.lon().0, inside_lonlat.lat().0);
        let moc = match result {
            Status::OUT => moc.not(),
            _ => moc,
        };
        HEALPixCoverage(moc)
    }

    pub fn allsky() -> Self {
        let mut moc_builder = BMOCBuilderFixedDepth::new(0, true);
        for hash in 0..12 {
            moc_builder.push(hash);
        }

        let bmoc = moc_builder.to_bmoc().unwrap();
        Self(bmoc)
    }
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = BMOC;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
