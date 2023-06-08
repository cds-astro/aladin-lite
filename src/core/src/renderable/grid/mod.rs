use crate::math::MINUS_HALF_PI;
use crate::math::lonlat::LonLat;
use crate::math::projection::coo_space::XYNDC;
use crate::math::projection::coo_space::XYScreen;
use crate::math::projection::coo_space::XYZWModel;
use crate::math::sph_geom::region::Intersection;
use cdshealpix::nested::center;
use web_sys::WebGl2RenderingContext;
use cdshealpix::sph_geom::coo3d::{Coo3D};
use crate::renderable::line;

use al_core::{log, inforec, info};

use crate::math::TWICE_PI;
use crate::math::angle;
use cgmath::Vector4;

use crate::camera::CameraViewPort;
use crate::ProjectionType;
use crate::LonLatT;
use crate::math::angle::ToAngle;

use al_api::grid::GridCfg;
use al_core::VertexArrayObject;
use al_api::color::ColorRGB;
use crate::Abort;
pub struct ProjetedGrid {
    // Properties
    pub color: ColorRGB,
    pub opacity: f32,
    pub show_labels: bool,
    pub enabled: bool,
    pub label_scale: f32,

    // The vertex array object of the screen in NDC
    vao: VertexArrayObject,

    labels: Vec<Option<Label>>,
    sizes: Vec<usize>,
    offsets: Vec<usize>,

    num_vertices: usize,

    gl: WebGlContext,

    // Render Text Manager
    text_renderer: TextRenderManager,
    fmt: angle::SerializeFmt,
}

use crate::shader::ShaderManager;
use al_core::VecData;
use al_core::WebGlContext;
use wasm_bindgen::JsValue;

use super::labels::RenderManager;

use super::TextRenderManager;

use al_api::resources::Resources;
impl ProjetedGrid {
    pub fn new(
        gl: &WebGlContext,
        camera: &CameraViewPort,
        resources: &Resources,
        projection: &ProjectionType
    ) -> Result<ProjetedGrid, JsValue> {
        let vao = {
            let mut vao = VertexArrayObject::new(gl);
            let vertices = vec![];
            // layout (location = 0) in vec2 ndc_pos;
            #[cfg(feature = "webgl2")]
            vao.bind_for_update().add_array_buffer(
                "ndc_pos",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&vertices),
            );
            #[cfg(feature = "webgl1")]
            vao.bind_for_update().add_array_buffer(
                2,
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&vertices),
            );

