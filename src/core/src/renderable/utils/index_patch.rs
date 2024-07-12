use std::ops::RangeInclusive;

use crate::CameraViewPort;
use cgmath::Vector2;

// This iterator construct indices from a set of vertices defining
// a grid.
// Triangles that are in a clockwise order will not be renderer
// Whereas other counter-clockwise triangle will be
pub struct CCWCheckPatchIndexIter<'a> {
    patch_iter: DefaultPatchIndexIter,

    ndc: &'a [Option<[f32; 2]>],
    camera: &'a CameraViewPort,
}

impl<'a> CCWCheckPatchIndexIter<'a> {
    pub fn new(
        idx_x_range: &RangeInclusive<usize>,
        idx_y_range: &RangeInclusive<usize>,
        num_x_vertices: usize,
        ndc: &'a [Option<[f32; 2]>],
        camera: &'a CameraViewPort,
    ) -> Self {
        let patch_iter = DefaultPatchIndexIter::new(idx_x_range, idx_y_range, num_x_vertices);

        Self {
            patch_iter,
            ndc,
            camera,
        }
    }
}

impl<'a> Iterator for CCWCheckPatchIndexIter<'a> {
    type Item = [(u16, u16, u16); 2];

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(indices) = self.patch_iter.next() {
            let idx_tl = indices[0].0;
            let idx_tr = indices[0].1;
            let idx_bl = indices[0].2;
            let idx_br = indices[1].1;

            let ndc_tl = &self.ndc[idx_tl as usize];
            let ndc_tr = &self.ndc[idx_tr as usize];
            let ndc_bl = &self.ndc[idx_bl as usize];
            let ndc_br = &self.ndc[idx_br as usize];

            match (ndc_tl, ndc_tr, ndc_bl, ndc_br) {
                (Some(ndc_tl), Some(ndc_tr), Some(ndc_bl), Some(ndc_br)) => {
                    let tlc = crate::math::projection::ndc_to_clip_space(
                        &Vector2::new(ndc_tl[0] as f64, ndc_tl[1] as f64),
                        &self.camera,
                    );
                    let brc = crate::math::projection::ndc_to_clip_space(
                        &Vector2::new(ndc_br[0] as f64, ndc_br[1] as f64),
                        &self.camera,
                    );

                    let d1 = crate::math::vector::dist2::<f64>(tlc.as_ref(), brc.as_ref());
                    if d1 > 0.1 {
                        self.next()
                    } else {
                        let trc = crate::math::projection::ndc_to_clip_space(
                            &Vector2::new(ndc_tr[0] as f64, ndc_tr[1] as f64),
                            &self.camera,
                        );
                        let blc = crate::math::projection::ndc_to_clip_space(
                            &Vector2::new(ndc_bl[0] as f64, ndc_bl[1] as f64),
                            &self.camera,
                        );

                        let d2 = crate::math::vector::dist2::<f64>(trc.as_ref(), blc.as_ref());
                        if d2 > 0.1 {
                            self.next()
                        } else {
                            Some(indices)
                        }
                    }
                }
                _ => self.next(), // out of proj
            }
        } else {
            None
        }
    }
}

pub struct DefaultPatchIndexIter {
    pub idx_x_range: RangeInclusive<usize>,
    pub idx_y_range: RangeInclusive<usize>,

    pub num_x_vertices: usize,

    cur_idx_x: usize,
    cur_idx_y: usize,
}

impl DefaultPatchIndexIter {
    pub fn new(
        idx_x_range: &RangeInclusive<usize>,
        idx_y_range: &RangeInclusive<usize>,
        num_x_vertices: usize,
    ) -> Self {
        let cur_idx_x = *idx_x_range.start();
        let cur_idx_y = *idx_y_range.start();

        Self {
            idx_x_range: idx_x_range.clone(),
            idx_y_range: idx_y_range.clone(),
            num_x_vertices,
            cur_idx_x,
            cur_idx_y,
        }
    }

    fn get_index_value(&self, idx_x: usize, idx_y: usize) -> usize {
        idx_x + idx_y * self.num_x_vertices
    }
}

impl Iterator for DefaultPatchIndexIter {
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

        Some([
            (idx_tl as u16, idx_tr as u16, idx_bl as u16),
            (idx_tr as u16, idx_br as u16, idx_bl as u16),
        ])
    }
}
