use crate::core::{
 VecData,
 VertexArrayObject
};

use web_sys::WebGl2RenderingContext;

use crate::color::Color;

use cgmath::Vector4;
use crate::renderable::angle;
use crate::renderable::TextManager;

use crate::camera::CameraViewPort;
use web_sys::{WebGlVertexArrayObject, WebGlBuffer, CanvasRenderingContext2d};

pub struct ProjetedGrid {
    // The color of the grid
    color: Color,
    opacity: f32,

    // The vertex array object of the screen in NDC
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,
    // A pointer over the 2d context where we can write text
    ctx2d: CanvasRenderingContext2d,

    labels: Vec<Label>,
    size_vertices_buf: usize,
    sizes: Vec<usize>,
    offsets: Vec<usize>,

    num_vertices: usize,

    gl: WebGl2Context,
}

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
        //_text_manager: &TextManager
    ) -> ProjetedGrid {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        gl.line_width(1.0);
        let data = vec![0.0_f32; 1000];
        let size_vertices_buf = 1000;
        let num_vertices = 0;
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&data) },
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        let num_bytes_per_f32 = std::mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 ndc_pos;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 2 * num_bytes_per_f32, (0 * num_bytes_per_f32) as i32);
        gl.enable_vertex_attrib_array(0);

        let labels = vec![];
        /*let vertex_array_object = {
            let mut vao = VertexArrayObject::new(gl);

            let shader = shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("GridVS"),
                    Cow::Borrowed("GridOrthoFS"),
                )
            ).unwrap();
            shader.bind(gl)
                .bind_vertex_array_object(&mut vao)
                    // Store the screen and uv of the billboard in a VBO
                    .add_array_buffer(
                        2 * std::mem::size_of::<f32>(),
                        &[2],
                        &[0],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(
                            &vec![
                                -1.0, -1.0,
                                1.0, -1.0,
                                1.0, 1.0,
                                -1.0, 1.0,
                            ]
                        ),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(
                            &vec![
                                0_u16, 1_u16, 2_u16,
                                0_u16, 2_u16, 3_u16,
                            ]
                        ),
                    )
                    // Unbind the buffer
                    .unbind();
            vao
        };*/

        let color = Color::new(0_f32, 1_f32, 0_f32, 0.2_f32);
        let gl = gl.clone();
        let opacity = 0.4;
        let sizes = vec![];
        let offsets = vec![];

        // Get the canvas rendering context
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_elements_by_class_name("aladin-gridCanvas")
            .get_with_index(0)
            .unwrap();
        canvas.set_attribute("style", "z-index:1; position:absolute; top:0; left:0;");
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let size_screen = camera.get_screen_size();
        canvas.set_width(size_screen.x as u32);
        canvas.set_height(size_screen.y as u32);

        let ctx2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        ProjetedGrid {
            color,
            opacity,

            vao,
            vbo,

            labels,
            size_vertices_buf,
            num_vertices,

            sizes,
            offsets,

            ctx2d,
            gl
        }
    }
    
    // Update the grid whenever the camera moved
    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort) {
        if camera.has_moved() {
            let lines = lines::<P>(camera);

            /*let num_lines = lines.len();
            let num_vertices: usize = lines.iter().fold(0, |mut sum, line| {
                sum += line.vertices.len();
                sum
            });*/
            self.offsets.clear();
            self.sizes.clear();
            let (mut vertices, labels): (Vec<Vec<Vector2<f32>>>, Vec<Label>) = lines
                .into_iter()
                .map(|line| {
                    if self.sizes.is_empty() {
                        self.offsets.push(0);
                    } else {
                        let last_offset = self.offsets.last().unwrap();
                        self.offsets.push(last_offset + self.sizes.last().unwrap());
                    }
                    self.sizes.push(line.vertices.len());

                    (line.vertices, line.label)
                })
                .unzip();
            self.labels = labels;
            let mut vertices = vertices.into_iter().flatten().collect::<Vec<_>>();
            //self.lines = lines;
            self.num_vertices = vertices.len();

            let vertices: Vec<f32> = unsafe {
                /*crate::log(&format!("sum {:?}", vertices));

                let ptr = vertices.as_mut_ptr() as *mut f32;
                let len = vertices.len() << 1;

                Vec::from_raw_parts(ptr, len, len)*/
                vertices.set_len(vertices.len() * 2);
                std::mem::transmute(vertices)
            };

            let buf_vertices = unsafe { js_sys::Float32Array::view(&vertices) };

            self.gl.bind_vertex_array(Some(&self.vao));
            self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo));
            if vertices.len() > self.size_vertices_buf {
                self.size_vertices_buf =  vertices.len();
                //crate::log(&format!("realloc num floats: {}", self.size_vertices_buf));
    
                self.gl.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    &buf_vertices,
                    WebGl2RenderingContext::DYNAMIC_DRAW
                );
            } else {
                self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &buf_vertices
                );
            }
        }
    }

    /*pub fn update_label_positions<P: Projection>(&mut self, gl: &WebGl2Context, text_manager: &mut TextManager, camera: &CameraViewPort, shaders: &ShaderManager) {
        if !camera.is_camera_updated() {
            return;
        }
        
        let great_circles = camera.get_great_circles_inside();
        let labels = great_circles.get_labels::<angle::DMS>();

        for (content, pos_world_space) in labels {
            let pos_world_space = Vector4::new(
                pos_world_space.x,
                pos_world_space.y,
                pos_world_space.z,
                1_f32
            );

            text_manager.add_text_on_sphere::<P>(&pos_world_space, &content, camera);
        }

        // Update the VAO
        text_manager.update();
    }*/

    pub fn draw<P: Projection>(
        &self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        let shader = shaders.get(
            &self.gl,
            &ShaderId(
                Cow::Borrowed("GridVS_CPU"),
                Cow::Borrowed("GridFS_CPU"),
            )
        ).unwrap();

        shader.bind(&self.gl)
            .attach_uniforms_from(camera)
            .attach_uniform("color", &self.color)
            .attach_uniform("opacity", &self.opacity);
        //crate::log("raster");
        // The raster vao is bound at the lib.rs level
        self.gl.bind_vertex_array(Some(&self.vao));
        for (offset, size) in self.offsets.iter().zip(self.sizes.iter()) {
            self.gl.draw_arrays(
                WebGl2RenderingContext::LINES,
                *offset as i32,
                *size as i32
            );
        }

        // Draw the labels here
        let size_screen = &camera.get_screen_size();
        self.ctx2d.clear_rect(
            0.0, 0.0,
            size_screen.x as f64, size_screen.y as f64
        );
        self.ctx2d.set_fill_style(&JsValue::from_str("green"));
        self.ctx2d.set_font("30px Verdana");
        let text_height = 30.0;
        self.ctx2d.set_text_align("center");
        for label in self.labels.iter() {
            let dim = self.ctx2d.measure_text(&label.content)?;
            //let to_center = (size_screen/2.0) - label.position;
            self.ctx2d.save();
            //self.ctx2d.translate(label.position.x as f64 - dim.width() / 2.0, label.position.y as f64 + (text_height/2.0));
            let k = Vector2::new(label.rot.cos() as f64, label.rot.sin() as f64) * (dim.width() * 0.5 + 10.0);

            self.ctx2d.translate(label.position.x as f64 + k.x, label.position.y as f64 + k.y);
            let a = label.rot;
            let rot = if k.y > 0.0 {
                if a > HALF_PI {
                    -PI + a
                } else {
                    a
                }
            } else {
                if a < -HALF_PI {
                    PI + a
                } else {
                    a
                }
            };

            self.ctx2d.rotate(rot as f64);
            self.ctx2d.fill_text(&label.content, 0.0, text_height / 4.0).unwrap();
            self.ctx2d.restore();
            //self.ctx2d.fill_text(&label.content, label.position.x as f64 - dim.width() / 2.0 + (to_center.x as f64 * 0.05), label.position.y as f64 + text_height/2.0 + (to_center.y as f64 * 0.05)).unwrap();
        }

        Ok(())

        //if camera.is_allsky() {
            /*let shader = P::get_grid_shader(gl, shaders);
            //let great_circles = camera.get_great_circles_inside();
    
            shader.bind(gl)
                // Attach all the uniforms from the camera
                .attach_uniforms_from(camera)
                // Attach grid specialized uniforms
                .attach_uniform("grid_color", &self.color)
                .attach_uniform("model2world", camera.get_m2w())
                .attach_uniform("world2model", camera.get_w2m());
                //.attach_uniforms_from(great_circles)
                // Bind the Vertex Array Object for drawing
            gl.bind_vertex_array(Some(&self.vertex_array_object));
            gl.draw_elements_with_i32(
                // Mode of render
                WebGl2RenderingContext::TRIANGLES,
                // Number of elements, by default None
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT
            );*/
        /*} else {

        }*/
    }
}

