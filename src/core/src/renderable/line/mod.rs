/// This module handles the lines rendering code
pub mod great_circle_arc;
pub mod parallel_arc;

use al_core::WebGlContext;
use al_core::VertexArrayObject;
use al_core::shader::Shader;
use crate::Abort;
use crate::math::projection::coo_space::XYNDC;
use al_api::color::ColorRGBA;
use super::Renderer;

use lyon::algorithms::{
    math::point,
    path::Path,
    length::approximate_length,
    measure::{PathMeasurements, SampleType},
};

use al_core::{info, inforec, log};

struct Meta {
    color: ColorRGBA,
    style: Style,
    thickness: f32,
    off_indices: usize,
    num_indices: usize,
}

#[derive(Clone)]
pub enum Style {
    None,
    Dashed,
    Dotted
}

pub struct RasterizedLineRenderer {
    gl: WebGlContext,
    shader: Shader,
    vao: VertexArrayObject,

    vertices: Vec<f32>,
    indices: Vec<u16>,
    meta: Vec<Meta>,
}
use wasm_bindgen::JsValue;
use cgmath::Vector2;
use al_core::VecData;
use web_sys::WebGl2RenderingContext;
use crate::Color;
use crate::camera::CameraViewPort;

use lyon::tessellation::*;

impl RasterizedLineRenderer {
    /// Init the buffers, VAO and shader
    pub fn new(gl: &WebGlContext) -> Result<Self, JsValue> {
        let vertices = vec![];
        let indices = vec![];
        // Create the VAO for the screen
        let shader = Shader::new(
            &gl,
            include_str!("../../../../glsl/webgl2/line/line_vertex.glsl"),
            include_str!("../../../../glsl/webgl2/line/line_frag.glsl")
        )?;
        let mut vao = VertexArrayObject::new(&gl);

        vao
            .bind_for_update()
                .add_array_buffer(
                    "ndc_pos",
                    2 * std::mem::size_of::<f32>(),
                    &[2],
                    &[0],
                    WebGl2RenderingContext::STREAM_DRAW,
                    VecData::<f32>(&vertices),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STREAM_DRAW,
                    VecData::<u16>(&indices),
                )
            .unbind();

        let meta = vec![];
        let gl = gl.clone();
        Ok(Self {
            gl,
            shader,
            vao,
            meta,
            vertices,
            indices
        })
    }

    pub fn add_paths<'a>(&mut self, paths: impl Iterator<Item=&'a [[f32; 2]]>, thickness: f32, color: &ColorRGBA, style: &Style) {
        let mut path_builder = Path::builder();

        let clamp_ndc_vertex = |v: &[f32; 2]| -> [f32; 2] {
            let x = v[0].clamp(-2.0, 2.0);
            let y = v[1].clamp(-2.0, 2.0);

            [x, y]
        };

        match &style {
            Style::None => {
                for line in paths {
                    if !line.is_empty() {
                        let v = clamp_ndc_vertex(&line[0]); 
                        path_builder.begin(point(v[0], v[1]));
        
                        for v in line.iter().skip(1) {
                            let v = clamp_ndc_vertex(v);
                            path_builder.line_to(point(v[0], v[1]));
                        }
        
                        path_builder.end(false);
                    }
                }
            },
            Style::Dashed => {
                for line in paths {
                    if !line.is_empty() {
                        let mut line_path_builder = Path::builder();
        
                        let v = clamp_ndc_vertex(&line[0]); 
                        line_path_builder.begin(point(v[0], v[1]));
        
                        for v in line.iter().skip(1) {
                            let v = clamp_ndc_vertex(v);
                            line_path_builder.line_to(point(v[0], v[1]));
                        }
        
                        line_path_builder.end(false);
                        let path = line_path_builder.build();
        
                        // Build the acceleration structure.
                        let measurements = PathMeasurements::from_path(&path, 1e-2);
                        let mut sampler = measurements.create_sampler(&path, SampleType::Normalized);
        
                        let path_len = sampler.length();
                        let step = 1e-2 / path_len;
        
                        for i in (0..((1.0/step) as usize)).step_by(2) {
                            let start = (i as f32) * step;
                            let end = (i as f32 + 1.0) * step;
        
                            sampler.split_range(start..end, &mut path_builder);
                        }
                    }
                }
            },
            Style::Dotted => {

            }
        }

        let p = path_builder.build();
        // Let's use our own custom vertex type instead of the default one.
        // Will contain the result of the tessellation.
        let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
        {
            let mut tessellator = StrokeTessellator::new();
            // Compute the tessellation.
            tessellator.tessellate(
                &p,
                &StrokeOptions::default()
                    .with_line_width(thickness),
                &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                    vertex.position().to_array()
                }),
            ).unwrap_abort();
        }

        let VertexBuffers {vertices, mut indices} = geometry;
        let num_vertices = (self.vertices.len() / 2) as u16;
        for idx in indices.iter_mut() {
            *idx += num_vertices;
        }

        let num_indices = indices.len();
        let off_indices = self.indices.len();

        self.vertices.extend(vertices.iter().flatten());
        self.indices.extend(indices.iter());

        self.meta.push(
            Meta {
                off_indices,
                num_indices,
                thickness,
                color: color.clone(),
                style: style.clone()
            }
        );
    }

    pub fn draw(&mut self, camera: &CameraViewPort) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        
        self.gl.disable(WebGl2RenderingContext::CULL_FACE);

        let shader = self.shader.bind(&self.gl);
        for meta in self.meta.iter() {
            shader
                .attach_uniform("u_color", &meta.color) // Strengh of the kernel
                .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(meta.num_indices as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        ((meta.off_indices as usize) * std::mem::size_of::<u16>()) as i32
                    );
        }

        self.gl.enable(WebGl2RenderingContext::CULL_FACE);
        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}

impl Renderer for RasterizedLineRenderer {
    fn begin(&mut self) {
        self.vertices.clear();
        self.indices.clear();

        self.meta.clear();
    }

    fn end(&mut self) {
        // update to the GPU
        self.vao.bind_for_update()
            .update_array("ndc_pos", WebGl2RenderingContext::STREAM_DRAW, VecData(&self.vertices))
            .update_element_array(WebGl2RenderingContext::STREAM_DRAW, VecData(&self.indices));
    }
}