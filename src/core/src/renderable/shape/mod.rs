use crate::math::{angle::Angle, lonlat::LonLatT};
use al_api::color::ColorRGBA;
use serde::Deserialize;

mod circle;
mod ellipsis;
mod image;
//mod polyline;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shape {
    Box {
        /// Center of the box
        c: LonLatT<f64>,
        /// Size following the RA axis
        ra_w: Angle<f64>,
        /// Size following the Dec axis
        dec_h: Angle<f64>,
        /// Rotation of the box in the RA-Dec space
        rot: Angle<f64>,
    },
    Circle {
        /// Center of the circle
        c: LonLatT<f64>,
        /// Radius of the circle
        rad: Angle<f64>,
    },
    PolyLine(Box<[LonLatT<f64>]>),
    Ellipsis {
        /// Center of the ellipsis
        c: LonLatT<f64>,
        /// Semi-major axis
        a: Angle<f64>,
        /// Semi-minor axis
        b: Angle<f64>,
        /// Rotation angle of the ellipsis. Origin aligns the ellipsis' major axis with the north pole. Positive angle points towards the east.
        rot: Angle<f64>,
    },
    // TODO
    Image,
}

use crate::SpaceMoc;
use crate::HEALPixCell;
impl Shape {
    /// This methods returns the HEALPix cells intersecting (or being fully contained)
    /// in the shape region
    pub(crate) fn to_flattened_hpx_cells(&self, order: u8) -> Vec<HEALPixCell> {
        match self {
            Self::Circle { c, rad } => {
                SpaceMoc::from_cone(&c, rad.to_radians(), order)
                    .to_flattened_hpx_cells(order)
                    .collect()
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum Style {
    None,
    Dashed,
    Dotted,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footprint {
    shapes: Vec<Shape>,
    /// Some styling meta data
    color: ColorRGBA,
    filled: bool,
    thickness: f32,
    style: Style,
}

pub type Catalog = Footprint;