use crate::{
    Shader,
    renderable::projection::*,
    shader::ShaderId
};
use std::borrow::Cow;
pub trait GridShaderProjection {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
}

impl GridShaderProjection for Aitoff {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridAitoffFS"),
            )
        ).unwrap()
    }
}
impl GridShaderProjection for Mollweide {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridMollFS"),
            )
        ).unwrap()
    }
}
impl GridShaderProjection for AzimuthalEquidistant {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridOrthoFS"),
            )
        ).unwrap()
    }
}
impl GridShaderProjection for Gnomonic {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridOrthoFS"),
            )
        ).unwrap()
    }
}
impl GridShaderProjection for Mercator {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridMercatorFS"),
            )
        ).unwrap()
    }
}
impl GridShaderProjection for Orthographic {
    fn get_grid_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("GridVS"),
                Cow::Borrowed("GridOrthoFS"),
            )
        ).unwrap()
    }
}

use crate::sphere_geometry::{FieldOfViewType, BoundingBox};

use cgmath::InnerSpace;
const MAX_ANGLE_BEFORE_SUBDIVISION: f32 = 5.0 * std::f32::consts::PI / 180.0;
fn subdivide<P: Projection>(vertices: &mut Vec<Vector2<f32>>, lonlat: [(f32, f32); 3], depth: usize, first_call: bool, camera: &CameraViewPort) {
    // Convert to cartesian
    let a: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[0].0), Angle(lonlat[0].1));
    let b: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[1].0), Angle(lonlat[1].1));
    let c: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[2].0), Angle(lonlat[2].1));

    // Project them. We are always facing the camera
    let A = P::model_to_ndc_space(&a, camera);
    let B = P::model_to_ndc_space(&b, camera);
    let C = P::model_to_ndc_space(&c, camera);
    match (A, B, C) {
        (None, None, None) => {
            return;
        },
        (Some(A), Some(B), Some(C)) => {
            // Compute the angle between a->b and b->c
            let AB = (B - A);
            let BC = (C - B);
            let AB_l = AB.magnitude2();
            let BC_l = BC.magnitude2();

            let theta = math::angle(&AB.normalize(), &BC.normalize());

            if theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION && !first_call {
                vertices.push(A);
                vertices.push(B);

                vertices.push(B);
                vertices.push(C);
            } else {
                if depth > 0 {
                    // Subdivide a->b and b->c
                    let lon_d = (lonlat[0].0 + lonlat[1].0) * 0.5_f32;
                    let lat_d = (lonlat[0].1 + lonlat[1].1) * 0.5_f32;
                    subdivide::<P>(vertices, [lonlat[0], (lon_d, lat_d), lonlat[1]], depth - 1, false, camera);

                    let lon_e = (lonlat[1].0 + lonlat[2].0) * 0.5_f32;
                    let lat_e = (lonlat[1].1 + lonlat[2].1) * 0.5_f32;
                    subdivide::<P>(vertices, [lonlat[1], (lon_e, lat_e), lonlat[2]], depth - 1, false, camera);
                } else {
                    if AB_l.min(BC_l) / AB_l.max(BC_l) < 0.1 {
                        if AB_l == AB_l.min(BC_l) {
                            vertices.push(A);
                            vertices.push(B);
                        } else {
                            vertices.push(B);
                            vertices.push(C);
                        }
                        return;
                    }
                }
            }
        },
        (Some(A), None, None) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(vertices,
                [
                    lonlat[0],
                    ((lonlat[0].0 + lonlat[1].0)*0.5, (lonlat[0].1 + lonlat[1].1)*0.5),
                    lonlat[1]
                ],
                depth - 1,
                false,
                camera
            );
        },
        (None, None, Some(C)) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(vertices,
                [
                    lonlat[1],
                    ((lonlat[1].0 + lonlat[2].0)*0.5, (lonlat[1].1 + lonlat[2].1)*0.5),
                    lonlat[2]
                ],
                depth - 1,
                false,
                camera
            );
        },
        (None, Some(B), None) => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(vertices,
                [
                    lonlat[0],
                    ((lonlat[0].0 + lonlat[1].0)*0.5, (lonlat[0].1 + lonlat[1].1)*0.5),
                    lonlat[1]
                ],
                depth - 1,
                false,
                camera
            );
            subdivide::<P>(vertices,
                [
                    lonlat[1],
                    ((lonlat[1].0 + lonlat[2].0)*0.5, (lonlat[1].1 + lonlat[2].1)*0.5),
                    lonlat[2]
                ],
                depth - 1,
                false,
                camera
            );
        },
        _ => {
            if depth == 0 {
                return;
            }
            subdivide::<P>(vertices,
                [
                    lonlat[0],
                    ((lonlat[0].0 + lonlat[1].0)*0.5, (lonlat[0].1 + lonlat[1].1)*0.5),
                    lonlat[1]
                ],
                depth - 1,
                false,
                camera
            );
            subdivide::<P>(vertices,
                [
                    lonlat[1],
                    ((lonlat[1].0 + lonlat[2].0)*0.5, (lonlat[1].1 + lonlat[2].1)*0.5),
                    lonlat[2]
                ],
                depth - 1,
                false,
                camera
            );
        }
    }
}
use crate::math::{self, LonLatT};
use cgmath::Vector2;
use core::ops::Range;
use crate::Angle;

