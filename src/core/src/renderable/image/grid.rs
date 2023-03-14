use wcs::ImgXY;
use std::ops::Range;
use std::ops::RangeInclusive;

use crate::camera::CameraViewPort;
use crate::math::projection::ProjectionType;
use wcs::WCS;
use al_api::coo_system::CooSystem;
use crate::coo_space::XYNDC;
use crate::math::angle::ToAngle;
use crate::Vector2;

#[allow(dead_code)]
pub fn get_grid_vertices(xy_min: &ImgXY, xy_max: &ImgXY, max_tex_size: u64, num_tri_per_tex_patch: u64, camera: &CameraViewPort, wcs: &WCS, projection: &ProjectionType) -> (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<u32>, Vec<u32>) {
    let x_range_len = (xy_max.x() - xy_min.x()) as u64;
    let y_range_len = (xy_max.y() - xy_min.y()) as u64;

    let xmin = xy_min.x() as u64;
    let ymin = xy_min.y() as u64;
    let xmax = xy_max.x() as u64;
    let ymax = xy_max.y() as u64;

    let step_x = (x_range_len / num_tri_per_tex_patch) as usize;
    let step_y = (y_range_len / num_tri_per_tex_patch) as usize;

    let step = step_x.max(step_y);

    let it_x = (xmin..=xmax).step_by(step);
    let it_y = (ymin..=ymax).step_by(step);

    let get_uv_in_tex_chunk = |x: u64| {
        ((x % max_tex_size) as f32) / (max_tex_size as f32)
    };

    let x = std::iter::once((xmin, get_uv_in_tex_chunk(xmin)))
        .chain(
            it_x.clone().skip(1).zip(it_x.skip(2))
                .map(|(x1, x2)| { 
                    let x1_t = x1 / max_tex_size;
                    let x2_t = x2 / max_tex_size;
        
                    let cross_tex_chunk = x2_t > x1_t && (x2 % max_tex_size > 0);
    
                    let uv1 = ((x1 % max_tex_size) as f32) / (max_tex_size as f32);
                    let uv2 = ((x2 % max_tex_size) as f32) / (max_tex_size as f32);
    
                    if cross_tex_chunk {
                        let xt = x1 - (x1 % max_tex_size) + max_tex_size;
                        vec![(x1, uv1), (xt, 0.0)]
                    } else {
                        vec![(x1, uv1)]
                    }
                })
                .flatten()
                .map(|(x, uv)| {
                    if x % max_tex_size == 0 {
                        vec![(x, 1.0), (x, 0.0)]
                    } else {
                        vec![(x, uv)]
                    }
                })
                .flatten()
        ).chain(std::iter::once((xmax, get_uv_in_tex_chunk(xmax))));

    dbg!(x.clone().collect::<Vec<_>>());

    let y = std::iter::once((ymin, get_uv_in_tex_chunk(ymin)))
    .chain(
        it_y.clone().skip(1).zip(it_y.skip(2))
            .map(|(y1, y2)| { 
                let y1_t = y1 / max_tex_size;
                let y2_t = y2 / max_tex_size;
    
                let cross_tex_chunk = y2_t > y1_t && (y2 % max_tex_size > 0);

                let uv1 = ((y1 % max_tex_size) as f32) / (max_tex_size as f32);
                let uv2 = ((y2 % max_tex_size) as f32) / (max_tex_size as f32);

                if cross_tex_chunk {
                    let yt = y1 - (y1 % max_tex_size) + max_tex_size;
                    vec![(y1, uv1), (yt, 0.0)]
                } else {
                    vec![(y1, uv1)]
                }
            })
            .flatten()
            .map(|(y, uv)| {
                if y % max_tex_size == 0 {
                    vec![(y, 1.0), (y, 0.0)]
                } else {
                    vec![(y, uv)]
                }
            })
            .flatten()
    ).chain(std::iter::once((ymax, get_uv_in_tex_chunk(ymax))));

    let x_bis = x.clone();
    let (pos, uv): (Vec<_>, Vec<_>) = y.clone().map(move |(y, uvy)|
        x_bis.clone().map(move |(x, uvx)| {
            let lonlat = wcs.unproj(&ImgXY::new(x as f64, y as f64)).unwrap(); // vertex belong to the image space so I can unproject
            let lon = lonlat.lon();
            let lat = lonlat.lat();

            let xyzw = crate::math::lonlat::radec_to_xyzw(lon.to_angle(), lat.to_angle());
            let xyzw = crate::coosys::apply_coo_system(&CooSystem::ICRSJ2000, camera.get_system(), &xyzw);

            let ndc = projection.model_to_normalized_device_space(&xyzw, camera)
                .map(|v| [v.x as f32, v.y as f32]);

            (ndc, [uvx, uvy])
        })
    ).flatten()
    .unzip();

    let mut idx_y_ranges = vec![];
    let mut idx_x_ranges = vec![];

    let mut idx_start = 0;
    let mut last_idx = 0;
    for (idx_c, ((x_c, _), (x_n, _))) in x.clone().zip(x.skip(1)).enumerate() {
        if x_c == x_n {
            // on a tex chunk frontier
            idx_x_ranges.push(idx_start..=idx_c);

            idx_start = idx_c + 1;
            last_idx = idx_c + 1;
        } else {
            last_idx = idx_c + 1;
        }
    }

    if last_idx > idx_start {
        idx_x_ranges.push(idx_start..=last_idx);
    }

    dbg!(&idx_x_ranges);

    // Get the range of y pos patches
    let mut idx_start = 0;
    let mut last_idx = 0;
    for (idx_c, ((y_c, _), (y_n, _))) in y.clone().zip(y.skip(1)).enumerate() {
        if y_c == y_n {
            // on a tex chunk frontier
            idx_y_ranges.push(idx_start..=idx_c);

            idx_start = idx_c + 1;
            last_idx = idx_c + 1;
        } else {
            last_idx = idx_c + 1;
        }
    }

    if last_idx > idx_start {
        idx_y_ranges.push(idx_start..=last_idx);
    }

    dbg!(&idx_y_ranges);


    let num_x_vertices = idx_x_ranges.last().unwrap().end() + 1;

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

#[cfg(tests)]
mod tests {
    use wcs::ImgXY;

    #[test]
    fn test_grid_vertices() {
        let (pos, uv, indices, num_indices) = super::get_grid_vertices(
            &ImgXY::new(0.0, 0.0),
            &ImgXY::new(40.0, 40.0),
            20,
            4
        );

        assert_eq!(pos.len(), 36);
        assert_eq!(uv.len(), 36);

        let (pos, uv, indices, num_indices) = super::get_grid_vertices(
            &ImgXY::new(0.0, 0.0),
            &ImgXY::new(50.0, 40.0),
            20,
            5
        );

        assert_eq!(pos.len(), 48);
        assert_eq!(uv.len(), 48);

        let (pos, uv, indices, num_indices) = super::get_grid_vertices(
            &ImgXY::new(0.0, 0.0),
            &ImgXY::new(7000.0, 7000.0),
            4096,
            2
        );

        assert_eq!(pos.len(), 25);
        assert_eq!(uv.len(), 25);
    }
}