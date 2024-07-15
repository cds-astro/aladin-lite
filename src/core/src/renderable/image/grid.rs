use cgmath::Vector4;
use std::ops::RangeInclusive;
use wcs::ImgXY;

use crate::camera::CameraViewPort;
use crate::math::projection::ProjectionType;
use crate::renderable::utils::index_patch::CCWCheckPatchIndexIter;
use al_api::coo_system::CooSystem;
use wcs::WCS;

pub fn get_grid_params(
    xy_min: &(f64, f64),
    xy_max: &(f64, f64),
    max_tex_size_x: u64,
    max_tex_size_y: u64,
    num_tri_per_tex_patch: u64,
) -> (
    impl Iterator<Item = (u64, f32)> + Clone,
    impl Iterator<Item = (u64, f32)> + Clone,
) {
    let x_range_len = (xy_max.0 - xy_min.0) as u64;
    let y_range_len = (xy_max.1 - xy_min.1) as u64;

    let xmin = xy_min.0 as u64;
    let ymin = xy_min.1 as u64;
    let xmax = xy_max.0 as u64;
    let ymax = xy_max.1 as u64;

    let step_x = (x_range_len / num_tri_per_tex_patch) as usize;
    let step_y = (y_range_len / num_tri_per_tex_patch) as usize;

    let step = (step_x.max(step_y)).max(1); // at least one pixel!

    (
        get_coord_uv_it(xmin, xmax, step, max_tex_size_x),
        get_coord_uv_it(ymin, ymax, step, max_tex_size_y),
    )
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
            cur,
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

fn get_coord_uv_it(
    xmin: u64,
    xmax: u64,
    step: usize,
    max_tex_size: u64,
) -> impl Iterator<Item = (u64, f32)> + Clone {
    let get_uv_in_tex_chunk = move |x: u64| ((x % max_tex_size) as f32) / (max_tex_size as f32);

    let tex_patch_x = StepCoordIterator::new(xmin, xmax, max_tex_size);

    let x_it = std::iter::once((xmin, get_uv_in_tex_chunk(xmin)))
        .chain(
            tex_patch_x
                .clone()
                .skip(1)
                .map(|x1| vec![(x1, 1.0), (x1, 0.0)])
                .flatten(),
        )
        .chain(std::iter::once((
            xmax,
            if xmax % max_tex_size == 0 {
                1.0
            } else {
                get_uv_in_tex_chunk(xmax)
            },
        )));

    let mut step_x = (xmin..xmax).step_by(step as usize);
    let mut cur_step = step_x.next().unwrap();

    x_it.clone()
        .zip(x_it.clone().skip(1))
        .map(move |(x1, x2)| {
            let mut xk = vec![x1];

            while cur_step < x2.0 {
                if cur_step > x1.0 {
                    xk.push((cur_step, get_uv_in_tex_chunk(cur_step)));
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
        .chain(std::iter::once((
            xmax,
            if xmax % max_tex_size == 0 {
                1.0
            } else {
                get_uv_in_tex_chunk(xmax)
            },
        )))
}

fn build_range_indices(it: impl Iterator<Item = (u64, f32)> + Clone) -> Vec<RangeInclusive<usize>> {
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
pub fn vertices(
    xy_min: &(f64, f64),
    xy_max: &(f64, f64),
    max_tex_size_x: u64,
    max_tex_size_y: u64,
    num_tri_per_tex_patch: u64,
    camera: &CameraViewPort,
    wcs: &WCS,
    projection: &ProjectionType,
) -> (Vec<f32>, Vec<f32>, Vec<u16>, Vec<u32>) {
    let (x_it, y_it) = get_grid_params(
        xy_min,
        xy_max,
        max_tex_size_x,
        max_tex_size_y,
        num_tri_per_tex_patch,
    );

    let idx_x_ranges = build_range_indices(x_it.clone());
    let idx_y_ranges = build_range_indices(y_it.clone());

    let num_x_vertices = idx_x_ranges.last().unwrap().end() + 1;

    let mut uv = vec![];
    let pos = y_it
        .map(|(y, uvy)| {
            x_it.clone().map(move |(x, uvx)| {
                let ndc = if let Some(xyz) = wcs.unproj_xyz(&ImgXY::new(x as f64, y as f64)) {
                    let xyzw = crate::coosys::apply_coo_system(
                        CooSystem::ICRS,
                        camera.get_coo_system(),
                        &Vector4::new(xyz.y(), xyz.z(), xyz.x(), 1.0),
                    );

                    projection
                        .model_to_normalized_device_space(&xyzw, camera)
                        .map(|v| [v.x as f32, v.y as f32])
                } else {
                    None
                };

                (ndc, [uvx, uvy])
            })
        })
        .flatten()
        .map(|(p, uu)| {
            uv.extend_from_slice(&uu);
            p
        })
        .collect::<Vec<_>>();

    let mut indices = vec![];
    let mut num_indices = vec![];
    for idx_x_range in &idx_x_ranges {
        for idx_y_range in &idx_y_ranges {
            let build_indices_iter =
                CCWCheckPatchIndexIter::new(idx_x_range, idx_y_range, num_x_vertices, &pos, camera);

            let patch_indices = build_indices_iter
                .flatten()
                .map(|indices| [indices.0, indices.1, indices.2])
                .flatten()
                .collect::<Vec<_>>();

            num_indices.push(patch_indices.len() as u32);
            indices.extend(patch_indices);
        }
    }

    let pos = pos
        .into_iter()
        .flat_map(|ndc| ndc.unwrap_or([0.0, 0.0]))
        .collect();

    (pos, uv, indices, num_indices)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_grid_vertices() {
        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(40.0, 40.0), 20, 20, 4);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 6);
        assert_eq!(y.len(), 6);

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(50.0, 40.0), 20, 20, 5);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 8);
        assert_eq!(y.len(), 6);

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(7000.0, 7000.0), 4096, 4096, 2);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x.len(), 5);
        assert_eq!(y.len(), 5);

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(3000.0, 7000.0), 4096, 4096, 2);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (3000, 0.7324219)]);
        assert_eq!(
            y,
            &[
                (0, 0.0),
                (3500, 0.8544922),
                (4096, 1.0),
                (4096, 0.0),
                (7000, 0.7089844)
            ]
        );

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(4096.0, 4096.0), 4096, 4096, 1);

        let x_idx_rng = super::build_range_indices(x.clone());
        let y_idx_rng = super::build_range_indices(y.clone());

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (4096, 1.0)]);
        assert_eq!(y, &[(0, 0.0), (4096, 1.0)]);

        assert_eq!(x_idx_rng, &[0..=1]);
        assert_eq!(y_idx_rng, &[0..=1]);

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(11000.0, 7000.0), 4096, 4096, 1);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(
            x,
            &[
                (0, 0.0),
                (4096, 1.0),
                (4096, 0.0),
                (8192, 1.0),
                (8192, 0.0),
                (11000, 0.6855469)
            ]
        );
        assert_eq!(y, &[(0, 0.0), (4096, 1.0), (4096, 0.0), (7000, 0.7089844)]);

        let (x, y) = super::get_grid_params(&(0.0, 0.0), &(4096.0, 4096.0), 4096, 4096, 1);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(0, 0.0), (4096, 1.0)]);
        assert_eq!(y, &[(0, 0.0), (4096, 1.0)]);

        let (x, y) = super::get_grid_params(&(3000.0, 4000.0), &(4096.0, 7096.0), 4096, 4096, 1);

        let x = x.collect::<Vec<_>>();
        let y = y.collect::<Vec<_>>();

        assert_eq!(x, &[(3000, 0.7324219), (4096, 1.0)]);
        assert_eq!(
            y,
            &[
                (4000, 0.9765625),
                (4096, 1.0),
                (4096, 0.0),
                (7096, 0.7324219)
            ]
        );
    }
}
