use crate::math::angle::Angle;
use cgmath::Vector2;

pub trait Triangulate {
    fn triangulate() -> Triangulation;
}

use crate::{healpix::cell::HEALPixCell, math::projection::*};
impl Triangulate for Orthographic {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
impl Triangulate for Aitoff {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
impl Triangulate for Mollweide {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
impl Triangulate for AzimuthalEquidistant {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
impl Triangulate for Mercator {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
impl Triangulate for Gnomonic {
    fn triangulate() -> Triangulation {
        build::<Self>()
    }
}
use crate::math;
impl Triangulate for HEALPix {
    fn triangulate() -> Triangulation {
        fn counter_clockwise_tri(x: Vector2<f64>, y: Vector2<f64>, z: Vector2<f64>) -> bool {
            // From: https://math.stackexchange.com/questions/1324179/how-to-tell-if-3-connected-points-are-connected-clockwise-or-counter-clockwise
            // | x.x, x.y, 1 |
            // | y.x, y.y, 1 | > 0 => the triangle is given in counterclockwise order
            // | z.x, z.y, 1 |

            x.x * y.y + x.y * z.x + y.x * z.y - z.x * y.y - z.y * x.x - y.x * x.y >= 0.0
        }

        // The HEALPix 2d projection space is not convex
        // We can define it by creating triangles from the projection
        // of the HEALPix cells at order 2
        let mut off_idx = 0_u16;
        let mut idx = Vec::new();

        let vertices = HEALPixCell::allsky(3)
            .map(|cell| {
                idx.extend([
                    off_idx,
                    off_idx + 1,
                    off_idx + 2,
                    off_idx + 3,
                    off_idx + 4,
                    off_idx + 5,
                ]);

                let (c_ra, c_dec) = cell.center();

                let v = cell.vertices().map(|(ra, dec)| {
                    let ra = lerp(ra, c_ra, 1e-5);
                    let dec = lerp(dec, c_dec, 1e-5);

                    let v = math::lonlat::radec_to_xyzw(Angle(ra), Angle(dec));
                    HEALPix::world_to_clip_space(&v).unwrap()
                });

                let mut vertices = [v[0], v[3], v[2], v[2], v[1], v[0]];

                if !counter_clockwise_tri(vertices[3], vertices[4], vertices[5]) {
                    // triangles are crossing
                    vertices[3].x = 1.0;
                    vertices[5].x = 1.0;
                }

                off_idx += 6;

                vertices
            })
            .flatten()
            .collect::<Vec<_>>();

        Triangulation { vertices, idx }
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a * (1.0 - t) + b * t
}

pub struct Triangulation {
    pub vertices: Vec<Vector2<f64>>,
    pub idx: Vec<u16>,
}

fn build<P: Projection>() -> Triangulation {
    let (mut vertices, mut idx) = (Vec::new(), Vec::new());

    let root = Face::new(Vector2::new(-1_f64, -1_f64), Vector2::new(1_f64, 1_f64));
    let children = root.split();

    let depth = 5;
    let mut first = false;
    recursive_triangulation::<P>(&children[0], &mut vertices, &mut idx, depth, &mut first);
    recursive_triangulation::<P>(&children[1], &mut vertices, &mut idx, depth, &mut first);
    recursive_triangulation::<P>(&children[2], &mut vertices, &mut idx, depth, &mut first);
    recursive_triangulation::<P>(&children[3], &mut vertices, &mut idx, depth, &mut first);

    Triangulation { vertices, idx }
}

struct Face {
    min: Vector2<f64>,
    max: Vector2<f64>,
}

#[derive(Clone, Copy)]
pub enum Direction {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}
impl Face {
    fn new(min: Vector2<f64>, max: Vector2<f64>) -> Face {
        Face { min, max }
    }

    fn split(self) -> [Face; 4] {
        let bl = self.get_child(Direction::BottomLeft);
        let br = self.get_child(Direction::BottomRight);
        let tr = self.get_child(Direction::TopRight);
        let tl = self.get_child(Direction::TopLeft);

        [bl, br, tr, tl]
    }

    fn get_farthest_vertex(&self) -> (Vector2<f64>, Direction) {
        let x_neg = self.min.x < 0_f64;
        let y_neg = self.min.y < 0_f64;

        if x_neg && y_neg {
            // bottom-left
            (self.min, Direction::BottomLeft)
        } else if !x_neg && !y_neg {
            // top-right
            (self.max, Direction::TopRight)
        } else if !x_neg && y_neg {
            // bottom-right
            (Vector2::new(self.max.x, self.min.y), Direction::BottomRight)
        } else {
            // top-left
            (Vector2::new(self.min.x, self.max.y), Direction::TopLeft)
        }
    }

