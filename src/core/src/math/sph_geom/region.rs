use super::bbox::BoundingBox;
use crate::math::angle::ToAngle;

use crate::math::{lonlat::LonLatT, projection::coo_space::XYZWModel, MINUS_HALF_PI};
use cgmath::Vector3;
use healpix::sph_geom::coo3d::Vec3;
use healpix::sph_geom::coo3d::{Coo3D, UnitVect3};
use healpix::sph_geom::ContainsSouthPoleMethod;
use healpix::sph_geom::Polygon;
use mapproj::math::HALF_PI;

pub enum Region {
    AllSky,
    Polygon {
        polygon: Polygon,
        // A fast way to query if a position is contained
        // is to check first the bounding box
        bbox: BoundingBox,
        // Some informations about the poles
        poles: PoleContained,

        is_intersecting_zero_meridian: bool,
    },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PoleContained {
    None,
    South,
    North,
    Both,
}

#[derive(Debug)]
pub enum Intersection {
    // The segment is fully included into the region
    Included,
    // The segment does not intersect the region
    Empty,
    // The segment does intersect the region
    Intersect { vertices: Box<[XYZWModel<f64>]> },
}

impl Region {
    pub fn from_vertices(vertices: &[XYZWModel<f64>], control_point: &XYZWModel<f64>) -> Self {
        let (vertices, (lon, lat)): (Vec<_>, (Vec<_>, Vec<_>)) = vertices
            .iter()
            .map(|v| {
                let coo = healpix::sph_geom::coo3d::Coo3D::from_vec3(v.z, v.x, v.y);
                let (lon, lat) = coo.lonlat();

                (coo, (lon, lat))
            })
            .unzip();

        let polygon = Polygon::new_custom_vec3(
            vertices.into_boxed_slice(),
            &ContainsSouthPoleMethod::ControlPointIn(Coo3D::from_vec3(
                control_point.z,
                control_point.x,
                control_point.y,
            )),
        );

        let north_pole_coo = &Coo3D::from_sph_coo(0.0, HALF_PI);
        let south_pole_coo = &Coo3D::from_sph_coo(0.0, -HALF_PI);
        let north_pole_contained = polygon.contains(north_pole_coo);
        let south_pole_contained = polygon.contains(south_pole_coo);

        let poles = match (south_pole_contained, north_pole_contained) {
            (false, false) => PoleContained::None,
            (false, true) => PoleContained::North,
            (true, false) => PoleContained::South,
            (true, true) => PoleContained::Both,
        };
        // The arc length must be < PI, so we create an arc from [(0, -PI/2); (0, PI/2)[
        // see the cdshealpix doc:
        // https://docs.rs/cdshealpix/latest/cdshealpix/sph_geom/struct.Polygon.html#method.intersect_great_circle_arc
        let is_intersecting_zero_meridian = polygon.is_intersecting_great_circle_arc(
            &Coo3D::from_sph_coo(0.0, -HALF_PI),
            &Coo3D::from_sph_coo(0.0, HALF_PI - 1e-6),
        );
        let bbox = BoundingBox::from_polygon(&poles, lon, &lat, is_intersecting_zero_meridian);
        // Allsky case
        Region::Polygon {
            polygon,
            bbox,
            poles,
            is_intersecting_zero_meridian,
        }
    }

    pub fn intersects_parallel(&self, lat: f64) -> Intersection {
        if lat == 0.0 {
            self.intersects_great_circle(&Vector3::unit_y())
        } else {
            match self {
                // The polygon is included inside the region
                Region::AllSky => Intersection::Included,
                Region::Polygon { polygon, .. } => {
                    let vertices = polygon
                        .intersect_parallel_all(lat)
                        .iter()
                        .map(|v| XYZWModel::new(v.y(), v.z(), v.x(), 1.0))
                        .collect::<Vec<_>>();

                    if !vertices.is_empty() {
                        Intersection::Intersect {
                            vertices: vertices.into_boxed_slice(),
                        }
                    // test whether a point on the parallel is included
                    } else if self.contains(&LonLatT::new(0.0.to_angle(), lat.to_angle())) {
                        Intersection::Included
                    } else {
                        Intersection::Empty
                    }
                }
            }
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Region::Polygon {
                    polygon: p1,
                    bbox: bbox1,
                    ..
                },
                Region::Polygon {
                    polygon: p2,
                    bbox: bbox2,
                    ..
                },
            ) => {
                if !bbox1.intersects(bbox2) {
                    return false;
                }

                for v1 in p1.vertices() {
                    let ll = v1.lonlat();
                    let ll = LonLatT::new(ll.0.to_angle(), ll.1.to_angle());
                    if other.contains(&ll) {
                        return true;
                    }
                }

                for v2 in p2.vertices() {
                    let ll = v2.lonlat();
                    let ll = LonLatT::new(ll.0.to_angle(), ll.1.to_angle());
                    if self.contains(&ll) {
                        return true;
                    }
                }

                let vertices = p2.vertices();
                let mut j = vertices.len() - 1;
                for i in 0..vertices.len() {
                    let llj = vertices[j].lonlat();
                    let lli = vertices[i].lonlat();

                    let llj = LonLatT::new(llj.0.to_angle(), llj.1.to_angle());
                    let lli = LonLatT::new(lli.0.to_angle(), lli.1.to_angle());

                    let inter = self.intersects_great_circle_arc(&llj, &lli);
                    match inter {
                        Intersection::Empty => {}
                        _ => {
                            return true;
                        }
                    }

                    j = i;
                }

                false
            }
            _ => true,
        }
    }

