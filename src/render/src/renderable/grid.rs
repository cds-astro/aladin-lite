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
pub struct ProjetedGrid {
    // The color of the grid
    color: Color,

    // The vertex array object of the screen in NDC
    vertex_array_object: VertexArrayObject,
}

use crate::renderable::projection::Projection;

use crate::ShaderManager;
use crate::WebGl2Context;

impl ProjetedGrid {
    pub fn new<P: Projection>(
        gl: &WebGl2Context,
        _camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        _text_manager: &TextManager
    ) -> ProjetedGrid {
        let vertex_array_object = {
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
        };

        let color = Color::new(0_f32, 1_f32, 0_f32, 0.2_f32);

        ProjetedGrid {
            color,

            vertex_array_object,
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
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        text_manager: &TextManager,
    ) {
        let shader = P::get_grid_shader(gl, shaders);
        //let great_circles = camera.get_great_circles_inside();

        shader.bind(gl)
            // Attach all the uniforms from the camera
            .attach_uniforms_from(camera)
            // Attach grid specialized uniforms
            .attach_uniform("grid_color", &self.color)
            .attach_uniform("model2world", camera.get_m2w())
            .attach_uniform("world2model", camera.get_w2m())
            //.attach_uniforms_from(great_circles)
            // Bind the Vertex Array Object for drawing
            .bind_vertex_array_object_ref(&self.vertex_array_object)
                .draw_elements_with_i32(
                    // Mode of render
                    WebGl2RenderingContext::TRIANGLES,
                    // Number of elements, by default None
                    None,
                    WebGl2RenderingContext::UNSIGNED_SHORT
                );
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
impl GridShaderProjection for AzimutalEquidistant {
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