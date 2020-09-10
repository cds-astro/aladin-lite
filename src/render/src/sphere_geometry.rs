use cgmath::Rad;
use cgmath::{Vector4, Vector3};

const PI: f32 = std::f32::consts::PI;
const TWICE_PI: f32 = PI * 2_f32;
const HALF_PI: f32 = PI * 0.5_f32;

use crate::renderable::angle::{Angle, ArcDeg, ArcMin, ArcSec, SerializeToString};

use cgmath::InnerSpace;

pub enum GreatCirclesInFieldOfView {
    AllSkyGrid(AllSkyGrid),
    PolygonGrid(PolygonGrid)
}

//use cgmath::Vector2;
use std::collections::HashMap;

//#[repr(C)]
//pub struct ClipSpacePosition<S: BaseFloat>(Vector2<S>);

impl GreatCirclesInFieldOfView {
    pub fn new_allsky() -> GreatCirclesInFieldOfView {
        let allsky = GreatCirclesInFieldOfView::AllSkyGrid(AllSkyGrid::new());

        allsky
    }
    
    pub fn new_polygon(vertices: Vec<Vector4<f32>>, aspect: f32) -> GreatCirclesInFieldOfView {
        GreatCirclesInFieldOfView::PolygonGrid(PolygonGrid::new(vertices, aspect))
    }

    pub fn get_labels<F: FormatType>(&self) -> HashMap<String, Vector3<f32>> {
        let mut great_circles_labels = HashMap::new();

        if let GreatCirclesInFieldOfView::PolygonGrid(polygon) = self {
            let meridians_labels = polygon.get_meridians_intersecting_fov::<F>();
            great_circles_labels.extend(meridians_labels.into_iter());
        }

        great_circles_labels
    }

    /*pub fn intersect_meridian<LonT: Into<Rad<f32>>>(&self, lon: LonT) -> bool {
        match self {
            FieldOfViewType::AllSkyGrid(_grid) => {
                // Allsky case
                // We do an approx saying allsky fovs intersect all meridian
                // but this is not true for example for the orthographic projection
                // Some meridians may not be visible
                true
            },
            FieldOfViewType::PolygonGrid(polygon) => {
                polygon.intersect_meridian(lon)
            }
        }
    }

    pub fn intersect_parallel<LatT: Into<Rad<f32>>>(&self, lat: LatT) -> bool {
        match self {
            FieldOfViewType::AllSkyGrid(_grid) => {
                true
            },
            FieldOfViewType::PolygonGrid(polygon) => {
                polygon.intersect_parallel(lat)
            }
        }
    }*/
}

impl HasUniforms for GreatCirclesInFieldOfView {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {        
        match self {
            GreatCirclesInFieldOfView::AllSkyGrid(ref allsky) => {
                shader.attach_uniforms_from(allsky);
            },
            GreatCirclesInFieldOfView::PolygonGrid(polygon) => {
                shader.attach_uniforms_from(polygon);
            }
        }

        shader
    }
}

pub trait ZoneFieldOfView {
    fn meridians(&self) -> &[Angle<f32>];
    fn parallels(&self) -> &[Angle<f32>];
}

use crate::shader::HasUniforms;
use crate::shader::ShaderBound;

use crate::renderable::angle::transmute_angles;
impl<T> HasUniforms for T where T: ZoneFieldOfView {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {        
        // Meridians
        let meridians = unsafe { transmute_angles(self.meridians()) };
        let name = "meridians[0]";
        shader.attach_uniform(name, &meridians);
        shader.attach_uniform("num_meridians", &(meridians.len() as i32));

        // Parallels
        let parallels = unsafe { transmute_angles(self.parallels()) };
        let name = "parallels[0]";
        shader.attach_uniform(name, &parallels);
        shader.attach_uniform("num_parallels", &(parallels.len() as i32));

        shader
    }
}

pub struct AllSkyGrid {
    meridians: Vec<Angle<f32>>,
    parallels: Vec<Angle<f32>>,
}