            vao
        };

        let num_vertices = 0;

        let labels = vec![];

        let gl = gl.clone();
        let sizes = vec![];
        let offsets = vec![];

        let text_renderer = TextRenderManager::new(gl.clone(), &resources)?;

        let color = ColorRGB { r: 0.0, g: 1.0, b: 0.0 };
        let opacity = 1.0;
        let show_labels = true;
        let enabled = false;
        let label_scale = 1.0;

        let fmt = angle::SerializeFmt::DMS;

        let mut grid = ProjetedGrid {
            color,
            opacity,
            show_labels,
            enabled,
            label_scale,

            vao,
            //vbo,
            labels,
            num_vertices,

            sizes,
            offsets,

            gl,

            text_renderer,
            fmt,
        };
        // Initialize the vertices & labels
        grid.force_update(camera, projection);

        Ok(grid)
    }

    pub fn set_cfg(&mut self, new_cfg: GridCfg, camera: &CameraViewPort, projection: &ProjectionType) -> Result<(), JsValue> {
        let GridCfg {
            color,
            opacity,
            show_labels,
            label_size,
            enabled,
            fmt,
        } = new_cfg;

        if let Some(color) = color {
            self.color = color;
        }

        if let Some(opacity) = opacity {
            self.opacity = opacity;
        }

        if let Some(show_labels) = show_labels {
            self.show_labels = show_labels;
        }

        if let Some(fmt) = fmt {
            self.fmt = fmt.into();
        }

        if let Some(enabled) = enabled {
            self.enabled = enabled;
            if enabled {
                self.force_update(camera, projection);
            }
        }

        if let Some(label_size) = label_size {
            self.label_scale = label_size;
        }

        self.text_renderer.begin_frame();
        for label in self.labels.iter().flatten() {
            self.text_renderer.add_label(
                &label.content,
                &label.position.cast::<f32>().unwrap_abort(),
                cgmath::Rad(label.rot as f32),
            );
        }
        self.text_renderer.end_frame();


        Ok(())
    }

    fn force_update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) {
        self.text_renderer.begin_frame();
        //let text_height = text_renderer.text_size();
        let lines = lines(camera, projection, &self.fmt);

        self.offsets.clear();
        self.sizes.clear();
        let (vertices, labels): (Vec<Vec<Vector2<f64>>>, Vec<Option<Label>>) = lines
            .into_iter()
            .map(|line| {
                if self.sizes.is_empty() {
                    self.offsets.push(0);
                } else {
                    let last_offset = *self.offsets.last().unwrap_abort();
                    self.offsets.push(last_offset + self.sizes.last().unwrap_abort());
                }
                self.sizes.push(line.vertices.len());

                (line.vertices, line.label)
            })
            .unzip();
        self.labels = labels;

        for label in self.labels.iter().flatten() {
            self.text_renderer.add_label(
                &label.content,
                &label.position.cast::<f32>().unwrap_abort(),
                cgmath::Rad(label.rot as f32),
            );
        }

        let vertices = vertices
            .into_iter()
            .flatten()
            .flat_map(|v| [v.x as f32, v.y as f32])
            .collect::<Vec<_>>();

        self.num_vertices = vertices.len() >> 1;

        #[cfg(feature = "webgl2")]
        self.vao.bind_for_update().update_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&vertices),
        );
        #[cfg(feature = "webgl1")]
        self.vao.bind_for_update().update_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&vertices),
        );

        self.text_renderer.end_frame();
    }

    // Update the grid whenever the camera moved
    pub fn update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) {
        if !self.enabled {
            return;
        }

        self.force_update(camera, projection);
    }

    fn draw_lines_cpu(&self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        let shader = shaders
            .get(
                &self.gl,
                &ShaderId(Cow::Borrowed("GridVS_CPU"), Cow::Borrowed("GridFS_CPU")),
            )
            .unwrap_abort();
        let shader = shader.bind(&self.gl);
        shader
            .attach_uniforms_from(camera)
            .attach_uniform("opacity", &self.opacity)
            .attach_uniform("color", &self.color);

        // The raster vao is bound at the lib.rs level
        let drawer = shader.bind_vertex_array_object_ref(&self.vao);
        for (offset, size) in self.offsets.iter().zip(self.sizes.iter()) {
            if *size > 0 {
                drawer.draw_arrays(WebGl2RenderingContext::LINES, *offset as i32, *size as i32);
            }
        }
    }

    pub fn draw(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        if self.enabled {
            self.gl.enable(WebGl2RenderingContext::BLEND);
            self.draw_lines_cpu(camera, shaders);

            self.gl.disable(WebGl2RenderingContext::BLEND);

            if self.show_labels {
                self.text_renderer.draw(camera, &self.color, self.opacity, self.label_scale)?;
            }
        }

        Ok(())
    }
}

use crate::shader::ShaderId;

use core::num;
use std::borrow::Cow;
use std::path::is_separator;

use crate::math::{
    angle::Angle,
};
use crate::camera::fov::FieldOfView;
use cgmath::InnerSpace;
use cgmath::Vector2;
use core::ops::Range;

