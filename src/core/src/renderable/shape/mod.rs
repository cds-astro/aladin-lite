use crate::math::{angle::Angle, lonlat::LonLatT};
use al_api::color::ColorRGBA;
use serde::Deserialize;

mod circle;
mod ellipsis;
mod image;
mod polyline;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shape {
    Box {
        /// Center of the box
        c: LonLatT<f32>,
        /// Size following the RA axis
        ra_w: Angle<f32>,
        /// Size following the Dec axis
        dec_h: Angle<f32>,
        /// Rotation of the box in the RA-Dec space
        rot: Angle<f32>,
    },
    Circle {
        /// Center of the circle
        c: LonLatT<f32>,
        /// Radius of the circle
        rad: Angle<f32>,
    },
    PolyLine(Box<[LonLatT<f32>]>),
    Ellipsis {
        /// Center of the ellipsis
        c: LonLatT<f32>,
        /// Semi-major axis
        a: Angle<f32>,
        /// Semi-minor axis
        b: Angle<f32>,
        /// Rotation angle of the ellipsis. Origin aligns the ellipsis' major axis with the north pole. Positive angle points towards the east.
        rot: Angle<f32>,
    },
    // TODO
    Image,
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
