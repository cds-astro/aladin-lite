use cgmath::Rad;
use cgmath::{Vector3, Vector4};
const PI: f64 = std::f64::consts::PI;
const ZERO: f64 = 0.0;
const TWICE_PI: f64 = std::f64::consts::PI * 2.0;
const HALF_PI: f64 = std::f64::consts::PI * 0.5;
const MINUS_HALF_PI: f64 = -std::f64::consts::PI * 0.5;

use crate::math::angle::Angle;



use crate::math::lonlat::LonLat;
use cdshealpix::sph_geom::{
    coo3d::{Coo3D, Vec3},
    ContainsSouthPoleMethod, Polygon,
};
pub enum FieldOfViewType {
    Allsky,
    Polygon {
        poly: Polygon,

        bbox: BoundingBox,
        poles: PoleContained,
    },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PoleContained {
    None,
    South,
    North,
    Both,
}
use crate::math::lonlat::LonLatT;
//use cgmath::Vector2;
use crate::CameraViewPort;
impl FieldOfViewType {
    pub fn new_polygon(vertices: &[Vector4<f64>], control_point: &Vector4<f64>) -> FieldOfViewType {
        let (vertices, (lon, lat)): (Vec<_>, (Vec<_>, Vec<_>)) = vertices
            .iter()
            .map(|v| {
                let coo = cdshealpix::sph_geom::coo3d::Coo3D::from_vec3(v.z, v.x, v.y);
                let (lon, lat) = coo.lonlat();

                (coo, (lon, lat))
            })
            .unzip();

        let control_point = Coo3D::from_vec3(control_point.z, control_point.x, control_point.y);
        let poly = Polygon::new_custom_vec3(
            vertices.into_boxed_slice(),
            &ContainsSouthPoleMethod::ControlPointIn(control_point),
        );

        let north_pole_coo = &Coo3D::from_sph_coo(0.0, HALF_PI);
        let south_pole_coo = &Coo3D::from_sph_coo(0.0, -HALF_PI);
        let north_pole_contained = poly.contains(north_pole_coo);
        let south_pole_contained = poly.contains(south_pole_coo);

        let poles = match (south_pole_contained, north_pole_contained) {
            (false, false) => PoleContained::None,
            (false, true) => PoleContained::North,
            (true, false) => PoleContained::South,
            (true, true) => PoleContained::Both,
        };
        // The arc length must be < PI, so we create an arc from [(0, -PI/2); (0, PI/2)[
        // see the cdshealpix doc:
        // https://docs.rs/cdshealpix/latest/cdshealpix/sph_geom/struct.Polygon.html#method.intersect_great_circle_arc
        let poly_intersects_meridian = poly.is_intersecting_great_circle_arc(
            &Coo3D::from_sph_coo(0.0, -HALF_PI),
            &Coo3D::from_sph_coo(0.0, HALF_PI - 1e-6),
        );
        let bbox = BoundingBox::from_polygon(&poles, lon, &lat, poly_intersects_meridian);

        FieldOfViewType::Polygon { poly, poles, bbox }
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        match self {
            FieldOfViewType::Allsky => &ALLSKY_BBOX,
            FieldOfViewType::Polygon { bbox, .. } => bbox,
        }
    }

    pub fn intersect_meridian<LonT: Into<Rad<f64>>>(
        &self,
        lon: LonT,
        camera: &CameraViewPort,
    ) -> Option<Vector3<f64>> {
        let Rad::<f64>(lon) = lon.into();

        match self {
            FieldOfViewType::Allsky => {
                // Allsky case
                // We do an approx saying allsky fovs intersect all meridian
                // but this is not true for example for the orthographic projection
                // Some meridians may not be visible
                let center = camera.get_center();
                let pos: Vector3<f64> = LonLatT::new(Angle(lon), center.lat()).vector();
                Some(pos)
            }
            FieldOfViewType::Polygon { poly, bbox: _, poles: _ } => {
                let lon = lon.into();
                let lon = if lon < 0.0 { lon + TWICE_PI } else { lon };

                // The arc length must be < PI, so we create an arc from [(lon, -PI/2); (lon, PI/2)[
                // see the cdshealpix doc:
                // https://docs.rs/cdshealpix/latest/cdshealpix/sph_geom/struct.Polygon.html#method.intersect_great_circle_arc
                let a = Coo3D::from_sph_coo(lon, -HALF_PI);
                let b = Coo3D::from_sph_coo(lon, HALF_PI - 1e-6);

                // For those intersecting, perform the intersection
                poly.intersect_great_circle_arc(&a, &b)
                    .and_then(|v| Some(Vector3::new(v.y(), v.z(), v.x())))
                    .or_else(|| {
                        // If no intersection has been found, e.g. because the
                        // great circle is fully contained in the bounding box
                        let center = camera.get_center();
                        let pos: Vector3<f64> = LonLatT::new(Angle(lon), center.lat()).vector();
                        Some(pos)
                    })
            }
        }
    }

    pub fn intersect_parallel<LatT: Into<Rad<f64>>>(
        &self,
        lat: LatT,
        camera: &CameraViewPort,
    ) -> Option<Vector3<f64>> {
        let Rad::<f64>(lat) = lat.into();

        match self {
            FieldOfViewType::Allsky => {
                let center = camera.get_center();
                let pos: Vector3<f64> = LonLatT::new(center.lon(), Angle(lat)).vector();
                Some(pos)
            }
            FieldOfViewType::Polygon { poly, bbox, poles: _ } => {
                // Prune parallels that do not intersect the fov
                if bbox.contains_latitude(lat) {
                    // For those intersecting, perform the intersection
                    poly.intersect_parallel(lat)
                        .and_then(|v| Some(Vector3::new(v.y(), v.z(), v.x())))
                        .or_else(|| {
                            // If no intersection has been found, e.g. because the
                            // great circle is fully contained in the bounding box
                            let center = camera.get_center();
                            let pos: Vector3<f64> = LonLatT::new(center.lon(), Angle(lat)).vector();
                            Some(pos)
                        })
                } else {
                    None
                }
            }
        }
    }