#[derive(Debug)]
struct Label {
    // The position
    position: XYScreen,
    // the string content
    content: String,
    // in radians
    rot: f64,
}
impl Label {
    fn from_meridian(
        lonlat: &LonLatT<f64>,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        fmt: &angle::SerializeFmt
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        let d = if fov.contains_north_pole() {
            Vector3::new(0.0, 1.0, 0.0)
        } else if fov.contains_south_pole() {
            Vector3::new(0.0, -1.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let m1: Vector3<_> = lonlat.vector();
        let m2 = (m1 + d * 1e-3).normalize();

        //let s1 = projection.model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m1), camera, reversed_longitude)?;
        let s1 = projection.model_to_screen_space(&m1.extend(1.0), camera)?;
        let s2 = projection.model_to_screen_space(&m2.extend(1.0), camera)?;

        //let s2 = projection.model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m2), camera, reversed_longitude)?;

        let ds = (s2 - s1).normalize();

        let mut lon = m1.lon().to_radians();
        if lon < 0.0 {
            lon += TWICE_PI;
        }

        let content = fmt.to_string(lon.to_angle());
        let position = if !fov.is_allsky() {
            //let dim = ctx2d.measure_text(&content).unwrap_abort();
            let dim = 100.0;
            let k = ds * (dim * 0.5 + 10.0);

            s1 + k
        } else {
            s1
        };

        // rot is between -PI and +PI
        let rot = if ds.y > 0.0 {
            ds.x.acos()
        } else {
            -ds.x.acos()
        };
        let rot = if ds.y > 0.0 {
            if rot > HALF_PI {
                -PI + rot
            } else {
                rot
            }
        } else if rot < -HALF_PI {
            PI + rot
        } else {
            rot
        };

        Some(Label {
            position,
            content,
            rot,
        })
    }

    fn from_parallel(
        lonlat: &LonLatT<f64>,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Option<Self> {
        let m1: Vector3<_> = lonlat.vector();

        let mut t = Vector3::new(-m1.z, 0.0, m1.x).normalize();
        let center = camera.get_center().truncate();

        let dot_t_center = center.dot(t);
        if dot_t_center.abs() < 1e-4 {
            t = -t;
        } else {
            t = dot_t_center.signum() * t;
        }

        let m2 = (m1 + t * 1e-3).normalize();

        let s1 = projection.model_to_screen_space(&m1.extend(1.0), camera)?;
        let s2 = projection.model_to_screen_space(&m2.extend(1.0), camera)?;

        let ds = (s2 - s1).normalize();

        let content = angle::SerializeFmt::DMS.to_string(lonlat.lat());

        let fov = camera.get_field_of_view();
        let position = if !fov.is_allsky() && !fov.contains_pole() {
            let dim = 100.0;
            let k = ds * (dim * 0.5 + 10.0);

            s1 + k
        } else {
            s1
        };

        // rot is between -PI and +PI
        let rot = if ds.y > 0.0 {
            ds.x.acos()
        } else {
            -ds.x.acos()
        };

        let rot = if ds.y > 0.0 && rot > HALF_PI {
            -PI + rot
        } else if ds.y <= 0.0 && rot < -HALF_PI {
            PI + rot
        } else {
            rot
        };

        Some(Label {
            position,
            content,
            rot,
        })
    }
}

#[derive(Debug)]
struct GridLine {
    vertices: Vec<XYNDC>,
    label: Option<Label>,
}
use cgmath::{Rad, Vector3};
//use math::angle::SerializeToString;
const PI: f64 = std::f64::consts::PI;
const HALF_PI: f64 = 0.5 * PI;
use crate::math::{
    angle::ArcDeg,
};

impl GridLine {
    fn meridian(
        lon: f64,
        lat: &Range<f64>,
        sp: Option<&Vector2<f64>>,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        fmt: &angle::SerializeFmt
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        if fov.contains_both_poles() {
            let camera_center = camera.get_center();

            let center_lat = camera_center.lat().to_radians();
            let label_lat = if center_lat > 70.0_f64.to_radians() {
                70.0_f64.to_radians()
            } else if center_lat < -70.0_f64.to_radians() {
                (-70.0_f64).to_radians()
            } else {
                center_lat
            };

            let lonlat = LonLatT::new(
                lon.to_angle(),
                label_lat.to_angle()
            );

            let label = Label::from_meridian(&lonlat, camera, projection, fmt);

            // Draw the full parallel
            let vertices = line::great_circle_arc::project(lon, -HALF_PI, lon, HALF_PI, camera, projection);
            Some(GridLine { vertices, label })
        } else {
            let i = fov.intersects_meridian(lon);
            match i {
                Intersection::Included => {
                    // Longitude fov >= PI
                    let camera_center = camera.get_center();

                    let center_lat = camera_center.lat().to_radians();

                    let lonlat = LonLatT::new(
                        lon.to_angle(),
                        center_lat.to_angle()
                    );

                    let label = Label::from_meridian(&lonlat, camera, projection, fmt);

                    // Draw the full parallel
                    let vertices = line::great_circle_arc::project(lon, -HALF_PI, lon, HALF_PI, camera, projection);
                    Some(GridLine { vertices, label })
                },
                Intersection::Intersect { vertices } => {
                    let num_intersections = vertices.len();
                    let (vertices, label) = match num_intersections {
                        1 => {
                            let v1 = &vertices[0];
                            let lonlat1 = v1.lonlat();
                            let lat1 = lonlat1.lat().to_radians();

                            let line_vertices = if fov.contains_north_pole() {
                                line::great_circle_arc::project(
                                    lon,
                                    lat1,
                                    lon,
                                    HALF_PI,
                                    camera,
                                    projection
                                )
                            } else {
                                line::great_circle_arc::project(
                                    lon,
                                    lat1,
                                    lon,
                                    MINUS_HALF_PI,
                                    camera,
                                    projection
                                )
                            };

                            let label = Label::from_meridian(&lonlat1, camera, projection, fmt);
                            (line_vertices, label)
                        },
                        2 => {
                            // full intersection
                            let v1 = &vertices[0];
                            let v2 = &vertices[1];

                            let lat1 = v1.lat().to_radians();
                            let lat2 = v2.lat().to_radians();

                            let line_vertices = line::great_circle_arc::project(
                                lon,
                                lat1,
                                lon,
                                lat2,
                                camera,
                                projection
                            );

                            let label = Label::from_meridian(&v1.lonlat(), camera, projection, fmt);
                            (line_vertices, label)
                        },
                        _ => {
                            let mut vertices = vertices.into_vec();
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
                            (line_vertices, label)
                        }
                    };

                    Some(GridLine { vertices, label })
                },
                Intersection::Empty => {
                    None
                },
            }
        }
    }

    fn parallel(
        lat: f64,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        if fov.get_bounding_box().get_lon_size() > PI {
            // Longitude fov >= PI
            let camera_center = camera.get_center();
            let center_lon = camera_center.lon();
            let lonlat = LonLatT::new(
                center_lon,
                lat.to_angle()
            );

            let label = Label::from_parallel(&lonlat, camera, projection);

            // Draw the full parallel
            let mut vertices = line::parallel_arc::project(lat, center_lon.to_radians(), center_lon.to_radians() + PI, camera, projection);
            vertices.append(&mut line::parallel_arc::project(lat, center_lon.to_radians() + PI, center_lon.to_radians() + TWICE_PI, camera, projection));
            Some(GridLine { vertices, label })
        } else {
            // Longitude fov < PI
            let i = fov.intersects_parallel(lat);
            match i {
                Intersection::Included => {
                    let camera_center = camera.get_center();
                    let center_lon = camera_center.lon();
                    let lonlat = LonLatT::new(
                        center_lon,
                        lat.to_angle()
                    );

                    let label = Label::from_parallel(&lonlat, camera, projection);

                    // Draw the full parallel
                    let vertices = line::parallel_arc::project(lat, center_lon.to_radians(), center_lon.to_radians() + TWICE_PI, camera, projection);
                    Some(GridLine { vertices, label })
                },
                Intersection::Intersect { vertices } => {
                    let v1 = &vertices[0];
                    let v2 = &vertices[1];

                    let mut lon1 = v1.lon().to_radians();
                    let mut lon2 = v2.lon().to_radians();

                    let lon_len = crate::math::sph_geom::distance_from_two_lon(lon1, lon2);

                    // The fov should be contained into PI length
                    if lon_len >= PI {
                        std::mem::swap(&mut lon1, &mut lon2);
                    }

                    let line_vertices = line::parallel_arc::project(lat, lon1, lon2, camera, projection);
                    let label = Label::from_parallel(&v1.lonlat(), camera, projection);
    
                    Some(GridLine { vertices: line_vertices, label })
                },
                Intersection::Empty => {
                    None
                },
            }
        }
    }
}

const GRID_STEPS: &[f64] = &[
    0.0000000000048481367,
    0.000000000009696274,
    0.000000000024240685,
    0.000000000048481369,
    0.000000000096962737,
    0.00000000024240683,
    0.00000000048481364,
    0.0000000009696274,
    0.0000000024240686,
    0.000000004848138,
    0.000000009696275,
    0.000000024240685,
    0.00000004848138,
    0.00000009696275,
    0.00000024240687,
    0.0000004848138,
    0.0000009696275,
    0.0000024240686,
    0.000004848138,
    0.000009696275,
    0.000024240685,
    0.000048481369,
    0.000072722055,
    0.00014544412,
    0.00029088823,
    0.00058177644,
    0.0014544412,
    0.0029088823,
    0.004363324,
    0.008726647,
    0.017453293,
    0.034906586,
    0.08726647,
    0.17453293,
    0.34906585,
    std::f64::consts::FRAC_PI_4,
];

fn lines(
    camera: &CameraViewPort,
    //text_height: f64,
    projection: &ProjectionType,
    fmt: &angle::SerializeFmt,
) -> Vec<GridLine> {
    // Get the screen position of the nearest pole
    let fov = camera.get_field_of_view();
    let sp = if fov.contains_pole() {
        if fov.contains_north_pole() {
            // Project the pole into the screen
            // This is an information needed
            // for plotting labels
            // screen north pole
            projection.view_to_screen_space(
                //&(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, 1.0, 0.0, 1.0)),
                &Vector4::new(0.0, 1.0, 0.0, 1.0),
                camera,
            )
        } else {
            // screen south pole
            projection.view_to_screen_space(
                //&(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, -1.0, 0.0, 1.0)),
                &Vector4::new(0.0, -1.0, 0.0, 1.0),
                camera,
            )
        }
    } else {
        None
    };

    let bbox = camera.get_field_of_view().get_bounding_box();

    let max_dim_px = camera.get_width().max(camera.get_height()) as f64;
    let step_line_px =  max_dim_px * 0.2;

    let step_lon_precised = (bbox.get_lon_size() as f64) * step_line_px / (camera.get_width() as f64);
    let step_lat_precised = (bbox.get_lat_size() as f64) * step_line_px / (camera.get_height() as f64);

    // Select the good step with a binary search
    let step_lon = select_fixed_step(step_lon_precised);
    let step_lat = select_fixed_step(step_lat_precised);

    let mut lines = vec![];
    // Add meridians
    let mut theta = bbox.lon_min() - (bbox.lon_min() % step_lon);
    let mut stop_theta = bbox.lon_max();
    if bbox.all_lon() {
        stop_theta -= 1e-3;
    }

    while theta < stop_theta {
        if let Some(line) =
            GridLine::meridian(theta, &bbox.get_lat(), sp.as_ref(), camera, projection, fmt)
        {
            lines.push(line);
        }
        theta += step_lon;
    }

    let mut alpha = bbox.lat_min() - (bbox.lat_min() % step_lat);
    if alpha == -HALF_PI {
        alpha += step_lat;
    }
    let stop_alpha = bbox.lat_max();

    while alpha < stop_alpha {
        if let Some(line) = GridLine::parallel(alpha, camera, projection) {
            lines.push(line);
        }
        alpha += step_lat;
    }

    lines
}

/*fn select_grid_step(fov: f64, max_lines: usize) -> f64 {
    // Select the best meridian grid step
    let mut i = 0;
    let mut step = GRID_STEPS[0];
    while i < GRID_STEPS.len() {
        if fov >= GRID_STEPS[i] {
            let num_meridians_in_fov = (fov / GRID_STEPS[i]) as usize;
            if num_meridians_in_fov >= max_lines - 1 {
                //let idx_grid = if i == 0 { 0 } else { i - 1 };
                //step = GRID_STEPS[idx_grid];
                step = GRID_STEPS[i];
                break;
            }
        }

        step = GRID_STEPS[i];
        i += 1;
    }

    step
}*/

fn select_fixed_step(fov: f64) -> f64 {
    match GRID_STEPS.binary_search_by(|v| {
        v.partial_cmp(&fov).expect("Couldn't compare values, maybe because the fov given is NaN")
    }) {
        Ok(idx) => GRID_STEPS[idx],
        Err(idx) => {
            if idx == 0 {
                GRID_STEPS[0]
            } else if idx == GRID_STEPS.len() {
                GRID_STEPS[idx - 1]
            } else {
                let a = GRID_STEPS[idx];
                let b = GRID_STEPS[idx - 1];

                if a - fov > fov - b {
                    b
                } else {
                    a
                }
            }
        }
    }
}