impl AllSkyGrid {
    fn new() -> AllSkyGrid {
        let meridians = vec![
            ArcDeg(0_f32).into(),
            ArcDeg(45_f32).into(),
            ArcDeg(90_f32).into(),
            ArcDeg(135_f32).into(),
            ArcDeg(180_f32).into(),
            ArcDeg(225_f32).into(),
            ArcDeg(270_f32).into(),
            ArcDeg(315_f32).into(),
        ];
        let parallels = vec![
            ArcDeg(-60_f32).into(),
            ArcDeg(-30_f32).into(),
            ArcDeg(0_f32).into(),
            ArcDeg(30_f32).into(),
            ArcDeg(60_f32).into(),
        ];

        AllSkyGrid {
            meridians,
            parallels
        }
    }
}

impl ZoneFieldOfView for AllSkyGrid {
    fn meridians(&self) -> &[Angle<f32>] {
        &self.meridians
    }
    fn parallels(&self) -> &[Angle<f32>] {
        &self.parallels
    }
}

#[derive(PartialEq, Eq)]
enum PoleContained {
    North,
    South,
    None,
}

impl PoleContained {
    // This checks whether the polygon contains a pole
    // The code is inspired by the formula given here:
    // https://www.edwilliams.org/avform.htm#Crs
    fn new(lon: &[Angle<f32>], lat: &[Angle<f32>]) -> PoleContained {
        // For each edge of the polygon, we compute the heading angle (i.e. course)
        // from the starting vertex of the edge to the ending one.
        let mut sum_delta_lon = Angle::new(Rad(0_f32));

        let mut num_vertices_in_south = 0 as usize;

        let num_lon = lon.len();
        let mut last = num_lon - 1;

        for cur in 0..num_lon {
            let delta_lon = lon[cur] - lon[last];
            let abs_delta_lon = delta_lon.abs();

            if abs_delta_lon <= PI {
                sum_delta_lon += delta_lon;
            } else if delta_lon > 0_f32 {
                sum_delta_lon -= -abs_delta_lon + TWICE_PI;
            } else {
                sum_delta_lon += -abs_delta_lon + TWICE_PI;
            }

            if lat[cur] < 0_f32 {
                num_vertices_in_south += 1;
            }

            last = cur;
        }

        if sum_delta_lon.abs() > PI {
            let num_vertices = lon.len();
            // More than the half of the vertices are located
            // in the south hemisphere
            if (num_vertices_in_south << 1) >= num_vertices {
                PoleContained::South
            } else {
                PoleContained::North
            }
        } else {
            PoleContained::None
        }
    }
}

use std::ops::Range;
struct BoundingBox {
    lon: Range<Angle<f32>>,
    lat: Range<Angle<f32>>,
}

impl BoundingBox {
    #[inline]
    fn get_lon_size(&self) -> Angle<f32> {
        self.lon.end - self.lon.start
    }
    #[inline]
    fn get_lat_size(&self) -> Angle<f32> {
        self.lat.end - self.lat.start
    }
    #[inline]
    fn lon_min(&self) -> Angle<f32> {
        self.lon.start
    }
    #[inline]
    fn lon_max(&self) -> Angle<f32> {
        self.lon.end
    }
    #[inline]
    fn lat_min(&self) -> Angle<f32> {
        self.lat.start
    }
    #[inline]
    fn lat_max(&self) -> Angle<f32> {
        self.lat.end
    }
}

use crate::math::LonLatT;
struct EdgeIterator<'a, S: BaseFloat> {
    vertices: &'a [Vector4<S>],
    prev: usize,
    curr: usize,
    finished: bool,
}