    fn get_nearest_dir(&self) -> Direction {
        let x_neg = self.min.x < 0_f64;
        let y_neg = self.min.y < 0_f64;

        if x_neg && y_neg {
            // bottom-left
            Direction::TopRight
        } else if !x_neg && !y_neg {
            // top-right
            Direction::BottomLeft
        } else if !x_neg && y_neg {
            // bottom-right
            Direction::TopLeft
        } else {
            // top-left
            Direction::BottomRight
        }
    }

    fn get_nearest_vertex(&self) -> Vector2<f64> {
        let x_neg = self.min.x < 0_f64;
        let y_neg = self.min.y < 0_f64;

        if x_neg && y_neg {
            // bottom-left
            self.max
        } else if !x_neg && !y_neg {
            // top-right
            self.min
        } else if !x_neg && y_neg {
            // bottom-right
            Vector2::new(self.min.x, self.max.y)
        } else {
            // top-left
            Vector2::new(self.max.x, self.min.y)
        }
    }

    fn get_vertex(&self, d: Direction) -> Vector2<f64> {
        match d {
            Direction::BottomLeft => self.min,
            Direction::BottomRight => Vector2::new(self.max.x, self.min.y),
            Direction::TopLeft => Vector2::new(self.min.x, self.max.y),
            Direction::TopRight => self.max,
        }
    }

    pub fn add(
        &self,
        vertices: &mut Vec<Vector2<f64>>,
        idx: &mut Vec<u16>,
        dir_farthest_vertex: Direction,
    ) {
        let bl = self.get_vertex(Direction::BottomLeft);
        let br = self.get_vertex(Direction::BottomRight);
        let tr = self.get_vertex(Direction::TopRight);
        let tl = self.get_vertex(Direction::TopLeft);

        let off_idx = vertices.len() as u16;

        // push the 4 vertices
        vertices.push(bl);
        vertices.push(br);
        vertices.push(tr);
        vertices.push(tl);

        match dir_farthest_vertex {
            Direction::TopLeft | Direction::BottomRight => {
                // push the 6 indexes
                idx.extend(
                    [
                        off_idx,
                        off_idx + 1,
                        off_idx + 3,
                        off_idx + 1,
                        off_idx + 2,
                        off_idx + 3,
                    ]
                    .iter(),
                );
                // LINES drawing
                /*idx.extend([
                    off_idx,
                    off_idx + 1,
                    off_idx + 1,
                    off_idx + 3,
                    off_idx + 3,
                    off_idx,

                    off_idx + 1,
                    off_idx + 2,
                    off_idx + 2,
                    off_idx + 3,
                    off_idx + 3,
                    off_idx + 1,
                ].iter());*/
            }
            _ => {
                // push the 6 indexes
                idx.extend(
                    [
                        off_idx,
                        off_idx + 1,
                        off_idx + 2,
                        off_idx,
                        off_idx + 2,
                        off_idx + 3,
                    ]
                    .iter(),
                );
                // LINES drawing
                /*idx.extend([
                    off_idx,
                    off_idx + 1,
                    off_idx + 1,
                    off_idx + 2,
                    off_idx + 2,
                    off_idx,

                    off_idx,
                    off_idx + 2,
                    off_idx + 2,
                    off_idx + 3,
                    off_idx + 3,
                    off_idx,
                ].iter());*/
            }
        }
    }

    fn add_triangle(
        &self,
        p: &[Vector2<f64>; 3],
        vertices: &mut Vec<Vector2<f64>>,
        idx: &mut Vec<u16>,
    ) {
        let off_idx = vertices.len() as u16;

        // push the 4 vertices
        vertices.push(p[0]);
        vertices.push(p[1]);
        vertices.push(p[2]);

        // push the 6 indexes
        idx.extend([off_idx, off_idx + 1, off_idx + 2].iter());
        // LINES drawing
        /*idx.extend([
            off_idx,
            off_idx + 1,
            off_idx + 1,
            off_idx + 2,
            off_idx + 2,
            off_idx,
        ].iter());*/
    }

