//! Mostly a carbon-copy of `webgl1.rs`.

use {
    js_sys::WebAssembly,
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::{
        WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader,
        WebGlTexture, WebGlVertexArrayObject,
    },
};

use std::borrow::Cow;

use cgmath::Vector2;
use {
    egui::{
        self,
        emath::vec2,
        epaint::{Color32, Texture},
    },
};
use web_sys::console;

use crate::{core::{VecData, VertexArrayObject}, shader::{Shader, ShaderId, ShaderManager}};
type Gl = WebGl2RenderingContext;
macro_rules! loog {
    ($l:expr) => { crate::log(&format("{:?}", $l)); }
}
pub struct WebGl2Painter {
    canvas_id: String,
    canvas: web_sys::HtmlCanvasElement,
    gl: WebGl2Context,
    shader: Shader,
    /*index_buffer: WebGlBuffer,
    pos_buffer: WebGlBuffer,
    tc_buffer: WebGlBuffer,
    color_buffer: WebGlBuffer,*/
    vao: VertexArrayObject,
    post_process: PostProcess,

    egui_texture: Texture2D,
    egui_texture_version: Option<u64>,

    /// `None` means unallocated (freed) slot.
    user_textures: Vec<Option<UserTexture>>,
}

#[derive(Default)]
struct UserTexture {
    size: (usize, usize),

    /// Pending upload (will be emptied later).
    pixels: Vec<u8>,

    /// Lazily uploaded
    gl_texture: Option<Texture2D>,
}

