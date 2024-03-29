use crate::healpix::cell::MAX_HPX_DEPTH;
use crate::{healpix::cell::HEALPixCell, math::lonlat::LonLat};
use al_api::Abort;
use cgmath::BaseFloat;

use std::cmp::{Ord, Ordering};

#[derive(PartialEq, Eq)]
pub enum HEALPixBBox {
    AllSky,
    Cell(HEALPixCell),
}

impl PartialOrd for HEALPixBBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (HEALPixBBox::AllSky, HEALPixBBox::AllSky) => Some(Ordering::Equal),
            (HEALPixBBox::AllSky, HEALPixBBox::Cell(_)) => Some(Ordering::Greater),
            (HEALPixBBox::Cell(_), HEALPixBBox::AllSky) => Some(Ordering::Less),
            (HEALPixBBox::Cell(c1), HEALPixBBox::Cell(c2)) => c1.partial_cmp(c2),
        }
    }
}

impl Ord for HEALPixBBox {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

pub struct GreatCircleArc {
    /// Smallest HEALPix cell containing the arc
    hpx_bbox: HEALPixBBox,
}

impl GreatCircleArc {
    pub fn new<S, T, U>(v1: T, v2: U) -> Self
    where
        S: BaseFloat,
        T: LonLat<S>,
        U: LonLat<S>,
    {
        // Compute the HPX bbox
        let lonlat1 = v1.lonlat();
        let lonlat2 = v2.lonlat();

        let c1 = HEALPixCell::new(
            MAX_HPX_DEPTH,
            lonlat1.lon().to_radians().to_f64().unwrap_abort(),
            lonlat1.lat().to_radians().to_f64().unwrap_abort(),
        );
        let c2 = HEALPixCell::new(
            MAX_HPX_DEPTH,
            lonlat2.lon().to_radians().to_f64().unwrap_abort(),
            lonlat2.lat().to_radians().to_f64().unwrap_abort(),
        );

        let hpx_bbox = if let Some(common_ancestor) = c1.smallest_common_ancestor(&c2) {
            HEALPixBBox::Cell(common_ancestor)
        } else {
            HEALPixBBox::AllSky
        };

        Self { hpx_bbox }
    }

    pub fn get_containing_hpx_cell(&self) -> &HEALPixBBox {
        &self.hpx_bbox
    }
}
