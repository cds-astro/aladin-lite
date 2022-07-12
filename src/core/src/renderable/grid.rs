
use web_sys::WebGl2RenderingContext;

use crate::math::angle;
use cgmath::Vector4;

use crate::camera::CameraViewPort;

use al_api::grid::GridCfg;
use al_core::VertexArrayObject;
pub struct ProjetedGrid {
    // Properties
    pub color: Color,
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
}

use crate::math::projection::Projection;
use crate::shader::ShaderManager;
use al_core::VecData;
use al_core::WebGlContext;
use wasm_bindgen::JsValue;

use super::labels::RenderManager;
use al_api::color::Color;

use super::TextRenderManager;

impl ProjetedGrid {
    pub fn new<P: Projection>(
        gl: &WebGlContext,
        camera: &CameraViewPort,
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

        let text_renderer = TextRenderManager::new(gl.clone(), camera)?;

        let color = Color::new(0.0, 1.0, 0.0, 1.0);
        let show_labels = true;
        let enabled = false;
        let label_scale = 1.0;

        let mut grid = ProjetedGrid {
            color,
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
        };
        // Initialize the vertices & labels
        grid.force_update::<P>(camera);

        Ok(grid)
    }

    pub fn set_cfg<P: Projection>(&mut self, new_cfg: GridCfg, camera: &CameraViewPort) -> Result<(), JsValue> {
        let GridCfg {
            color,
            show_labels,
            label_size,
            enabled,
        } = new_cfg;

        if let Some(color) = color {
            self.color = color;
        }

        if let Some(show_labels) = show_labels {
            self.show_labels = show_labels;
        }

        if let Some(enabled) = enabled {
            self.enabled = enabled;
            if enabled {
                self.force_update::<P>(camera);
            }
        }

        if let Some(label_size) = label_size {
            self.label_scale = label_size;

            self.text_renderer.set_text_size(label_size)?;
        }

        self.text_renderer.begin_frame();
        for label in self.labels.iter() {
            if let Some(label) = label {
                self.text_renderer.add_label(
                    &label.content,
                    &label.position.cast::<f32>().unwrap(),
                    1.0,
                    &self.color,
                    cgmath::Rad(label.rot as f32),
                );
            }
        }
        self.text_renderer.end_frame();

        Ok(())
    }

    fn force_update<P: Projection>(&mut self, camera: &CameraViewPort) {
        self.text_renderer.begin_frame();
        //let text_height = text_renderer.text_size();
        let lines = lines::<P>(camera, &self.text_renderer);

        self.offsets.clear();
        self.sizes.clear();
        let (vertices, labels): (Vec<Vec<Vector2<f64>>>, Vec<Option<Label>>) = lines
            .into_iter()
            .map(|line| {
                if self.sizes.is_empty() {
                    self.offsets.push(0);
                } else {
                    let last_offset = *self.offsets.last().unwrap();
                    self.offsets.push(last_offset + self.sizes.last().unwrap());
                }
                self.sizes.push(line.vertices.len());

                (line.vertices, line.label)
            })
            .unzip();
        self.labels = labels;

        for label in self.labels.iter() {
            if let Some(label) = label {
                self.text_renderer.add_label(
                    &label.content,
                    &label.position.cast::<f32>().unwrap(),
                    1.0,
                    &self.color,
                    cgmath::Rad(label.rot as f32),
                );
            }
        }

        let mut vertices = vertices
            .into_iter()
            .flatten()
            .map(|v| Vector2::new(v.x as f32, v.y as f32))
            .collect::<Vec<_>>();
        //self.lines = lines;
        self.num_vertices = vertices.len();

        /*let vertices = unsafe {
            let len = vertices.len() << 1;
            let cap = len;

            Vec::from_raw_parts(vertices.as_mut_ptr() as *mut f32, len, cap)
        };*/
        let vertices = unsafe {
            vertices.set_len(self.num_vertices << 1);
            std::mem::transmute::<_, Vec<f32>>(vertices)
        };

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
    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort) {
        if !self.enabled {
            return;
        }

        self.force_update::<P>(camera);
    }

