use wcs::ImgXY;
use std::ops::RangeInclusive;

use crate::camera::CameraViewPort;
use crate::math::projection::ProjectionType;
use wcs::WCS;
use al_api::coo_system::CooSystem;
use crate::math::angle::ToAngle;
use crate::Vector2;

pub fn get_grid_params(xy_min: &(f64, f64), xy_max: &(f64, f64), max_tex_size: u64, num_tri_per_tex_patch: u64) -> (impl Iterator<Item=(u64, f32)> + Clone, impl Iterator<Item=(u64, f32)> + Clone) {
    let x_range_len = (xy_max.0 - xy_min.0) as u64;
    let y_range_len = (xy_max.1 - xy_min.1) as u64;

    let xmin = xy_min.0 as u64;
    let ymin = xy_min.1 as u64;
    let xmax = xy_max.0 as u64;
    let ymax = xy_max.1 as u64;

    let step_x = (x_range_len / num_tri_per_tex_patch) as usize;
    let step_y = (y_range_len / num_tri_per_tex_patch) as usize;

    let step = (step_x.max(step_y)).max(1); // at least one pixel!

    (get_coord_uv_it(xmin, xmax, step, max_tex_size), get_coord_uv_it(ymin, ymax, step, max_tex_size))
}

#[derive(Clone)]
struct StepCoordIterator {
    start: u64,
    end: u64,

    step: u64,

    cur: u64,
}

impl StepCoordIterator {
    fn new(start: u64, end: u64, step: u64) -> Self {
        let cur = start;

        Self {
            start,
            step,
            end,
            cur
        }
    }
}

impl Iterator for StepCoordIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {

        if self.cur == self.start {
            // starting case
            self.cur = self.start - (self.start % self.step) + self.step;
           
            Some(self.start)
        } else if self.cur < self.end {
            // ongoing case
            let cur = self.cur;

            self.cur += self.step;
            Some(cur)
        } else {
            None
        }
    }
}

fn get_coord_uv_it(xmin: u64, xmax: u64, step: usize, max_tex_size: u64) -> impl Iterator<Item=(u64, f32)> + Clone {
    let get_uv_in_tex_chunk = move |x: u64| {
        ((x % max_tex_size) as f32) / (max_tex_size as f32)
    };

    let tex_patch_x = StepCoordIterator::new(xmin, xmax, max_tex_size);

    let x_it = std::iter::once((xmin, get_uv_in_tex_chunk(xmin)))
        .chain(
            tex_patch_x.clone().skip(1)
                .map(|x1| {
                    vec![(x1, 1.0), (x1, 0.0)]
                })
                .flatten()
        )
        .chain(
            std::iter::once((
                xmax,
                if xmax % max_tex_size == 0 {
                    1.0
                } else {
                    get_uv_in_tex_chunk(xmax)
                }
            ))
        );

    let mut step_x = (xmin..xmax).step_by(step as usize);


    let mut cur_step = step_x.next().unwrap();

    x_it.clone().zip(x_it.clone().skip(1))
        .map(move |(x1, x2)| {
            let mut xk = vec![x1];

            while cur_step < x2.0 {
                if cur_step > x1.0 {
                    xk.push(
                        (cur_step, get_uv_in_tex_chunk(cur_step))
                    );
                }

                if let Some(step) = step_x.next() {
                    cur_step = step;
                } else {
                    break;
                }
            }

            xk
        })
        .flatten()
        .chain(
            std::iter::once((
                xmax,
                if xmax % max_tex_size == 0 {
                    1.0
                } else {
                    get_uv_in_tex_chunk(xmax)
                }
            ))
        )
} 

fn build_range_indices(it: impl Iterator<Item=(u64, f32)> + Clone) -> Vec<RangeInclusive<usize>> {
    let mut idx_ranges = vec![];

    let mut idx_start = 0;
    let mut last_idx = 0;
    for (idx_c, ((x_c, _), (x_n, _))) in it.clone().zip(it.skip(1)).enumerate() {
        let idx_n = idx_c + 1;
        if x_c == x_n {
            // on a tex chunk frontier
            idx_ranges.push(idx_start..=idx_c);

            idx_start = idx_n;
            last_idx = idx_n;
        } else {
            last_idx = idx_n;
        }
    }

    if last_idx > idx_start {
        idx_ranges.push(idx_start..=last_idx);
    }

    idx_ranges
}

#[allow(dead_code)]
pub fn get_grid_vertices(xy_min: &(f64, f64), xy_max: &(f64, f64), max_tex_size: u64, num_tri_per_tex_patch: u64, camera: &CameraViewPort, wcs: &WCS, projection: &ProjectionType) -> (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<u32>, Vec<u32>) {    
    let (x_it, y_it) = get_grid_params(xy_min, xy_max, max_tex_size, num_tri_per_tex_patch);

    let idx_x_ranges = build_range_indices(x_it.clone());
    let idx_y_ranges = build_range_indices(y_it.clone());

    let num_x_vertices = idx_x_ranges.last().unwrap().end() + 1;

    let (pos, uv): (Vec<_>, Vec<_>) = y_it.map(move |(y, uvy)|
        x_it.clone().map(move |(x, uvx)| {
            let ndc = if let Some(lonlat) = wcs.unproj(&ImgXY::new(x as f64, y as f64)) {
                let lon = lonlat.lon();
                let lat = lonlat.lat();
    
                let xyzw = crate::math::lonlat::radec_to_xyzw(lon.to_angle(), lat.to_angle());
                let xyzw = crate::coosys::apply_coo_system(&CooSystem::ICRSJ2000, camera.get_system(), &xyzw);
    
                projection.model_to_normalized_device_space(&xyzw, camera)
                    .map(|v| [v.x as f32, v.y as f32])
            } else {
                None
            };

            (ndc, [uvx, uvy])
        })
    ).flatten()
    .unzip();

    let mut indices = vec![];
    let mut num_indices = vec![];
    for idx_x_range in &idx_x_ranges {
        for idx_y_range in &idx_y_ranges {
            let build_indices_iter = BuildPatchIndicesIter::new(idx_x_range, idx_y_range, num_x_vertices, &pos, camera);

            let patch_indices = build_indices_iter.flatten()
                .map(|indices| [indices.0, indices.1, indices.2])
                .flatten()
                .collect::<Vec<_>>();

            num_indices.push(patch_indices.len() as u32);
            indices.extend(patch_indices);
        }
    }

    let pos = pos.into_iter()
        .map(|ndc| {
            if let Some(ndc) = ndc {
                ndc
            } else {
                [0.0, 0.0]
            }
        })
        .collect();

    (pos, uv, indices, num_indices)
}