    pub fn get_child(&self, d: Direction) -> Self {
        let center = (self.min + self.max) * 0.5_f64;
        let (min, max) = match d {
            Direction::BottomLeft => {
                let min = self.min;
                let max = center;
                (min, max)
            }
            Direction::BottomRight => {
                let min = Vector2::new(center.x, self.min.y);
                let max = Vector2::new(self.max.x, center.y);
                (min, max)
            }
            Direction::TopLeft => {
                let min = Vector2::new(self.min.x, center.y);
                let max = Vector2::new(center.x, self.max.y);
                (min, max)
            }
            Direction::TopRight => {
                let min = center;
                let max = self.max;
                (min, max)
            }
        };

        Face { min, max }
    }
}

fn recursive_triangulation<P: Projection>(
    face: &Face,
    vertices: &mut Vec<Vector2<f64>>,
    idx: &mut Vec<u16>,
    depth: u8,
    first: &mut bool,
) {
    let (farthest_vertex, dir_farthest_vertex) = face.get_farthest_vertex();
    if depth > 0 {
        // Look if the square is totally included in the projection
        if P::is_included_inside_projection(&farthest_vertex) && depth < 1 {
            face.add(vertices, idx, dir_farthest_vertex);
        // If not check if is traversed by the border of the projection
        } else {
            let nearest_vertex = face.get_nearest_vertex();
            if P::is_included_inside_projection(&nearest_vertex) {
                // The nearest is included and the farthest not,
                // so let's subdivide the cell
                // subdivision
                // top-left
                recursive_triangulation::<P>(
                    &face.get_child(Direction::TopLeft),
                    vertices,
                    idx,
                    depth - 1,
                    first,
                );
                // top-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::TopRight),
                    vertices,
                    idx,
                    depth - 1,
                    first,
                );
                // bottom-left
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomLeft),
                    vertices,
                    idx,
                    depth - 1,
                    first,
                );
                // bottom-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomRight),
                    vertices,
                    idx,
                    depth - 1,
                    first,
                );
            }
        }
    } else {
        if P::is_included_inside_projection(&farthest_vertex) {
            face.add(vertices, idx, dir_farthest_vertex);

            return;
        }

        match face.get_nearest_dir() {
            // x < 0 && y < 0
            Direction::TopRight => {
                let tr = face.get_vertex(Direction::TopRight);
                if !P::is_included_inside_projection(&tr) {
                    return;
                }

                let tl = face.get_vertex(Direction::TopLeft);
                let br = face.get_vertex(Direction::BottomRight);

                if !P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br)
                {
                    let (x1, _) = P::solve_along_abscissa(tl.y).unwrap();
                    let (y1, _) = P::solve_along_ordinate(br.x).unwrap();

                    let tl_r = Vector2::new(x1, tl.y);
                    let br_r = Vector2::new(br.x, y1);
                    face.add_triangle(&[tl_r, br_r, tr], vertices, idx);
                } else if P::is_included_inside_projection(&tl)
                    && !P::is_included_inside_projection(&br)
                {
                    let (y1, _) = P::solve_along_ordinate(br.x).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tl.x).unwrap();

                    face.add_triangle(&[tl, Vector2::new(tl.x, y2), tr], vertices, idx);
                    face.add_triangle(
                        &[Vector2::new(tl.x, y2), Vector2::new(br.x, y1), tr],
                        vertices,
                        idx,
                    );
                } else if !P::is_included_inside_projection(&tl)
                    && P::is_included_inside_projection(&br)
                {
                    let (x1, _) = P::solve_along_abscissa(tr.y).unwrap();
                    let (x2, _) = P::solve_along_abscissa(br.y).unwrap();

                    face.add_triangle(
                        &[Vector2::new(x1, tr.y), Vector2::new(x2, br.y), tr],
                        vertices,
                        idx,
                    );
                    face.add_triangle(&[Vector2::new(x2, br.y), br, tr], vertices, idx);
                } else if P::is_included_inside_projection(&tl)
                    && P::is_included_inside_projection(&br)
                {
                    let (y1, _) = P::solve_along_ordinate(tl.x).unwrap();
                    let (x2, _) = P::solve_along_abscissa(br.y).unwrap();

                    let u = Vector2::new(tl.x, y1);
                    let v = Vector2::new(x2, br.y);
                    face.add_triangle(&[tl, u, tr], vertices, idx);
                    face.add_triangle(&[tr, u, v], vertices, idx);
                    face.add_triangle(&[v, br, tr], vertices, idx);
                }
            }
            // x > 0 && y > 0
            Direction::BottomLeft => {
                let bl = face.get_vertex(Direction::BottomLeft);
                if !P::is_included_inside_projection(&bl) {
                    return;
                }

                let tl = face.get_vertex(Direction::TopLeft);
                let br = face.get_vertex(Direction::BottomRight);

                if !P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br)
                {
                    let (_, x2) = P::solve_along_abscissa(br.y).unwrap();
                    let (_, y2) = P::solve_along_ordinate(tl.x).unwrap();

                    let u = Vector2::new(x2, br.y);
                    let v = Vector2::new(tl.x, y2);
                    face.add_triangle(&[u, v, bl], vertices, idx);
                } else if P::is_included_inside_projection(&tl)
                    && !P::is_included_inside_projection(&br)
                {
                    let (_, x1) = P::solve_along_abscissa(tl.y).unwrap();
                    let (_, x2) = P::solve_along_abscissa(br.y).unwrap();

                    let u = Vector2::new(x1, tl.y);
                    let v = Vector2::new(x2, br.y);

                    face.add_triangle(&[tl, bl, u], vertices, idx);
                    face.add_triangle(&[u, bl, v], vertices, idx);
                } else if !P::is_included_inside_projection(&tl)
                    && P::is_included_inside_projection(&br)
                {
                    let (_, y1) = P::solve_along_ordinate(tl.x).unwrap();
                    let (_, y2) = P::solve_along_ordinate(br.x).unwrap();

                    let u = Vector2::new(tl.x, y1);
                    let v = Vector2::new(br.x, y2);

                    face.add_triangle(&[u, bl, v], vertices, idx);
                    face.add_triangle(&[v, bl, br], vertices, idx);
                } else if P::is_included_inside_projection(&tl)
                    && P::is_included_inside_projection(&br)
                {
                    let (_, x1) = P::solve_along_abscissa(tl.y).unwrap();
                    let (_, y2) = P::solve_along_ordinate(br.x).unwrap();

                    let u = Vector2::new(x1, tl.y);
                    let v = Vector2::new(br.x, y2);

                    face.add_triangle(&[tl, bl, u], vertices, idx);
                    face.add_triangle(&[v, u, bl], vertices, idx);
                    face.add_triangle(&[v, bl, br], vertices, idx);
                }
            }
            // x > 0 && y < 0
            Direction::TopLeft => {
                let tl = face.get_vertex(Direction::TopLeft);
                if !P::is_included_inside_projection(&tl) {
                    return;
                }

                let tr = face.get_vertex(Direction::TopRight);
                let bl = face.get_vertex(Direction::BottomLeft);

                if !P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr)
                {
                    let (y1, _) = P::solve_along_ordinate(bl.x).unwrap();
                    let (_, x2) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(x2, tr.y);
                    face.add_triangle(&[u, v, tl], vertices, idx);
                } else if P::is_included_inside_projection(&bl)
                    && !P::is_included_inside_projection(&tr)
                {
                    let (_, x1) = P::solve_along_abscissa(bl.y).unwrap();
                    let (_, x2) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(&[tl, bl, u], vertices, idx);
                    face.add_triangle(&[tl, u, v], vertices, idx);
                } else if !P::is_included_inside_projection(&bl)
                    && P::is_included_inside_projection(&tr)
                {
                    let (y1, _) = P::solve_along_ordinate(bl.x).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(&[tl, u, v], vertices, idx);
                    face.add_triangle(&[tl, v, tr], vertices, idx);
                } else if P::is_included_inside_projection(&bl)
                    && P::is_included_inside_projection(&tr)
                {
                    let (_, x1) = P::solve_along_abscissa(bl.y).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(&[tl, bl, u], vertices, idx);
                    face.add_triangle(&[tl, u, v], vertices, idx);
                    face.add_triangle(&[tl, v, tr], vertices, idx);
                }
            }
            // x < 0 && y > 0
            Direction::BottomRight => {
                let br = face.get_vertex(Direction::BottomRight);
                if !P::is_included_inside_projection(&br) {
                    return;
                }

                let tr = face.get_vertex(Direction::TopRight);
                let bl = face.get_vertex(Direction::BottomLeft);

                if !P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr)
                {
                    let (x2, _) = P::solve_along_abscissa(bl.y).unwrap();
                    let (_, y1) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(x2, bl.y);
                    let v = Vector2::new(tr.x, y1);
                    face.add_triangle(&[u, br, v], vertices, idx);
                } else if P::is_included_inside_projection(&bl)
                    && !P::is_included_inside_projection(&tr)
                {
                    let (_, y1) = P::solve_along_ordinate(bl.x).unwrap();
                    let (_, y2) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(&[u, bl, br], vertices, idx);
                    face.add_triangle(&[u, br, v], vertices, idx);
                } else if !P::is_included_inside_projection(&bl)
                    && P::is_included_inside_projection(&tr)
                {
                    let (x1, _) = P::solve_along_abscissa(bl.y).unwrap();
                    let (x2, _) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(&[u, br, v], vertices, idx);
                    face.add_triangle(&[v, br, tr], vertices, idx);
                } else if P::is_included_inside_projection(&bl)
                    && P::is_included_inside_projection(&tr)
                {
                    let (_, y1) = P::solve_along_ordinate(bl.x).unwrap();
                    let (x2, _) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(&[bl, br, u], vertices, idx);
                    face.add_triangle(&[u, br, v], vertices, idx);
                    face.add_triangle(&[v, br, tr], vertices, idx);
                }
            }
        }
    }
}
