/// This module handles the lines rendering code
pub mod great_circle_arc;
pub mod parallel_arc;

use al_core::WebGlContext;
use al_core::VertexArrayObject;
use al_core::shader::Shader;
use crate::Abort;
use crate::math::projection::coo_space::XYNDC;
use al_api::color::ColorRGBA;

struct LineMeta {
    color: ColorRGBA,
    thickness: f32,
    off_idx: usize,
    num_idx: usize,
}

pub struct RasterizedLinesRenderManager {
    gl: WebGlContext,
    shader: Shader,
    vao: VertexArrayObject,

    vertices: Vec<f32>,
    indices: Vec<u16>,
    meta: Vec<LineMeta>,
}
use wasm_bindgen::JsValue;
use cgmath::Vector2;
use al_core::VecData;
use web_sys::WebGl2RenderingContext;
use crate::Color;
use crate::camera::CameraViewPort;

use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::*;

impl RasterizedLinesRenderManager {
    /// Init the buffers, VAO and shader
    pub fn new(gl: WebGlContext, camera: &CameraViewPort) -> Result<Self, JsValue> {
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
        Ok(Self {
            gl,
            shader,
            vao,
            meta,
            vertices,
            indices
        })
    }

    pub fn add_path(&mut self, path: &[XYNDC], thickness: f32, color: &ColorRGBA) {
        let mut builder = Path::builder();
        if path.is_empty() {
            return;
        }

        builder.begin(point(path[0].x as f32, path[0].y as f32));

        for p in path.iter().skip(1) {
            builder.line_to(point(p.x as f32, p.y as f32));
        }

        builder.end(true);
        let path = builder.build();
        // Let's use our own custom vertex type instead of the default one.
        // Will contain the result of the tessellation.
        let mut geometry: VertexBuffers<[f32; 2], u16> = VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        {
            // Compute the tessellation.
            tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                    vertex.position().to_array()
                }),
            ).unwrap_abort();
        }
        let num_vertices = (self.vertices.len() / 2) as u16;

        self.vertices.extend(geometry.vertices.iter().flatten());
        for i in geometry.indices.iter_mut() {
            *i += num_vertices;
        }

        let num_idx = geometry.indices.len();
        let off_idx = self.indices.len();
        self.indices.extend(geometry.indices);

        self.meta.push(
            LineMeta {
                off_idx,
                num_idx,
                thickness,
                color: color.clone(),
            }
        );
    }


    fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();

        self.meta.clear();
    }

    fn end_frame(&mut self) {
        // update to the GPU
        self.vao.bind_for_update()
            .update_array("ndc_pos", WebGl2RenderingContext::STREAM_DRAW, VecData(&self.vertices))
            .update_element_array(WebGl2RenderingContext::STREAM_DRAW, VecData(&self.indices));
    }

    fn draw(&mut self, window_size: &Vector2<f32>) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA); // premultiplied alpha

        let shader = self.shader.bind(&self.gl);
        self.vao.bind(&shader);

        for meta in self.meta.iter() {
            shader
                .attach_uniform("u_color", &meta.color) // Strengh of the kernel
                .attach_uniform("u_screen_size", window_size);

            self.gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                meta.num_idx as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                (meta.off_idx as i32) * (std::mem::size_of::<u16>() as i32)
            );
        }

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
