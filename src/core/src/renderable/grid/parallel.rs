use super::label::Label;
use crate::math::projection::ProjectionType;
use crate::math::sph_geom::region::Intersection;
use crate::CameraViewPort;

use crate::math::lonlat::LonLat;
use crate::math::{PI, TWICE_PI};


use crate::renderable::line;

use core::ops::Range;

pub fn get_intersecting_parallel(
    lat: f64,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> Option<Parallel> {
    let fov = camera.get_field_of_view();
    if fov.get_bounding_box().get_lon_size() > PI {
        // Longitude fov >= PI
        let camera_center = camera.get_center();
        let lon_start = camera_center.lon().to_radians();

        Some(Parallel::new(
            lat,
            &(lon_start..(lon_start + TWICE_PI)),
            camera,
            LabelOptions::Centered,
            projection,
        ))
    } else {
        // Longitude fov < PI
        let i = fov.intersects_parallel(lat);
        match i {
            Intersection::Included => {
                let camera_center = camera.get_center();
                let lon_start = camera_center.lon().to_radians();

                Some(Parallel::new(
                    lat,
                    &(lon_start..(lon_start + TWICE_PI)),
                    camera,
                    LabelOptions::Centered,
                    projection,
                ))
            }
            Intersection::Intersect { vertices } => {
                let v1 = &vertices[0];
                let v2 = &vertices[1];

                let mut lon1 = v1.lon().to_radians();
                let mut lon2 = v2.lon().to_radians();

                let lon_len = crate::math::sph_geom::distance_from_two_lon(lon1, lon2);
                let _len_vert = vertices.len();
                // The fov should be contained into PI length
                if lon_len >= PI {
                    std::mem::swap(&mut lon1, &mut lon2);
                }

                Some(Parallel::new(
                    lat,
                    &(lon1..lon2),
                    camera,
                    LabelOptions::OnSide,
                    projection,
                ))
            }
            Intersection::Empty => None,
        }
    }
}

pub struct Parallel {
    // List of vertices
    vertices: Vec<[f32; 2]>,
    // Line vertices indices
    indices: Vec<Range<usize>>,
    label: Option<Label>,
}

use super::label::LabelOptions;

impl Parallel {
    pub fn new(
        lat: f64,
        lon: &Range<f64>,
        camera: &CameraViewPort,
        label_options: LabelOptions,
        projection: &ProjectionType,
    ) -> Self {
        let label = Label::from_parallel(lat, lon, label_options, camera, projection);

        // Draw the full parallel
        let vertices = if lon.end - lon.start > PI {
            let mut vertices =
                line::parallel_arc::project(lat, lon.start, lon.start + PI, camera, projection);
            vertices.append(&mut line::parallel_arc::project(
                lat,
                lon.start + PI,
                lon.end,
                camera,
                projection,
            ));

            vertices
        } else {
            line::parallel_arc::project(lat, lon.start, lon.end, camera, projection)
        };

        /*let mut prev_v = [vertices[0].x as f32, vertices[0].y as f32];
        let vertices: Vec<_> = std::iter::once(prev_v)
            .chain(
                vertices.into_iter().skip(1)
                    .filter_map(|v| {
                        let cur_v = [v.x as f32, v.y as f32];
                        if cur_v == prev_v {
                            None
                        } else {
                            prev_v = cur_v;
                            Some(cur_v)
                        }
                    })
            )
            .collect();

        let indices = vec![0..vertices.len()];
        */
        let mut start_idx = 0;

        let mut indices = if vertices.len() >= 3 {
            let v_iter = (1..(vertices.len() - 1)).map(|i| &vertices[i]);

            v_iter
                .clone()
                .zip(v_iter.skip(1))
                .enumerate()
                .step_by(2)
                .filter_map(|(i, (v1, v2))| {
                    if v1 == v2 {
                        None
                    } else {
                        let res = Some(start_idx..(i + 2));
                        start_idx = i + 2;
                        res
                    }
                })
                .collect()
        } else {
            vec![]
        };

        indices.push(start_idx..vertices.len());

        Self {
            vertices,
            indices,
            label,
        }
    }

    #[inline]
    pub fn get_lines_vertices(&self) -> Vec<&[[f32; 2]]> {
        self.indices
            .iter()
            .map(|range| &self.vertices[range.start..range.end])
            .collect()
    }

    #[inline]
    pub fn get_label(&self) -> Option<&Label> {
        self.label.as_ref()
    }
}
