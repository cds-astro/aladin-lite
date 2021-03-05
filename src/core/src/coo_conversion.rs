use cgmath::Matrix4;
pub trait CooBaseFloat: Sized {
    const GALACTIC_TO_J2000: Matrix4<Self>;
    const J2000_TO_GALACTIC: Matrix4<Self>;
}

impl CooBaseFloat for f32 {
    const GALACTIC_TO_J2000: Matrix4<f32> = Matrix4::new(
        -0.4448296299195045,
        0.7469822444763707,
        0.4941094279435681,
        0.0,
    
        -0.1980763734646737,
        0.4559837762325372,
        -0.8676661489811610,
        0.0,
    
        -0.873437090247923,
        -0.4838350155267381,
        -0.0548755604024359,
        0.0,
    
        0.0,
        0.0,
        0.0,
        1.0
    );

    const J2000_TO_GALACTIC: Matrix4<f32> = Matrix4::new(
        -0.4448296299195045,
        -0.1980763734646737,
        -0.873437090247923,
        0.0,
    
        0.7469822444763707,
        0.4559837762325372,
        -0.4838350155267381,
        0.0,
    
        0.4941094279435681,
        -0.8676661489811610,
        -0.0548755604024359,
        0.0,
    
        0.0,
        0.0,
        0.0,
        1.0
    );
}
impl CooBaseFloat for f64 {
    const GALACTIC_TO_J2000: Matrix4<f64> = Matrix4::new(
        -0.4448296299195045,
        0.7469822444763707,
        0.4941094279435681,
        0.0,
    
        -0.1980763734646737,
        0.4559837762325372,
        -0.8676661489811610,
        0.0,
    
        -0.873437090247923,
        -0.4838350155267381,
        -0.0548755604024359,
        0.0,
    
        0.0,
        0.0,
        0.0,
        1.0
    );

    const J2000_TO_GALACTIC: Matrix4<f64> = Matrix4::new(
        -0.4448296299195045,
        -0.1980763734646737,
        -0.873437090247923,
        0.0,
    
        0.7469822444763707,
        0.4559837762325372,
        -0.4838350155267381,
        0.0,
    
        0.4941094279435681,
        -0.8676661489811610,
        -0.0548755604024359,
        0.0,
    
        0.0,
        0.0,
        0.0,
        1.0
    );
}

use crate::LonLatT;
use crate::Vector4;
use crate::math::LonLat;
use cgmath::BaseFloat;

// Some utility functions converting the spherical coordinates
// from icrs j2000 to galactic
pub fn to_galactic<S>(lonlat: LonLatT<S>) -> LonLatT<S>
where
    S: BaseFloat + CooBaseFloat
{
    let j2000_coo: Vector4<S> = lonlat.vector();
    let gal_coo = S::J2000_TO_GALACTIC * j2000_coo;
    gal_coo.lonlat()
}

// or from galactic to icrs j2000
pub fn to_icrs_j2000<S>(lonlat: LonLatT<S>) -> LonLatT<S>
where
    S: BaseFloat + CooBaseFloat
{
    let gal_coo: Vector4<S> = lonlat.vector();
    let j2000_coo = S::GALACTIC_TO_J2000 * gal_coo;
    j2000_coo.lonlat()
}
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub enum CooSystem {
    ICRSJ2000,
    GAL,
}

#[allow(dead_code)]
#[wasm_bindgen(js_name = "GALCooSys")]
pub fn galcoo_sys() -> Result<CooSystem, JsValue> {
    Ok(CooSystem::GAL)
}

#[allow(dead_code)]
#[wasm_bindgen(js_name = "ICRSJ2000CooSys")]
pub fn icrsj2000_coo_sys() -> Result<CooSystem, JsValue> {
    Ok(CooSystem::ICRSJ2000)
}

use cgmath::SquareMatrix;
impl CooSystem {
    /// This is conversion method returning a transformation
    /// matrix when the system requested by the user is not
    /// icrs j2000.
    /// The core projections are always performed in icrs j2000
    /// so one must call these methods to convert them to icrs before.
    #[inline]
    pub fn to_icrs_j2000<'a, S>(&self) -> Matrix4<S>
    where
        S: BaseFloat + CooBaseFloat
    {
        if *self == CooSystem::GAL {
            S::GALACTIC_TO_J2000
        } else {
            Matrix4::<S>::identity()
        }
    }

    /// This method is called by the grid to convert it
    /// from icrs to galactic coo system
    #[inline]
    pub fn to_gal<'a, S>(&self) -> Matrix4<S>
    where
        S: BaseFloat + CooBaseFloat
    {
        if *self == CooSystem::GAL {
            S::J2000_TO_GALACTIC
        } else {
            Matrix4::<S>::identity()
        }
    }
}



mod tests {
    #[allow(unused_macros)]
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) { panic!(); }
        }
    }

    #[test]
    fn j2000_to_gal() {
        use crate::{LonLatT, ArcDeg};
        let lonlat = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let gal_lonlat = super::to_galactic(lonlat);
        
        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 96.33723581, 1e-3);
        assert_delta!(gal_lat_deg, -60.18845577, 1e-3);
    }

    #[test]
    fn gal_to_j2000() {
        use crate::{LonLatT, ArcDeg};

        let lonlat = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());
        let j2000_lonlat = super::to_icrs_j2000(lonlat);
        let j2000_lon_deg = j2000_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let j2000_lat_deg = j2000_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(j2000_lon_deg, 266.40506655, 1e-3);
        assert_delta!(j2000_lat_deg, -28.93616241, 1e-3);
    }

    #[test]
    fn j2000_gal_roundtrip() {
        use crate::{LonLatT, ArcDeg};
        let gal_lonlat = LonLatT::new(ArcDeg(0.0).into(), ArcDeg(0.0).into());

        let gal_lonlat = super::to_galactic(super::to_icrs_j2000(gal_lonlat));

        let gal_lon_deg = gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI);
        let gal_lat_deg = gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI);

        assert_delta!(gal_lon_deg, 0.0, 1e-3);
        assert_delta!(gal_lat_deg, 0.0, 1e-3);
    }
}