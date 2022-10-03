use al_core::shader::Shader;
use al_core::text::LetterTexPosition;
use al_core::texture::Texture2D;
use al_core::webgl_ctx::WebGlContext;
use al_core::VertexArrayObject;

use std::collections::HashMap;

pub trait RenderManager {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn draw(&mut self, camera: &CameraViewPort) -> Result<(), JsValue>;
}

use cgmath::Matrix2;
struct LabelMeta {
    rot: Matrix2<f32>,
    color: ColorRGB,
    opacity: f32,
    screen_pos: Vector2<f32>,
    scale: f32,
    off_idx: u16,
    num_idx: u16,
}

pub struct TextRenderManager {
    gl: WebGlContext,
    shader: Shader,
    vao: VertexArrayObject,

    font_texture: Texture2D,
    text_size: f32,
    letters: HashMap<char, LetterTexPosition>,

    #[cfg(feature = "webgl2")]
    vertices: Vec<f32>,
    #[cfg(feature = "webgl1")]
    pos: Vec<f32>,
    #[cfg(feature = "webgl1")]
    tx: Vec<f32>,

    indices: Vec<u16>,
    labels: Vec<LabelMeta>,
}
use al_core::VecData;
use cgmath::{Basis2, Rad, Rotation2, Vector2};
use wasm_bindgen::JsValue;

use crate::camera::CameraViewPort;
use al_api::color::ColorRGB;
use web_sys::WebGl2RenderingContext;

impl TextRenderManager {
    /// Init the buffers, VAO and shader
    pub fn new(gl: WebGlContext) -> Result<Self, JsValue> {
        // Create the VAO for the screen
        #[cfg(feature = "webgl1")]
        let shader = Shader::new(
            &gl,
            include_str!("../../../glsl/webgl1/text/text_vertex.glsl"),
            include_str!("../../../glsl/webgl1/text/text_frag.glsl"),
        )?;
        #[cfg(feature = "webgl2")]
        let shader = Shader::new(
            &gl,
            include_str!("../../../glsl/webgl2/text/text_vertex.glsl"),
            include_str!("../../../glsl/webgl2/text/text_frag.glsl"),
        )?;
        let mut vao = VertexArrayObject::new(&gl);
        #[cfg(feature = "webgl2")]
        let vertices = vec![];
        #[cfg(feature = "webgl1")]
        let pos = vec![];
        #[cfg(feature = "webgl1")]
        let tx = vec![];

        let indices = vec![];
        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer(
                "vertices",
                4 * std::mem::size_of::<f32>(),
                &[2, 2],
                &[0, 2 * std::mem::size_of::<f32>()],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&vertices),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&indices),
            );
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            .add_array_buffer(
                2,
                "pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&pos),
            )
            .add_array_buffer(
                2,
                "tx",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&tx),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&indices),
            );
        let text_size = 16.0;
        let al_core::text::Font {
            size,
            bitmap,
            letters,
            ..
        } = al_core::text::rasterize_font(text_size);
        let font_texture = Texture2D::create_from_raw_pixels::<al_core::image::format::RGBA8U>(
            &gl,
            size.x as i32,
            size.y as i32,
            &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
            ],
            Some(&bitmap),
        )?;

        let labels = vec![];

        Ok(Self {
            gl,
            shader,
            vao,
            letters,
            font_texture,
            text_size,
            #[cfg(feature = "webgl2")]
            vertices: vec![],
            #[cfg(feature = "webgl1")]
            pos: vec![],
            #[cfg(feature = "webgl1")]
            tx: vec![],
            indices: vec![],
            labels,
        })
    }

    pub fn set_text_size(&mut self, text_size: f32) -> Result<(), JsValue> {
        self.text_size = text_size;

        let al_core::text::Font {
            size,
            bitmap,
            letters,
            font: _,
        } = al_core::text::rasterize_font(text_size);
        self.font_texture = Texture2D::create_from_raw_pixels::<al_core::image::format::RGBA8U>(
            &self.gl,
            size.x as i32,
            size.y as i32,
            &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
            ],
            Some(&bitmap),
        )?;

        self.letters = letters;

        Ok(())
    }

    pub fn text_size(&self) -> f32 {
        self.text_size
    }

    pub fn add_label<A: Into<Rad<f32>>>(
        &mut self,
        text: &str,
        screen_pos: &Vector2<f32>,
        scale: f32,
        color: &ColorRGB,
        opacity: f32,
        angle_rot: A,
    ) {
        // 1. Loop over the text chars to compute the size of the text to plot
        let (mut w, mut h) = (0, 0);
        for c in text.chars() {
            if let Some(l) = self.letters.get(&c) {
                w += l.x_advance;
                h = std::cmp::max(h, l.h);
            }
        }

        let x_pos = -(w as f32) * 0.5;
        let y_pos = -(h as f32) * 0.5;

        let f_tex_size = &self.font_texture.get_size();

        let mut x_offset = 0.0;

        let off_idx = self.indices.len() as u16;
        let mut num_idx = 0;

        for c in text.chars() {
            if let Some(l) = self.letters.get(&c) {
                let u1 = (l.x_min as f32) / (f_tex_size.0 as f32);
                let v1 = (l.y_min as f32) / (f_tex_size.1 as f32);

                let u2 = (l.x_min as f32 + l.w as f32) / (f_tex_size.0 as f32);
                let v2 = (l.y_min as f32) / (f_tex_size.1 as f32);

                let u3 = (l.x_min as f32 + l.w as f32) / (f_tex_size.0 as f32);
                let v3 = (l.y_min as f32 + l.h as f32) / (f_tex_size.1 as f32);

                let u4 = (l.x_min as f32) / (f_tex_size.0 as f32);
                let v4 = (l.y_min as f32 + l.h as f32) / (f_tex_size.1 as f32);

                #[cfg(feature = "webgl2")]
                let num_vertices = (self.vertices.len() / 4) as u16;
                #[cfg(feature = "webgl1")]
                let num_vertices = (self.pos.len() / 2) as u16;

                let xmin = l.bounds.xmin;
                let ymin = l.bounds.ymin + (l.h as f32);

                #[cfg(feature = "webgl2")]
                self.vertices.extend([
                    x_pos + x_offset + xmin,
                    y_pos - ymin,
                    u1,
                    v1,
                    x_pos + x_offset + (l.w as f32) + xmin,
                    y_pos - ymin,
                    u2,
                    v2,
                    x_pos + x_offset + (l.w as f32) + xmin,
                    y_pos + (l.h as f32) - ymin,
                    u3,
                    v3,
                    x_pos + x_offset + xmin,
                    y_pos + (l.h as f32) - ymin,
                    u4,
                    v4,
                ]);
                #[cfg(feature = "webgl1")]
                self.pos.extend([
                    x_pos + x_offset + xmin,
                    y_pos - ymin,
                    x_pos + x_offset + (l.w as f32) + xmin,
                    y_pos - ymin,
                    x_pos + x_offset + (l.w as f32) + xmin,
                    y_pos + (l.h as f32) - ymin,
                    x_pos + x_offset + xmin,
                    y_pos + (l.h as f32) - ymin,
                ]);
                #[cfg(feature = "webgl1")]
                self.tx.extend([u1, v1, u2, v2, u3, v3, u4, v4]);
                self.indices.extend([
                    num_vertices,
                    num_vertices + 2,
                    num_vertices + 1,
                    num_vertices,
                    num_vertices + 3,
                    num_vertices + 2,
                ]);
                num_idx += 6;

                x_offset += l.x_advance as f32;
            }
        }

        let angle: Rad<_> = angle_rot.into();
        let rot: Basis2<f32> = Rotation2::from_angle(angle);
        self.labels.push(LabelMeta {
            off_idx,
            num_idx,
            scale,
            color: *color,
            opacity,
            screen_pos: *screen_pos,
            rot: rot.into(),
        });
    }

    pub fn get_width_pixel_size(&self, content: &str) -> f64 {
        let mut w = 0;
        for c in content.chars() {
            if let Some(l) = self.letters.get(&c) {
                w += l.x_advance;
            }
        }

        w as f64
    }
}

