//! Mostly a carbon-copy of `webgl1.rs`.

#[cfg(feature = "webgl2")]
pub type WebGlRenderingCtx = web_sys::WebGl2RenderingContext;
#[cfg(feature = "webgl1")]
pub type WebGlRenderingCtx = web_sys::WebGlRenderingContext;

use {
    wasm_bindgen::{prelude::*, JsCast},
    web_sys::{WebGlBuffer, WebGlTexture},
};

use al_core::shader::Shader;
use cgmath::Vector2;
use egui::{
    self,
    emath::vec2,
    epaint::{Color32, Texture},
};

use al_core::FrameBufferObject;
pub struct WebGl2Painter {
    pub canvas_id: String,
    pub canvas: web_sys::HtmlCanvasElement,
    gl: WebGlContext,
    shader: Shader,

    pos_buffer: WebGlBuffer,
    tc_buffer: WebGlBuffer,
    color_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,

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

use al_core::Texture2D;
use al_core::WebGlContext;
impl WebGl2Painter {
    pub fn new(aladin_lite_div: &str, gl: WebGlContext) -> Result<WebGl2Painter, JsValue> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            // Get the aladin div element
            .get_element_by_id(aladin_lite_div)
            .unwrap()
            // Inside it, retrieve the canvas
            .get_elements_by_class_name("aladin-imageCanvas")
            .get_with_index(0)
            .unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let canvas_id = canvas.id();
        let egui_texture = Texture2D::create_empty_unsized(
            &gl,
            &[
                (
                    WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                    WebGlRenderingCtx::LINEAR,
                ),
                (
                    WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                    WebGlRenderingCtx::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_S,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGlRenderingCtx::TEXTURE_WRAP_T,
                    WebGlRenderingCtx::CLAMP_TO_EDGE,
                ),
            ],
        )?;

        #[cfg(feature = "webgl1")]
        let shader = Shader::new(
            &gl,
            include_str!("../shaders/webgl1/main_vertex_100es.glsl"),
            include_str!("../shaders/webgl1/main_fragment_100es.glsl"),
        )?;
        #[cfg(feature = "webgl2")]
        let shader = Shader::new(
            &gl,
            include_str!("../shaders/webgl2/main_vertex_100es.glsl"),
            include_str!("../shaders/webgl2/main_fragment_100es.glsl"),
        )?;

        let pos_buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        let tc_buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        let color_buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        //gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(&pos_buffer));

        /*let num_bytes_per_f32 = std::mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 pos;
        gl.vertex_attrib_pointer_with_i32(
            0,
            2,
            WebGlRenderingCtx::FLOAT,
            false,
            8 * num_bytes_per_f32,
            0,
        );
        gl.enable_vertex_attrib_array(0);

        // layout (location = 1) in vec2 tx;
        gl.vertex_attrib_pointer_with_i32(
            1,
            2,
            WebGlRenderingCtx::FLOAT,
            false,
            8 * num_bytes_per_f32,
            (2 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(1);

        // layout (location = 2) in vec4 color;
        gl.vertex_attrib_pointer_with_i32(
            2,
            4,
            WebGlRenderingCtx::FLOAT,
            false,
            8 * num_bytes_per_f32,
            (4 * num_bytes_per_f32) as i32,
        );
        gl.enable_vertex_attrib_array(2);*/

        let index_buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        // Bind the buffer
        /*gl.bind_buffer(WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        let data = vec![0_u16, 1, 2];
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            unsafe { &js_sys::Uint16Array::view(&data) },
            WebGlRenderingCtx::STREAM_DRAW,
        );
        //gl.bind_vertex_array(None);

        gl.bind_buffer(WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, None);
        */
        Ok(WebGl2Painter {
            canvas_id: canvas_id.to_owned(),
            canvas,
            gl,
            shader,

            //vao,
            pos_buffer,
            tc_buffer,
            color_buffer,

            index_buffer,
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

    pub fn alloc_user_texture(
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
            if user_texture.gl_texture.is_none() {
                let pixels = std::mem::take(&mut user_texture.pixels);

                let gl_texture = Texture2D::create_from_raw_pixels::<al_core::format::RGBA8U>(
                    &gl,
                    user_texture.size.0 as i32,
                    user_texture.size.1 as i32,
                    &[
                        (
                            WebGlRenderingCtx::TEXTURE_MIN_FILTER,
                            WebGlRenderingCtx::LINEAR,
                        ),
                        (
                            WebGlRenderingCtx::TEXTURE_MAG_FILTER,
                            WebGlRenderingCtx::LINEAR,
                        ),
                        // Prevents s-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_S,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                        // Prevents t-coordinate wrapping (repeating)
                        (
                            WebGlRenderingCtx::TEXTURE_WRAP_T,
                            WebGlRenderingCtx::CLAMP_TO_EDGE,
                        ),
                    ],
                    Some(&pixels),
                )
                .unwrap();

                user_texture.gl_texture = Some(gl_texture);
            }
        }
    }

    #[deprecated = "Use: `NativeTexture::register_native_texture` instead"]
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
    }

