/// A simple wrapper around sore core methods
/// of cdshealpix
/// 
/// cdshealpix is developped by F-X. Pineau.
/// Please check its github repo: https://github.com/cds-astro/cds-healpix-rust

/// Get the vertices of an HEALPix cell
use cgmath::BaseFloat;
use crate::math::LonLatT;
use crate::healpix_cell::HEALPixCell;
use crate::renderable::Angle;
pub fn vertices_lonlat<S: BaseFloat>(cell: &HEALPixCell) -> [LonLatT<S>; 4] {
    let (lon, lat): (Vec<_>, Vec<_>) = healpix::nested::vertices(cell.depth(), cell.idx())
        .iter()
        .map(|(lon, lat)| {
            // Risky wrapping here
            let lon = S::from(*lon).unwrap();
            let lat = S::from(*lat).unwrap();

            (lon, lat)
        })
        .unzip();

    [
        LonLatT::new(Angle(lon[0]), Angle(lat[0])),
        LonLatT::new(Angle(lon[1]), Angle(lat[1])),
        LonLatT::new(Angle(lon[2]), Angle(lat[2])),
        LonLatT::new(Angle(lon[3]), Angle(lat[3]))
    ]
}

/// Get the grid 
pub fn grid_lonlat<S: BaseFloat>(cell: &HEALPixCell, n_segments_by_side: u16) -> Vec<LonLatT<S>> {
    assert!(n_segments_by_side > 0);
    healpix::nested::grid(cell.depth(), cell.idx(), n_segments_by_side)
        .iter()
        .map(|(lon, lat)| {
            // Risky wrapping here
            let lon = S::from(*lon).unwrap();
            let lat = S::from(*lat).unwrap();

            LonLatT::new(Angle(lon), Angle(lat))
        })
        .collect()
}

use healpix::nested::bmoc::{BMOC, Status};
use crate::math;
use cgmath::{Vector4, Vector3};
use crate::math::LonLat;
pub struct HEALPixCoverage(BMOC);


pub fn from_polygon(
    // The depth of the smallest HEALPix cells contained in it
    depth: u8,
    // The vertices of the polygon delimiting the coverage
    vertices: &[Vector4<f32>],
    // A vertex being inside the coverage,
    // typically the center of projection
    inside: &Vector3<f32>
) -> HEALPixCoverage {
    let lonlat = vertices.iter()
        .map(|vertex| {
            let (lon, lat) = math::xyzw_to_radec(vertex);
            (lon.0 as f64, lat.0 as f64)
        })
        .collect::<Vec<_>>();
    let moc = healpix::nested::polygon_coverage(depth, &lonlat[..], false);
    let inside_lonlat = inside.lonlat();
    let result = moc.test_coo(inside_lonlat.lon().0 as f64, inside_lonlat.lat().0 as f64);
    let moc = match result {
        Status::OUT => {
            moc.not()
        },
        _ => moc
    };
    HEALPixCoverage(moc)
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = BMOC;

    fn deref (&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

    