use crate::webgl_ctx::WebGl2Context;
use crate::core::Texture2D;
impl WebGl2Painter {
    pub fn new(gl: WebGl2Context) -> Result<WebGl2Painter, JsValue> {
        let canvas = gl.canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let canvas_id = canvas.id();
        let egui_texture = Texture2D::create_empty_unsized(
            &gl,
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
        )?;
        /*let egui_texture = gl.create_texture().unwrap();
        gl.bind_texture(Gl::TEXTURE_2D, Some(&egui_texture));
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);*/        
        let shader = Shader::new(
            &gl,
            include_str!("shaders/main_vertex_100es.glsl"),
            include_str!("shaders/main_fragment_100es.glsl"),
        )?;

        let mut vao = VertexArrayObject::new(&gl);
        shader
            .bind(&gl)
                .bind_vertex_array_object(&mut vao)
                    // positions and texcoords buffers
                    .add_array_buffer(
                        8 * std::mem::size_of::<f32>(),
                        &[2, 2, 4],
                        &[0, 2 * std::mem::size_of::<f32>(), 4 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::STREAM_DRAW,
                        VecData(&Vec::<f32>::new()),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STREAM_DRAW,
                        VecData(&Vec::<u16>::new()),
                    )
            // Unbind the buffer
            .unbind();
        
        /*let index_buffer = gl.create_buffer().ok_or("failed to create index_buffer")?;
        let pos_buffer = gl.create_buffer().ok_or("failed to create pos_buffer")?;
        let tc_buffer = gl.create_buffer().ok_or("failed to create tc_buffer")?;
        let color_buffer = gl.create_buffer().ok_or("failed to create color_buffer")?;*/

        let post_process =
            PostProcess::new(gl.clone(), canvas.width() as i32, canvas.height() as i32)?;

        Ok(WebGl2Painter {
            canvas_id: canvas_id.to_owned(),
            canvas,
            gl,
            shader,
            /*index_buffer,
            pos_buffer,
            tc_buffer,
            color_buffer,*/
            vao,
            post_process,
            egui_texture,
            egui_texture_version: None,
            user_textures: Default::default(),
        })
    }

    fn alloc_user_texture_index(&mut self) -> usize {
        for (index, tex) in self.user_textures.iter_mut().enumerate() {
            if tex.is_none() {
                *tex = Some(Default::default());
                return index;
            }
        }
        let index = self.user_textures.len();
        self.user_textures.push(Some(Default::default()));
        index
    }

    fn alloc_user_texture(
        &mut self,
        size: (usize, usize),
        srgba_pixels: &[Color32],
    ) -> egui::TextureId {
        let index = self.alloc_user_texture_index();
        assert_eq!(
            size.0 * size.1,
            srgba_pixels.len(),
            "Mismatch between texture size and texel count"
        );

        if let Some(Some(user_texture)) = self.user_textures.get_mut(index) {
            let mut pixels: Vec<u8> = Vec::with_capacity(srgba_pixels.len() * 4);
            for srgba in srgba_pixels {
                pixels.push(srgba.r());
                pixels.push(srgba.g());
                pixels.push(srgba.b());
                pixels.push(srgba.a());
            }

            *user_texture = UserTexture {
                size,
                pixels,
                gl_texture: None,
            };
        }

        egui::TextureId::User(index as u64)
    }

    fn free_user_texture(&mut self, id: egui::TextureId) {
        if let egui::TextureId::User(id) = id {
            let index = id as usize;
            if index < self.user_textures.len() {
                self.user_textures[index] = None;
            }
        }
    }

    pub fn get_texture(&self, texture_id: egui::TextureId) -> Option<&Texture2D> {
        match texture_id {
            egui::TextureId::Egui => Some(&self.egui_texture),
            egui::TextureId::User(id) => self
                .user_textures
                .get(id as usize)?
                .as_ref()?
                .gl_texture
                .as_ref(),
        }
    }

    fn upload_user_textures(&mut self) {
        let gl = &self.gl;
        for user_texture in self.user_textures.iter_mut().flatten() {
            crate::log("sdsdfs");
            if user_texture.gl_texture.is_none() {
                let pixels = std::mem::take(&mut user_texture.pixels);

                let gl_texture = Texture2D::create_empty(
                    &gl,
                    user_texture.size.0 as i32,
                    user_texture.size.1 as i32,
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
                    crate::image_fmt::FormatImageType::PNG
                ).unwrap();

                crate::log(
                    &format!("width: {:?}", user_texture.size.0),
                );
                crate::log(
                    &format!("height: {:?}", user_texture.size.1),
                );
                /*let gl_texture = gl.create_texture().unwrap();
                gl.bind_texture(Gl::TEXTURE_2D, Some(&gl_texture));
                gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32);
                gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32);
                gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::LINEAR as i32);
                gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::LINEAR as i32);

                gl.bind_texture(Gl::TEXTURE_2D, Some(&gl_texture));

                let level = 0;
                let internal_format = Gl::SRGB8_ALPHA8;
                let border = 0;
                let src_format = Gl::RGBA;
                let src_type = Gl::UNSIGNED_BYTE;
                gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);
                gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    Gl::TEXTURE_2D,
                    level,
                    internal_format as i32,
                    user_texture.size.0 as i32,
                    user_texture.size.1 as i32,
                    border,
                    src_format,
                    src_type,
                    Some(&pixels),
                )
                .unwrap();*/
                gl_texture.bind()
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(0, 0, user_texture.size.0 as i32, user_texture.size.1 as i32, Some(&pixels));

                user_texture.gl_texture = Some(gl_texture);
            }
        }
    }

    /*#[deprecated = "Use: `NativeTexture::register_native_texture` instead"]
    pub fn register_webgl_texture(&mut self, texture: WebGlTexture) -> egui::TextureId {
        let id = self.alloc_user_texture_index();
        if let Some(Some(user_texture)) = self.user_textures.get_mut(id) {
            *user_texture = UserTexture {
                size: (0, 0),
                pixels: vec![],
                gl_texture: None,
            }
        }
        egui::TextureId::User(id as u64)
    }*/

    fn paint_mesh(&mut self, mesh: &egui::epaint::Mesh16) -> Result<(), JsValue> {
        debug_assert!(mesh.is_valid());
        let mut vertices = Vec::with_capacity(8 * mesh.vertices.len());
        //let mut colors: Vec<u8> = Vec::with_capacity(4 * mesh.vertices.len());
        for v in &mesh.vertices {
            vertices.push(v.pos.x);
            vertices.push(v.pos.y);
            vertices.push(v.uv.x);
            vertices.push(v.uv.y);
            vertices.push((v.color[0] as f32)/255.0);
            vertices.push((v.color[1] as f32)/255.0);
            vertices.push((v.color[2] as f32)/255.0);
            vertices.push((v.color[3] as f32)/255.0);
        }

        // --------------------------------------------------------------------

        let gl = &self.gl;

        let shader_bound = self.shader
            .bind(gl);
        let mut shader_bound = shader_bound
                .bind_vertex_array_object(&mut self.vao);
        shader_bound
            .update_array(0, Gl::STREAM_DRAW, VecData(&vertices))
            .update_element_array(Gl::STREAM_DRAW, VecData(&mesh.indices));
            
        
        /*let indices_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let indices_ptr = mesh.indices.as_ptr() as u32 / 2;
        let indices_array = js_sys::Int16Array::new(&indices_memory_buffer)
            .subarray(indices_ptr, indices_ptr + mesh.indices.len() as u32);

        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        gl.buffer_data_with_array_buffer_view(
            Gl::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            Gl::STREAM_DRAW,
        );

        // --------------------------------------------------------------------

        let pos_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let pos_ptr = positions.as_ptr() as u32 / 4;
        let pos_array = js_sys::Float32Array::new(&pos_memory_buffer)
            .subarray(pos_ptr, pos_ptr + positions.len() as u32);

        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.pos_buffer));
        gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &pos_array, Gl::STREAM_DRAW);

        let a_pos_loc = gl.get_attrib_location(&shader.program, "a_pos");
        assert!(a_pos_loc >= 0);
        let a_pos_loc = a_pos_loc as u32;

        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(a_pos_loc, 2, Gl::FLOAT, normalize, stride, offset);
        gl.enable_vertex_attrib_array(a_pos_loc);

        // --------------------------------------------------------------------

        let tc_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let tc_ptr = tex_coords.as_ptr() as u32 / 4;
        let tc_array = js_sys::Float32Array::new(&tc_memory_buffer)
            .subarray(tc_ptr, tc_ptr + tex_coords.len() as u32);

        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.tc_buffer));
        gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &tc_array, Gl::STREAM_DRAW);

        let a_tc_loc = gl.get_attrib_location(&self.program, "a_tc");
        assert!(a_tc_loc >= 0);
        let a_tc_loc = a_tc_loc as u32;

        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(a_tc_loc, 2, Gl::FLOAT, normalize, stride, offset);
        gl.enable_vertex_attrib_array(a_tc_loc);

        // --------------------------------------------------------------------

        let colors_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()?
            .buffer();
        let colors_ptr = colors.as_ptr() as u32;
        let colors_array = js_sys::Uint8Array::new(&colors_memory_buffer)
            .subarray(colors_ptr, colors_ptr + colors.len() as u32);

        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&self.color_buffer));
        gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &colors_array, Gl::STREAM_DRAW);

        let a_srgba_loc = gl.get_attrib_location(&self.program, "a_srgba");
        assert!(a_srgba_loc >= 0);
        let a_srgba_loc = a_srgba_loc as u32;

        let normalize = false;
        let stride = 0;
        let offset = 0;
        gl.vertex_attrib_pointer_with_i32(
            a_srgba_loc,
            4,
            Gl::UNSIGNED_BYTE,
            normalize,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(a_srgba_loc);
        */
        // --------------------------------------------------------------------

        gl.draw_elements_with_i32(
            Gl::TRIANGLES,
            mesh.indices.len() as i32,
            Gl::UNSIGNED_SHORT,
            0,
        );

        Ok(())
    }
}

impl epi::TextureAllocator for WebGl2Painter {
    fn alloc_srgba_premultiplied(
        &mut self,
        size: (usize, usize),
        srgba_pixels: &[egui::Color32],
    ) -> egui::TextureId {
        self.alloc_user_texture(size, srgba_pixels)
    }

