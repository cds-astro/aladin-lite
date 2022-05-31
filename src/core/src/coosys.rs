use cgmath::{BaseFloat, Vector4};

use al_api::coo_system::CooBaseFloat;
use al_api::coo_system::CooSystem;
/// This is conversion method returning a transformation
/// matrix when the system requested by the user is not
/// icrs j2000.
/// The core projections are always performed in icrs j2000
/// so one must call these methods to convert them to icrs before.
#[inline]
pub fn apply_coo_system<'a, S>(c1: &CooSystem, c2: &CooSystem, v1: &Vector4<S>) -> Vector4<S>
where
    S: BaseFloat + CooBaseFloat,
{
    let c1_2_c2_mat = c1.to::<S>(c2);
    c1_2_c2_mat * (*v1)
}

mod tests {
    #[allow(unused_macros)]
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    use crate::math::{
        angle::ArcDeg,
        lonlat::{LonLat, LonLatT},
    };
    use al_api::coo_system::CooSystem;

    #[test]
    fn j2000_to_gal() {
        let lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let gal_lonlat =
            super::apply_coo_system(&CooSystem::ICRSJ2000, &CooSystem::GAL, &lonlat.vector())
                .lonlat();

        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 96.33723581, 1e-3);
        assert_delta!(gal_lat_deg, -60.18845577, 1e-3);
    }

    #[test]
    fn gal_to_j2000() {
        let lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let j2000_lonlat =
            super::apply_coo_system(&CooSystem::GAL, &CooSystem::ICRSJ2000, &lonlat.vector())
                .lonlat();
        let j2000_lon_deg = j2000_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let j2000_lat_deg = j2000_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(j2000_lon_deg, 266.40506655, 1e-3);
        assert_delta!(j2000_lat_deg, -28.93616241, 1e-3);
    }

    #[test]
    fn j2000_gal_roundtrip() {
        let gal_lonlat: LonLatT<f64> = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());

        let icrsj2000_pos =
            super::apply_coo_system(&CooSystem::GAL, &CooSystem::ICRSJ2000, &gal_lonlat.vector());

        let gal_lonlat =
            super::apply_coo_system(&CooSystem::ICRSJ2000, &CooSystem::GAL, &icrsj2000_pos);

        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 0.0, 1e-3);
        assert_delta!(gal_lat_deg, 0.0, 1e-3);
    }
}
