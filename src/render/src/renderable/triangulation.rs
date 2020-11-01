use cgmath::Vector2;

pub struct Triangulation {
    vertices: Vec<Vector2<f32>>,
    idx: Vec<u16>,
}

struct Face {
    min: Vector2<f32>,
    max: Vector2<f32>,
}

#[derive(Clone, Copy)]
pub enum Direction {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}
impl Face {
    fn new(min: Vector2<f32>, max: Vector2<f32>) -> Face {
        Face {
            min, 
            max
        }
    }

    fn split(self) -> [Face; 4] {
        let bl = self.get_child(Direction::BottomLeft);
        let br = self.get_child(Direction::BottomRight);
        let tr = self.get_child(Direction::TopRight);
        let tl = self.get_child(Direction::TopLeft);

        [bl, br, tr, tl]
    }

    fn get_farthest_vertex(&self) -> (Vector2<f32>, Direction) {
        let x_neg = self.min.x < 0_f32;
        let y_neg = self.min.y < 0_f32;

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
        let x_neg = self.min.x < 0_f32;
        let y_neg = self.min.y < 0_f32;

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

    fn get_nearest_vertex(&self) -> Vector2<f32> {
        let x_neg = self.min.x < 0_f32;
        let y_neg = self.min.y < 0_f32;

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

    fn get_vertex(&self, d: Direction) -> Vector2<f32> {
        match d {
            Direction::BottomLeft => self.min,
            Direction::BottomRight => Vector2::new(self.max.x, self.min.y),
            Direction::TopLeft => Vector2::new(self.min.x, self.max.y),
            Direction::TopRight => self.max,
        }
    }

    pub fn add(&self, vertices: &mut Vec<Vector2<f32>>, idx: &mut Vec<u16>, dir_farthest_vertex: Direction) {
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
                idx.extend([
                    off_idx,
                    off_idx + 1,
                    off_idx + 3,

                    off_idx + 1,
                    off_idx + 2,
                    off_idx + 3
                ].iter());
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
            },
            _ => {
                // push the 6 indexes
                idx.extend([
                    off_idx,
                    off_idx + 1,
                    off_idx + 2,

                    off_idx,
                    off_idx + 2,
                    off_idx + 3
                ].iter());
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

    fn add_triangle(&self, p: &[Vector2<f32>; 3], vertices: &mut Vec<Vector2<f32>>, idx: &mut Vec<u16>) {
        let off_idx = vertices.len() as u16;

        // push the 4 vertices
        vertices.push(p[0]);
        vertices.push(p[1]);
        vertices.push(p[2]);

        // push the 6 indexes
        idx.extend([
            off_idx,
            off_idx + 1,
            off_idx + 2,
        ].iter());
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
        let center = (self.min + self.max) * 0.5_f32;
        let (min, max) = match d {
            Direction::BottomLeft => {
                let min = self.min;
                let max = center;
                (min, max)
            },
            Direction::BottomRight => {
                let min = Vector2::new(center.x, self.min.y);
                let max = Vector2::new(self.max.x, center.y);
                (min, max)
            },
            Direction::TopLeft => {
                let min = Vector2::new(self.min.x, center.y);
                let max = Vector2::new(center.x, self.max.y);
                (min, max)
            },
            Direction::TopRight => {
                let min = center;
                let max = self.max;
                (min, max)
            },
        };

        Face {
            min, 
            max
        }
    }
}

fn recursive_triangulation<P: Projection>(
    face: &Face,
    vertices: &mut Vec<Vector2<f32>>,
    idx: &mut Vec<u16>,
    depth: u8,
    first: &mut bool,
) {
    let (farthest_vertex, dir_farthest_vertex) = face.get_farthest_vertex();
    if depth > 0 {
        // Look if the square is totally included in the projection
        if P::is_included_inside_projection(&farthest_vertex) && depth < 2 {
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
                    first
                );
                // top-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::TopRight),
                    vertices,
                    idx,
                    depth - 1,
                    first
                );
                // bottom-left
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomLeft),
                    vertices,
                    idx,
                    depth - 1,
                    first
                );
                // bottom-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomRight),
                    vertices,
                    idx,
                    depth - 1,
                    first
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

                if !P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br) {
                    let (x1, _) = P::solve_along_abscissa(tl.y).unwrap();
                    let (y1, _) = P::solve_along_ordinate(br.x).unwrap();

                    let tl_r = Vector2::new(x1, tl.y);
                    let br_r = Vector2::new(br.x, y1);
                    face.add_triangle(
                        &[
                            tl_r,
                            br_r,
                            tr
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br) {
                    let (y1, _) = P::solve_along_ordinate(br.x).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tl.x).unwrap();

                    face.add_triangle(
                        &[
                            tl,
                            Vector2::new(tl.x, y2),
                            tr
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            Vector2::new(tl.x, y2),
                            Vector2::new(br.x, y1),
                            tr
                        ],
                        vertices,
                        idx
                    );
                } else if !P::is_included_inside_projection(&tl) && P::is_included_inside_projection(&br) {
                    let (x1, _) = P::solve_along_abscissa(tr.y).unwrap();
                    let (x2, _) = P::solve_along_abscissa(br.y).unwrap();

                    face.add_triangle(
                        &[
                            Vector2::new(x1, tr.y),
                            Vector2::new(x2, br.y),
                            tr
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            Vector2::new(x2, br.y),
                            br,
                            tr
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&tl) && P::is_included_inside_projection(&br) {
                    let (y1, _) = P::solve_along_ordinate(tl.x).unwrap();
                    let (x2, _) = P::solve_along_abscissa(br.y).unwrap();

                    let u = Vector2::new(tl.x, y1);
                    let v = Vector2::new(x2, br.y);
                    face.add_triangle(
                        &[
                            tl,
                            u,
                            tr
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            tr,
                            u,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            br,
                            tr
                        ],
                        vertices,
                        idx
                    );
                }
            },
            // x > 0 && y > 0
            Direction::BottomLeft => {
                let bl = face.get_vertex(Direction::BottomLeft);
                if !P::is_included_inside_projection(&bl) {
                    return;
                }

                let tl = face.get_vertex(Direction::TopLeft);
                let br = face.get_vertex(Direction::BottomRight);

                if !P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br) {
                    let (_, x2) = P::solve_along_abscissa(br.y).unwrap();
                    let (_, y2) = P::solve_along_ordinate(tl.x).unwrap();

                    let u = Vector2::new(x2, br.y);
                    let v = Vector2::new(tl.x, y2);
                    face.add_triangle(
                        &[
                            u,
                            v,
                            bl
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&tl) && !P::is_included_inside_projection(&br) {
                    let (_, x1) = P::solve_along_abscissa(tl.y).unwrap();
                    let (_, x2) = P::solve_along_abscissa(br.y).unwrap();

                    let u = Vector2::new(x1, tl.y);
                    let v = Vector2::new(x2, br.y);

                    face.add_triangle(
                        &[
                            tl,
                            bl,
                            u,
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            u,
                            bl,
                            v
                        ],
                        vertices,
                        idx
                    );
                } else if !P::is_included_inside_projection(&tl) && P::is_included_inside_projection(&br) {
                    let (_, y1) = P::solve_along_ordinate(tl.x).unwrap();
                    let (_, y2) = P::solve_along_ordinate(br.x).unwrap();

                    let u = Vector2::new(tl.x, y1);
                    let v = Vector2::new(br.x, y2);

                    face.add_triangle(
                        &[
                            u,
                            bl,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            bl,
                            br
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&tl) && P::is_included_inside_projection(&br) {
                    let (_, x1) = P::solve_along_abscissa(tl.y).unwrap();
                    let (_, y2) = P::solve_along_ordinate(br.x).unwrap();

                    let u = Vector2::new(x1, tl.y);
                    let v = Vector2::new(br.x, y2);

                    face.add_triangle(
                        &[
                            tl,
                            bl,
                            u
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            u,
                            bl
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            bl,
                            br
                        ],
                        vertices,
                        idx
                    );
                }
            },
            // x > 0 && y < 0
            Direction::TopLeft => {
                let tl = face.get_vertex(Direction::TopLeft);
                if !P::is_included_inside_projection(&tl) {
                    return;
                }

                let tr = face.get_vertex(Direction::TopRight);
                let bl = face.get_vertex(Direction::BottomLeft);

                if !P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr) {
                    let (y1, _) = P::solve_along_ordinate(bl.x).unwrap();
                    let (_, x2) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(x2, tr.y);
                    face.add_triangle(
                        &[
                            u,
                            v,
                            tl
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr) {
                    let (_, x1) = P::solve_along_abscissa(bl.y).unwrap();
                    let (_, x2) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(
                        &[
                            tl,
                            bl,
                            u,
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            tl,
                            u,
                            v
                        ],
                        vertices,
                        idx
                    );
                } else if !P::is_included_inside_projection(&bl) && P::is_included_inside_projection(&tr) {
                    let (y1, _) = P::solve_along_ordinate(bl.x).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(
                        &[
                            tl,
                            u,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            tl,
                            v,
                            tr
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&bl) && P::is_included_inside_projection(&tr) {
                    let (_, x1) = P::solve_along_abscissa(bl.y).unwrap();
                    let (y2, _) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(
                        &[
                            tl,
                            bl,
                            u
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            tl,
                            u,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            tl,
                            v,
                            tr
                        ],
                        vertices,
                        idx
                    );
                }
            },
            // x < 0 && y > 0
            Direction::BottomRight => {
                let br = face.get_vertex(Direction::BottomRight);
                if !P::is_included_inside_projection(&br) {
                    return;
                }

                let tr = face.get_vertex(Direction::TopRight);
                let bl = face.get_vertex(Direction::BottomLeft);

                if !P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr) {
                    let (x2, _) = P::solve_along_abscissa(bl.y).unwrap();
                    let (_, y1) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(x2, bl.y);
                    let v = Vector2::new(tr.x, y1);
                    face.add_triangle(
                        &[
                            u,
                            br,
                            v
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&bl) && !P::is_included_inside_projection(&tr) {
                    let (_, y1) = P::solve_along_ordinate(bl.x).unwrap();
                    let (_, y2) = P::solve_along_ordinate(tr.x).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(tr.x, y2);

                    face.add_triangle(
                        &[
                            u,
                            bl,
                            br,
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            u,
                            br,
                            v
                        ],
                        vertices,
                        idx
                    );
                } else if !P::is_included_inside_projection(&bl) && P::is_included_inside_projection(&tr) {
                    let (x1, _) = P::solve_along_abscissa(bl.y).unwrap();
                    let (x2, _) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(x1, bl.y);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(
                        &[
                            u,
                            br,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            br,
                            tr
                        ],
                        vertices,
                        idx
                    );
                } else if P::is_included_inside_projection(&bl) && P::is_included_inside_projection(&tr) {
                    let (_, y1) = P::solve_along_ordinate(bl.x).unwrap();
                    let (x2, _) = P::solve_along_abscissa(tr.y).unwrap();

                    let u = Vector2::new(bl.x, y1);
                    let v = Vector2::new(x2, tr.y);

                    face.add_triangle(
                        &[
                            bl,
                            br,
                            u
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            u,
                            br,
                            v
                        ],
                        vertices,
                        idx
                    );
                    face.add_triangle(
                        &[
                            v,
                            br,
                            tr
                        ],
                        vertices,
                        idx
                    );
                }
            },
            _ => ()
        }
    }
}


use crate::renderable::projection::Projection;
impl Triangulation {
    pub fn new<P: Projection>() -> Triangulation {
        let (mut vertices, mut idx) = (Vec::new(), Vec::new());

        let root = Face::new(Vector2::new(-1_f32, -1_f32), Vector2::new(1_f32, 1_f32));
        let children = root.split();

        let depth = 4;
        let mut first = false;
        recursive_triangulation::<P>(&children[0], &mut vertices, &mut idx, depth, &mut first);
        recursive_triangulation::<P>(&children[1], &mut vertices, &mut idx, depth, &mut first);
        recursive_triangulation::<P>(&children[2], &mut vertices, &mut idx, depth, &mut first);
        recursive_triangulation::<P>(&children[3], &mut vertices, &mut idx, depth, &mut first);

        Triangulation {
            vertices,
            idx
        }
    }

    pub fn vertices(&self) -> &Vec<Vector2<f32>> {
        &self.vertices
    }

    pub fn idx(&self) -> &Vec<u16> {
        &self.idx
    }
}

impl From<Triangulation> for (Vec<Vector2<f32>>, Vec<u16>) {
    fn from(t: Triangulation) -> Self {
        (t.vertices, t.idx)
    }
}