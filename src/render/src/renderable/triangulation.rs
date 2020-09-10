use cgmath::Vector2;

pub struct Triangulation {
    vertices: Vec<Vector2<f32>>,
    idx: Vec<u16>,
}

struct Face {
    min: Vector2<f32>,
    max: Vector2<f32>,
}

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

    fn get_farthest_vertex(&self) -> Vector2<f32> {
        let x_neg = self.min.x < 0_f32;
        let y_neg = self.min.y < 0_f32;

        if x_neg && y_neg {
            // bottom-left
            self.min
        } else if !x_neg && !y_neg {
            // top-right
            self.max
        } else if !x_neg && y_neg {
            // bottom-right
            Vector2::new(self.max.x, self.min.y)
        } else {
            // top-left
            Vector2::new(self.min.x, self.max.y)
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

    pub fn add(&self, vertices: &mut Vec<Vector2<f32>>, idx: &mut Vec<u16>) {
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

        // push the 6 indexes
        idx.extend([
            off_idx,
            off_idx + 1,
            off_idx + 2,

            off_idx,
            off_idx + 2,
            off_idx + 3
        ].iter());
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
    depth: u8
) {
    if depth > 0 {
        // Look if the square is totally included in the projection
        let farthest_vertex = face.get_farthest_vertex();
        if P::is_included_inside_projection(&farthest_vertex) && depth < 5 {
            face.add(vertices, idx);
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
                    depth - 1
                );
                // top-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::TopRight),
                    vertices,
                    idx,
                    depth - 1
                );
                // bottom-left
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomLeft),
                    vertices,
                    idx,
                    depth - 1
                );
                // bottom-right
                recursive_triangulation::<P>(
                    &face.get_child(Direction::BottomRight),
                    vertices,
                    idx,
                    depth - 1
                );
            }
        } 
    } else {
        // Leaf
        // TODO
    }
}


use crate::renderable::projection::Projection;
impl Triangulation {
    pub fn new<P: Projection>() -> Triangulation {
        let (mut vertices, mut idx) = (Vec::new(), Vec::new());

        let root = Face::new(Vector2::new(-1_f32, -1_f32), Vector2::new(1_f32, 1_f32));
        let children = root.split();

        let depth = 8;
        recursive_triangulation::<P>(&children[0], &mut vertices, &mut idx, depth);
        recursive_triangulation::<P>(&children[1], &mut vertices, &mut idx, depth);
        recursive_triangulation::<P>(&children[2], &mut vertices, &mut idx, depth);
        recursive_triangulation::<P>(&children[3], &mut vertices, &mut idx, depth);

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

