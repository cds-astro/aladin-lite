use crate::healpix::cell::HEALPixCell;
use crate::math::{angle::Angle, lonlat::LonLatT};
/// A simple wrapper around sore core methods
/// of cdshealpix
///
/// cdshealpix is developped by F-X. Pineau.
/// Please check its github repo: https://github.com/cds-astro/cds-healpix-rust

/// Get the vertices of an HEALPix cell
use cgmath::BaseFloat;
#[allow(dead_code)]
pub fn vertices_lonlat<S: BaseFloat>(cell: &HEALPixCell) -> [LonLatT<S>; 4] {
    let (lon, lat): (Vec<_>, Vec<_>) = healpix::nested::vertices(cell.depth(), cell.idx())
        .iter()
        .map(|(lon, lat)| {
            // Risky wrapping here
            let lon = S::from(*lon).unwrap_abort();
            let lat = S::from(*lat).unwrap_abort();

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
use crate::Abort;
/// Get the grid
#[allow(dead_code)]
pub fn grid_lonlat<S: BaseFloat>(cell: &HEALPixCell, n_segments_by_side: u16) -> Vec<LonLatT<S>> {
    debug_assert!(n_segments_by_side > 0);
    healpix::nested::grid(cell.depth(), cell.idx(), n_segments_by_side)
        .iter()
        .map(|(lon, lat)| {
            // Risky wrapping here
            let lon = S::from(*lon).unwrap_abort();
            let lat = S::from(*lat).unwrap_abort();

            LonLatT::new(Angle(lon), Angle(lat))
        })
        .collect()
}

pub fn hash_with_dxdy(depth: u8, lonlat: &LonLatT<f64>) -> (u64, f64, f64) {
    healpix::nested::hash_with_dxdy(depth, lonlat.lon().0, lonlat.lat().0)
}

pub const MEAN_HPX_CELL_RES: &[f64; 30] = &[
    1.0233267079464885,
    0.5116633539732443,
    0.2558316769866221,
    0.12791583849331106,
    0.06395791924665553,
    0.031978959623327766,
    0.015989479811663883,
    0.007994739905831941,
    0.003997369952915971,
    0.0019986849764579854,
    0.0009993424882289927,
    0.0004996712441144963,
    0.00024983562205724817,
    0.00012491781102862408,
    0.00006245890551431204,
    0.00003122945275715602,
    0.00001561472637857801,
    0.00000780736318928901,
    0.00000390368159464450,
    0.00000195184079732225,
    0.00000097592039866113,
    0.00000048796019933056,
    0.00000024398009966528,
    0.00000012199004983264,
    0.00000006099502491632,
    0.00000003049751245816,
    0.00000001524875622908,
    0.00000000762437811454,
    0.00000000381218905727,
    0.00000000190609452864,
];
