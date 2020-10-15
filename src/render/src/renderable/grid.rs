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
use web_sys::{WebGlVertexArrayObject, WebGlBuffer};
pub struct ProjetedGrid {
    // The color of the grid
    color: Color,
    opacity: f32,

    // The vertex array object of the screen in NDC
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,

    lines: Vec<GridLine>,
    size_vertices_buf: usize,

    num_vertices: usize,

    gl: WebGl2Context,
}

use crate::renderable::projection::Projection;

use crate::ShaderManager;
use crate::WebGl2Context;

impl ProjetedGrid {
    pub fn new<P: Projection>(
        gl: &WebGl2Context,
        _camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        //_text_manager: &TextManager
    ) -> ProjetedGrid {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        gl.line_width(2.0);
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

        let lines = vec![];
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
        let opacity = 1.0;
        ProjetedGrid {
            color,
            opacity,

            vao,
            vbo,

            lines,
            size_vertices_buf,
            num_vertices,

            gl
        }
    }

    // Update the grid whenever the camera moved
    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort) {
        if camera.has_moved() {
            let mut lines = lines::<P>(camera);
            crate::log(&format!("length {:?}", lines));

            /*let num_lines = lines.len();
            let num_vertices: usize = lines.iter().fold(0, |mut sum, line| {
                sum += line.vertices.len();
                sum
            });*/
            
            let mut vertices: Vec<Vector2<f32>> = lines.into_iter().map(|line| line.vertices).flatten().collect();
            self.num_vertices = vertices.len();

            let vertices: Vec<f32> = unsafe {
                /*crate::log(&format!("sum {:?}", vertices));

                let ptr = vertices.as_mut_ptr() as *mut f32;
                let len = vertices.len() << 1;

                Vec::from_raw_parts(ptr, len, len)*/
                vertices.set_len(vertices.len() * 2);
                std::mem::transmute(vertices)
            };
            crate::log(&format!("vertices {:?}", vertices));

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
        //text_manager: &TextManager,
    ) {
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
        self.gl.draw_arrays(
            WebGl2RenderingContext::LINE_STRIP,
            0,
            self.num_vertices as i32
        );

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

use crate::sphere_geometry::{GreatCircles, BoundingBox};

use cgmath::InnerSpace;
const MAX_ANGLE_BEFORE_SUBDIVISION: f32 = 20.0 * std::f32::consts::PI / 180.0;
fn subdivide<P: Projection>(vertices: &mut Vec<Vector2<f32>>, lonlat: [(f32, f32); 3], depth: usize, camera: &CameraViewPort) {
    // Convert to cartesian
    let a: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[0].0), Angle(lonlat[0].1));
    let b: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[1].0), Angle(lonlat[1].1));
    let c: Vector4<f32> = math::radec_to_xyzw(Angle(lonlat[2].0), Angle(lonlat[2].1));

    // Project them. We are always facing the camera
    let A = P::model_to_ndc_space(&a, camera).unwrap();
    let B = P::model_to_ndc_space(&b, camera).unwrap();
    let C = P::model_to_ndc_space(&c, camera).unwrap();

    // Compute the angle between a->b and b->c
    let AB = (B - A).normalize();
    let BC = (C - B).normalize();
    let theta = math::angle(&AB, &BC);

    if theta.abs() < MAX_ANGLE_BEFORE_SUBDIVISION || depth == 0 {
        // We do not need to subdivide the line anymore
        if vertices.is_empty() {
            vertices.push(A);
        }

        vertices.push(B);
        vertices.push(C);
    } else {
        // Subdivide a->b and b->c
        let lon_d = (lonlat[0].0 + lonlat[1].0) * 0.5_f32;
        let lat_d = (lonlat[0].1 + lonlat[1].1) * 0.5_f32;
        subdivide::<P>(vertices, [lonlat[0], (lon_d, lat_d), lonlat[1]], depth - 1, camera);

        let lon_e = (lonlat[1].0 + lonlat[2].0) * 0.5_f32;
        let lat_e = (lonlat[1].1 + lonlat[2].1) * 0.5_f32;
        subdivide::<P>(vertices, [lonlat[1], (lon_e, lat_e), lonlat[2]], depth - 1, camera);
    }
}
use crate::math::{self, LonLatT};
use cgmath::Vector2;
use core::ops::Range;
use crate::Angle;
#[derive(Debug)]
struct GridLine {
    vertices: Vec<Vector2<f32>>
}
impl GridLine {
    fn meridian<P: Projection>(lon: f32, lat: &Range<f32>, camera: &CameraViewPort) -> Self {
        let mut vertices = vec![];
        subdivide::<P>(
            &mut vertices,
            [
                (lon, lat.start),
                (lon, (lat.start + lat.end)*0.5_f32),
                (lon, lat.end),
            ],
            5,
            camera,
        );

        GridLine {
            vertices
        }
    }
    fn parallel<P: Projection>(lon: &Range<f32>, lat: f32, camera: &CameraViewPort) -> Self {
        let mut vertices = vec![];
        subdivide::<P>(
            &mut vertices,
            [
                (lon.start, lat),
                (0.5*(lon.start + lon.end), lat),
                (lon.end, lat),
            ],
            5,
            camera,
        );

        GridLine {
            vertices
        }
    }
}

const NUM_LINES_LONGITUDES: usize = 5;
fn lines<P: Projection>(camera: &CameraViewPort) -> Vec<GridLine> {
    if !camera.is_allsky() {
        let BoundingBox { lat, lon } = camera.get_bounding_box().unwrap();
        let lon = lon.start.0..lon.end.0;
        let lat = lat.start.0..lat.end.0;
    
        let alpha_step = (lat.end - lat.start)/(NUM_LINES_LONGITUDES as f32 + 1.0);
        let theta_step = (lon.end - lon.start)/(NUM_LINES_LONGITUDES as f32 + 1.0);
        crate::log(&format!("bounding box {:?} {:?}", lon, lat));

        let mut lines = vec![];
        // Add meridians
        let mut theta = lon.start + theta_step;
        for i in 0..NUM_LINES_LONGITUDES {
            let line = GridLine::meridian::<P>(theta, &lat, camera);
            lines.push(line);
            theta += theta_step;
        }
        // Add parallels
        let mut alpha = lat.start + alpha_step;
        for i in 0..NUM_LINES_LONGITUDES {
            let line = GridLine::parallel::<P>(&lon, alpha, camera);
            lines.push(line);
            alpha += alpha_step;
        }
    
        lines
    } else {
        crate::log("allsky");
        // Allsky
        vec![]
    }
}