    fn free(&mut self, id: egui::TextureId) {
        self.free_user_texture(id)
    }
}


impl egui_web::Painter for WebGl2Painter {
    fn as_tex_allocator(&mut self) -> &mut dyn epi::TextureAllocator {
        self
    }

    fn debug_info(&self) -> String {
        format!(
            "Stored canvas size: {} x {}\n\
             gl context size: {} x {}",
            self.canvas.width(),
            self.canvas.height(),
            self.gl.drawing_buffer_width(),
            self.gl.drawing_buffer_height(),
        )
    }

    /// id of the canvas html element containing the rendering
    fn canvas_id(&self) -> &str {
        &self.canvas_id
    }

    fn upload_egui_texture(&mut self, texture: &Texture) {
        if self.egui_texture_version == Some(texture.version) {
            return; // No change
        }

        let mut pixels: Vec<u8> = Vec::with_capacity(texture.pixels.len() * 4);
        for srgba in texture.srgba_pixels(1.0) {
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }

        let gl = &self.gl;
        //gl.bind_texture(Gl::TEXTURE_2D, Some(&self.egui_texture));

        let internal_format = Gl::SRGB8_ALPHA8;
        let src_format = Gl::RGBA;
        let src_type = Gl::UNSIGNED_BYTE;
        self.egui_texture.bind_mut()
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                dbg!(texture.width as i32), 
                dbg!(texture.height as i32),
                internal_format as i32,
                src_format,
                src_type,
                Some(&pixels)
            );
        console::log_1(
            &format!("width: {:?}", texture.width).into(),
        );
        console::log_1(
            &format!("height: {:?}", texture.height).into(),
        );

