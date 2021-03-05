use web_sys::WebGl2RenderingContext;

use crate::color::Color;

use crate::renderable::angle;
use cgmath::Vector4;

use crate::camera::CameraViewPort;
use web_sys::{CanvasRenderingContext2d, WebGlBuffer, WebGlVertexArrayObject};

pub struct ProjetedGrid {
    // The color of the grid
    color: Color,

    // The vertex array object of the screen in NDC
    vao: WebGlVertexArrayObject,
    vao_gpu: VertexArrayObject,

    vbo: WebGlBuffer,
    // A pointer over the 2d context where we can write text
    ctx2d: CanvasRenderingContext2d,

    labels: Vec<Option<Label>>,
    size_vertices_buf: usize,
    sizes: Vec<usize>,
    offsets: Vec<usize>,

    num_vertices: usize,

    gl: WebGl2Context,
    enabled: bool,
    hide_labels: bool,
}

use crate::core::{VecData, VertexArrayObject};
use crate::renderable::projection::Projection;
use crate::ShaderManager;
use crate::WebGl2Context;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
impl ProjetedGrid {
    pub fn new<P: Projection>(
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Result<ProjetedGrid, JsValue> {
        let vao_gpu = {
            let mut vao = VertexArrayObject::new(gl);

            let shader = shaders
                .get(
                    &gl,
                    &ShaderId(Cow::Borrowed("GridVS_CPU"), Cow::Borrowed("GridFS_CPU")),
                )
                .unwrap();
            shader
                .bind(gl)
                .bind_vertex_array_object(&mut vao)
                // Store the screen and uv of the billboard in a VBO
                .add_array_buffer(
                    2 * std::mem::size_of::<f32>(),
                    &[2],
                    &[0],
                    WebGl2RenderingContext::STATIC_DRAW,
                    VecData(&vec![-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0]),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STATIC_DRAW,
                    VecData(&vec![0_u16, 1_u16, 2_u16, 0_u16, 2_u16, 3_u16]),
                )
                // Unbind the buffer
                .unbind();
            vao
        };

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        gl.line_width(1.0);
        let data = vec![0.0_f32; 1000];
        let size_vertices_buf = 1000;
        let num_vertices = 0;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let num_bytes_per_f32 = std::mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 ndc_pos;
        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            2 * num_bytes_per_f32,
            (0 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(0);

        let labels = vec![];

        let color = Color::new(0_f32, 1_f32, 0_f32, 0.3_f32);
        let gl = gl.clone();
        let sizes = vec![];
        let offsets = vec![];

        // Get the canvas rendering context
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_elements_by_class_name("aladin-gridCanvas")
            .get_with_index(0)
            .unwrap();
        canvas.set_attribute("style", "z-index:1; position:absolute; top:0; left:0;")?;
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
        let size_screen = camera.get_screen_size();
        canvas.set_width(2 * size_screen.x as u32);
        canvas.set_height(2 * size_screen.y as u32);

        let ctx2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        ctx2d.scale(2.0, 2.0)?;

        let enabled = false;
        let hide_labels = false;
        let grid = ProjetedGrid {
            color,

            vao,
            vbo,

            labels,
            size_vertices_buf,
            num_vertices,

            sizes,
            offsets,

            ctx2d,
            gl,
            enabled,
            hide_labels,
            vao_gpu,
        };
        Ok(grid)
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn enable<P: Projection>(&mut self, camera: &CameraViewPort) {
        self.enabled = true;
        self.force_update::<P>(camera);
    }

    pub fn disable(&mut self, camera: &CameraViewPort) {
        let size_screen = &camera.get_screen_size();

        self.enabled = false;
        self.ctx2d.clear_rect(
            0.0,
            0.0,
            2.0 * size_screen.x as f64,
            2.0 * size_screen.y as f64,
        );
    }

    pub fn hide_labels(&mut self, camera: &CameraViewPort) {
        let size_screen = &camera.get_screen_size();

        self.hide_labels = true;
        self.ctx2d.clear_rect(
            0.0,
            0.0,
            2.0 * size_screen.x as f64,
            2.0 * size_screen.y as f64,
        );
    }
    pub fn show_labels(&mut self) {
        self.hide_labels = false;
    }
    fn force_update<P: Projection>(&mut self, camera: &CameraViewPort) {
        let text_height = Label::size(camera);
        let lines = lines::<P>(camera, &self.ctx2d, text_height);

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
        let mut vertices = vertices
            .into_iter()
            .flatten()
            .map(|v| Vector2::new(v.x as f32, v.y as f32))
            .collect::<Vec<_>>();
        //self.lines = lines;
        self.num_vertices = vertices.len();

        let vertices: Vec<f32> = unsafe {
            vertices.set_len(vertices.len() * 2);
            std::mem::transmute(vertices)
        };

        let buf_vertices = unsafe { js_sys::Float32Array::view(&vertices) };

        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
        if vertices.len() > self.size_vertices_buf {
            self.size_vertices_buf = vertices.len();

            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &buf_vertices,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        } else {
            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                0,
                &buf_vertices,
            );
        }
    }

    // Update the grid whenever the camera moved
    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort, force: bool) {
        if !self.enabled {
            return;
        }

        if camera.has_moved() || force {
            self.force_update::<P>(camera);
        }
    }

    fn draw_lines_gpu<P: Projection>(&self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        let shader = P::get_grid_shader(&self.gl, shaders);

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        let (meridians, parallels) = lines_gpu::<P>(camera);

        shader
            .bind(&self.gl)
            .attach_uniforms_from(camera)
            .attach_uniform("num_meridians", &(meridians.len() as i32))
            .attach_uniform("num_parallels", &(parallels.len() as i32))
            .attach_uniform("meridians[0]", &meridians.as_slice())
            .attach_uniform("parallels[0]", &parallels.as_slice())
            .attach_uniform("color", &self.color)
            // Bind the Vertex Array Object for drawing
            .bind_vertex_array_object_ref(&self.vao_gpu)
            .draw_elements_with_i32(
                // Mode of render
                WebGl2RenderingContext::TRIANGLES,
                // Number of elements, by default None
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT,
            );
    }

    fn draw_lines_cpu<P: Projection>(&self, camera: &CameraViewPort, shaders: &mut ShaderManager) {
        let shader = shaders
            .get(
                &self.gl,
                &ShaderId(Cow::Borrowed("GridVS_CPU"), Cow::Borrowed("GridFS_CPU")),
            )
            .unwrap();

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        shader
            .bind(&self.gl)
            .attach_uniforms_from(camera)
            .attach_uniform("color", &self.color);

        // The raster vao is bound at the lib.rs level
        self.gl.bind_vertex_array(Some(&self.vao));
        for (offset, size) in self.offsets.iter().zip(self.sizes.iter()) {
            if *size > 0 {
                self.gl
                    .draw_arrays(WebGl2RenderingContext::LINES, *offset as i32, *size as i32);
            }
        }
    }

    pub fn draw<P: Projection>(
        &self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        if self.enabled {
            if camera.get_aperture() < ArcDeg(10.0) {
                self.draw_lines_cpu::<P>(camera, shaders);
            } else {
                self.draw_lines_gpu::<P>(camera, shaders);
            }

            // Draw the labels here
            if !self.hide_labels {
                let size_screen = &camera.get_screen_size();
                self.ctx2d.clear_rect(
                    0.0,
                    0.0,
                    2.0 * size_screen.x as f64,
                    2.0 * size_screen.y as f64,
                );

                let text_height = Label::size(camera);
                self.ctx2d
                    .set_font(&format!("{}px Arial", text_height as usize));
                self.ctx2d.set_text_align("center");
                let fill_style: String = (&self.color).into();
                self.ctx2d.set_fill_style(&JsValue::from_str(&fill_style));
                for label in self.labels.iter() {
                    if let Some(label) = &label {
                        self.ctx2d.save();
                        self.ctx2d
                            .translate(label.position.x as f64, label.position.y as f64)?;
                        self.ctx2d.rotate(label.rot as f64)?;
                        self.ctx2d
                            .fill_text(&label.content, 0.0, text_height / 4.0)
                            .unwrap();
                        self.ctx2d.restore();
                    }
                }
            }
        }

        Ok(())
    }
}

use crate::{renderable::projection::*, shader::ShaderId, Shader};
use std::borrow::Cow;
pub trait GridShaderProjection {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
}

impl GridShaderProjection for Aitoff {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridAitoffFS")),
            )
            .unwrap()
    }
}
impl GridShaderProjection for Mollweide {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridMollFS")),
            )
            .unwrap()
    }
}
impl GridShaderProjection for Mercator {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridMercatorFS")),
            )
            .unwrap()
    }
}
impl GridShaderProjection for Orthographic {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridOrthoFS")),
            )
            .unwrap()
    }
}
impl GridShaderProjection for Gnomonic {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridTanFS")),
            )
            .unwrap()
    }
}
impl GridShaderProjection for AzimuthalEquidistant {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("GridVS"), Cow::Borrowed("GridArcFS")),
            )
            .unwrap()
    }
}

