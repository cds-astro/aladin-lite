use cgmath::Rad;
use cgmath::{Vector3, Vector4};
const PI: Angle<f64> = Angle(std::f64::consts::PI);
const TWICE_PI: Angle<f64> = Angle(std::f64::consts::PI * 2.0);
const HALF_PI: Angle<f64> = Angle(std::f64::consts::PI * 0.5);

use crate::angle::Angle;

use cgmath::InnerSpace;

pub enum FieldOfViewType {
    Allsky(Allsky),
    Polygon(Polygon),
}

//use cgmath::Vector2;
use crate::CameraViewPort;
impl FieldOfViewType {
    pub fn new_allsky() -> FieldOfViewType {
        let allsky = FieldOfViewType::Allsky(Allsky::new());

        allsky
    }

    pub fn new_polygon(vertices: &[Vector4<f64>]) -> FieldOfViewType {
        FieldOfViewType::Polygon(Polygon::new(vertices))
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        match self {
            FieldOfViewType::Allsky(allsky) => allsky.get_bbox(),
            FieldOfViewType::Polygon(poly) => poly.get_bbox(),
        }
    }

    /*pub fn get_labels<F: FormatType>(&self) -> HashMap<String, Vector3<f32>> {
        let mut great_circles_labels = HashMap::new();

        if let FieldOfViewType::Polygon(polygon) = self {
            let meridians_labels = polygon.get_meridians_intersecting_fov::<F>();
            great_circles_labels.extend(meridians_labels.into_iter());
        }

        great_circles_labels
    }*/

    pub fn intersect_meridian<LonT: Into<Rad<f64>>>(
        &self,
        lon: LonT,
        camera: &CameraViewPort,
    ) -> Option<Vector3<f64>> {
        match self {
            FieldOfViewType::Allsky(_) => {
                // Allsky case
                // We do an approx saying allsky fovs intersect all meridian
                // but this is not true for example for the orthographic projection
                // Some meridians may not be visible
                let system = camera.get_system();
                let center = (system.to_gal::<f64>() * camera.get_center()).lonlat();
                let lon: Rad<f64> = lon.into();
                let pos: Vector3<f64> = LonLatT::new(lon.into(), center.lat()).vector();
                Some(pos)
            }
            FieldOfViewType::Polygon(polygon) => polygon.intersect_meridian(lon),
        }
    }

    pub fn intersect_parallel<LatT: Into<Rad<f64>>>(
        &self,
        lat: LatT,
        camera: &CameraViewPort,
    ) -> Option<Vector3<f64>> {
        match self {
            FieldOfViewType::Allsky(_) => {
                let system = camera.get_system();

                let center = (system.to_gal::<f64>() * camera.get_center()).lonlat();
                let lat: Rad<f64> = lat.into();
                let pos: Vector3<f64> = LonLatT::new(center.lon(), lat.into()).vector();
                Some(pos)
            }
            FieldOfViewType::Polygon(poly) => poly.intersect_parallel(lat, camera),
        }
    }

    pub fn is_allsky(&self) -> bool {
        matches!(self, FieldOfViewType::Allsky(_))
    }

    pub fn contains_pole(&self) -> bool {
        match self {
            FieldOfViewType::Allsky(_) => true,
            FieldOfViewType::Polygon(poly) => poly.contains_pole(),
        }
    }

    pub fn contains_north_pole(&self, camera: &CameraViewPort) -> bool {
        match self {
            FieldOfViewType::Allsky(_) => {
                let center = camera.get_center();
                center.y >= 0.0
            }
            FieldOfViewType::Polygon(poly) => poly.contains_north_pole(),
        }
    }

    pub fn contains_south_pole(&self, camera: &CameraViewPort) -> bool {
        match self {
            FieldOfViewType::Allsky(_) => {
                let center = camera.get_center();
                center.y < 0.0
            }
            FieldOfViewType::Polygon(poly) => poly.contains_south_pole(),
        }
    }
}

