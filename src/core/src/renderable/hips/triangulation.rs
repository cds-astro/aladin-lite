use crate::math::vector::NormedVector2;
use cgmath::Vector2;

pub struct Triangulation {
    pub vertices: Vec<Vector2<f64>>,
    pub idx: Vec<u16>,
}

use crate::math::projection::domain::sdf::{self, ProjDef, ProjDefType};
impl Triangulation {
    pub(super) fn build(proj_def: &ProjDefType) -> Triangulation {
        let (mut vertices, mut idx) = (Vec::new(), Vec::new());

        // get the validity domain
        let root = Face::new(Vector2::new(-1_f64, -1_f64), Vector2::new(1_f64, 1_f64));
        let children = root.split(2);

        let depth = 3;
        for child in children {
            recursive_triangulation(&child, &mut vertices, &mut idx, depth, proj_def);
        }

        Triangulation { vertices, idx }
    }
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

    fn split_rec(self, sub: usize, faces: &mut Vec<Face>) {
        let bl = self.get_child(Direction::BottomLeft);
        let br = self.get_child(Direction::BottomRight);
        let tr = self.get_child(Direction::TopRight);
        let tl = self.get_child(Direction::TopLeft);

        if sub == 0 {
            faces.extend([bl, br, tr, tl]);
        } else {
            bl.split_rec(sub - 1, faces);
            br.split_rec(sub - 1, faces);
            tr.split_rec(sub - 1, faces);
            tl.split_rec(sub - 1, faces);
        }
    }

    fn split(self, sub: usize) -> Vec<Face> {
        let mut faces = vec![];
        self.split_rec(sub, &mut faces);

        faces
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
        off_idx: u16,
        //dir_farthest_vertex: Direction,
    ) -> ([Vector2<f64>; 4], [u16; 6]) {
        let bl = self.get_vertex(Direction::BottomLeft);
        let br = self.get_vertex(Direction::BottomRight);
        let tr = self.get_vertex(Direction::TopRight);
        let tl = self.get_vertex(Direction::TopLeft);

        // push the 4 vertices
        let vertices = [bl, br, tr, tl];
        /*let idx = match dir_farthest_vertex {
            Direction::TopLeft | Direction::BottomRight => {
                // push the 6 indexes
                [
                    off_idx,
                    off_idx + 1,
                    off_idx + 3,
                    off_idx + 1,
                    off_idx + 2,
                    off_idx + 3,
                ]
            }
            _ => {
                // push the 6 indexes
                [
                    off_idx,
                    off_idx + 1,
                    off_idx + 2,
                    off_idx,
                    off_idx + 2,
                    off_idx + 3,
                ]
            }
        };*/
        let idx = [
            off_idx,
            off_idx + 1,
            off_idx + 3,
            off_idx + 1,
            off_idx + 2,
            off_idx + 3,
        ];

        (vertices, idx)
    }