    pub fn is_allsky(&self) -> bool {
        matches!(self, FieldOfViewType::Allsky)
    }

    pub fn contains_pole(&self) -> bool {
        match self {
            FieldOfViewType::Allsky => true,
            FieldOfViewType::Polygon { poles, .. } => *poles != PoleContained::None,
        }
    }

    pub fn contains_north_pole(&self) -> bool {
        match self {
            FieldOfViewType::Allsky => {
                //let center = camera.get_center();
                //center.y >= 0.0
                true
            }
            FieldOfViewType::Polygon { poles, .. } => {
                *poles == PoleContained::North || *poles == PoleContained::Both
            }
        }
    }

    pub fn contains_south_pole(&self) -> bool {
        match self {
            FieldOfViewType::Allsky => {
                //let center = camera.get_center();
                //center.y < 0.0
                true
            }
            FieldOfViewType::Polygon { poles, .. } => {
                *poles == PoleContained::South || *poles == PoleContained::Both
            }
        }
    }

    pub fn contains_both_poles(&self) -> bool {
        match self {
            FieldOfViewType::Allsky => {
                true
            }
            FieldOfViewType::Polygon { poles, .. } => {
                *poles == PoleContained::Both
            }
        }
    }
}

const ALLSKY_BBOX: BoundingBox = BoundingBox {
    lon: ZERO..TWICE_PI,
    lat: MINUS_HALF_PI..HALF_PI,
};

use std::ops::{Range};
#[derive(Debug)]
pub struct BoundingBox {
    pub lon: Range<f64>,
    pub lat: Range<f64>,
}

impl BoundingBox {
    fn from_polygon(
        pole_contained: &PoleContained,
        mut lon: Vec<f64>,
        lat: &[f64],
        intersect_zero_meridian: bool,
    ) -> Self {
        // The longitudes must be readjust if the
        // polygon crosses the 0deg meridian
        // We make the assumption the polygon is not too big
        // (i.e. < PI length on the longitude so that it does not
        // crosses both the 0 and 180deg meridians)
        if intersect_zero_meridian {
            lon = lon
                .iter()
                .map(|&lon| if lon > PI { lon - TWICE_PI } else { lon })
                .collect();
        }

        let (lon, lat) = match pole_contained {
            PoleContained::None => {
                // The polygon does not contain any pole
                // Meridian 0deg is not crossing the polygon
                let (min_lat, max_lat) = lat
                    .iter()
                    .fold((std::f64::MAX, std::f64::MIN), |(min, max), &b| {
                        (min.min(b), max.max(b))
                    });

                let (min_lon, max_lon) = lon
                    .iter()
                    .fold((std::f64::MAX, std::f64::MIN), |(min, max), &b| {
                        (min.min(b), max.max(b))
                    });

                (min_lon..max_lon, min_lat..max_lat)
            }
            PoleContained::South => {
                let max_lat = lat.iter().fold(std::f64::MIN, |a, b| a.max(*b));
                (
                    if intersect_zero_meridian {
                        -PI..PI
                    } else {
                        ZERO..TWICE_PI
                    },
                    -HALF_PI..max_lat,
                )
            }
            PoleContained::North => {
                let min_lat = lat.iter().fold(std::f64::MAX, |a, b| a.min(*b));
                (
                    if intersect_zero_meridian {
                        -PI..PI
                    } else {
                        ZERO..TWICE_PI
                    },
                    min_lat..HALF_PI,
                )
            }
            PoleContained::Both => (
                if intersect_zero_meridian {
                    -PI..PI
                } else {
                    ZERO..TWICE_PI
                },
                -HALF_PI..HALF_PI,
            ),
        };

        BoundingBox { lon, lat }
    }

    #[inline]
    pub fn get_lon_size(&self) -> f64 {
        self.lon.end - self.lon.start
    }

    #[inline]
    pub fn get_lat_size(&self) -> f64 {
        self.lat.end - self.lat.start
    }

    #[inline]
    pub fn all_lon(&self) -> bool {
        (self.lon.end - self.lon.start) == TWICE_PI
    }

    #[inline]
    pub fn lon_min(&self) -> f64 {
        self.lon.start
    }

    #[inline]
    pub fn lon_max(&self) -> f64 {
        self.lon.end
    }

    #[inline]
    pub fn lat_min(&self) -> f64 {
        self.lat.start
    }

    #[inline]
    pub fn lat_max(&self) -> f64 {
        self.lat.end
    }

    #[inline]
    pub fn get_lon(&self) -> Range<f64> {
        self.lon.start..self.lon.end
    }

    #[inline]
    pub fn get_lat(&self) -> Range<f64> {
        self.lat.start..self.lat.end
    }

    #[inline]
    pub fn contains_latitude(&self, lat: f64) -> bool {
        self.lat.contains(&lat)
    }

    #[inline]
    pub fn contains_meridian(&self, lon: f64) -> bool {
        self.lon.contains(&lon)
    }

    #[inline]
    pub const fn fullsky() -> Self {
        BoundingBox {
            lon: ZERO..TWICE_PI,
            lat: MINUS_HALF_PI..HALF_PI,
        }
    }
}