/*
use crate::shader::SendUniforms;
use crate::shader::ShaderBound;
impl SendUniforms for FieldOfViewType {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        match self {
            FieldOfViewType::Allsky(ref allsky) => {
                shader.attach_uniforms_from(allsky);
            },
            FieldOfViewType::Polygon(polygon) => {
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

use crate::renderable::angle::transmute_angles;
impl<T> SendUniforms for T where T: ZoneFieldOfView {
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
}*/

pub struct Allsky {
    bbox: BoundingBox,
}

impl Allsky {
    fn new() -> Allsky {
        let bbox = BoundingBox::fullsky();

        Allsky { bbox }
    }

    fn get_bbox(&self) -> &BoundingBox {
        &self.bbox
    }
}
/*
impl ZoneFieldOfView for Allsky {
    fn meridians(&self) -> &[Angle<f32>] {
        &self.meridians
    }
    fn parallels(&self) -> &[Angle<f32>] {
        &self.parallels
    }
}
*/
// Pole contained in polygon fov mode
#[derive(PartialEq, Eq)]
enum Pole {
    North,
    South,
}

impl Pole {
    // This checks whether the polygon contains a pole
    // The code is inspired by the formula given here:
    // https://www.edwilliams.org/avform.htm#Crs
    fn contained_in_polygon(lon: &[Angle<f64>], lat: &[Angle<f64>]) -> Option<Self> {
        // For each edge of the polygon, we compute the heading angle (i.e. course)
        // from the starting vertex of the edge to the ending one.
        let mut sum_delta_lon = Angle::new(Rad(0.0));

        let mut num_vertices_in_south = 0 as usize;

        let num_lon = lon.len();
        let mut last = num_lon - 1;

        for cur in 0..num_lon {
            let delta_lon = lon[cur] - lon[last];
            let abs_delta_lon = delta_lon.abs();

            if abs_delta_lon <= PI {
                sum_delta_lon += delta_lon;
            } else if delta_lon > Angle(0.0) {
                sum_delta_lon -= -abs_delta_lon + TWICE_PI;
            } else {
                sum_delta_lon += -abs_delta_lon + TWICE_PI;
            }

            if lat[cur] < Angle(0.0) {
                num_vertices_in_south += 1;
            }

            last = cur;
        }

        if sum_delta_lon.abs() > PI {
            let num_vertices = lon.len();
            // More than the half of the vertices are located
            // in the south hemisphere
            if (num_vertices_in_south << 1) >= num_vertices {
                Some(Pole::South)
            } else {
                Some(Pole::North)
            }
        } else {
            None
        }
    }

    #[inline]
    fn is_south(&self) -> bool {
        *self == Pole::South
    }

    #[inline]
    fn is_north(&self) -> bool {
        *self == Pole::North
    }
}

use std::ops::Range;
#[derive(Clone)]
pub struct BoundingBox {
    pub lon: Range<Angle<f64>>,
    pub lat: Range<Angle<f64>>,
}

impl BoundingBox {
    fn from_polygon(pole_contained: &Option<Pole>, lon: &[Angle<f64>], lat: &[Angle<f64>]) -> Self {
        if let Some(pole) = pole_contained {
            let lat = if pole.is_south() {
                // All the latitudes lower than the maximum latitude
                // of the vertices are included or intersect the polygon
                let max_lat = lat.iter().fold(Angle::min_value(), |a, b| a.max(*b));

                -HALF_PI..max_lat
            } else {
                // All the latitudes upper than the minimum latitude
                // of the vertices are included or intersect the polygon
                let min_lat = lat.iter().fold(Angle::max_value(), |a, b| a.min(*b));

                min_lat..HALF_PI
            };

            let lon = Angle(0.0)..TWICE_PI;
            BoundingBox { lon, lat }
        } else {
            // The polygon does not contain any pole
            // Meridian 0deg is not crossing the polygon
            let (min_lat, max_lat) = lat
                .iter()
                .fold((Angle::max_value(), Angle::min_value()), |(min, max), b| {
                    (min.min(*b), max.max(*b))
                });

            let lat = min_lat..max_lat;

            let (min_lon, max_lon) = lon
                .iter()
                .fold((Angle::max_value(), Angle::min_value()), |(min, max), b| {
                    (min.min(*b), max.max(*b))
                });

            let lon = min_lon..max_lon;
            BoundingBox { lon, lat }
        }
    }