impl<'a, S> EdgeIterator<'a, S>
where S: BaseFloat {
    fn new(vertices: &'a [Vector4<S>]) -> EdgeIterator<'a, S> {
        let prev = vertices.len() - 1;
        let curr = 0;

        let finished = false;

        EdgeIterator {
            vertices,
            curr,
            prev,
            finished,
        }
    }
}

use cgmath::BaseFloat;
struct Edge<S: BaseFloat> {
    pub v1: LonLatT<S>,
    pub v2: LonLatT<S>
}

use crate::math::LonLat;
use crate::math;
impl<S> Edge<S>
where S: BaseFloat {
    // Swap the vertices of the edge
    #[inline]
    fn swap(&mut self) {
        let tmp = self.v1;
        self.v1 = self.v2;
        self.v2 = tmp;
    }

    #[inline]
    fn is_in_lon_range(&self, p: &Vector3<S>) -> bool {
        let a = self.v1.vector::<Vector3<S>>();
        let b = self.v2.vector::<Vector3<S>>();

        let pa = math::ang_between_vect(&a, p);
        let pb = math::ang_between_vect(&b, p);
        let ab = math::ang_between_vect(&a, &b);

        (pa + pb - ab).0 <= S::from(1e-3).unwrap()
    }
}

impl<'a, S> Iterator for EdgeIterator<'a, S>
where S: BaseFloat {
    type Item = Edge<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finished {
            let prev = self.prev;
            let curr = self.curr;
            let v1 = self.vertices[prev].lonlat();
            let v2 = self.vertices[curr].lonlat();

            let edge = Edge {
                v1,
                v2
            };

            if self.curr == self.vertices.len() - 1 {
                self.finished = true;
            } else {
                // There are still edges, we increment self.curr
                self.prev = curr;
                self.curr += 1;
            }

            Some(edge)
        } else {
            None
        }
    }
}

struct EdgesSortedLon<S: BaseFloat>(Vec<Edge<S>>);

impl<S> EdgesSortedLon<S>
where S: BaseFloat {
    fn new(vertices: &[Vector4<S>]) -> EdgesSortedLon<S> {
        let mut edges = EdgeIterator::new(&vertices).collect::<Vec<_>>();
        edges.sort_unstable_by(|e1, e2| {
            // Get the minimum longitudes from e1 and e2 vertices
            let e1_min_lon = e1.v1.lon().min(e1.v2.lon());
            let e2_min_lon = e2.v1.lon().min(e2.v2.lon());

            e1_min_lon.partial_cmp(&e2_min_lon).unwrap()
        });
        // Swap the edges vertices in increasing longitude order
        edges = edges.into_iter()
            .map(|mut e| {
                if e.v1.lon() > e.v2.lon() {
                    e.swap();
                }
                e
            })
            .collect();

        EdgesSortedLon(edges)
    }
}

