use crate::math;
use crate::math::lonlat::LonLatT;
use crate::math::PI;

use cgmath::{Vector3, Vector4};
use moclib::{moc::range::RangeMOC, qty::Hpx, ranges::SNORanges};
pub type Smoc = RangeMOC<u64, Hpx<u64>>;

use crate::healpix::cell::HEALPixCell;
#[derive(Clone, Debug)]
pub struct HEALPixCoverage(pub Smoc);


impl HEALPixCoverage {
    pub fn from_3d_coos<'a>(
        // The depth of the smallest HEALPix cells contained in it
        depth: u8,
        // The vertices of the polygon delimiting the coverage
        vertices_iter: impl Iterator<Item = Vector4<f64>>,
        // A vertex being inside the coverage,
        // typically the center of projection
        inside: &Vector3<f64>,
    ) -> Self {
        let lonlat = vertices_iter
            .map(|vertex| {
                let (lon, lat) = math::lonlat::xyzw_to_radec(&vertex);
                (lon.0, lat.0)
            })
            .collect::<Vec<_>>();
        let (inside_lon, inside_lat) = math::lonlat::xyz_to_radec(inside);

        let moc = RangeMOC::from_polygon_with_control_point(
            &lonlat[..],
            (inside_lon.0, inside_lat.0),
            depth,
        );
        HEALPixCoverage(moc)
    }

    pub fn from_fixed_hpx_cells(
        depth: u8,
        hpx_idx: impl Iterator<Item = u64>,
        cap: Option<usize>,
    ) -> Self {
        let moc = RangeMOC::from_fixed_depth_cells(depth, hpx_idx, cap);
        HEALPixCoverage(moc)
    }

    pub fn from_hpx_cells<'a>(
        depth: u8,
        hpx_cell_it: impl Iterator<Item = &'a HEALPixCell>,
        cap: Option<usize>,
    ) -> Self {
        let cells_it = hpx_cell_it.map(|HEALPixCell(depth, idx)| (*depth, *idx));

        let moc = RangeMOC::from_cells(depth, cells_it, cap);
        HEALPixCoverage(moc)
    }

    pub fn from_cone(lonlat: &LonLatT<f64>, rad: f64, depth: u8) -> Self {
        if rad >= PI {
            Self::allsky(depth)
        } else {
            HEALPixCoverage(RangeMOC::from_cone(
                lonlat.lon().to_radians(),
                lonlat.lat().to_radians(),
                rad,
                depth,
                0,
            ))
        }
    }

    pub fn allsky(depth_max: u8) -> Self {
        let moc = RangeMOC::new_full_domain(depth_max);
        HEALPixCoverage(moc)
    }

    pub fn contains_coo(&self, coo: &Vector4<f64>) -> bool {
        let (lon, lat) = math::lonlat::xyzw_to_radec(coo);
        self.0.is_in(lon.0, lat.0)
    }

    // O(log2(N))
    pub fn intersects_cell(&self, cell: &HEALPixCell) -> bool {
        let z29_rng = cell.z_29_rng();

        self.0.moc_ranges().intersects_range(&z29_rng)
    }

    pub fn is_intersecting(&self, other: &Self) -> bool {
        !self.0.intersection(&other.0).is_empty()
    }

    pub fn depth(&self) -> u8 {
        self.0.depth_max()
    }

    pub fn sky_fraction(&self) -> f64 {
        self.0.coverage_percentage()
    }

    pub fn empty(depth: u8) -> Self {
        HEALPixCoverage(RangeMOC::new_empty(depth))
    }
}

use core::ops::Deref;
impl Deref for HEALPixCoverage {
    type Target = Smoc;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