    #[inline]
    pub fn get_lon_size(&self) -> Angle<f64> {
        self.lon.end - self.lon.start
    }

    #[inline]
    pub fn all_lon(&self) -> bool {
        (self.lon.end - self.lon.start) == TWICE_PI
    }

    #[inline]
    pub fn all_lat(&self) -> bool {
        (self.lat.end - self.lat.start) == PI
    }

    #[inline]
    pub fn get_lat_size(&self) -> Angle<f64> {
        self.lat.end - self.lat.start
    }
    #[inline]
    pub fn lon_min(&self) -> Angle<f64> {
        self.lon.start
    }
    #[inline]
    pub fn lon_max(&self) -> Angle<f64> {
        self.lon.end
    }
    #[inline]
    pub fn lat_min(&self) -> Angle<f64> {
        self.lat.start
    }
    #[inline]
    pub fn lat_max(&self) -> Angle<f64> {
        self.lat.end
    }
    #[inline]
    pub fn get_lon(&self) -> Range<f64> {
        self.lon.start.0..self.lon.end.0
    }
    #[inline]
    pub fn get_lat(&self) -> Range<f64> {
        self.lat.start.0..self.lat.end.0
    }

    pub fn fullsky() -> Self {
        BoundingBox {
            lon: Angle(0.0)..TWICE_PI,
            lat: -HALF_PI..HALF_PI,
        }
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
where
    S: BaseFloat,
{
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
    pub v2: LonLatT<S>,
}

use crate::math;
use crate::math::LonLat;
impl<S> Edge<S>
where
    S: BaseFloat,
{
    // Swap the vertices of the edge
    #[inline]
    fn swap(&mut self) {
        std::mem::swap(&mut self.v1, &mut self.v2);
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
where
    S: BaseFloat,
{
    type Item = Edge<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finished {
            let prev = self.prev;
            let curr = self.curr;
            let v1 = self.vertices[prev].lonlat();
            let v2 = self.vertices[curr].lonlat();

            if v1.lon().0.is_nan()
                || v1.lat().0.is_nan()
                || v2.lon().0.is_nan()
                || v2.lat().0.is_nan()
            {
                if self.curr == self.vertices.len() - 1 {
                    self.finished = true;
                } else {
                    // There are still edges, we increment self.curr
                    self.prev = curr;
                    self.curr += 1;
                }

                self.next()
            } else {
                let edge = Edge { v1, v2 };

                if self.curr == self.vertices.len() - 1 {
                    self.finished = true;
                } else {
                    // There are still edges, we increment self.curr
                    self.prev = curr;
                    self.curr += 1;
                }

                Some(edge)
            }
        } else {
            None
        }
    }
}

struct EdgesSortedLon<S: BaseFloat>(Vec<Edge<S>>);

impl<S> EdgesSortedLon<S>
where
    S: BaseFloat,
{
    fn new(vertices: &[Vector4<S>]) -> EdgesSortedLon<S> {
        let mut edges = EdgeIterator::new(&vertices).collect::<Vec<_>>();
        edges.sort_unstable_by(|e1, e2| {
            // Get the minimum longitudes from e1 and e2 vertices
            let e1_min_lon = e1.v1.lon().min(e1.v2.lon());
            let e2_min_lon = e2.v1.lon().min(e2.v2.lon());

            e1_min_lon.partial_cmp(&e2_min_lon).unwrap()
        });
        // Swap the edges vertices in increasing longitude order
        edges = edges
            .into_iter()
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
where
    S: BaseFloat,
{
    type Target = Vec<Edge<S>>;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

struct EdgesSortedLat<S: BaseFloat>(Vec<Edge<S>>);

impl<S> EdgesSortedLat<S>
where
    S: BaseFloat + std::cmp::PartialOrd,
{
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
        edges = edges
            .into_iter()
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
where
    S: BaseFloat,
{
    type Target = Vec<Edge<S>>;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

pub struct Polygon {
    bbox: BoundingBox,
    // Edges of the polygon sorted by increasing longitudes
    edges_sorted_lon: EdgesSortedLon<f64>,
    edges_sorted_lat: EdgesSortedLat<f64>,
    // Pole contained
    pole: Option<Pole>,
}
// A polygon must contain at least 3 vertices
impl Polygon {
    fn new(vertices: &[Vector4<f64>]) -> Polygon {
        assert!(vertices.len() >= 3);

        // Compute longitudes and latitudes
        let (lon, lat): (Vec<_>, Vec<_>) = vertices
            .iter()
            .map(|vertex| {
                let lonlat: LonLatT<f64> = vertex.lonlat();
                (lonlat.lon(), lonlat.lat())
            })
            .unzip();

        // The longitudes must be readjust if the
        // polygon crosses the 0deg meridian
        // We make the assumption the polygon is not too big
        // (i.e. < PI length on the longitude so that it does not
        // crosses both the 0 and 180deg meridians)
        let lon = if is_intersecting_meridian(&lon, Rad(0.0)) {
            lon.into_iter()
                .map(|lon| if lon > PI { lon - TWICE_PI } else { lon })
                .collect::<Vec<_>>()
        } else {
            lon.into_iter()
                .map(|lon| {
                    if lon < Angle(0.0) {
                        lon + TWICE_PI
                    } else {
                        lon
                    }
                })
                .collect::<Vec<_>>()
        };

        let pole = Pole::contained_in_polygon(&lon, &lat);
        let bbox = BoundingBox::from_polygon(&pole, &lon, &lat);

        let vertices = lon
            .into_iter()
            .zip(lat.into_iter())
            .map(|(lon, lat)| LonLatT::new(lon, lat).vector())
            .collect::<Vec<_>>();

        let edges_sorted_lat = EdgesSortedLat::new(&vertices);
        let edges_sorted_lon = EdgesSortedLon::new(&vertices);

        Polygon {
            edges_sorted_lat,
            edges_sorted_lon,
            bbox,
            pole,
        }
    }
    #[inline]
    fn contains_pole(&self) -> bool {
        self.pole.is_some()
    }

    #[inline]
    fn contains_north_pole(&self) -> bool {
        if let Some(p) = &self.pole {
            p.is_north()
        } else {
            false
        }
    }

    #[inline]
    fn contains_south_pole(&self) -> bool {
        if let Some(p) = &self.pole {
            p.is_south()
        } else {
            false
        }
    }

    #[inline]
    fn get_bbox(&self) -> &BoundingBox {
        &self.bbox
    }

    // Return if it exists, the intersection between a polygon and a parallel
    //
    // There can be many intersections. The intersection returned is the one
    // having the min longitude
    pub fn intersect_parallel<LatT: Into<Rad<f64>>>(
        &self,
        lat: LatT,
        camera: &CameraViewPort,
    ) -> Option<Vector3<f64>> {
        if let Some(pole) = &self.pole {
            // A pole is contained in the polygon
            // We know there is an intersection if lat is
            if pole.is_north() {
                // North pole
                let lat_min: Rad<f64> = self.bbox.lat_min().into();
                let lat: Rad<f64> = lat.into();
                if lat > lat_min {
                    let center = camera.get_center().lonlat();
                    let inter: Vector3<f64> =
                        LonLatT::from_radians(center.lon().into(), lat).vector();
                    Some(inter)
                } else {
                    None
                }
            } else {
                // South pole
                let lat_max: Rad<f64> = self.bbox.lat_max().into();
                let lat: Rad<f64> = lat.into();
                if lat < lat_max {
                    let center = camera.get_center().lonlat();
                    let inter: Vector3<f64> =
                        LonLatT::from_radians(center.lon().into(), lat).vector();
                    Some(inter)
                } else {
                    None
                }
            }
        } else {
            let lat = (lat.into() as Rad<f64>).0;
            for edge in self.edges_sorted_lon.iter() {
                // Return the first intersection found
                if let Some(vertex) = Self::get_parallel_intersect(lat, edge) {
                    return Some(vertex);
                }
            }

            None
        }
    }

    // Return if it exists, the intersection between a polygon and a meridian
    //
    // There can be many intersections. The intersection returned is the one
    // having the min latitude
    pub fn intersect_meridian<LonT: Into<Rad<f64>>>(&self, lon: LonT) -> Option<Vector3<f64>> {
        let lon_rad: Rad<f64> = lon.into();
        let mut lon: Angle<f64> = lon_rad.into();
        if lon > PI {
            lon -= TWICE_PI;
        }

        // Normal of a meridian
        for edge in self.edges_sorted_lat.iter() {
            // Return the first intersection found
            if let Some(vertex) = Self::get_meridian_intersect(lon, edge) {
                return Some(vertex);
            }
        }

        // All the edges have been processed and
        // no intersections have been found
        None
    }
}

impl Polygon {
    // Compute the intersection between a great-circle defined by its normal vector
    // with an arc of great-circle defined by two vertices
    // Precondition:
    // - ``n`` is a normal vector that has to be normalized
    // - ``a`` and ``b`` are positions on the sphere, they are normalized too
    fn get_meridian_intersect(lon: Angle<f64>, edge: &Edge<f64>) -> Option<Vector3<f64>> {
        if !is_in_lon_range(lon, edge.v1.lon(), edge.v2.lon()) {
            None
        } else {
            let v1 = edge.v1.vector();
            let v2 = edge.v2.vector();
            let n = Vector3::new(lon.cos(), 0.0, -lon.sin());
            // The intersection between the two great-circles is given
            // by r = n x (v1 x v2)
            //      = dot(n, v2) x v1 - dot(n, v1) x v2
            let mut r = n.dot(v2) * v1 - n.dot(v1) * v2;
            r = r.normalize();

            if edge.is_in_lon_range(&r) {
                Some(r)
            } else if edge.is_in_lon_range(&(-r)) {
                Some(-r)
            } else {
                None
            }
        }
    }

    fn get_parallel_intersect(lat: f64, edge: &Edge<f64>) -> Option<Vector3<f64>> {
        let lat_max = edge.v1.lat().0.max(edge.v2.lat().0);
        let lat_min = edge.v1.lat().0.min(edge.v2.lat().0);

        if lat < lat_min || lat > lat_max {
            // No intersection possible
            None
        } else {
            // Code from https://math.stackexchange.com/questions/1157278/find-the-intersection-point-of-a-great-circle-arc-and-latitude-line
            // that computes the intersection between an arc that lies in a great circle
            // and a parallel
            let p1: Vector3<f64> = edge.v1.vector();
            let p2: Vector3<f64> = edge.v2.vector();

            let gc = p1.cross(p2).normalize();

            let a = p1.y;
            let b = gc.z * p1.x - gc.x * p1.z;
            let c = lat.sin();
            let r = (a * a + b * b).sqrt();

            let e = (b / a).atan();
            let f = (c / r).acos();
            let alpha = e - f;

            let ca = alpha.cos();
            let sa = alpha.sin();
            let c = gc.cross(p1);
            let inter = p1 * ca + c * sa;

            if edge.is_in_lon_range(&inter) {
                Some(inter)
            } else if edge.is_in_lon_range(&(-inter)) {
                Some(-inter)
            } else {
                let alpha = e + f;
                let ca = alpha.cos();
                let sa = alpha.sin();
                let inter = p1 * ca + c * sa;

                if edge.is_in_lon_range(&inter) {
                    Some(inter)
                } else if edge.is_in_lon_range(&(-inter)) {
                    Some(-inter)
                } else {
                    None
                }
            }
        }
    }
}

#[inline]
fn is_in_lon_range(l: Angle<f64>, l1: Angle<f64>, l2: Angle<f64>) -> bool {
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
    if dlon < Angle(0.0) {
        (dlon >= -PI) == (l2 <= l && l < l1)
    } else {
        (dlon <= PI) == (l1 <= l && l < l2)
    }
}

fn is_intersecting_meridian<MeridianT: Into<Angle<f64>>>(
    lon: &[Angle<f64>],
    meridian: MeridianT,
) -> bool {
    let meridian: Angle<f64> = meridian.into();
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