    fn draw_lines_cpu<P: Projection>(&self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
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
            .unwrap();
        let shader = shader.bind(&self.gl);
        shader
            .attach_uniforms_from(camera)
            .attach_uniform("color", &self.color);

        // The raster vao is bound at the lib.rs level
        let drawer = shader.bind_vertex_array_object_ref(&self.vao);
        for (offset, size) in self.offsets.iter().zip(self.sizes.iter()) {
            if *size > 0 {
                drawer.draw_arrays(WebGl2RenderingContext::LINES, *offset as i32, *size as i32);
            }
        }
    }

    pub fn draw<P: Projection>(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        if self.enabled {
            self.gl.enable(WebGl2RenderingContext::BLEND);
            self.draw_lines_cpu::<P>(camera, shaders);

            self.gl.disable(WebGl2RenderingContext::BLEND);

            if self.show_labels {
                self.text_renderer.draw(camera)?;
            }
        }

        Ok(())
    }
}

use crate::shader::ShaderId;

use std::borrow::Cow;

use crate::math::{
    angle::Angle,
    spherical::{BoundingBox, FieldOfViewType},
};
use cgmath::InnerSpace;
use cgmath::Vector2;
use core::ops::Range;

#[derive(Debug)]
struct Label {
    position: Vector2<f64>,
    content: String,
    rot: f64,
}
impl Label {
    fn meridian<P: Projection>(
        fov: &FieldOfViewType,
        lon: f64,
        m1: &Vector4<f64>,
        camera: &CameraViewPort,
        sp: Option<&Vector2<f64>>,
        text_renderer: &TextRenderManager,
    ) -> Option<Self> {
        let _system = camera.get_system();

        //let LonLatT(_, lat) = &(system.to_gal::<f64>() * camera.get_center()).lonlat();
        let LonLatT(.., lat) = camera.get_center().lonlat();
        // Do not plot meridian labels when the center of fov
        // is above 80deg
        if fov.is_allsky() {
            // If allsky label plotting mode
            // check if we are not too near of a pole
            // If so, do not plot the meridian labels because
            // they can overlap
            if lat.abs() > ArcDeg(80.0) {
                return None;
            }
        }

        let d = if fov.contains_north_pole() {
            Vector3::new(0.0, 1.0, 0.0)
        } else if fov.contains_south_pole() {
            Vector3::new(0.0, -1.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let m2 = ((m1.truncate() + d * 1e-3).normalize()).extend(1.0);

        //let s1 = P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m1), camera, reversed_longitude)?;
        let s1 = P::model_to_screen_space(&m1, camera)?;

        if !fov.is_allsky() && fov.contains_pole() {
            // If a pole is contained in the view
            // we will have its screen projected position
            if let Some(sp) = sp {
                // Distance factor between the label position
                // and the nearest pole position
                let dy = sp.y - s1.y;
                let dx = sp.x - s1.x;
                let dd2 = dx * dx + dy * dy;
                let ss = camera.get_screen_size();
                let ds2 = (ss.x * ss.x + ss.y * ss.y) as f64;
                // This distance is divided by the size of the
                // screen diagonal to be pixel agnostic
                let fdd2 = dd2 / ds2;
                if fdd2 < 0.004 {
                    return None;
                }
            } else {
                return None;
            }
        }

        let s2 = P::model_to_screen_space(&m2, camera)?;

        //let s2 = P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m2), camera, reversed_longitude)?;

        let ds = (s2 - s1).normalize();

        let content = Angle(lon).to_string::<angle::DMS>();
        let position = if !fov.is_allsky() {
            //let dim = ctx2d.measure_text(&content).unwrap();
            let dim = text_renderer.get_width_pixel_size(&content);
            let k = ds * (dim * 0.5 + 10.0);

            s1 + k
        } else {
            s1
        };

        //position += dv * text_height * 0.5;

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

    fn parallel<P: Projection>(
        fov: &FieldOfViewType,
        lat: f64,
        m1: &Vector3<f64>,
        camera: &CameraViewPort,
        // in pixels
        text_renderer: &TextRenderManager,
    ) -> Option<Self> {
        let mut d = Vector3::new(-m1.z, 0.0, m1.x).normalize();
        let _system = camera.get_system();
        let center = camera.get_center().truncate();
        //let center = (system.to_gal::<f64>() * camera.get_center()).truncate();
        if center.dot(d) < 0.0 {
            d = -d;
        }
        let m2 = (m1 + d * 1e-3).normalize();

        let s1 =
            //P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m1.extend(1.0)), camera, reversed_longitude)?;
            P::model_to_screen_space(&m1.extend(1.0), camera)?;
        let s2 =
            //P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m2.extend(1.0)), camera, reversed_longitude)?;
            P::model_to_screen_space(&m2.extend(1.0), camera)?;

        let ds = (s2 - s1).normalize();

        let content = Angle(lat).to_string::<angle::DMS>();
        let position = if !fov.is_allsky() && !fov.contains_pole() {
            let dim = text_renderer.get_width_pixel_size(&content);
            let k = ds * (dim * 0.5 + 10.0);
            //let k = Vector2::new(0.0, 0.0);

            s1 + k
        } else {
            s1
        };
        //position += dv * text_height * 0.5;

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

    /*fn size(camera: &CameraViewPort) -> f64 {
        let ndc1 =
            crate::projection::clip_to_ndc_space(&Vector2::new(-1.0, 0.0), camera);
        let ndc2 =
            crate::projection::clip_to_ndc_space(&Vector2::new(1.0, 0.0), camera);

        let dx = ndc2.x - ndc1.x;
        let allsky = dx < 2.0;

        if allsky {
            let dw = dx / 2.0; // [0..1]
            dw.max(0.75)
        } else {
            1.0
        }
    }*/
}

#[derive(Debug)]
struct GridLine {
    vertices: Vec<Vector2<f64>>,
    label: Option<Label>,
}
use cgmath::{Rad, Vector3};
use math::angle::SerializeToString;
const PI: f64 = std::f64::consts::PI;
const HALF_PI: f64 = 0.5 * PI;
use crate::math::{
    self,
    angle::ArcDeg,
    lonlat::{LonLat, LonLatT},
};
impl GridLine {
    fn meridian<P: Projection>(
        lon: f64,
        lat: &Range<f64>,
        sp: Option<&Vector2<f64>>,
        camera: &CameraViewPort,
        //text_height: f64,
        text_renderer: &TextRenderManager,
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        let mut vertices = vec![];

        let _system = camera.get_system();
        let a = Vector2::new(lon, lat.start);
        let c = Vector2::new(lon, lat.end);
        let b = (a + c) * 0.5;

        crate::line::subdivide_along_longitude_and_latitudes::<P>(
            &mut vertices,
            [&a, &b, &c],
            camera,
        );

        let p = (fov.intersect_meridian(Rad(lon), camera)?).extend(1.0);
        let label = Label::meridian::<P>(fov, lon, &p, camera, sp, text_renderer);

        Some(GridLine { vertices, label })
    }

    fn parallel<P: Projection>(
        lon: &Range<f64>,
        lat: f64,
        camera: &CameraViewPort,
        text_renderer: &TextRenderManager,
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();

        if let Some(p) = fov.intersect_parallel(Rad(lat), camera) {
            let mut vertices = vec![];
            let _system = camera.get_system();

            //let a = (system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle(lon.start), Angle(lat))).truncate();
            let _a = math::lonlat::radec_to_xyz(Angle(lon.start), Angle(lat));
            //let b = (system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle((lon.start + lon.end)*0.5), Angle(lat))).truncate();
            let _b = math::lonlat::radec_to_xyz(Angle((lon.start + lon.end) * 0.5), Angle(lat));
            //let c = (system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle(lon.end), Angle(lat))).truncate();
            let _c = math::lonlat::radec_to_xyz(Angle(lon.end), Angle(lat));

            crate::line::subdivide_along_longitude_and_latitudes::<P>(
                &mut vertices,
                [
                    &Vector2::new(lon.start, lat),
                    &Vector2::new(0.5 * (lon.start + lon.end), lat),
                    &Vector2::new(lon.end, lat),
                ],
                camera,
            );

            let label = Label::parallel::<P>(fov, lat, &p, camera, text_renderer);

            Some(GridLine { vertices, label })
        } else {
            None
        }
    }
}

const GRID_STEPS: &[f64] = &[
    0.78539816339,
    0.34906584,
    0.17453292,
    0.08726646,
    0.034906585,
    0.017453292,
    0.008726646,
    0.004363323,
    0.0029088822,
    0.0014544411,
    0.00058177643,
    0.00029088822,
    0.00014544411,
    0.000072722054,
    0.000048481368,
    0.000024240684,
    0.000009696274,
    0.000004848137,
    0.0000024240685,
    0.0000009696274,
    0.0000004848137,
    0.00000024240686,
    0.00000009696274,
    0.00000004848137,
    0.000000024240684,
    0.000000009696274,
    0.000000004848137,
    0.0000000024240685,
    0.0000000009696273,
    0.00000000048481363,
    0.00000000024240682,
    0.000000000096962736,
    0.000000000048481368,
    0.000000000024240684,
    0.000000000009696273,
    0.0000000000048481366,
];

const NUM_LINES: usize = 4;
fn lines<P: Projection>(
    camera: &CameraViewPort,
    //text_height: f64,
    text_renderer: &TextRenderManager,
) -> Vec<GridLine> {
    // Get the screen position of the nearest pole
    let _system = camera.get_system();
    let fov = camera.get_field_of_view();
    let sp = if fov.contains_pole() {
        if fov.contains_north_pole() {
            // Project the pole into the screen
            // This is an information needed
            // for plotting labels
            // screen north pole
            P::view_to_screen_space(
                //&(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, 1.0, 0.0, 1.0)),
                &Vector4::new(0.0, 1.0, 0.0, 1.0),
                camera,
            )
        } else {
            // screen south pole
            P::view_to_screen_space(
                //&(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, -1.0, 0.0, 1.0)),
                &Vector4::new(0.0, -1.0, 0.0, 1.0),
                camera,
            )
        }
    } else {
        None
    };

    let bbox = camera.get_bounding_box();

    let step_lon = select_grid_step(
        bbox,
        bbox.get_lon_size() as f64,
        //(NUM_LINES_LATITUDES as f64 * (camera.get_aspect() as f64)) as usize,
        //((NUM_LINES_LATITUDES as f64) * fs.0) as usize
        NUM_LINES,
    );

    let mut lines = vec![];
    // Add meridians
    let mut theta = bbox.lon_min() - (bbox.lon_min() % step_lon);
    let mut stop_theta = bbox.lon_max();
    if bbox.all_lon() {
        stop_theta -= 1e-3;
    }

    while theta < stop_theta {
        if let Some(line) =
            GridLine::meridian::<P>(theta, &bbox.get_lat(), sp.as_ref(), camera, text_renderer)
        {
            lines.push(line);
        }
        theta += step_lon;
    }

    // Add parallels
    let step_lat = select_grid_step(bbox, bbox.get_lat_size() as f64, NUM_LINES);
    let mut alpha = bbox.lat_min() - (bbox.lat_min() % step_lat);
    if alpha == -HALF_PI {
        alpha += step_lat;
    }
    let stop_alpha = bbox.lat_max();
    /*if stop_alpha == HALF_PI {
        stop_alpha -= 1e-3;
    }*/

    while alpha < stop_alpha {
        if let Some(line) = GridLine::parallel::<P>(&bbox.get_lon(), alpha, camera, text_renderer) {
            lines.push(line);
        }
        alpha += step_lat;
    }

    lines
}

fn select_grid_step(_bbox: &BoundingBox, fov: f64, max_lines: usize) -> f64 {
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
}