use crate::sphere_geometry::BoundingBox;

use cgmath::InnerSpace;
pub const MAX_ANGLE_BEFORE_SUBDIVISION: Angle<f64> = Angle(5.0 * std::f64::consts::PI / 180.0);
fn subdivide<P: Projection>(
    vertices: &mut Vec<Vector2<f64>>,
    lonlat: [(f64, f64); 3],
    depth: usize,
    min_subdivision_level: i32,
    camera: &CameraViewPort,
) {
    // Convert to cartesian
    let system = camera.get_system();
    let a: Vector4<f64> =
        system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle(lonlat[0].0), Angle(lonlat[0].1));
    let b: Vector4<f64> =
        system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle(lonlat[1].0), Angle(lonlat[1].1));
    let c: Vector4<f64> =
        system.to_icrs_j2000::<f64>() * math::radec_to_xyzw(Angle(lonlat[2].0), Angle(lonlat[2].1));

    // Project them. We are always facing the camera
    let a = P::model_to_ndc_space(&a, camera);
    let b = P::model_to_ndc_space(&b, camera);
    let c = P::model_to_ndc_space(&c, camera);
    match (a, b, c) {
        (None, None, None) => {}
        (Some(a), Some(b), Some(c)) => {
            // Compute the angle between a->b and b->c
            let ab = b - a;
            let bc = c - b;
            let ab_l = ab.magnitude2();
            let bc_l = bc.magnitude2();

            let theta = math::angle(&ab.normalize(), &bc.normalize());

            if theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION && min_subdivision_level <= 0 {
                vertices.push(a);
                vertices.push(b);

                vertices.push(b);
                vertices.push(c);
            } else if depth > 0 {
                // Subdivide a->b and b->c
                let lon_d = (lonlat[0].0 + lonlat[1].0) * 0.5;
                let lat_d = (lonlat[0].1 + lonlat[1].1) * 0.5;
                subdivide::<P>(
                    vertices,
                    [lonlat[0], (lon_d, lat_d), lonlat[1]],
                    depth - 1,
                    min_subdivision_level - 1,
                    camera,
                );

                let lon_e = (lonlat[1].0 + lonlat[2].0) * 0.5;
                let lat_e = (lonlat[1].1 + lonlat[2].1) * 0.5;
                subdivide::<P>(
                    vertices,
                    [lonlat[1], (lon_e, lat_e), lonlat[2]],
                    depth - 1,
                    min_subdivision_level - 1,
                    camera,
                );
            } else if ab_l.min(bc_l) / ab_l.max(bc_l) < 0.1 {
                if ab_l == ab_l.min(bc_l) {
                    vertices.push(a);
                    vertices.push(b);
                } else {
                    vertices.push(b);
                    vertices.push(c);
                }
                return;
            }
        }
        (Some(_), None, None) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(
                vertices,
                [
                    lonlat[0],
                    (
                        (lonlat[0].0 + lonlat[1].0) * 0.5,
                        (lonlat[0].1 + lonlat[1].1) * 0.5,
                    ),
                    lonlat[1],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
        }
        (None, None, Some(_)) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(
                vertices,
                [
                    lonlat[1],
                    (
                        (lonlat[1].0 + lonlat[2].0) * 0.5,
                        (lonlat[1].1 + lonlat[2].1) * 0.5,
                    ),
                    lonlat[2],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
        }
        (None, Some(_), None) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(
                vertices,
                [
                    lonlat[0],
                    (
                        (lonlat[0].0 + lonlat[1].0) * 0.5,
                        (lonlat[0].1 + lonlat[1].1) * 0.5,
                    ),
                    lonlat[1],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
            subdivide::<P>(
                vertices,
                [
                    lonlat[1],
                    (
                        (lonlat[1].0 + lonlat[2].0) * 0.5,
                        (lonlat[1].1 + lonlat[2].1) * 0.5,
                    ),
                    lonlat[2],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
        }
        _ => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(
                vertices,
                [
                    lonlat[0],
                    (
                        (lonlat[0].0 + lonlat[1].0) * 0.5,
                        (lonlat[0].1 + lonlat[1].1) * 0.5,
                    ),
                    lonlat[1],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
            subdivide::<P>(
                vertices,
                [
                    lonlat[1],
                    (
                        (lonlat[1].0 + lonlat[2].0) * 0.5,
                        (lonlat[1].1 + lonlat[2].1) * 0.5,
                    ),
                    lonlat[2],
                ],
                depth - 1,
                min_subdivision_level - 1,
                camera,
            );
        }
    }
}
use crate::math::{self};
use crate::sphere_geometry::FieldOfViewType;
use crate::Angle;
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
        ctx2d: &CanvasRenderingContext2d,
        text_height: f64,
    ) -> Option<Self> {
        let system = camera.get_system();

        let LonLatT(_, lat) = &(system.to_gal::<f64>() * camera.get_center()).lonlat();
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

        let d = if fov.contains_north_pole(camera) {
            Vector3::new(0.0, 1.0, 0.0)
        } else if fov.contains_south_pole(camera) {
            Vector3::new(0.0, -1.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };

        let m2 = ((m1.truncate() + d * 1e-3).normalize()).extend(1.0);

        let s1 = P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m1), camera)?;
        if !fov.is_allsky() && fov.contains_pole() {
            // If a pole is contained in the view
            // we will have its screen projected position
            let sp = sp.unwrap();
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
        }
        let s2 = P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m2), camera)?;

        let ds = (s2 - s1).normalize();

        let dv = if ds.x < 0.0 {
            Vector2::new(-ds.y, ds.x)
        } else {
            Vector2::new(ds.y, -ds.x)
        };

        let content = Angle(lon).to_string::<angle::DMS>();
        let mut position = if !fov.is_allsky() {
            let dim = ctx2d.measure_text(&content).unwrap();
            let k = ds * (dim.width() * 0.5 + 10.0);

            s1 + k
        } else {
            s1
        };

        position += dv * text_height * 0.5;

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
        ctx2d: &CanvasRenderingContext2d,
        // in pixels
        text_height: f64,
    ) -> Option<Self> {
        let mut d = Vector3::new(-m1.z, 0.0, m1.x).normalize();
        let system = camera.get_system();
        let center = (system.to_gal::<f64>() * camera.get_center()).truncate();
        if center.dot(d) < 0.0 {
            d = -d;
        }
        let m2 = (m1 + d * 1e-3).normalize();

        let s1 =
            P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m1.extend(1.0)), camera)?;
        let s2 =
            P::model_to_screen_space(&(system.to_icrs_j2000::<f64>() * m2.extend(1.0)), camera)?;

        let ds = (s2 - s1).normalize();

        let dv = if ds.x >= 0.0 && ds.y <= 0.0 {
            Vector2::new(ds.y, -ds.x)
        } else if ds.x >= 0.0 && ds.y >= 0.0 {
            Vector2::new(ds.y, -ds.x)
        } else if ds.x <= 0.0 && ds.y <= 0.0 {
            Vector2::new(-ds.y, ds.x)
        } else {
            Vector2::new(-ds.y, ds.x)
        };

        let content = Angle(lat).to_string::<angle::DMS>();
        let mut position = if !fov.is_allsky() && !fov.contains_pole() {
            let dim = ctx2d.measure_text(&content).unwrap();
            let k = ds * (dim.width() * 0.5 + 10.0);
            //let k = Vector2::new(0.0, 0.0);

            s1 + k
        } else {
            s1
        };
        position += dv * text_height * 0.5;

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

    /// Size of the label in pixels
    fn size(camera: &CameraViewPort) -> f64 {
        let ndc1 =
            crate::renderable::projection::clip_to_ndc_space(&Vector2::new(-1.0, 0.0), camera);
        let ndc2 =
            crate::renderable::projection::clip_to_ndc_space(&Vector2::new(1.0, 0.0), camera);

        let dx = ndc2.x - ndc1.x;
        let allsky = dx < 2.0;

        let ss = camera.get_screen_size();
        let size_not_allsky = ((ss.y.max(ss.x) as f64) * 0.1).min(13.0);

        if allsky {
            let dw = dx / 2.0; // [0..1]
            (dw * size_not_allsky).max(10.0)
        } else {
            size_not_allsky
        }
    }
}