    fn paint_mesh(
        &self,
        mesh: &egui::epaint::Mesh16,
        screen_size_points: &egui::Vec2,
    ) -> Result<(), JsValue> {
        //debug_assert!(mesh.is_valid());
        let mut positions = Vec::with_capacity(2 * mesh.vertices.len());
        let mut texcoords = Vec::with_capacity(2 * mesh.vertices.len());
        let mut colors = Vec::with_capacity(4 * mesh.vertices.len());

        //let mut colors: Vec<u8> = Vec::with_capacity(4 * mesh.vertices.len());
        for v in &mesh.vertices {
            positions.push(v.pos.x);
            positions.push(v.pos.y);
            texcoords.push(v.uv.x);
            texcoords.push(v.uv.y);
            colors.push(v.color[0] as f32);
            colors.push(v.color[1] as f32);
            colors.push(v.color[2] as f32);
            colors.push(v.color[3] as f32);
        }

        let gl = &self.gl;

        //self.gl.bind_vertex_array(Some(&self.vao));

        // Bind the buffer
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(&self.pos_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingCtx::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&positions) },
            WebGlRenderingCtx::STREAM_DRAW,
        );

        let num_bytes_per_f32 = std::mem::size_of::<f32>() as i32;
        // layout (location = 0) in vec2 pos;
        let pos_loc = self.shader.get_attrib_location(gl, "pos") as u32;
        gl.vertex_attrib_pointer_with_i32(pos_loc, 2, WebGlRenderingCtx::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(pos_loc);

        // layout (location = 1) in vec2 tx;
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(&self.tc_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingCtx::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&texcoords) },
            WebGlRenderingCtx::STREAM_DRAW,
        );
        let tx_loc = self.shader.get_attrib_location(gl, "tx") as u32;
        gl.vertex_attrib_pointer_with_i32(tx_loc, 2, WebGlRenderingCtx::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(tx_loc);

        // layout (location = 2) in vec4 color;
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(&self.color_buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingCtx::ARRAY_BUFFER,
            unsafe { &js_sys::Float32Array::view(&colors) },
            WebGlRenderingCtx::STREAM_DRAW,
        );
        let color_loc = self.shader.get_attrib_location(gl, "color") as u32;
        gl.vertex_attrib_pointer_with_i32(color_loc, 4, WebGlRenderingCtx::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(color_loc);

        gl.bind_buffer(
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            Some(&self.index_buffer),
        );
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            unsafe { &js_sys::Uint16Array::view(&mesh.indices) },
            WebGlRenderingCtx::STREAM_DRAW,
        );

        let shader = self.shader.bind(&self.gl);

        shader
            .attach_uniform(
                "u_screen_size",
                &Vector2::new(screen_size_points.x, screen_size_points.y),
            )
            .attach_uniform("u_sampler", self.get_texture(mesh.texture_id).unwrap());

        // The raster vao is bound at the lib.rs level
        self.gl.draw_elements_with_i32(
            //WebGlRenderingCtx::LINES,
            WebGlRenderingCtx::TRIANGLES,
            mesh.indices.len() as i32,
            WebGlRenderingCtx::UNSIGNED_SHORT,
            0,
        );
        //self.gl.bind_vertex_array(None);

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
        #[cfg(feature = "webgl1")]
        let (src_format, src_internal_format) = (
            web_sys::ExtSRgb::SRGB_ALPHA_EXT,
            web_sys::ExtSRgb::SRGB_ALPHA_EXT,
        );
        #[cfg(feature = "webgl2")]
        let (src_format, src_internal_format) =
            (WebGlRenderingCtx::RGBA, WebGlRenderingCtx::SRGB8_ALPHA8);

        let src_type = WebGlRenderingCtx::UNSIGNED_BYTE;
        let gl = &self.gl;
        self.egui_texture
            .bind_mut()
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                texture.width as i32,
                texture.height as i32,
                src_internal_format as i32,
                src_format,
                src_type,
                Some(&pixels),
            );

        self.egui_texture_version = Some(texture.version);
    }

    fn clear(&mut self, clear_color: egui::Rgba) {
        unimplemented!();
    }

    fn paint_meshes(
        &mut self,
        clipped_meshes: Vec<egui::ClippedMesh>,
        pixels_per_point: f32,
    ) -> Result<(), JsValue> {
        /* Upload user textures */
        self.upload_user_textures();

        /* Draw the ui */
        self.gl.enable(WebGlRenderingCtx::SCISSOR_TEST);
        self.gl.disable(WebGlRenderingCtx::CULL_FACE); // egui is not strict about winding order.

        let canvas_width = self.canvas.width();
        let canvas_height = self.canvas.height();

        let screen_size_pixels = vec2(canvas_width as f32, canvas_height as f32);
        let screen_size_points = screen_size_pixels / pixels_per_point;

        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes {
            if let Some(_) = self.get_texture(mesh.texture_id) {
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
                self.gl.scissor(
                    clip_min_x,
                    canvas_height as i32 - clip_max_y,
                    clip_max_x - clip_min_x,
                    clip_max_y - clip_min_y,
                );

                for mesh in mesh.split_to_u16() {
                    self.paint_mesh(&mesh, &screen_size_points)?;
                }
            } else {
                egui_web::console_warn(format!(
                    "WebGL: Failed to find texture {:?}",
                    mesh.texture_id
                ));
            }
        }

        self.gl.disable(WebGlRenderingCtx::SCISSOR_TEST);
        self.gl.enable(WebGlRenderingCtx::CULL_FACE);

        /* End draw the ui */

        Ok(())
    }
}
