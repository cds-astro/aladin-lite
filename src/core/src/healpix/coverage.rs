use crate::{
    math::{
        self,
        lonlat::LonLat,
    }
};
use cgmath::{Vector3, Vector4};
use healpix::nested::bmoc::{Status, BMOC};
pub struct HEALPixCoverage(BMOC);

pub fn from_polygon(
    // The depth of the smallest HEALPix cells contained in it
    depth: u8,
    // The vertices of the polygon delimiting the coverage
    vertices: &[Vector4<f64>],
    // A vertex being inside the coverage,
    // typically the center of projection
    inside: &Vector3<f64>,
) -> HEALPixCoverage {
    let lonlat = vertices
        .iter()
        .map(|vertex| {
            let (lon, lat) = math::lonlat::xyzw_to_radec(vertex);
            (lon.0, lat.0)
        })
        .collect::<Vec<_>>();
    let moc = healpix::nested::polygon_coverage(depth, &lonlat[..], false);
    let inside_lonlat = inside.lonlat();
    let result = moc.test_coo(inside_lonlat.lon().0, inside_lonlat.lat().0);
    let moc = match result {
        Status::OUT => moc.not(),
        _ => moc,
    };
    HEALPixCoverage(moc)
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = BMOC;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
