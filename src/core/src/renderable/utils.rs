use std::ops::RangeInclusive;
use cgmath::BaseFloat;

use crate::CameraViewPort;

// This iterator construct indices from a set of vertices defining
// a grid.
// Triangles that are in a clockwise order will not be renderer
// Whereas other counter-clockwise triangle will be
pub struct BuildPatchIndicesIter<'a> {
    pub idx_x_range: RangeInclusive<usize>,
    pub idx_y_range: RangeInclusive<usize>,

    pub num_x_vertices: usize,

    cur_idx_x: usize,
    cur_idx_y: usize,

    ndc: &'a [Option<[f32; 2]>],
    camera: &'a CameraViewPort,
}

impl<'a> BuildPatchIndicesIter<'a> {
    pub fn new(idx_x_range: &RangeInclusive<usize>, idx_y_range: &RangeInclusive<usize>, num_x_vertices: usize, ndc: &'a [Option<[f32; 2]>], camera: &'a CameraViewPort) -> Self {
        let cur_idx_x = *idx_x_range.start();
        let cur_idx_y = *idx_y_range.start();

        Self {
            idx_x_range: idx_x_range.clone(),
            idx_y_range: idx_y_range.clone(),
            num_x_vertices,
            cur_idx_x,
            cur_idx_y,
            ndc,
            camera,
        }
    }

    fn get_index_value(&self, idx_x: usize, idx_y: usize) -> usize {
        idx_x + idx_y * self.num_x_vertices
    }
}

impl<'a> Iterator for BuildPatchIndicesIter<'a> {
    type Item = [(u16, u16, u16); 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx_x == *self.idx_x_range.end() {
            self.cur_idx_x = *self.idx_x_range.start();
            self.cur_idx_y += 1;

            if self.cur_idx_y == *self.idx_y_range.end() {
                return None;
            }
        }

        let idx_tl = self.get_index_value(self.cur_idx_x, self.cur_idx_y);
        let idx_tr = self.get_index_value(self.cur_idx_x + 1, self.cur_idx_y);
        let idx_bl = self.get_index_value(self.cur_idx_x, self.cur_idx_y + 1);
        let idx_br = self.get_index_value(self.cur_idx_x + 1, self.cur_idx_y + 1);

        self.cur_idx_x += 1;

        let ndc_tl = &self.ndc[idx_tl];
        let ndc_tr = &self.ndc[idx_tr];
        let ndc_bl = &self.ndc[idx_bl];
        let ndc_br = &self.ndc[idx_br];

        match (ndc_tl, ndc_tr, ndc_bl, ndc_br) {
            (Some(ndc_tl), Some(ndc_tr), Some(ndc_bl), Some(ndc_br)) => {
                let t1 = Triangle::new(&ndc_tl, &ndc_tr, &ndc_bl);
                let t2 = Triangle::new(&ndc_tr, &ndc_br, &ndc_bl);

                if !t1.is_invalid(&self.camera) || !t2.is_invalid(&self.camera) {
                    self.next() // crossing projection tri
                } else {
                    Some([
                        (idx_tl as u16, idx_tr as u16, idx_bl as u16),
                        (idx_tr as u16, idx_br as u16, idx_bl as u16)
                    ])
                }
            },
            _ => self.next() // out of proj
        }
    }
}

pub struct Triangle<'a, S>
where
    S: BaseFloat
{
    v1: &'a [S; 2],
    v2: &'a [S; 2],
    v3: &'a [S; 2],
}

impl<'a, S> Triangle<'a, S>
where
    S: BaseFloat
{
    pub fn new(v1: &'a [S; 2], v2: &'a [S; 2], v3: &'a [S; 2]) -> Self {
        Self { v1, v2, v3 }
    }

    pub fn is_invalid(&self, camera: &CameraViewPort) -> bool {
        let tri_ccw = self.is_ccw();
        let reversed_longitude = camera.get_longitude_reversed();

        (!reversed_longitude && tri_ccw) || (reversed_longitude && !tri_ccw)
    }

    pub fn is_ccw(&self) -> bool {
        crate::math::utils::ccw_tri(&self.v1, &self.v2, &self.v3)
    }
}