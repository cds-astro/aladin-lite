use super::label::{Label, LabelOptions};
use crate::math::lonlat::LonLat;
use crate::math::sph_geom::region::Intersection;
use crate::CameraViewPort;
use core::ops::Range;

use crate::math::MINUS_HALF_PI;
use crate::ProjectionType;

use super::angle::SerializeFmt;
use crate::math::HALF_PI;

pub fn get_intersecting_meridian(
    lon: f64,
    camera: &CameraViewPort,
    projection: &ProjectionType,
    fmt: &SerializeFmt,
) -> Option<Meridian> {
    let fov = camera.get_field_of_view();
    if fov.contains_both_poles() {
        let meridian = Meridian::new(
            lon,
            &(-HALF_PI..HALF_PI),
            LabelOptions::Centered,
            camera,
            projection,
            fmt,
        );
        Some(meridian)
    } else {
        let i = fov.intersects_meridian(lon);
        match i {
            Intersection::Included => {
                // Longitude fov >= PI
                let meridian = Meridian::new(
                    lon,
                    &(-HALF_PI..HALF_PI),
                    LabelOptions::Centered,
                    camera,
                    projection,
                    fmt,
                );
                Some(meridian)
            }
            Intersection::Intersect { vertices } => {
                let num_intersections = vertices.len();
                let meridian = match num_intersections {
                    1 => {
                        let v1 = &vertices[0];
                        let lonlat1 = v1.lonlat();
                        let lat1 = lonlat1.lat().to_radians();

                        let lat = if fov.contains_north_pole() {
                            lat1..HALF_PI
                        } else {
                            lat1..MINUS_HALF_PI
                        };

                        Meridian::new(lon, &lat, LabelOptions::OnSide, camera, projection, fmt)
                    }
                    2 => {
                        // full intersection
                        let v1 = &vertices[0];
                        let v2 = &vertices[1];

                        let lat1 = v1.lat().to_radians();
                        let lat2 = v2.lat().to_radians();

                        Meridian::new(
                            lon,
                            &(lat1..lat2),
                            LabelOptions::OnSide,
                            camera,
                            projection,
                            fmt,
                        )
                    }
                    _ => {
                        /*let mut vertices = vertices.into_vec();
                        // One segment over two will be in the field of view
                        vertices.push(Vector4::new(0.0, 1.0, 0.0, 1.0));
                        vertices.push(Vector4::new(0.0, -1.0, 0.0, 1.0));

                        vertices.sort_by(|i1, i2| {
                            i1.y.total_cmp(&i2.y)
                        });

                        let v1 = &vertices[0];
                        let v2 = &vertices[1];

                        // meridian are part of great circles so the mean between v1 & v2 also lies on it
                        let vm = (v1 + v2).truncate().normalize();

                        let vertices = if !fov.contains_south_pole() {
                            &vertices[1..]
                        } else {
                            &vertices
                        };

                        let line_vertices = vertices.iter().zip(vertices.iter().skip(1))
                            .step_by(2)
                            .map(|(i1, i2)| {
                                line::great_circle_arc::project(
                                    lon,
                                    i1.lat().to_radians(),
                                    lon,
                                    i2.lat().to_radians(),
                                    camera,
                                    projection
                                )
                            })
                            .flatten()
                            .collect::<Vec<_>>();

                        let label = Label::from_meridian(&v1.lonlat(), camera, projection, fmt);
                        */
                        Meridian::new(
                            lon,
                            &(-HALF_PI..HALF_PI),
                            LabelOptions::OnSide,
                            camera,
                            projection,
                            fmt,
                        )
                    }
                };

                Some(meridian)
            }
            Intersection::Empty => None,
        }
    }
}

pub struct Meridian {
    // List of vertices
    vertices: Vec<[f32; 2]>,
    // Line vertices indices
    indices: Vec<Range<usize>>,
    label: Option<Label>,
}

impl Meridian {
    pub fn new(
        lon: f64,
        lat: &Range<f64>,
        label_options: LabelOptions,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        fmt: &SerializeFmt,
    ) -> Self {
        let label = Label::from_meridian(lon, lat, label_options, camera, projection, fmt);

        // Draw the full parallel
        let vertices = crate::renderable::line::great_circle_arc::project(
            lon, lat.start, lon, lat.end, camera, projection,
        )
        .into_iter()
        .map(|v| [v.x as f32, v.y as f32])
        .collect::<Vec<_>>();

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

        // Create subsets of vertices referring to different lines
        let indices = if vertices.len() >= 3 {
            let mut indices = vec![];

            let mut v0 = 0;
            let mut v1 = 1;
            let mut v2 = 2;

            let mut s = 0;

            let n_segment = vertices.len() - 1;

            for i in 0..n_segment {
                if Triangle::new(&vertices[v0], &vertices[v1], &vertices[v2]).is_valid(camera) {
                    indices.push(s..(i+1));
                    s = i;
                }

                v0 = v1;
                v1 = v2;
                v2 = (v2 + 1) % vertices.len();
            }

            //indices.push(start_line_i..vertices.len());
            //vec![0..vertices.len()]
            vec![0..2]
        } else {
            vec![0..vertices.len()]
        };*/

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
            .map(|r| &self.vertices[r.start..r.end])
            .collect()
    }

    #[inline]
    pub fn get_label(&self) -> Option<&Label> {
        self.label.as_ref()
    }
}
