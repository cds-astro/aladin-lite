use al_api::angle::Formatter;

use super::label::{Label, LabelOptions};
use crate::math::lonlat::LonLat;
use crate::math::sph_geom::region::Intersection;
use crate::CameraViewPort;
use core::ops::Range;

use crate::math::MINUS_HALF_PI;
use crate::ProjectionType;

use crate::math::HALF_PI;

pub fn get_intersecting_meridian(
    lon: f64,
    camera: &CameraViewPort,
    projection: &ProjectionType,
    fmt: Formatter,
    grid_decimal_prec: u8
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
            grid_decimal_prec
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
                    grid_decimal_prec
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

                        Meridian::new(lon, &lat, LabelOptions::OnSide, camera, projection, fmt, grid_decimal_prec)
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
                            grid_decimal_prec
                        )
                    }
                    _ => Meridian::new(
                        lon,
                        &(-HALF_PI..HALF_PI),
                        LabelOptions::OnSide,
                        camera,
                        projection,
                        fmt,
                        grid_decimal_prec
                    )
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
        fmt: Formatter,
        grid_decimal_prec: u8
    ) -> Self {
        let label = Label::from_meridian(lon, lat, label_options, camera, projection, fmt, grid_decimal_prec);

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
