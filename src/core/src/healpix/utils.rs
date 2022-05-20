use crate::math::{
    angle::Angle,
    lonlat::LonLatT
};
use crate::healpix::cell::HEALPixCell;
/// A simple wrapper around sore core methods
/// of cdshealpix
///
/// cdshealpix is developped by F-X. Pineau.
/// Please check its github repo: https://github.com/cds-astro/cds-healpix-rust

/// Get the vertices of an HEALPix cell
use cgmath::BaseFloat;
#[allow(dead_code)]
pub fn vertices_lonlat<S: BaseFloat>(cell: &HEALPixCell) -> [LonLatT<S>; 4] {
    let (lon, lat): (Vec<_>, Vec<_>) = cdshealpix::nested::vertices(cell.depth(), cell.idx())
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
        LonLatT::new(Angle(lon[3]), Angle(lat[3])),
    ]
}

/// Get the grid
pub fn grid_lonlat<S: BaseFloat>(cell: &HEALPixCell, n_segments_by_side: u16) -> Vec<LonLatT<S>> {
    debug_assert!(n_segments_by_side > 0);
    cdshealpix::nested::grid(cell.depth(), cell.idx(), n_segments_by_side)
        .iter()
        .map(|(lon, lat)| {
            // Risky wrapping here
            let lon = S::from(*lon).unwrap();
            let lat = S::from(*lat).unwrap();

            LonLatT::new(Angle(lon), Angle(lat))
        })
        .collect()
}

pub fn hash_with_dxdy(depth: u8, lonlat: &LonLatT<f64>) -> (u64, f64, f64) {
    cdshealpix::nested::hash_with_dxdy(depth, lonlat.lon().0, lonlat.lat().0)
}