    fn add_triangle(
        &self,
        p: &[Vector2<f64>; 3],
        vertices: &mut Vec<Vector2<f64>>,
        idx: &mut Vec<u16>,
    ) {
        let off_idx = vertices.len() as u16;

        // push the 4 vertices
        vertices.extend(p);

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

fn recursive_triangulation(
    face: &Face,
    vertices: &mut Vec<Vector2<f64>>,
    idx: &mut Vec<u16>,
    depth: u8,
    proj_def: &ProjDefType,
) {
    //let (farthest_vertex, dir_farthest_vertex) = face.get_farthest_vertex();

    // Look for which vertices lies inside the region
    let tl = face.get_vertex(Direction::TopLeft);
    let tr = face.get_vertex(Direction::TopRight);
    let bl = face.get_vertex(Direction::BottomLeft);
    let br = face.get_vertex(Direction::BottomRight);

    let tl_in = proj_def.is_in(&tl);
    let tr_in = proj_def.is_in(&tr);
    let bl_in = proj_def.is_in(&bl);
    let br_in = proj_def.is_in(&br);

    if depth > 0 {
        //let nearest_vertex = face.get_nearest_vertex();
        let at_least_one_in = tl_in || tr_in || bl_in || br_in;
        if at_least_one_in {
            // so let's subdivide the cell
            // subdivision
            // top-left
            recursive_triangulation(
                &face.get_child(Direction::TopLeft),
                vertices,
                idx,
                depth - 1,
                proj_def,
            );
            // top-right
            recursive_triangulation(
                &face.get_child(Direction::TopRight),
                vertices,
                idx,
                depth - 1,
                proj_def,
            );
            // bottom-left
            recursive_triangulation(
                &face.get_child(Direction::BottomLeft),
                vertices,
                idx,
                depth - 1,
                proj_def,
            );
            // bottom-right
            recursive_triangulation(
                &face.get_child(Direction::BottomRight),
                vertices,
                idx,
                depth - 1,
                proj_def,
            );
        }
    } else {
        // Final case
        let all_in = tl_in && tr_in && bl_in && br_in;

        if all_in {
            let off_idx = vertices.len() as u16;

            let (v, i) = face.add(off_idx);
            vertices.extend(&v);
            idx.extend(&i);

            return;
        }

        const D_TO_EAST: NormedVector2 = unsafe { NormedVector2::new_unsafe(1.0, 0.0) };
        const D_TO_WEST: NormedVector2 = unsafe { NormedVector2::new_unsafe(-1.0, 0.0) };
        const D_TO_NORTH: NormedVector2 = unsafe { NormedVector2::new_unsafe(0.0, 1.0) };
        const D_TO_SOUTH: NormedVector2 = unsafe { NormedVector2::new_unsafe(0.0, -1.0) };

        match (bl_in, br_in, tr_in, tl_in) {
            // 0 VERTEX case
            (false, false, false, false) => {}
            // 1 VERTEX cases
            // 1 triangle to plot
            // (x bl, br, tr, tl)
            (true, false, false, false) => {
                let u =
                    sdf::ray_marching(&tl, &D_TO_SOUTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&br, &D_TO_WEST, proj_def).expect("should intersect domain");

                face.add_triangle(&[bl, v, u], vertices, idx);
            }
            // (bl, x br, tr, tl)
            (false, true, false, false) => {
                let u =
                    sdf::ray_marching(&bl, &D_TO_EAST, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tr, &D_TO_SOUTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[br, v, u], vertices, idx);
            }
            // (bl, br, x tr, tl)
            (false, false, true, false) => {
                let v =
                    sdf::ray_marching(&br, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let u =
                    sdf::ray_marching(&tl, &D_TO_EAST, proj_def).expect("should intersect domain");

                face.add_triangle(&[tr, u, v], vertices, idx);
            }
            // (bl, br, tr, x tl)
            (false, false, false, true) => {
                let u =
                    sdf::ray_marching(&bl, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tr, &D_TO_WEST, proj_def).expect("should intersect domain");

                face.add_triangle(&[tl, u, v], vertices, idx);
            }
            // 2 VERTICES cases
            // (bl, x br, tr, x tl)
            (false, true, false, true) => {
                let u =
                    sdf::ray_marching(&bl, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&bl, &D_TO_EAST, proj_def).expect("should intersect domain");
                let w =
                    sdf::ray_marching(&tr, &D_TO_WEST, proj_def).expect("should intersect domain");
                let x =
                    sdf::ray_marching(&tr, &D_TO_SOUTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[u, w, tl], vertices, idx);
                face.add_triangle(&[u, v, w], vertices, idx);
                face.add_triangle(&[v, x, w], vertices, idx);
                face.add_triangle(&[v, br, x], vertices, idx);
            }
            // (x bl, br, x tr, tl)
            (true, false, true, false) => {
                let u =
                    sdf::ray_marching(&tl, &D_TO_SOUTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tl, &D_TO_EAST, proj_def).expect("should intersect domain");
                let w =
                    sdf::ray_marching(&br, &D_TO_WEST, proj_def).expect("should intersect domain");
                let x =
                    sdf::ray_marching(&br, &D_TO_NORTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[bl, w, u], vertices, idx);
                face.add_triangle(&[w, x, u], vertices, idx);
                face.add_triangle(&[x, v, u], vertices, idx);
                face.add_triangle(&[x, tr, v], vertices, idx);
            }
            // (bl, br, x tr, x tl)
            (false, false, true, true) => {
                let u =
                    sdf::ray_marching(&bl, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&br, &D_TO_NORTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[v, tl, u], vertices, idx);
                face.add_triangle(&[v, tr, tl], vertices, idx);
            }
            // (x bl, x br, tr, tl)
            (true, true, false, false) => {
                let u =
                    sdf::ray_marching(&tl, &D_TO_SOUTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tr, &D_TO_SOUTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[u, bl, v], vertices, idx);
                face.add_triangle(&[v, bl, br], vertices, idx);
            }
            // (x bl, br, tr, x tl)
            (true, false, false, true) => {
                let u =
                    sdf::ray_marching(&tr, &D_TO_WEST, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&br, &D_TO_WEST, proj_def).expect("should intersect domain");

                face.add_triangle(&[bl, u, tl], vertices, idx);
                face.add_triangle(&[u, bl, v], vertices, idx);
            }
            // (bl, x br, x tr, tl)
            (false, true, true, false) => {
                let u =
                    sdf::ray_marching(&tl, &D_TO_EAST, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&bl, &D_TO_EAST, proj_def).expect("should intersect domain");

                face.add_triangle(&[br, tr, u], vertices, idx);
                face.add_triangle(&[br, u, v], vertices, idx);
            }
            // 3 VERTICES cases
            // (x bl, x br, x tr, tl)
            (true, true, true, false) => {
                let u =
                    sdf::ray_marching(&tl, &D_TO_EAST, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tl, &D_TO_SOUTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[u, v, bl], vertices, idx);
                face.add_triangle(&[u, bl, tr], vertices, idx);
                face.add_triangle(&[tr, bl, br], vertices, idx);
            }
            // (bl, x br, x tr, x tl)
            (false, true, true, true) => {
                let u =
                    sdf::ray_marching(&bl, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&bl, &D_TO_EAST, proj_def).expect("should intersect domain");

                face.add_triangle(&[tl, u, tr], vertices, idx);
                face.add_triangle(&[tr, u, v], vertices, idx);
                face.add_triangle(&[v, br, tr], vertices, idx);
            }
            // (x bl, br, x tr, x tl)
            (true, false, true, true) => {
                let u =
                    sdf::ray_marching(&br, &D_TO_NORTH, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&br, &D_TO_WEST, proj_def).expect("should intersect domain");

                face.add_triangle(&[tl, bl, v], vertices, idx);
                face.add_triangle(&[u, tl, v], vertices, idx);
                face.add_triangle(&[tr, tl, u], vertices, idx);
            }
            // (x bl, x br, tr, x tl)
            (true, true, false, true) => {
                let u =
                    sdf::ray_marching(&tr, &D_TO_WEST, proj_def).expect("should intersect domain");
                let v =
                    sdf::ray_marching(&tr, &D_TO_SOUTH, proj_def).expect("should intersect domain");

                face.add_triangle(&[bl, u, tl], vertices, idx);
                face.add_triangle(&[bl, v, u], vertices, idx);
                face.add_triangle(&[bl, br, v], vertices, idx);
            }
            // full case must not happen
            _ => unreachable!(),
        }
    }
}