struct BuildPatchIndicesIter<'a> {
    pub idx_x_range: RangeInclusive<usize>,
    pub idx_y_range: RangeInclusive<usize>,

    pub num_x_vertices: usize,

    cur_idx_x: usize,
    cur_idx_y: usize,

    ndc: &'a [Option<[f32; 2]>],
    camera: &'a CameraViewPort,
}

impl<'a> BuildPatchIndicesIter<'a> {
    fn new(idx_x_range: &RangeInclusive<usize>, idx_y_range: &RangeInclusive<usize>, num_x_vertices: usize, ndc: &'a [Option<[f32; 2]>], camera: &'a CameraViewPort) -> Self {
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

    fn invalid_tri(&self, tri_ccw: bool) -> bool {
        let reversed_longitude = self.camera.get_longitude_reversed();
        (!reversed_longitude && !tri_ccw) || (reversed_longitude && tri_ccw)
    }
}

impl<'a> Iterator for BuildPatchIndicesIter<'a> {
    type Item = [(u32, u32, u32); 2];

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
                let ndc_tl = Vector2::new(ndc_tl[0] as f64, ndc_tl[1] as f64);
                let ndc_tr = Vector2::new(ndc_tr[0] as f64, ndc_tr[1] as f64);
                let ndc_bl = Vector2::new(ndc_bl[0] as f64, ndc_bl[1] as f64);
                let ndc_br = Vector2::new(ndc_br[0] as f64, ndc_br[1] as f64);

                let c_tl = crate::math::projection::ndc_to_screen_space(&ndc_tl, self.camera);
                let c_tr = crate::math::projection::ndc_to_screen_space(&ndc_tr, self.camera);
                let c_bl = crate::math::projection::ndc_to_screen_space(&ndc_bl, self.camera);
                let c_br = crate::math::projection::ndc_to_screen_space(&ndc_br, self.camera);

                let tri_ccw_1 = !crate::math::vector::ccw_tri(&c_tl, &c_tr, &c_bl);
                let tri_ccw_2 = !crate::math::vector::ccw_tri(&c_tr, &c_br, &c_bl);

                if self.invalid_tri(tri_ccw_1) || self.invalid_tri(tri_ccw_2) {
                    self.next() // crossing projection tri
                } else {
                    Some([
                        (idx_tl as u32, idx_tr as u32, idx_bl as u32),
                        (idx_tr as u32, idx_br as u32, idx_bl as u32)
                    ])
                }
            },
            _ => self.next() // out of proj
        }
    }
}

#[cfg(test)]
mod tests {
    use wcs::ImgXY;

    #[test]
    fn test_grid_vertices() {
        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(40.0, 40.0),
            20,
            4
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 6);
        assert_eq!(y.len(), 6);

        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(50.0, 40.0),
            20,
            5
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 8);
        assert_eq!(y.len(), 6);

        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(7000.0, 7000.0),
            4096,
            2
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 5);
        assert_eq!(y.len(), 5);

        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(3000.0, 7000.0),
            4096,
            2
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (3000, 0.7324219)]);
        assert_eq!(y, &[(0, 0.0), (3500, 0.8544922), (4096, 1.0), (4096, 0.0), (7000, 0.7089844)]);

        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(4096.0, 4096.0),
            4096,
            1
        );

        let x_idx_rng = super::build_range_indices(x.clone());
        let y_idx_rng = super::build_range_indices(y.clone());

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (4096, 1.0)]);
        assert_eq!(y, &[(0, 0.0), (4096, 1.0)]);

        assert_eq!(x_idx_rng, &[0..=1]);
        assert_eq!(y_idx_rng, &[0..=1]);


        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(11000.0, 7000.0),
            4096,
            1
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (4096, 1.0), (4096, 0.0), (8192, 1.0), (8192, 0.0), (11000, 0.6855469)]);
        assert_eq!(y, &[(0, 0.0), (4096, 1.0), (4096, 0.0), (7000, 0.7089844)]);

        let (x, y) = super::get_grid_params(
            &(0.0, 0.0),
            &(4096.0, 4096.0),
            4096,
            1
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (4096, 1.0)]);
        assert_eq!(y, &[(0, 0.0), (4096, 1.0)]);

        let (x, y) = super::get_grid_params(
            &(3000.0, 4000.0),
            &(4096.0, 7096.0),
            4096,
            1
        );

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(3000, 0.7324219), (4096, 1.0)]);
        assert_eq!(y, &[(4000, 0.9765625), (4096, 1.0), (4096, 0.0), (7096, 0.7324219)]);
    }
}