    pub fn intersects_great_circle_arc(
        &self,
        lonlat1: &LonLatT<f64>,
        lonlat2: &LonLatT<f64>,
    ) -> Intersection {
        match self {
            // The polygon is included inside the region
            Region::AllSky => Intersection::Included,
            Region::Polygon { polygon, .. } => {
                let coo1 =
                    Coo3D::from_sph_coo(lonlat1.lon().to_radians(), lonlat1.lat().to_radians());
                let coo2 =
                    Coo3D::from_sph_coo(lonlat2.lon().to_radians(), lonlat2.lat().to_radians());

                let vertices: Vec<cgmath::Vector4<f64>> = polygon
                    .intersect_great_circle_arc_all(&coo1, &coo2)
                    .iter()
                    .map(|v| XYZWModel::new(v.y(), v.z(), v.x(), 1.0))
                    .collect::<Vec<_>>();

                if !vertices.is_empty() {
                    Intersection::Intersect {
                        vertices: vertices.into_boxed_slice(),
                    }
                // test whether a point on the meridian is included
                } else if self.contains(lonlat1) {
                    Intersection::Included
                } else {
                    Intersection::Empty
                }
            }
        }
    }

    pub fn intersects_meridian(&self, lon: f64) -> Intersection {
        let n_pole_lonlat = LonLatT::new(lon.to_angle(), (HALF_PI - 1e-4).to_angle());
        let s_pole_lonlat = LonLatT::new(lon.to_angle(), (MINUS_HALF_PI + 1e-4).to_angle());

        self.intersects_great_circle_arc(&s_pole_lonlat, &n_pole_lonlat)
    }

    fn intersects_great_circle(&self, n: &Vector3<f64>) -> Intersection {
        match self {
            // The polygon is included inside the region
            Region::AllSky => Intersection::Included,
            Region::Polygon { polygon, .. } => {
                let vertices: Vec<cgmath::Vector4<f64>> = polygon
                    .intersect_great_circle_all(&UnitVect3::new_unsafe(n.z, n.x, n.y))
                    .iter()
                    .map(|v| XYZWModel::new(v.y(), v.z(), v.x(), 1.0))
                    .collect::<Vec<_>>();

                // Test whether a point on the meridian is included
                match vertices.len() {
                    0 => Intersection::Empty,
                    1 => Intersection::Included,
                    _ => Intersection::Intersect {
                        vertices: vertices.into_boxed_slice(),
                    },
                }
            }
        }
    }

    pub fn contains(&self, lonlat: &LonLatT<f64>) -> bool {
        match self {
            Region::AllSky => true,
            Region::Polygon { polygon, bbox, .. } => {
                // Fast checking with the bbox
                if !bbox.contains_lonlat(&lonlat) {
                    return false;
                }

                let coo = Coo3D::from_sph_coo(lonlat.lon().to_radians(), lonlat.lat().to_radians());
                polygon.contains(&coo)
            }
        }
    }

    // Is intersecting API
    pub fn is_intersecting_parallel(&self, lat: f64) -> bool {
        match self {
            // The polygon is included inside the region
            Region::AllSky => true,
            Region::Polygon { polygon, .. } => polygon.is_intersecting_parallel(lat),
        }
    }

    pub fn is_intersecting_great_circle_arc(
        &self,
        lonlat1: &LonLatT<f64>,
        lonlat2: &LonLatT<f64>,
    ) -> bool {
        match self {
            Region::AllSky => true,
            Region::Polygon { polygon, .. } => {
                let coo1 =
                    Coo3D::from_sph_coo(lonlat1.lon().to_radians(), lonlat1.lat().to_radians());
                let coo2 =
                    Coo3D::from_sph_coo(lonlat2.lon().to_radians(), lonlat2.lat().to_radians());

                polygon.is_intersecting_great_circle_arc(&coo1, &coo2)
            }
        }
    }

    pub fn is_intersecting_meridian(&self, lon: f64) -> bool {
        let n_pole_lonlat = LonLatT::new(HALF_PI.to_angle(), lon.to_angle());
        let s_pole_lonlat = LonLatT::new(MINUS_HALF_PI.to_angle(), lon.to_angle());

        self.is_intersecting_great_circle_arc(&s_pole_lonlat, &n_pole_lonlat)
    }
}