#[derive(Debug)]
struct GridLine {
    vertices: Vec<Vector2<f64>>,
    label: Option<Label>,
}
use super::angle::SerializeToString;
use cgmath::{Rad, Vector3};
const PI: f64 = std::f64::consts::PI;
const HALF_PI: f64 = 0.5 * PI;
use crate::math::LonLat;
use crate::{ArcDeg, LonLatT};
impl GridLine {
    fn meridian<P: Projection>(
        ctx2d: &CanvasRenderingContext2d,
        lon: f64,
        lat: &Range<f64>,
        sp: Option<&Vector2<f64>>,
        camera: &CameraViewPort,
        text_height: f64,
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();
        let mut vertices = vec![];
        subdivide::<P>(
            &mut vertices,
            [
                (lon, lat.start),
                (lon, (lat.start + lat.end) * 0.5),
                (lon, lat.end),
            ],
            7,
            2,
            camera,
        );

        /*let too_unzoomed = camera.get_aperture() > ArcDeg(720.0);
        let label = if !too_unzoomed {
            Label::meridian::<P>(fov, lon, &p, camera, sp, ctx2d)
        } else {
            None
        };*/
        let p = (fov.intersect_meridian(Rad(lon), camera)?).extend(1.0);
        let label = Label::meridian::<P>(fov, lon, &p, camera, sp, ctx2d, text_height);

        Some(GridLine { vertices, label })
    }

