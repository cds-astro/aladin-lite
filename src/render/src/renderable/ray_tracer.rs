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
fn create_vertex_array_object<P: Projection>(
    gl: &WebGl2Context,
    _viewport: &CameraViewPort,
    shaders: &mut ShaderManager
) -> VertexArrayObject {
    let (vertices, idx) = create_vertices_array::<P>(gl);
    
    let mut vertex_array_object = VertexArrayObject::new(gl);

    let shader = shaders.get(
        gl,
        &ShaderId(
            Cow::Borrowed("RayTracerVS"),
            Cow::Borrowed("RayTracerFS")
        )
    ).unwrap();
    shader.bind(gl)
        // VAO for per-pixel computation mode (only in case of large fovs and 2D projections)
        .bind_vertex_array_object(&mut vertex_array_object)
            // Store the projeted and 3D vertex positions in a VBO
            .add_array_buffer(
                5 * std::mem::size_of::<f32>(),
                &[2, 3],
                &[0, 2 * std::mem::size_of::<f32>()],
                WebGl2RenderingContext::STATIC_DRAW,
                VecData(vertices.as_ref()),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::STATIC_DRAW,
                VecData(idx.as_ref()),
            )
            // Unbind the buffer
            .unbind();

    vertex_array_object
}

pub struct RayTracer {
    vao: VertexArrayObject,
}

use crate::{
    buffer::TileBuffer,
    Shader
};
use std::borrow::Cow;
use crate::shader::ShaderId;
impl RayTracer {
    pub fn new<P: Projection>(gl: &WebGl2Context, viewport: &CameraViewPort, shaders: &mut ShaderManager) -> RayTracer {
        let vao = create_vertex_array_object::<P>(gl, viewport, shaders);

        RayTracer {
            vao
        }
    }

    pub fn get_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager, buffer: &TileBuffer) -> &'a Shader {
        // Fits tiles are handled by other shaders
        if buffer.fits_tiles_requested() {
            if buffer.fits_i_format() {
                shaders.get(
                    gl,
                    &ShaderId(
                        Cow::Borrowed("RayTracerVS"),
                        Cow::Borrowed("RayTracerFITSIFS")
                    )
                ).unwrap()
            } else {
                shaders.get(
                    gl,
                    &ShaderId(
                        Cow::Borrowed("RayTracerVS"),
                        Cow::Borrowed("RayTracerFITSFS")
                    )
                ).unwrap()            }
        } else {
            shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerFS")
                )
            ).unwrap()
        }
    }

    pub fn draw(
        &self,
        _gl: &WebGl2Context,
        shader: &ShaderBound,
    ) {
        //let vertex_array_object = P::get_raytracer_vertex_array_object(&self);
        shader.bind_vertex_array_object_ref(&self.vao)
            .draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                //WebGl2RenderingContext::LINE_LOOP,
                //WebGl2RenderingContext::POINTS,
                None,
                WebGl2RenderingContext::UNSIGNED_SHORT
            );
    }
}