        /*gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Gl::TEXTURE_2D,
            level,
            internal_format as i32,
            texture.width as i32,
            texture.height as i32,
            border,
            src_format,
            src_type,
            Some(&pixels),
        )
        .unwrap();*/

        self.egui_texture_version = Some(texture.version);
    }

    fn clear(&mut self, clear_color: egui::Rgba) {
        let gl = &self.gl;

        //gl.disable(Gl::SCISSOR_TEST);

        let width = self.canvas.width() as i32;
        let height = self.canvas.height() as i32;
        gl.viewport(0, 0, width, height);

        /*let clear_color: Color32 = clear_color.into();
        gl.clear_color(
            clear_color[0] as f32 / 255.0,
            clear_color[1] as f32 / 255.0,
            clear_color[2] as f32 / 255.0,
            clear_color[3] as f32 / 255.0,
        );
        gl.clear(Gl::COLOR_BUFFER_BIT);*/
    }

    fn paint_meshes(
        &mut self,
        clipped_meshes: Vec<egui::ClippedMesh>,
        pixels_per_point: f32,
    ) -> Result<(), JsValue> {
        //self.upload_user_textures();

        self.post_process
            .begin(self.canvas.width() as i32, self.canvas.height() as i32)?;
        
        //self.gl.clear(Gl::COLOR_BUFFER_BIT);
        let screen_size_pixels = vec2(self.canvas.width() as f32, self.canvas.height() as f32);
        let screen_size_points = screen_size_pixels / pixels_per_point;
        //self.shader.bind(&self.gl)
        //.attach_uniform("u_screen_size", &Vector2::new(screen_size_points.x, screen_size_points.y));
            
        //gl.use_program(Some(&self.program));
        //gl.active_texture(Gl::TEXTURE0);

        //let u_sampler_loc = gl.get_uniform_location(&self.program, "u_sampler").unwrap();
        //gl.uniform1i(Some(&u_sampler_loc), 0);

        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes {
            if let Some(gl_texture) = self.get_texture(mesh.texture_id) {
                self.shader.bind(&self.gl)
                    .attach_uniform("u_screen_size", &Vector2::new(screen_size_points.x, screen_size_points.y));
                    //.attach_uniform("u_sampler", gl_texture);

                crate::log(
                    "rendering ui"
                );
                let clip_min_x = pixels_per_point * clip_rect.min.x;
                let clip_min_y = pixels_per_point * clip_rect.min.y;
                let clip_max_x = pixels_per_point * clip_rect.max.x;
                let clip_max_y = pixels_per_point * clip_rect.max.y;
                let clip_min_x = clip_min_x.clamp(0.0, screen_size_pixels.x);
                let clip_min_y = clip_min_y.clamp(0.0, screen_size_pixels.y);
                let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size_pixels.x);
                let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size_pixels.y);
                let clip_min_x = clip_min_x.round() as i32;
                let clip_min_y = clip_min_y.round() as i32;
                let clip_max_x = clip_max_x.round() as i32;
                let clip_max_y = clip_max_y.round() as i32;

                // scissor Y coordinate is from the bottom
                /*self.gl.scissor(
                    clip_min_x,
                    self.canvas.height() as i32 - clip_max_y,
                    clip_max_x - clip_min_x,
                    clip_max_y - clip_min_y,
                );*/

                for mesh in mesh.split_to_u16() {
                    self.paint_mesh(&mesh)?;
                }
            } else {
                egui_web::console_warn(format!(
                    "WebGL: Failed to find texture {:?}",
                    mesh.texture_id
                ));
            }
        }

        self.post_process.end(self.canvas.width() as i32, self.canvas.height() as i32);

        Ok(())
    }
}

/// Uses a framebuffer to render everything in linear color space and convert it back to sRGB
/// in a separate "post processing" step
struct PostProcess {
    gl: WebGl2Context,
    vao: VertexArrayObject,
    texture: Texture2D,
    //texture_size: (i32, i32),
    fbo: WebGlFramebuffer,
    shader: Shader,
}

