use super::super::{HALF_PI, MINUS_HALF_PI, PI, TWICE_PI, ZERO};
use crate::math::{lonlat::LonLatT, sph_geom::region::PoleContained};

pub const ALLSKY_BBOX: BoundingBox = BoundingBox {
    lon: ZERO..TWICE_PI,
    lat: MINUS_HALF_PI..HALF_PI,
    intersect_zero_meridian: true,
};

use std::ops::Range;
#[derive(Debug)]
pub struct BoundingBox {
    pub lon: Range<f64>,
    pub lat: Range<f64>,
    intersect_zero_meridian: bool,
}

impl BoundingBox {
    pub fn from_polygon(
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

        BoundingBox {
            lon,
            lat,
            intersect_zero_meridian,
        }
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
    pub fn contains_longitude(&self, mut lon: f64) -> bool {
        lon = if self.intersect_zero_meridian && lon > PI {
            lon - TWICE_PI
        } else {
            lon
        };

        self.lon.contains(&lon)
    }

    #[inline]
    pub fn contains_lonlat(&self, lonlat: &LonLatT<f64>) -> bool {
        self.contains_longitude(lonlat.lon().to_radians())
            && self.contains_latitude(lonlat.lat().to_radians())
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let (sl, ol) = match (self.intersect_zero_meridian, other.intersect_zero_meridian) {
            (true, false) => {
                // self lon are in [-PI; PI]
                // other lon are in [0; 2PI]
                if other.lon.start >= PI {
                    (
                        self.lon.clone(),
                        (other.lon.start - TWICE_PI)..(other.lon.end - TWICE_PI),
                    )
                } else {
                    (self.lon.clone(), other.lon.clone())
                }
            }
            (false, true) => {
                // self lon are in [0; 2PI]
                // other lon are in [-PI; PI]
                if self.lon.start >= PI {
                    (
                        (self.lon.start - TWICE_PI)..(self.lon.end - TWICE_PI),
                        other.lon.clone(),
                    )
                } else {
                    (self.lon.clone(), other.lon.clone())
                }
            }
            _ => (self.lon.clone(), other.lon.clone()),
        };

        (sl.start <= ol.end && ol.start <= sl.end)
            && (self.lat.start <= other.lat.end && other.lat.start <= self.lat.end)
    }

    #[inline]
    pub const fn fullsky() -> Self {
        BoundingBox {
            lon: ZERO..TWICE_PI,
            lat: MINUS_HALF_PI..HALF_PI,
            intersect_zero_meridian: true,
        }
    }
}
