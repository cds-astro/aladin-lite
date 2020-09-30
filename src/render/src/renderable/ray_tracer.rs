use crate::{
    core::{VertexArrayObject, VecData},
    shader::{ShaderBound, ShaderManager},
    viewport::CameraViewPort,
    WebGl2Context,
    renderable::projection::Projection
};

pub trait RayTracingProjection {
    fn get_raytracer_vertex_array_object(raytracer: &RayTracer) -> &VertexArrayObject;
}

use crate::renderable::Triangulation;
fn create_vertices_array<P: Projection>(_gl: &WebGl2Context) -> (Vec<f32>, Vec<u16>) {
    let (vertices, idx) = Triangulation::new::<P>().into();

    let vertices = vertices
        .into_iter()
        .map(|pos_clip_space| {
            let pos_world_space = P::clip_to_world_space(&pos_clip_space).unwrap();

            vec![
                pos_clip_space.x,
                pos_clip_space.y,
                
                pos_world_space.x,
                pos_world_space.y,
                pos_world_space.z
            ]
        })
        .flatten()
        .collect::<Vec<_>>();

    (vertices, idx)
}

use web_sys::WebGl2RenderingContext;
use web_sys::WebGlVertexArrayObject;

pub struct RayTracer {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBufferObject,
    ebo: WebGlBufferObject
}

use crate::{
    buffer::TileBuffer,
    Shader
};
use std::borrow::Cow;
use crate::shader::ShaderId;
impl RayTracer {
    pub fn new<P: Projection>(gl: &WebGl2Context, viewport: &CameraViewPort, shaders: &mut ShaderManager) -> RayTracer {
        let (vertices, idx) = create_vertices_array::<P>(gl);

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));
    
        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 5 * mem::size_of::<f32>(), 0 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec3 position;
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, 5 * mem::size_of::<f32>(), 3 * mem::size_of::<f32>());
        gl.enable_vertex_attrib_array(1);

        let vbo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        let buf_vertices = unsafe { js_sys::Float32Array::view(&vertices) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &buf_vertices,
            WebGl2RenderingContext::STATIC_DRAW
        );
        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        let buf_indices = unsafe { js_sys::Uint16Array::view(&idx) };

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &buf_indices,
            WebGl2RenderingContext::STATIC_DRAW
        );

        let gl = gl.clone();
        RayTracer {
            vao,
            vbo,
            ebo,
            gl
        }
    }

    pub fn bind(&self) {
        self.gl.bind_vertex_array(Some(&self.vao));
    }

    pub fn draw(
        &self,
    ) {
        //let vertex_array_object = P::get_raytracer_vertex_array_object(&self);
        self.gl.draw_elements_with_i32(
            //WebGl2RenderingContext::LINES,
            WebGl2RenderingContext::TRIANGLES,
            None,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0
        ); 
    }
}