use crate::math::HALF_PI;
use crate::math::PI;
use crate::CameraViewPort;
use crate::LonLatT;
use crate::ProjectionType;
use cgmath::InnerSpace;
use cgmath::Vector3;

use crate::math::lonlat::LonLat;
use crate::math::projection::coo_space::XYScreen;
use crate::math::TWICE_PI;

use crate::math::angle::ToAngle;
use crate::math::angle::AngleFormatter;
use al_api::angle::Formatter;
use cgmath::Vector2;
use core::ops::Range;

const OFF_TANGENT: f64 = 70.0;
const OFF_BI_TANGENT: f64 = 5.0;

pub enum LabelOptions {
    Centered,
    OnSide,
}
#[derive(Debug)]
pub struct Label {
    // The position
    pub position: XYScreen<f64>,
    // the string content
    pub content: String,
    // in radians
    pub rot: f64,
}
impl Label {
    pub fn from_meridian(
        lon: f64,
        lat: &Range<f64>,
        options: LabelOptions,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        fmt: Formatter,
        grid_decimal_prec: u8
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        let d = if fov.contains_north_pole() {
            Vector3::new(0.0, 1.0, 0.0)
        } else if fov.contains_south_pole() {
            Vector3::new(0.0, -1.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let lonlat = match options {
            LabelOptions::Centered => {
                let mut lat = camera.get_center().lat().to_radians();
                if lat.abs() > 70.0_f64.to_radians() {
                    lat = lat.signum() * 70.0_f64.to_radians();
                }

                LonLatT::new(lon.to_angle(), lat.to_angle())
            }
            LabelOptions::OnSide => LonLatT::new(lon.to_angle(), lat.start.to_angle()),
        };

        let m1: Vector3<_> = lonlat.vector();
        let m2 = (m1 + d * 1e-3).normalize();

        let d1 = projection.model_to_screen_space(&m1, camera)?;
        let d2 = projection.model_to_screen_space(&m2, camera)?;

        let dt = (d2 - d1).normalize();
        let db = Vector2::new(dt.y.abs(), dt.x.abs());

        let mut lon = m1.lon().to_radians();
        if lon < 0.0 {
            lon += TWICE_PI;
        }

        let mut angle = lon.to_angle();
        let fmt = match fmt {
            Formatter::Decimal => {
                AngleFormatter::Decimal { prec: grid_decimal_prec }
            },
            Formatter::Sexagesimal => {
                // Sexagesimal formatting for longitudes is HMS
                AngleFormatter::Sexagesimal { prec: grid_decimal_prec, plus: false, hours: true }
            }
        };
        angle.set_format(fmt);
        let content = angle.to_string();

        let position = if !fov.is_allsky() {
            d1 + OFF_TANGENT * dt - OFF_BI_TANGENT * db
        } else {
            d1
        };

        // rot is between -PI and +PI
        let rot = dt.y.signum() * dt.x.acos();

        Some(Label {
            position,
            content,
            rot,
        })
    }

    pub fn from_parallel(
        lat: f64,
        lon: &Range<f64>,
        options: LabelOptions,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        fmt: Formatter,
        grid_decimal_prec: u8
    ) -> Option<Self> {
        let lonlat = match options {
            LabelOptions::Centered => {
                let lon = camera.get_center().lon();
                LonLatT::new(lon, lat.to_angle())
            }
            LabelOptions::OnSide => LonLatT::new(lon.start.to_angle(), lat.to_angle()),
        };

        let m1: Vector3<_> = lonlat.vector();

        let mut t = Vector3::new(-m1.z, 0.0, m1.x).normalize();
        let center = camera.get_center();

        let dot_t_center = center.dot(t);
        if dot_t_center.abs() < 1e-4 {
            t = -t;
        } else {
            t = dot_t_center.signum() * t;
        }

        let m2 = (m1 + t * 1e-3).normalize();

        let d1 = projection.model_to_screen_space(&m1, camera)?;
        let d2 = projection.model_to_screen_space(&m2, camera)?;

        let dt = (d2 - d1).normalize();
        let db = Vector2::new(dt.y.abs(), dt.x.abs());

        let mut angle = lat.to_angle();
        let fmt = match fmt {
            Formatter::Decimal => {
                AngleFormatter::Decimal { prec: grid_decimal_prec }
            },
            Formatter::Sexagesimal => {
                // Sexagesimal formatting for latitudes is DMS with an optional '+' character
                AngleFormatter::Sexagesimal { prec: grid_decimal_prec, plus: true, hours: false }
            }
        };
        angle.set_format(fmt);
        let content = angle.to_string();

        let fov = camera.get_field_of_view();
        let position = if !fov.is_allsky() && !fov.contains_pole() {
            d1 + OFF_TANGENT * dt - OFF_BI_TANGENT * db
        } else {
            d1
        };

        // rot is between -PI and +PI
        let mut angle = dt.y.signum() * dt.x.acos();

        // Detect if the label is upside-down fix the angle by adding PI
        if angle.abs() >= HALF_PI {
            angle += PI;
        }

        Some(Label {
            position,
            content,
            rot: angle,
        })
    }
}