impl RenderManager for TextRenderManager {
    fn begin_frame(&mut self) {
        #[cfg(feature = "webgl2")]
        self.vertices.clear();
        #[cfg(feature = "webgl1")]
        self.pos.clear();
        #[cfg(feature = "webgl1")]
        self.tx.clear();

        self.indices.clear();
        self.labels.clear();
    }

    fn end_frame(&mut self) {
        // update to the GPU
        #[cfg(feature = "webgl2")]
        self.vao
            .bind_for_update()
            .update_array(
                "vertices",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.vertices),
            )
            .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.indices));
        #[cfg(feature = "webgl1")]
        self.vao
            .bind_for_update()
            .update_array(
                "pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.pos),
            )
            .update_array(
                "tx",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.tx),
            )
            .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.indices));
    }

    fn draw(&mut self, camera: &CameraViewPort) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);
        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        ); // premultiplied alpha

        self.gl.disable(WebGl2RenderingContext::CULL_FACE);

        {
            let shader = self.shader.bind(&self.gl);
            shader
                .attach_uniform("u_sampler_font", &self.font_texture) // Font letters texture
                .attach_uniform("u_screen_size", &camera.get_screen_size());
            for label in self.labels.iter() {
                shader
                    .attach_uniform("u_color", &label.color) // Strengh of the kernel
                    .attach_uniform("u_opacity", &label.opacity)
                    .attach_uniform("u_screen_pos", &label.screen_pos)
                    .attach_uniform("u_rot", &label.rot)
                    .attach_uniform("u_scale", &label.scale)
                    .attach_uniform("u_dpi", &camera.get_dpi())
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(label.num_idx as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        (label.off_idx as i32) * (std::mem::size_of::<u16>() as i32),
                    );
            }
        }
        self.gl.enable(WebGl2RenderingContext::CULL_FACE);
        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