impl PostProcess {
    fn new(gl: WebGl2Context, width: i32, height: i32) -> Result<PostProcess, JsValue> {
        let fbo = gl
            .create_framebuffer()
            .ok_or("failed to create framebuffer")?;
        gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&fbo));

        let texture = Texture2D::create_empty_with_format(
            &gl,
            width,
            height,
    &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::NEAREST,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::NEAREST,
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
            Gl::SRGB8_ALPHA8 as i32,
            Gl::RGBA,
            Gl::UNSIGNED_BYTE
        )?;
        texture.attach_to_framebuffer();

        gl.bind_framebuffer(Gl::FRAMEBUFFER, None);

        let shader = Shader::new(
            &gl,
            include_str!("shaders/post_vertex_100es.glsl"),
            include_str!("shaders/post_fragment_100es.glsl"),
        )?;

        let positions = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let indices = vec![0u8, 1, 2, 1, 2, 3];
        let mut vao = VertexArrayObject::new(&gl);
        shader
            .bind(&gl)
                .bind_vertex_array_object(&mut vao)
                    // positions and texcoords buffers
                    .add_array_buffer(
                        2 * std::mem::size_of::<f32>(),
                        &[2],
                        &[0],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(&positions),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(&indices),
                    )
            // Unbind the buffer
            .unbind();

        Ok(PostProcess {
            gl,
            vao,
            texture,
            fbo,
            shader,
        })
    }

    fn begin(&mut self, canvas_w: i32, canvas_h: i32) -> Result<(), JsValue> {
        let gl = &self.gl;

        if (canvas_w, canvas_h) != (self.texture.width() as i32, self.texture.height() as i32) {
            //gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
            //gl.delete_framebuffer(Some(&self.fbo));

            /*self.fbo = gl
                .create_framebuffer()
                .ok_or("failed to create framebuffer")?;
            gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));*/

            //gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
            /*gl.bind_texture(Gl::TEXTURE_2D, Some(&self.texture));
            gl.pixel_storei(Gl::UNPACK_ALIGNMENT, 1);
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0,
                Gl::SRGB8_ALPHA8 as i32,
                width,
                height,
                0,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                None,
            )?;
            gl.bind_text    ure(Gl::TEXTURE_2D, None);*/
            crate::log("resize screen framebuffer");
            //gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
            let size_w = canvas_w as usize;
            let size_h = canvas_h as usize;
            let pixels = [120_u8, 0, 0, 255].iter().cloned().cycle().take(4*size_h*size_w).collect::<Vec<_>>();
            self.texture.bind_mut()
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array( 
                    size_w as i32, 
                    size_h as i32, 
                    Gl::SRGB8_ALPHA8 as i32,
                    Gl::RGBA,
                    Gl::UNSIGNED_BYTE,
                    None
                );
            //self.texture.attach_to_framebuffer();
            /*self.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                self.texture.texture.as_ref(),
                0,
            );*/
            //gl.clear(Gl::COLOR_BUFFER_BIT);
            //gl.bind_framebuffer(Gl::FRAMEBUFFER, None);

            //tex_bound.unbind();
            //self.texture_size = (width, height);
        }
        gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&self.fbo));
        gl.viewport(0, 0, self.texture.width() as i32, self.texture.height() as i32);
        self.gl.enable(Gl::SCISSOR_TEST);
        self.gl.disable(Gl::CULL_FACE); // egui is not strict about winding order.
        self.gl.enable(Gl::BLEND);
        self.gl.blend_func(Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA); // premultiplied alpha
        Ok(())
    }

    fn end(&self, canvas_w: i32, canvas_h: i32) {
        let gl = &self.gl;

        gl.bind_framebuffer(Gl::FRAMEBUFFER, None);
        gl.viewport(0, 0, canvas_w, canvas_h);

        gl.disable(Gl::SCISSOR_TEST);

        self.shader.bind(gl)
            .attach_uniform("fbo_tex", &self.texture)
                .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(Gl::TRIANGLES, Some(6), Gl::UNSIGNED_BYTE);

        gl.disable(WebGl2RenderingContext::BLEND);
        self.gl.enable(Gl::CULL_FACE);
    }
}

impl Drop for PostProcess {
    fn drop(&mut self) {
        let gl = &self.gl;
        /*gl.delete_vertex_array(Some(&self.vao));
        gl.delete_buffer(Some(&self.pos_buffer));
        gl.delete_buffer(Some(&self.index_buffer));
        gl.delete_program(Some(&self.program));
        gl.delete_framebuffer(Some(&self.fbo));*/
        //gl.delete_texture(Some(&self.texture));

        // The webgl texture is deleted when the Texture2D is dropped
    }
}
/*
fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    gl: &WebGl2RenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        gl.attach_shader(&program, shader)
    }
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, Gl::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}*/