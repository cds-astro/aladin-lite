/// This module handles the lines rendering code
pub mod great_circle_arc;
pub mod parallel_arc;

use crate::Abort;
use al_core::shader::Shader;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use super::Renderer;
use al_api::color::ColorRGBA;
use al_core::SliceData;

use lyon::algorithms::{
    math::point,
    measure::{PathMeasurements, SampleType},
    path::Path,
};

struct Meta {
    color: ColorRGBA,
    off_indices: usize,
    num_indices: usize,
}

#[derive(Clone)]
pub enum Style {
    None,
    Dashed,
    Dotted,
}

pub struct RasterizedLineRenderer {
    gl: WebGlContext,
    shader: Shader,
    vao: VertexArrayObject,

    vertices: Vec<f32>,
    indices: Vec<u32>,
    meta: Vec<Meta>,
}
use wasm_bindgen::JsValue;

use al_core::VecData;
use web_sys::WebGl2RenderingContext;

use crate::camera::CameraViewPort;

use lyon::tessellation::*;

pub struct PathVertices<T>
where
    T: AsRef<[[f32; 2]]>,
{
    pub vertices: T,
    pub closed: bool,
}

impl RasterizedLineRenderer {
    /// Init the buffers, VAO and shader
    pub fn new(gl: &WebGlContext) -> Result<Self, JsValue> {
        let vertices = vec![];
        let indices = vec![];
        // Create the VAO for the screen
        let shader = Shader::new(
            &gl,
            include_str!("../../../../glsl/webgl2/line/line_vertex.glsl"),
            include_str!("../../../../glsl/webgl2/line/line_frag.glsl"),
        )?;
        let mut vao = VertexArrayObject::new(&gl);

        vao.bind_for_update()
            .add_array_buffer(
                "ndc_pos",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&vertices),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&indices),
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
            indices,
        })
    }

    pub fn add_fill_paths<T>(
        &mut self,
        paths: impl Iterator<Item = PathVertices<T>>,
        color: &ColorRGBA,
    ) where
        T: AsRef<[[f32; 2]]>,
    {
        let mut num_indices = 0;
        let off_indices = self.indices.len();
        let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        //let mut num_vertices = 0;
        for path in paths {
            let mut path_builder = Path::builder();

            let PathVertices { vertices, closed } = path;

            let line: &[[f32; 2]] = vertices.as_ref();

            if !line.is_empty() {
                let v = &line[0];
                path_builder.begin(point(v[0], v[1]));

                for v in line.iter().skip(1) {
                    //let v = clamp_ndc_vertex(v);
                    path_builder.line_to(point(v[0], v[1]));
                }

                path_builder.end(closed);
            }

            // Create the destination vertex and index buffers.
            let p = path_builder.build();
            // Let's use our own custom vertex type instead of the default one.
            // Will contain the result of the tessellation.
            let num_vertices = (self.vertices.len() / 2) as u32;

            // Compute the tessellation.
            tessellator
                .tessellate_with_ids(
                    p.id_iter(),
                    &p,
                    Some(&p),
                    &FillOptions::default()
                        .with_intersections(false)
                        .with_fill_rule(FillRule::NonZero)
                        .with_tolerance(5e-3),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    })
                    .with_vertex_offset(num_vertices),
                )
                .unwrap_abort();
        }
        let VertexBuffers { vertices, indices } = geometry;
        num_indices += indices.len();

        self.vertices.extend(vertices.iter().flatten());
        self.indices.extend(indices.iter());

        //al_core::info!("num vertices fill", nv);

        self.meta.push(Meta {
            off_indices,
            num_indices,
            color: color.clone(),
        });
    }

    pub fn add_stroke_paths<T>(
        &mut self,
        paths: impl Iterator<Item = PathVertices<T>>,
        thickness: f32,
        color: &ColorRGBA,
        style: &Style,
    ) where
        T: AsRef<[[f32; 2]]>,
    {
        let num_vertices = (self.vertices.len() / 2) as u32;

        let mut path_builder = Path::builder();

        match &style {
            Style::None => {
                for path in paths {
                    let PathVertices { vertices, closed } = path;

                    let line: &[[f32; 2]] = vertices.as_ref();
                    if !line.is_empty() {
                        //let v = clamp_ndc_vertex(&line[0]);
                        let v = &line[0];
                        path_builder.begin(point(v[0], v[1]));

                        for v in line.iter().skip(1) {
                            //let v = clamp_ndc_vertex(v);
                            path_builder.line_to(point(v[0], v[1]));
                        }

                        path_builder.end(closed);
                    }
                }

                //al_core::info!("num vertices", nv);
            }
            Style::Dashed => {
                for path in paths {
                    let PathVertices { vertices, closed } = path;
                    let line: &[[f32; 2]] = vertices.as_ref();

                    if !line.is_empty() {
                        let mut line_path_builder = Path::builder();

                        //let v = clamp_ndc_vertex(&line[0]);
                        let v = &line[0];
                        line_path_builder.begin(point(v[0], v[1]));

                        for v in line.iter().skip(1) {
                            //let v = clamp_ndc_vertex(v);
                            line_path_builder.line_to(point(v[0], v[1]));
                        }

                        line_path_builder.end(closed);
                        let path = line_path_builder.build();

                        // Build the acceleration structure.
                        let measurements = PathMeasurements::from_path(&path, 1e-2);
                        let mut sampler =
                            measurements.create_sampler(&path, SampleType::Normalized);

                        let path_len = sampler.length();
                        let step = 1e-2 / path_len;

                        for i in (0..((1.0 / step) as usize)).step_by(2) {
                            let start = (i as f32) * step;
                            let end = (i as f32 + 1.0) * step;

                            sampler.split_range(start..end, &mut path_builder);
                        }
                    }
                }
            }
            Style::Dotted => {}
        }

        let p = path_builder.build();
        // Let's use our own custom vertex type instead of the default one.
        // Will contain the result of the tessellation.
        let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
        {
            let mut tessellator = StrokeTessellator::new();
            // Compute the tessellation.
            tessellator
                .tessellate(
                    &p,
                    &StrokeOptions::default().with_line_width(thickness),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| {
                        vertex.position().to_array()
                    })
                    .with_vertex_offset(num_vertices),
                )
                .unwrap_abort();
        }

        let VertexBuffers { vertices, indices } = geometry;

        let num_indices = indices.len();
        let off_indices = self.indices.len();

        self.vertices.extend(vertices.iter().flatten());
        self.indices.extend(indices.iter());

        self.meta.push(Meta {
            off_indices,
            num_indices,
            color: color.clone(),
        });
    }

    pub fn draw(&mut self, _camera: &CameraViewPort) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        //self.gl.disable(WebGl2RenderingContext::CULL_FACE);

        let shader = self.shader.bind(&self.gl);
        for meta in self.meta.iter() {
            shader
                .attach_uniform("u_color", &meta.color) // Strengh of the kernel
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    Some(meta.num_indices as i32),
                    WebGl2RenderingContext::UNSIGNED_INT,
                    ((meta.off_indices as usize) * std::mem::size_of::<u32>()) as i32,
                );
        }

        //self.gl.enable(WebGl2RenderingContext::CULL_FACE);
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
        self.vao
            .bind_for_update()
            .update_array(
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                SliceData(self.vertices.as_slice()),
            )
            .update_element_array(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                SliceData(self.indices.as_slice()),
            );
    }
}