#[derive(Debug)]
struct Label {
    position: Vector2<f32>,
    content: String,
    rot: f32,
}

#[derive(Debug)]
struct GridLine {
    vertices: Vec<Vector2<f32>>,
    label: Label,
}
use cgmath::Rad;
use super::angle::SerializeToString;
const PI: f32 = std::f32::consts::PI;
const TWICE_PI: f32 = 2.0*PI;
const HALF_PI: f32 = 0.5*PI;
impl GridLine {
    fn meridian<P: Projection>(lon: f32, lat: &Range<f32>, camera: &CameraViewPort) -> Option<Self> {
        let fov = camera.get_field_of_view();

        if let Some((p, u)) = fov.intersect_meridian(Rad(lon)) {
            if let Some(p1) = P::model_to_screen_space(&Vector4::new(p.x, p.y, p.z, 1.0), camera) {
                let t = (p + u*1e-3).normalize();

                if let Some(p2) = P::model_to_screen_space(&Vector4::new(t.x, t.y, t.z, 1.0), camera) {
                    let r = (p2 - p1).normalize();

                    // rot is between -PI and +PI
                    /*let rot = if r.y > 0.0 {
                        let a = r.x.acos();
                        if a > HALF_PI {
                            -PI + a
                        } else {
                            a
                        }
                    } else {
                        let a = -r.x.acos();
                        if a < -HALF_PI {
                            PI + a
                        } else {
                            a
                        }
                    };*/                  
                    let rot = if r.y > 0.0 {
                        r.x.acos()
                    } else {
                        -r.x.acos()
                    };
                    let content = Angle(lon).to_string::<angle::DMS>();

                    let label = Label {
                        position: p1,
                        content,
                        rot
                    };

                    let mut vertices = vec![];
                    subdivide::<P>(
                        &mut vertices,
                        [
                            (lon, lat.start),
                            (lon, (lat.start + lat.end)*0.5_f32),
                            (lon, lat.end),
                        ],
                        7,
                        true,
                        camera,
                    );
                    Some(GridLine {
                        vertices,
                        label
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parallel<P: Projection>(lon: &Range<f32>, lat: f32, camera: &CameraViewPort) -> Option<Self> {
        let fov = camera.get_field_of_view();
        //let labels = great_circles.get_labels::<angle::DMS>();

        if let Some((p, u)) = fov.intersect_parallel(Rad(lat)) {
            if let Some(p1) = P::model_to_screen_space(&Vector4::new(p.x, p.y, p.z, 1.0), camera) {
                let t = (p + u*1e-3).normalize();

                if let Some(p2) = P::model_to_screen_space(&Vector4::new(t.x, t.y, t.z, 1.0), camera) {
                    let r = (p2 - p1).normalize();

                    // rot is between -PI and +PI
                    /*let rot = if r.y > 0.0 {
                        let a = r.x.acos();
                        if a > HALF_PI {
                            -PI + a
                        } else {
                            a
                        }
                    } else {
                        let a = -r.x.acos();
                        if a < -HALF_PI {
                            PI + a
                        } else {
                            a
                        }
                    };*/                  
                    let rot = if r.y > 0.0 {
                        r.x.acos()
                    } else {
                        -r.x.acos()
                    };

                    let content = Angle(lat).to_string::<angle::DMS>();
                    let label = Label {
                        position: p1,
                        content,
                        rot
                    };

                    let mut vertices = vec![];
                    subdivide::<P>(
                        &mut vertices,
                        [
                            (lon.start, lat),
                            (0.5*(lon.start + lon.end), lat),
                            (lon.end, lat),
                        ],
                        7,
                        true,
                        camera,
                    );

                    Some(GridLine {
                        vertices,
                        label
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

const GRID_STEPS: &'static [f64] = &[
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
    0.0000000000048481366
];

const NUM_LINES_LATITUDES: usize = 5;
fn lines<P: Projection>(camera: &CameraViewPort) -> Vec<GridLine> {
    let bbox = camera.get_bounding_box();

    let step_lon = select_grid_step(&bbox, bbox.get_lon_size().0 as f64, (NUM_LINES_LATITUDES as f32 * camera.get_aspect()) as usize);

    let mut lines = vec![];
    // Add meridians
    let mut theta = bbox.lon_min().0 - (bbox.lon_min().0 % step_lon);
    let mut stop_theta = bbox.lon_max().0;
    if bbox.all_lon() {
        stop_theta -= 1e-3;
    }

    while theta < stop_theta {
        if let Some(line) = GridLine::meridian::<P>(theta, &bbox.get_lat(), camera) {
            lines.push(line);
        }
        theta += step_lon;
    }

    // Add parallels
    let step_lat = select_grid_step(&bbox, bbox.get_lat_size().0 as f64, NUM_LINES_LATITUDES);

    let mut alpha = bbox.lat_min().0 - (bbox.lat_min().0 % step_lat);
    if alpha == -HALF_PI {
        alpha += step_lat;
    }
    let mut stop_alpha = bbox.lat_max().0;
    if stop_alpha == HALF_PI {
        stop_alpha -= 1e-3;
    }

    while alpha < stop_alpha {
        if let Some(line) = GridLine::parallel::<P>(&bbox.get_lon(), alpha, camera) {
            lines.push(line);
        }
        alpha += step_lat;
    }

    lines
}

fn select_grid_step(bbox: &BoundingBox, fov: f64, max_lines: usize) -> f32 {

    // Select the best meridian grid step
    let mut i = 0;
    let mut step = GRID_STEPS[0];
    while i < GRID_STEPS.len() {
        if fov >= GRID_STEPS[i] {
            let num_meridians_in_fov = (fov / GRID_STEPS[i]) as usize;

            if num_meridians_in_fov >= max_lines - 1 {
                let idx_grid = if i == 0 {
                    0
                } else {
                    i - 1
                };
                step = GRID_STEPS[idx_grid];
                break;
            }
        }

        step = GRID_STEPS[i];
        i += 1;
    }

    step as f32
}