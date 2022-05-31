#[derive(Debug, Clone, PartialEq)]
#[repr(C, packed)]
pub struct Source {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Source {
    pub const fn num_f32() -> usize {
        std::mem::size_of::<Self>() / std::mem::size_of::<f32>()
    }
}

impl Eq for Source {}

use cgmath::Vector3;

use crate::math::{self, angle::Angle, lonlat::LonLat};

impl Source {
    pub fn new(lon: Angle<f32>, lat: Angle<f32> /*, mag: f32*/) -> Source {
        let world_pos = math::lonlat::radec_to_xyz(lon, lat);

        let x = world_pos.x;
        let y = world_pos.y;
        let z = world_pos.z;

        Source {
            x,
            y,
            z,
            //lon,
            //lat,
            //mag
        }
    }

    pub fn lonlat(&self) -> (f32, f32) {
        let lonlat = Vector3::new(self.x, self.y, self.z).lonlat();
        (lonlat.0 .0, lonlat.1 .0)
    }
}

use crate::math::angle::ArcDeg;
impl From<&[f32]> for Source {
    fn from(data: &[f32]) -> Source {
        let lon = ArcDeg(data[0]).into();
        let lat = ArcDeg(data[1]).into();
        //let mag = data[3];

        Source::new(lon, lat /*, mag*/)
    }
}