use core::ops::Deref;
impl<S> Deref for EdgesSortedLon<S>
where S: BaseFloat {
    type Target = Vec<Edge<S>>;

    fn deref (self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

struct EdgesSortedLat<S: BaseFloat>(Vec<Edge<S>>);

impl<S> EdgesSortedLat<S>
where S: BaseFloat {
    fn new(vertices: &[Vector4<S>]) -> EdgesSortedLat<S> {
        let mut edges = EdgeIterator::new(&vertices).collect::<Vec<_>>();
        edges.sort_unstable_by(|e1, e2| {
            // Get the minimum latitudes from e1 and e2 vertices
            let e1_min_lat = e1.v1.lat().min(e1.v2.lat());
            let e2_min_lat = e2.v1.lat().min(e2.v2.lat());

            let result = e1_min_lat.partial_cmp(&e2_min_lat);

            result.unwrap()
        });
        // Swap the edges vertices in increasing latitudes order
        edges = edges.into_iter()
            .map(|mut e| {
                if e.v1.lat() > e.v2.lat() {
                    e.swap();
                }
                e
            })
            .collect();

        EdgesSortedLat(edges)
    }
}

impl<S> Deref for EdgesSortedLat<S>
where S: BaseFloat {
    type Target = Vec<Edge<S>>;

    fn deref (self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

pub struct Polygon {
    // Coordinates longitudes
    lon: Vec<Angle<f32>>,
    // Coordinates latitudes
    lat: Vec<Angle<f32>>,
    // Edges of the polygon sorted by increasing longitudes
    _edges_sorted_lon: EdgesSortedLon<f32>,
    edges_sorted_lat: EdgesSortedLat<f32>,
    // Variable storing the info if the
    // polygon contains a pole
    pole: PoleContained,
}

// A polygon must contain at least 3 vertices
impl Polygon {
    fn new(vertices: Vec<Vector4<f32>>) -> Polygon {
        assert!(vertices.len() >= 3);

        // Compute longitudes and latitudes
        let (lon, lat): (Vec<_>, Vec<_>) = vertices.iter()
            .map(|vertex| {
                let lonlat: LonLatT<f32> = vertex.lonlat();
                (lonlat.lon(), lonlat.lat())
            })
            .unzip();

        // The longitudes must be readjust if the
        // polygon crosses the 0deg meridian
        // We make the assumption the polygon is not too big
        // (i.e. < PI length on the longitude so that it does not
        // crosses both the 0 and 180deg meridians)
        let lon = if is_intersecting_meridian(&lon, Rad(0_f32)) {
            lon.into_iter()
                .map(|lon| {
                    if lon > PI {
                        lon - TWICE_PI
                    } else {
                        lon
                    }
                })
                .collect::<Vec<_>>()
        } else {
            lon.into_iter()
                .map(|lon| {
                    if lon < 0_f32 {
                        lon + TWICE_PI
                    } else {
                        lon
                    }
                })
                .collect::<Vec<_>>()
        };

        let pole = PoleContained::new(&lon, &lat);

        let edges_sorted_lat = EdgesSortedLat::new(&vertices);
        let _edges_sorted_lon = EdgesSortedLon::new(&vertices);

        Polygon {
            lon,
            lat,
            edges_sorted_lat,
            _edges_sorted_lon,
            pole
        }
    }

    #[inline]
    fn is_containing_a_pole(&self) -> bool {
        self.pole == PoleContained::North || self.pole == PoleContained::South
    }

    #[inline]
    fn is_containing_south_pole(&self) -> bool {
        self.pole == PoleContained::South
    }

    #[inline]
    fn is_containing_north_pole(&self) -> bool {
        self.pole == PoleContained::North
    }

    fn get_bbox(&self) -> BoundingBox {
        let lat = if self.is_containing_south_pole() {
            // All the latitudes lower than the maximum latitude
            // of the vertices are included or intersect the polygon
            let max_lat = self.lat.iter()
                .fold(Angle::min_value(), |a, b| a.max(*b));

            Angle(-HALF_PI)..max_lat
        } else if self.is_containing_north_pole() {
            // All the latitudes upper than the minimum latitude
            // of the vertices are included or intersect the polygon
            let min_lat = self.lat.iter()
                .fold(Angle::max_value(), |a, b| a.min(*b));

            min_lat..Angle(HALF_PI)
        } else {
            // The polygon does not contain any pole
            // Meridian 0deg is not crossing the polygon
            let (min_lat, max_lat) = self.lat.iter()
                .fold((Angle::max_value(), Angle::min_value()),
                    |(min, max), b| (
                        min.min(*b),
                        max.max(*b)
                    )
                );

            min_lat..max_lat
        };

        let lon = if self.is_containing_a_pole() {
            Angle(0_f32)..Angle(TWICE_PI)
        } else {
            let (min_lon, max_lon) = self.lon.iter()
                .fold((Angle::max_value(), Angle::min_value()), 
                    |(min, max), b| (
                        min.min(*b),
                        max.max(*b)
                    )
                );

            min_lon..max_lon
        };

        BoundingBox {
            lon,
            lat
        }
    }

    #[inline]
    // Compute the intersection between a great-circle defined by its normal vector
    // with an arc of great-circle defined by two vertices
    // Precondition:
    // - ``n`` is a normal vector that has to be normalized
    // - ``a`` and ``b`` are positions on the sphere, they are normalized too
    fn intersect_great_circle(n: &Vector3<f32>, edge: &Edge<f32>) -> Option<Vector3<f32>> {
        let v1 = edge.v1.vector();
        let v2 = edge.v2.vector();
        // The intersection between the two great-circles is given
        // by r = n x (v1 x v2)
        //      = dot(n, v2) x v1 - dot(n, v1) x v2
        let mut r = n.dot(v2) * v1 - n.dot(v1) * v2;
        r = r.normalize();
        // Look whether r is in the arc of great-circle defined by a and b
/*        let l = r.lon();
        let left_l = edge.v1.lon();
        let right_l = edge.v2.lon();

        if is_in_lon_range(l, left_l, right_l) {
            Some(r)
        } else {
            // Check the opposite point 
            let l_opp = l + PI;
            if is_in_lon_range(l_opp, left_l, right_l) {
                Some(-r)
            } else {
                None
            }
        }*/
        if edge.is_in_lon_range(&r) {
            Some(r)
        } else if edge.is_in_lon_range(&(-r)) {
            Some(-r)
        } else {
            None
        }
    }

    // Return if it exists, the intersection between a polygon and a parallel
    //
    // There can be many intersections. The intersection returned is the one
    // having the min longitude
    fn _intersect_parallel<LatT: Into<Angle<f32>>>(&self, _lat: LatT) -> Option<Vector3<f32>> {
        if self.is_containing_a_pole() {
            // TODO
            None
        } else {
            // Normal of a parallel
            let n = Vector3::new(0_f32, 1_f32, 0_f32);

            for edge in self._edges_sorted_lon.iter() {
                // Return the first intersection found
                if let Some(vertex) = Self::intersect_great_circle(&n, edge) {
                    return Some(vertex);
                }
            }
            
            // All the edges have been processed and
            // no intersections have been found
            None
        }
    }

    // Return if it exists, the intersection between a polygon and a parallel
    //
    // There can be many intersections. The intersection returned is the one
    // having the min longitude
    fn intersect_meridian<LonT: Into<Angle<f32>>>(&self, lon: LonT) -> Option<Vector3<f32>> {
        if self.is_containing_a_pole() {
            // TODO
            None
        } else {
            let lon: Angle<f32> = lon.into();
            // Normal of a meridian
            let n = Vector3::new(lon.cos().0, 0_f32, -lon.sin().0);

            for edge in self.edges_sorted_lat.iter() {
                // Return the first intersection found
                if let Some(vertex) = Self::intersect_great_circle(&n, edge) {
                    return Some(vertex);
                }
            }
            
            // All the edges have been processed and
            // no intersections have been found
            None
        }
    }

    /*// Return the intesection between a polygon and a great-circle defined by its
    // normal vector
    fn intersect_great_circle(&self, n: &Vector3<f32>) -> Option<> {
        // If the polygon contains a pole, then it intersect all the meridians
        if self.contains_pole() {
            return true;
        }

        // Get the longitude of the meridian in radians
        let lon: Rad<f32> = lon.into();
        // Compute a vector normal to the plane intersecting the sphere
        // that gives the meridian great-circle
        // this vector is normalized
        let n = Vector3::new(lon.0.cos(), 0_f32, -lon.0.sin());

        let mut j = (self.lon.len() - 1) as usize;
        // Loop over all the edge of the polygon
        for i in 0..=j {
            // We approx the computation saying that the edges of the polygon
            // lie on great-circle arcs which may not be totally true
            // This is a decent approx because the fov has multiple control
            // vertices, it is not just 4 vertices

            // We compute the vector normal to the plane defining the great-circle
            // passing by i and j
            let vertex_i = math::radec_to_xyz(self.lon[i], self.lat[i]);
            let vertex_j = math::radec_to_xyz(self.lon[j], self.lat[j]);
            let edge_length = math::ang_between_vect(&vertex_i, &vertex_j);

            let u = vertex_j.cross(vertex_i).normalize();

            // The direction of the intersectional vertices is given
            // by n x u
            let r = n.cross(u).normalize();
            // Check the longitude of r to be equal to lon
            let (lon_r, _) = math::xyz_to_radec(&r);
            // We have two candidates, r and its antipodal vertex
            // Check r first to know if it is lying in the great-circle arc
            // corresponding to the edge
            if (lon.0 - lon_r.0).abs() < 1e-4 {
                let r_to_i_length = math::ang_between_vect(&r, &vertex_i);
                let r_to_j_length = math::ang_between_vect(&r, &vertex_j);
                let r_between_i_and_j = ((edge_length - (r_to_i_length + r_to_j_length)).0).abs() < 1e-4;

                if r_between_i_and_j {
                    return true;
                }
            } else {
                // consider the antipodal as the good longitude
                let anti_r_to_i_length = math::ang_between_vect(&(-r), &vertex_i);
                let anti_r_to_j_length = math::ang_between_vect(&(-r), &vertex_j);
                let anti_r_between_i_and_j = ((edge_length - (anti_r_to_i_length + anti_r_to_j_length)).0).abs() < 1e-4;
            
                if anti_r_between_i_and_j {
                    return true;
                }
            }

            j = i;
        }

        false
    }*/

    fn _is_intersecting_meridian<MeridianT: Into<Angle<f32>>>(&self, lon: MeridianT) -> bool {
        if self.is_containing_a_pole() {
            // Contains a pole
            true
        } else {
            // Contains no poles
            let lon: Angle<f32> = lon.into();
            let num_lon = self.lon.len();
            let mut last = num_lon - 1;
            // Loop over all the edge of the polygon
            for cur in 0..num_lon {
                let left_l = self.lon[last];
                let right_l = self.lon[cur];
                if is_in_lon_range(lon, left_l, right_l) {
                    return true;
                }

                last = cur;
            }

            false
        }
    }

    fn _is_intersecting_parallel<ParallelT: Into<Angle<f32>>>(&self, lat: ParallelT) -> bool {
        // Get the latitude of the meridian in radians
        let lat: Angle<f32> = lat.into();

        if self.is_containing_south_pole() {
            // All the latitudes lower than the maximum latitude
            // of the vertices are included or intersect the polygon
            let max_lat = self.lat.iter()
                .fold(Angle::min_value(), |a, b| a.max(*b));

            lat < max_lat
        } else if self.is_containing_north_pole() {
            // All the latitudes upper than the minimum latitude
            // of the vertices are included or intersect the polygon
            let min_lat = self.lat.iter()
                .fold(Angle::max_value(), |a, b| a.min(*b));

            lat > min_lat
        } else {
            // No poles included
            let num_lon = self.lon.len() as usize;
            let mut last = num_lon - 1;
            // Loop over all the edge of the polygon
            for cur in 0..num_lon {
                let (min_lat, max_lat) = (
                    self.lat[cur].min(self.lat[last]),
                    self.lat[cur].max(self.lat[last])
                );

                if lat < max_lat && lat > min_lat {
                    return true;
                }

                last = cur;
            }

            false
        }
    }
}

pub struct PolygonGrid {
    // Polygon of the field of view
    poly: Polygon,
    // Bounding box of the polygon
    bbox: BoundingBox,

    // Meridians intersecting the polygon
    meridians: Vec<Angle<f32>>,
    num_max_meridians: usize,
    // Parallels intersecting the polygon
    parallels: Vec<Angle<f32>>,
    num_max_parallels: usize,

    grid_steps: Vec<Angle<f32>>,
}

use crate::renderable::FormatType;
impl PolygonGrid {
    fn new(vertices: Vec<Vector4<f32>>, aspect: f32) -> PolygonGrid {
        let poly = Polygon::new(vertices);
        let bbox = poly.get_bbox();

        let grid_steps = vec![
            ArcDeg(45_f32).into(),
            ArcDeg(20_f32).into(),
            ArcDeg(10_f32).into(),
            ArcDeg(5_f32).into(),
            ArcDeg(2_f32).into(),
            ArcDeg(1_f32).into(),
            ArcMin(30_f32).into(),
            ArcMin(15_f32).into(),
            ArcMin(10_f32).into(),
            ArcMin(5_f32).into(),
            ArcMin(2_f32).into(),
            ArcMin(1_f32).into(),
            ArcSec(30_f32).into(),
            ArcSec(15_f32).into(),
            ArcSec(10_f32).into(),
            ArcSec(5_f32).into(),
            ArcSec(2_f32).into(),
            ArcSec(1_f32).into(),
            ArcSec(5e-1).into(),
            ArcSec(2e-1).into(),
            ArcSec(1e-1).into(),
            ArcSec(5e-2).into(),
            ArcSec(2e-2).into(),
            ArcSec(1e-2).into(),
            ArcSec(5e-3).into(),
            ArcSec(2e-3).into(),
            ArcSec(1e-3).into(),
            ArcSec(5e-4).into(),
            ArcSec(2e-4).into(),
            ArcSec(1e-4).into(),
            ArcSec(5e-5).into(),
            ArcSec(2e-5).into(),
            ArcSec(1e-5).into(),
            ArcSec(5e-6).into(),
            ArcSec(2e-6).into(),
            ArcSec(1e-6).into(),
        ];

        let parallels = vec![];
        let num_max_parallels = 5;
        let meridians = vec![];
        let num_max_meridians = ((num_max_parallels as f32) * aspect) as usize;

        let mut polygon = PolygonGrid {
            poly,
            bbox,

            meridians,
            parallels,
            grid_steps,
            num_max_meridians,
            num_max_parallels,
        };

        polygon.compute_meridians();
        polygon.compute_parallels();

        polygon
    }

    fn compute_meridians(&mut self) {
        let fov = self.bbox.get_lon_size();

        // Select the best meridian grid step
        let mut i = 0;
        let mut step = self.grid_steps[0];
        while i < self.grid_steps.len() {
            if fov >= self.grid_steps[i] {
                let num_meridians_in_fov = (fov / self.grid_steps[i]).0 as usize;

                if num_meridians_in_fov >= self.num_max_meridians - 1 {
                    let idx_grid = if i == 0 {
                        0
                    } else {
                        i - 1
                    };
                    step = self.grid_steps[idx_grid];
                    break;
                }
            }

            step = self.grid_steps[i];
            i += 1;
        }

        let mut meridian = self.bbox.lon_min() - (self.bbox.lon_min() % step);

        while meridian < self.bbox.lon_max() {
            self.meridians.push(meridian);
            meridian += step;
        }

        // Assert the number of meridians to plot does not
        // exceed the max number of meridians that the GPU
        // can handle
        //assert!(self.meridians.len() <= self.num_max_meridians);
    }
    fn compute_parallels(&mut self) {
        // We do have the bounding box, let's return the list of meridians intersecting
        let fov = self.bbox.get_lat_size();
        // Select the best meridian grid step
        let mut i = 0;
        let mut step = self.grid_steps[0];
        while i < self.grid_steps.len() {
            if fov >= self.grid_steps[i] {
                let num_parallels_in_fov = (fov / self.grid_steps[i]).0 as usize;

                if num_parallels_in_fov >= self.num_max_parallels - 1 {
                    let idx_grid = if i == 0 {
                        0
                    } else {
                        i - 1
                    };
                    step = self.grid_steps[idx_grid];
                    break;
                }
            }

            step = self.grid_steps[i];
            i += 1;
        }

        let mut parallel = self.bbox.lat_min() - (self.bbox.lat_min() % step);

        while parallel < self.bbox.lat_max() {
            self.parallels.push(parallel);
            parallel += step;
        }

        // Assert the number of parallels to plot does not
        // exceed the max number of parallels that the GPU
        // can handle
        assert!(self.parallels.len() <= self.num_max_parallels);
    }

    fn get_meridians_intersecting_fov<F: FormatType>(&self) -> HashMap<String, Vector3<f32>> {
        let mut meridians_labels = HashMap::with_capacity(self.num_max_meridians);
        for &meridian in self.meridians() {
            let vertex = self.poly.intersect_meridian(meridian);
            // If there is intersection
            if let Some(vertex) = vertex {
                meridians_labels.insert(meridian.to_string::<F>(), vertex);
            }
        }

        meridians_labels
    }

    // Return if it exists, the intersection between a polygon and a meridian
    //
    // There can be many intersections. The intersection returned is the one
    // having the min longitude
    fn _intersect_meridian<LonT: Into<Angle<f32>>>(&self, lon: LonT) -> Option<Vector3<f32>> {
        self.poly.intersect_meridian(lon)
    }

    // Return if it exists, the intersection between a polygon and a parallel
    //
    // There can be many intersections. The intersection returned is the one
    // having the min longitude
    fn _intersect_parallel<LatT: Into<Angle<f32>>>(&self, lat: LatT) -> Option<Vector3<f32>> {
        self.poly._intersect_parallel(lat)
    }
}

impl ZoneFieldOfView for PolygonGrid {
    fn meridians(&self) -> &[Angle<f32>] {
        &self.meridians
    }
    fn parallels(&self) -> &[Angle<f32>] {
        &self.parallels
    }
}

#[inline]
fn is_in_lon_range(l: Angle<f32>, l1: Angle<f32>, l2: Angle<f32>) -> bool {
    // First version of the code: 
    //   ((v2.lon() - v1.lon()).abs() > PI) != ((v2.lon() > coo.lon()) != (v1.lon() > coo.lon()))
    // 
    // Lets note 
    //   - lonA = v1.lon()
    //   - lonB = v2.lon()
    //   - lon0 = coo.lon()
    // When (lonB - lonA).abs() <= PI 
    //   => lonB > lon0 != lonA > lon0  like in PNPOLY
    //   A    B    lonA <= lon0 && lon0 < lonB
    // --[++++[--
    //   B    A    lonB <= lon0 && lon0 < lonA
    //
    // But when (lonB - lonA).abs() > PI, then the test should be 
    //  =>   lonA >= lon0 == lonB >= lon0 
    // <=> !(lonA >= lon0 != lonB >= lon0)
    //    A  |  B    (lon0 < lonB) || (lonA <= lon0)
    //  --[++|++[--
    //    B  |  A    (lon0 < lonA) || (lonB <= lon0)
    //
    // Instead of lonA > lon0 == lonB > lon0,
    //     i.e. !(lonA > lon0 != lonB > lon0).
    //    A  |  B    (lon0 <= lonB) || (lonA < lon0)
    //  --]++|++]--
    //    B  |  A    (lon0 <= lonA) || (lonB < lon0)
    //
    // So the previous code was bugged in this very specific case: 
    // - `lon0` has the same value as a vertex being part of:
    // - one segment that do not cross RA=0
    //   - plus one segment crossing RA=0.
    //   - the point have an odd number of intersections with the polygon 
    //     (since it will be counted 0 or 2 times instead of 1).
    let dlon = l2 - l1;
    if dlon < 0.0 {
        (dlon >= -PI) == (l2 <= l && l < l1)
    } else {
        (dlon <=  PI) == (l1 <= l && l < l2)
    }
}

fn is_intersecting_meridian<MeridianT: Into<Angle<f32>>>(lon: &[Angle<f32>], meridian: MeridianT) -> bool {
    let meridian: Angle<f32> = meridian.into();
    let num_lon = lon.len() as usize;
    // Loop over all the edge of the polygon
    let mut last = num_lon - 1;
    for cur in 0..num_lon {
        let left_l = lon[last];
        let right_l = lon[cur];
        if is_in_lon_range(meridian, left_l, right_l) {
            return true;
        }

        last = cur;
    }

    false
}