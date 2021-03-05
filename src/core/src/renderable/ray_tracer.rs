use crate::{
    camera::CameraViewPort,
    core::VertexArrayObject,
    renderable::projection::Projection,
    shader::{ShaderBound, ShaderManager},
    WebGl2Context,
};

pub trait RayTracingProjection {
    fn get_raytracer_vertex_array_object(raytracer: &RayTracer) -> &VertexArrayObject;
}

use crate::coo_conversion::CooSystem;
use crate::math::LonLat;
use crate::renderable::Triangulation;
fn create_vertices_array<P: Projection>(
    _gl: &WebGl2Context,
    camera: &CameraViewPort,
    _system: &CooSystem,
) -> (Vec<f32>, Vec<u16>) {
    let (vertices, idx) = Triangulation::new::<P>().into();

    let vertices = vertices
        .into_iter()
        .map(|pos_clip_space| {
            /*let pos_world_space = system.system_to_icrs_coo(
                P::clip_to_world_space(&pos_clip_space, camera.is_reversed_longitude()).unwrap()
            );*/
            let pos_world_space =
                P::clip_to_world_space(&pos_clip_space, camera.is_reversed_longitude()).unwrap();
            //let pos_world_space = system.to_icrs_j2000() * pos_world_space;
            let lonlat = pos_world_space.lonlat();

            // Cast all the double into float
            // simple precision because this buffer
            // is sent to the GPU
            vec![
                pos_clip_space.x as f32,
                pos_clip_space.y as f32,
                lonlat.lon().0 as f32,
                lonlat.lat().0 as f32,
                pos_world_space.x as f32,
                pos_world_space.y as f32,
                pos_world_space.z as f32,
            ]
        })
        .flatten()
        .collect::<Vec<_>>();

    (vertices, idx)
}

use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;
use web_sys::WebGlVertexArrayObject;

pub struct RayTracer {
    gl: WebGl2Context,

    vao: WebGlVertexArrayObject,

    vbo: WebGlBuffer,
    ebo: WebGlBuffer,

    num_indices: i32,
    //ang2pix: [Texture2D; 3],
}

use crate::Resources;
use std::mem;
impl RayTracer {
    pub fn new<P: Projection>(
        gl: &WebGl2Context,
        camera: &CameraViewPort,
        _shaders: &mut ShaderManager,
        _resources: &Resources,
        system: &CooSystem,
    ) -> RayTracer {
        let (vertices, idx) = create_vertices_array::<P>(gl, camera, system);

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        let buf_vertices = unsafe { js_sys::Float32Array::view(&vertices) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &buf_vertices,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // layout (location = 0) in vec2 pos_clip_space;
        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            (7 * mem::size_of::<f32>()) as i32,
            (0 * mem::size_of::<f32>()) as i32,
        );
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec2 lonlat;
        gl.vertex_attrib_pointer_with_i32(
            1,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            (7 * mem::size_of::<f32>()) as i32,
            (2 * mem::size_of::<f32>()) as i32,
        );
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec3 pos_world_space;
        gl.vertex_attrib_pointer_with_i32(
            2,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            (7 * mem::size_of::<f32>()) as i32,
            (4 * mem::size_of::<f32>()) as i32,
        );
        gl.enable_vertex_attrib_array(2);

        let ebo = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
        let num_indices = idx.len() as i32;

        let buf_indices = unsafe { js_sys::Uint16Array::view(&idx) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &buf_indices,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // Load the texture of the gaussian kernel
        /*let ang2pix = [
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
        ];*/

        let gl = gl.clone();
        RayTracer {
            gl,

            vao,

            vbo,
            ebo,

            num_indices,
            //ang2pix
        }
    }

    /*pub fn send_textures(surveys: &ImageSurveys, shader: &ShaderBound<'a>) {
        if self.is_ready() {
            // Send the textures
            let textures = self.get_allsky_textures();
            let mut num_textures = 0;
            for texture in textures.iter() {
                if texture.is_available() {
                    let texture_uniforms = TextureUniforms::new(
                        texture,
                        num_textures as i32
                    );

                    shader.attach_uniforms_from(&texture_uniforms);
                    num_textures += 1;
                }
            }
            num_textures += 1;
            //shader.attach_uniform("num_textures", &(num_textures as i32));
            shader.attach_uniforms_from(&self.config);
        }
    }*/

    pub fn bind(&self) {
        self.gl.bind_vertex_array(Some(&self.vao));
    }

    /*pub fn unbind(&self) {
        self.gl.bind_vertex_array(None);
    }*/

    pub fn draw<'a>(&self, _shader: &ShaderBound<'a>) {
        //shader.attach_uniform("ang2pixd", &self.ang2pix[0]);
        //self.gl.polygon_mode(WebGl2RenderingContext::FRONT_AND_BACK, WebGl2RenderingContext::LINES);
        self.gl.draw_elements_with_i32(
            //WebGl2RenderingContext::LINES,
            WebGl2RenderingContext::TRIANGLES,
            self.num_indices,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
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