    fn parallel<P: Projection>(
        ctx2d: &CanvasRenderingContext2d,
        lon: &Range<f64>,
        lat: f64,
        camera: &CameraViewPort,
        text_height: f64,
    ) -> Option<Self> {
        let fov = camera.get_field_of_view();

        if let Some(p) = fov.intersect_parallel(Rad(lat), camera) {
            let mut vertices = vec![];
            subdivide::<P>(
                &mut vertices,
                [
                    (lon.start, lat),
                    (0.5 * (lon.start + lon.end), lat),
                    (lon.end, lat),
                ],
                7,
                2,
                camera,
            );

            /*let too_unzoomed = camera.get_aperture() > ArcDeg(720.0);
            let label = if !too_unzoomed {
                Label::parallel::<P>(fov, lat, &p, camera, ctx2d)
            } else {
                None
            };*/
            let label = Label::parallel::<P>(fov, lat, &p, camera, ctx2d, text_height);

            Some(GridLine { vertices, label })
        } else {
            None
        }
    }
}

const GRID_STEPS: &[f64] = &[
    0.69813168,
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

fn lines_gpu<P: Projection>(camera: &CameraViewPort) -> (Vec<f64>, Vec<f64>) {
    let bbox = camera.get_bounding_box();
    let fov = camera.get_field_of_view();

    /*let num_max_lines = ((NUM_MIN_LINES as f32) * camera.get_aspect()) as usize;

    let c1 = camera.get_center().truncate();
    let c2 = (c1 + Vector3::new(0.0, 0.0, 1e-3)).normalize();
    let ndcc = P::model_to_ndc_space(&c2.extend(1.0), camera).unwrap();
    let d = ndcc.normalize();

    let a1 = d.x.abs() as f32;
    let a2 = d.y.abs() as f32;

    let num_lines_lon = (a1 * (num_max_lines as f32)  + (1.0 - a1) * (NUM_MIN_LINES as f32)) as usize;
    let num_lines_lat = ((1.0 - a1) * (num_max_lines as f32)  + a1 * (NUM_MIN_LINES as f32)) as usize;*/

    let step_lon = select_grid_step(
        &bbox,
        bbox.get_lon_size().0 as f64,
        //(NUM_LINES_LATITUDES as f64 * (camera.get_aspect() as f64)) as usize,
        //((NUM_LINES_LATITUDES as f64) * fs.0) as usize
        NUM_LINES,
    );

    // Add meridians
    let mut theta = bbox.lon_min().0 - (bbox.lon_min().0 % step_lon);

    let mut stop_theta = bbox.lon_max().0;
    if bbox.all_lon() {
        stop_theta -= 1e-3;
    }
    let mut meridians = vec![];
    while theta < stop_theta {
        meridians.push(theta);
        theta += step_lon;
    }

    // Add parallels
    let step_lat = if fov.contains_pole() {
        select_grid_step(&bbox, bbox.get_lat_size().0 as f64, NUM_LINES)
    } else {
        select_grid_step(&bbox, bbox.get_lat_size().0 as f64, NUM_LINES)
    };
    let mut alpha = bbox.lat_min().0 - (bbox.lat_min().0 % step_lat);
    if alpha == -HALF_PI {
        alpha += step_lat;
    }
    let mut stop_alpha = bbox.lat_max().0;
    if stop_alpha == HALF_PI {
        stop_alpha -= 1e-3;
    }
    let mut parallels = vec![];
    while alpha < stop_alpha {
        parallels.push(alpha);
        alpha += step_lat;
    }

    (meridians, parallels)
}

const NUM_LINES: usize = 4;
fn lines<P: Projection>(
    camera: &CameraViewPort,
    ctx2d: &CanvasRenderingContext2d,
    text_height: f64,
) -> Vec<GridLine> {
    // Get the screen position of the nearest pole
    let system = camera.get_system();
    let fov = camera.get_field_of_view();
    let sp = if fov.contains_pole() {
        if fov.contains_north_pole(camera) {
            // Project the pole into the screen
            // This is an information needed
            // for plotting labels
            // screen north pole
            if let Some(snp) = P::model_to_screen_space(
                &(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, 1.0, 0.0, 1.0)),
                camera,
            ) {
                Some(snp)
            } else {
                None
            }
        } else {
            // screen south pole
            if let Some(ssp) = P::model_to_screen_space(
                &(system.to_icrs_j2000::<f64>() * Vector4::new(0.0, -1.0, 0.0, 1.0)),
                camera,
            ) {
                Some(ssp)
            } else {
                None
            }
        }
    } else {
        None
    };

    let bbox = camera.get_bounding_box();
    let fov = camera.get_field_of_view();

    /*let num_max_lines = ((NUM_MIN_LINES as f32) * camera.get_aspect()) as usize;

    let c1 = camera.get_center().truncate();
    let c2 = (c1 + Vector3::new(0.0, 0.0, 1e-3)).normalize();
    let ndcc = P::model_to_ndc_space(&c2.extend(1.0), camera).unwrap();
    let d = ndcc.normalize();

    let a1 = d.x.abs() as f32;
    let a2 = d.y.abs() as f32;

    let num_lines_lon = (a1 * (num_max_lines as f32)  + (1.0 - a1) * (NUM_MIN_LINES as f32)) as usize;
    debug!(a1);
    debug!(num_max_lines);
    debug!(NUM_MIN_LINES);
    let num_lines_lat = ((1.0 - a1) * (num_max_lines as f32)  + a1 * (NUM_MIN_LINES as f32)) as usize;*/

    let step_lon = select_grid_step(
        &bbox,
        bbox.get_lon_size().0 as f64,
        //(NUM_LINES_LATITUDES as f64 * (camera.get_aspect() as f64)) as usize,
        //((NUM_LINES_LATITUDES as f64) * fs.0) as usize
        NUM_LINES,
    );

    let mut lines = vec![];
    // Add meridians
    let mut theta = bbox.lon_min().0 - (bbox.lon_min().0 % step_lon);
    let mut stop_theta = bbox.lon_max().0;
    if bbox.all_lon() {
        stop_theta -= 1e-3;
    }

    while theta < stop_theta {
        if let Some(line) = GridLine::meridian::<P>(
            ctx2d,
            theta,
            &bbox.get_lat(),
            sp.as_ref(),
            camera,
            text_height,
        ) {
            lines.push(line);
        }
        theta += step_lon;
    }

    // Add parallels
    let step_lat = if fov.contains_pole() {
        select_grid_step(&bbox, bbox.get_lat_size().0 as f64, NUM_LINES)
    } else {
        select_grid_step(&bbox, bbox.get_lat_size().0 as f64, NUM_LINES)
    };
    let mut alpha = bbox.lat_min().0 - (bbox.lat_min().0 % step_lat);
    if alpha == -HALF_PI {
        alpha += step_lat;
    }
    let mut stop_alpha = bbox.lat_max().0;
    if stop_alpha == HALF_PI {
        stop_alpha -= 1e-3;
    }

    while alpha < stop_alpha {
        if let Some(line) =
            GridLine::parallel::<P>(ctx2d, &bbox.get_lon(), alpha, camera, text_height)
        {
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
