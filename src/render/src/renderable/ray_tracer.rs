use crate::{
    core::{VertexArrayObject, VecData},
    shader::{ShaderBound, ShaderManager},
    camera::CameraViewPort,
    WebGl2Context,
    renderable::projection::Projection
};

pub trait RayTracingProjection {
    fn get_raytracer_vertex_array_object(raytracer: &RayTracer) -> &VertexArrayObject;
}

use crate::renderable::Triangulation;
fn create_vertices_array<P: Projection>(_gl: &WebGl2Context, camera: &CameraViewPort) -> (Vec<f32>, Vec<u16>) {
    let (vertices, idx) = Triangulation::new::<P>().into();

    let vertices = vertices
        .into_iter()
        .map(|pos_clip_space| {
            let pos_world_space = P::clip_to_world_space(&pos_clip_space, camera.is_reversed_longitude()).unwrap();

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
use web_sys::WebGlBuffer;
use crate::core::Texture2D;
pub struct RayTracer {
    gl: WebGl2Context,

    vao: WebGlVertexArrayObject,

    vbo: WebGlBuffer,
    ebo: WebGlBuffer,

    num_indices: i32, 

    ang2pix: [Texture2D; 3],
}

use crate::Shader;
use std::borrow::Cow;
use crate::shader::ShaderId;
use std::mem;
use crate::{FormatImageType, Resources};
impl RayTracer {
    pub fn new<P: Projection>(gl: &WebGl2Context, camera: &CameraViewPort, shaders: &mut ShaderManager, resources: &Resources) -> RayTracer {
        let (vertices, idx) = create_vertices_array::<P>(gl, camera);

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

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

        // layout (location = 0) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, (5 * mem::size_of::<f32>()) as i32, (0 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec3 position;
        gl.vertex_attrib_pointer_with_i32(1, 3, WebGl2RenderingContext::FLOAT, false, (5 * mem::size_of::<f32>()) as i32, (2 * mem::size_of::<f32>()) as i32);
        gl.enable_vertex_attrib_array(1);

        let ebo = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        let num_indices = idx.len() as i32;

        let buf_indices = unsafe { js_sys::Uint16Array::view(&idx) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &buf_indices,
            WebGl2RenderingContext::STATIC_DRAW
        );

        // Load the texture of the gaussian kernel
        let ang2pix = [
            Texture2D::create(
                gl,
                "ang2pixd0",
                &resources.get_filename("ang2pixd0").unwrap(),
                &[
                    (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::NEAREST),
                    (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST),
                    
                    // Prevents s-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::REPEAT),
                    // Prevents t-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::REPEAT),
                ],
                FormatImageType::PNG
            ),
            Texture2D::create(
                gl,
                "ang2pixd1",
                &resources.get_filename("ang2pixd1").unwrap(),
                &[
                    (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::NEAREST),
                    (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::NEAREST),
                    
                    // Prevents s-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::REPEAT),
                    // Prevents t-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::REPEAT),
                ],
                FormatImageType::PNG
            ),
            Texture2D::create(
                gl,
                "ang2pixd2",
                &resources.get_filename("ang2pixd2").unwrap(),
                &[
                    (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR),
                    (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR),
                    
                    // Prevents s-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE),
                    // Prevents t-coordinate wrapping (repeating)
                    (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE),
                ],
                FormatImageType::PNG
            ),
        ];

        let gl = gl.clone();
        RayTracer {
            gl,

            vao,

            vbo,
            ebo,

            num_indices,
            ang2pix
        }
    }

    pub fn bind(&self) {
        self.gl.bind_vertex_array(Some(&self.vao));
    }

    pub fn unbind(&self) {
        self.gl.bind_vertex_array(None);
    }

    pub fn draw<'a>(&self, shader: &ShaderBound<'a>) {
        /*shader.attach_uniform("ang2pixd[0]", &self.ang2pix[0])
            .attach_uniform("ang2pixd[1]", &self.ang2pix[1])
            .attach_uniform("ang2pixd[2]", &self.ang2pix[2]);*/

        self.gl.draw_elements_with_i32(
            //WebGl2RenderingContext::LINE_STRIP,
            WebGl2RenderingContext::TRIANGLES,
            self.num_indices,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0
        ); 
    }
}

impl Drop for RayTracer {
    fn drop(&mut self) {
        self.gl.delete_vertex_array(Some(&self.vao));
        self.gl.delete_buffer(Some(&self.vbo));
        self.gl.delete_buffer(Some(&self.ebo));
    }
}