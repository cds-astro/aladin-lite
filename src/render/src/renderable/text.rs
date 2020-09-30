#![allow(non_snake_case)]

use crate::viewport::CameraViewPort;
use crate::core::{VertexArrayObject, SliceData};
use crate::color::Color;
use web_sys::WebGl2RenderingContext;

// Text manager struct responsible
// for loading the font image and parse
// the json config file
use serde::Deserialize;
// Standard of a config font file generated with fontbm is
// described here:
// http://www.angelcode.com/products/bmfont/doc/file_format.html
#[derive(Deserialize, Debug)]
struct Char {
    id: u8,         //	The character id.
    x: u32,         //  The left position of the character image in the texture.
    y: u32,         //	The top position of the character image in the texture.
    width: u32,     //	The width of the character image in the texture.
    height: u32,    //	The height of the character image in the texture.
    xoffset: u32,   //  How much the current position should be offset when copying the image from the texture to the screen.
    yoffset: u32,   //  How much the current position should be offset when copying the image from the texture to the screen.
    xadvance: u32,  //  How much the current position should be advanced after drawing the character.
    page: u8,       //  The texture page where the character image is found.
    chnl: u8,       //  The texture channel where the character image is found (1 = blue, 2 = green, 4 = red, 8 = alpha, 15 = all channels).
}

#[derive(Deserialize, Debug)]
struct Common {
    lineHeight: u32,    //	This is the distance in pixels between each line of text.
    base: u32,          //	The number of pixels from the absolute top of the line to the base of the characters.
    scaleW: i32,        //	The width of the texture, normally used to scale the x pos of the character image.
    scaleH: i32,        //	The height of the texture, normally used to scale the y pos of the character image.
    pages: u32,         //	The number of texture pages included in the font.
    packed: u8,         //	Set to 1 if the monochrome characters have been packed into each of the texture channels. In this case alphaChnl describes what is stored in each channel.
    alphaChnl: u8,      //	Set to 0 if the channel holds the glyph data, 1 if it holds the outline, 2 if it holds the glyph and the outline, 3 if its set to zero, and 4 if its set to one.
    redChnl: u8,        //  Set to 0 if the channel holds the glyph data, 1 if it holds the outline, 2 if it holds the glyph and the outline, 3 if its set to zero, and 4 if its set to one.
    greenChnl: u8,      //	Set to 0 if the channel holds the glyph data, 1 if it holds the outline, 2 if it holds the glyph and the outline, 3 if its set to zero, and 4 if its set to one.
    blueChnl: u8,       //	Set to 0 if the channel holds the glyph data, 1 if it holds the outline, 2 if it holds the glyph and the outline, 3 if its set to zero, and 4 if its set to one.
}

#[derive(Deserialize, Debug)]
struct FontConfig {
    common: Common,
    chars: Vec<Char>,
    pages: Vec<String>
}

impl FontConfig {
    fn get_character(&self, id: char) -> Result<&Char, String> {
        // A binary search can be done as self.chars is sorted
        // by its id character field
        let id = id as u8;
        let character = self.chars.binary_search_by(|probe| probe.id.cmp(&id));
        if let Ok(idx) = character {
            Ok(&self.chars[idx])
        } else {
            let mut msg = String::from("Character ") + &(id as char).to_string();
            msg += " not found!";
            Err(msg)
        }
    }

    fn get_width_texture(&self) -> i32  {
        self.common.scaleW
    }
    fn get_height_texture(&self) -> i32  {
        self.common.scaleH
    }
}

use std::error::Error;

use std::rc::Rc;
fn read_font_config_from_str(content: &str) -> Result<FontConfig, Box<dyn Error>> {
    // Read the JSON contents of the file as an instance of `FontConfig`.
    let font = serde_json::from_str(content);
    if let Err(msg) = font {
        unreachable!();
    } else {
        let font = font?;
        Ok(font)
    }
}

use crate::core::Texture2DArray;
pub struct TextManager {
    font_textures: Rc<Texture2DArray>,
    letters: Vec<LetterInstance>, // letters that will be drawn at the next frame
    num_letters: usize,

    config: FontConfig,

    // The color of the text
    color: Color,

    // The vertex array object 
    vertex_array_object: VertexArrayObject,
}

use web_sys::console;
use crate::core::VecData;
use cgmath::{Vector2, Vector4};

#[derive(Clone)]
#[repr(C)]
struct LetterInstance {
    pos_center_screen: Vector2<f32>, // Position of the center of the letter on the screen
    size_screen: Vector2<f32>, // Size of the letter in pixel on the screen

    offset_uv: Vector2<f32>, // Position in the texture of the bottom-left of the letter
    size_uv: Vector2<f32>, // Size of the letter in the texture uv space

    idx_page: f32
}

struct UVCharacter(Vector2<f32>, Vector2<f32>);


use crate::ShaderManager;
use crate::WebGl2Context;

use crate::renderable::projection::Projection;
use crate::FormatImageType;
use std::borrow::Cow;
use crate::shader::ShaderId;
impl TextManager {
    pub fn new(gl: &WebGl2Context, font: &str, shaders: &mut ShaderManager) -> TextManager {
        let config = read_font_config_from_str(font).unwrap();

        let width_texture = config.get_width_texture();
        let height_texture = config.get_height_texture();
        let paths = &config.pages[..];

        let font_textures = Texture2DArray::create_from_slice_images(
            gl,
            paths,
            width_texture,
            height_texture,
            &[
                (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR),
                (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR),    
                // Prevents s-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE),
                // Prevents t-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE),
            ],
            FormatImageType::PNG
        );

        // Create a simple VAO mapped on the screen
        let vertex_array_object = {
            let mut vao = VertexArrayObject::new(gl);

            let vertices = vec![
                -0.5_f32, 0.5_f32, 0.0, 0.0,
                0.5_f32, 0.5_f32, 1.0, 0.0,
                0.5_f32, -0.5_f32, 1.0, 1.0,
                -0.5_f32, -0.5_f32, 0.0, 1.0,
            ];

            let indices: Vec<u16> = vec![
                0, 1, 2,
                0, 2, 3,
            ];

            let mut vao = VertexArrayObject::new(gl);
            let max_num_of_letters = 20 * 10; // 10 labels vertically and 10 labels horizontally, each being
            // let is say, max 10 letters

            let shader = shaders.get(
                gl,
                &ShaderId(
                    Cow::Borrowed("TextVS"),
                    Cow::Borrowed("TextFS")
                )
            ).unwrap();
            shader.bind(gl)
                .bind_vertex_array_object(&mut vao)
                    // Store the UV and the offsets of the billboard in a VBO
                    .add_array_buffer(
                        4 * std::mem::size_of::<f32>(),
                        &[2, 2],
                        &[0, 2 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(vertices.as_ref()),
                    )
                    // Store the cartesian position of the center of the source in the a instanced VBO
                    .add_instanced_array_buffer(
                        std::mem::size_of::<LetterInstance>(),
                        &[2, 2, 2, 2, 1],
                        &[0, 2 * std::mem::size_of::<f32>(), 4 * std::mem::size_of::<f32>(), 6 * std::mem::size_of::<f32>(), 8 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        SliceData(&vec![0_f32; max_num_of_letters * 9]),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(indices.as_ref()),
                    )
                    // Unbind the buffer
                    .unbind();
            vao
        };

        let color = Color::new(1_f32, 0_f32, 0_f32, 0.2_f32);
        let letters = vec![];
        let num_letters = 0;
        TextManager {
            letters,
            num_letters,
            config,
            vertex_array_object,
            color,
            font_textures,
        }
    }

    pub fn add_text(&mut self, pos: &Vector2<f32>, text: &str) {
        let (w, h) = self.get_string_pixel_size(text);

        // The top left position of the character in pixels
        let mut pos_center_screen = pos - Vector2::new((w as f32)/2_f32, (h as f32)/2_f32);
        for c in text.chars() {
            let character = self.config.get_character(c).unwrap();
            let idx_page = character.page as f32;

            // Get the size in pixels of the character
            let size_screen = Vector2::new(character.width as f32, character.height as f32);
            let UVCharacter(offset_uv, size_uv) = self.get_texture_uv(&character);

            self.letters.push(LetterInstance {
                size_screen,
                pos_center_screen,
                offset_uv,
                size_uv,
                idx_page,
            });

            pos_center_screen += Vector2::new(character.xadvance as f32, 0_f32);
        }
    }

    // Utilities methods used by add_text
    fn get_texture_uv(&self, character: &Char) -> UVCharacter {
        let width_texture = self.config.get_width_texture() as f32;
        let height_texture = self.config.get_height_texture() as f32;
        let p3 = Vector2::new(
            (character.x as f32) / width_texture,
            (character.y as f32) / height_texture
        );
        let mut p0 = p3 + Vector2::new(0_f32, (character.height as f32) / height_texture);
        p0.y = 1_f32 - p0.y;
        let size = Vector2::new(
            (character.width as f32) / width_texture,
            (character.height as f32) / height_texture
        );
        UVCharacter(p0, size)
    }

    fn get_string_pixel_size(&self, content: &str) -> (u32, u32) {
        let tl_pos = Vector2::new(0, 0);
        let mut h = 0;
        let mut w = 0;
        for c in content.chars() {
            let character = self.config.get_character(c).unwrap();
            w += character.xadvance;
            h = h.max(character.height);
        }
        (w, h)
    }

    pub fn add_text_on_sphere<P: Projection>(&mut self, pos: &Vector4<f32>, text: &str, viewport: &CameraViewPort) {
        let r = viewport.get_inverted_model_mat();
        let pos_model_space = r * pos;

        let in_front_of_camera = P::is_front_of_camera(&pos_model_space);

        if !in_front_of_camera {
            // If not in front of the camera, we do nothing
            return;
        }

        let pos = P::world_to_screen_space(&pos_model_space, viewport);

        self.add_text(&pos, text);
    }

    pub fn update(&mut self) {
        let data = unsafe { crate::utils::flatten_vec::<LetterInstance, f32>(self.letters.clone()) };
        //crate::log(&format!("data {:?}", data));

        // Update the VAO one time each frame
        self.vertex_array_object.bind_for_update()
            .update_instanced_array(0, VecData(&data));

        self.num_letters = self.letters.len();
        self.letters.clear();
    }

    pub fn draw(
        &self,
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        viewport: &CameraViewPort,
    ) {
        let shader = shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("TextVS"),
                Cow::Borrowed("TextFS")
            )
        ).unwrap();

        crate::log(&format!("num letters {:?}", self.num_letters));
        shader.bind(gl)
            // Attach all the uniforms from the viewport
            .attach_uniforms_from(viewport)
            // Attach grid specialized uniforms
            .attach_uniform("text_color", &self.color)
            .attach_uniform("scaling", &0.5_f32)
            .attach_uniform("font_textures", &*self.font_textures)
            // Bind the Vertex Array Object for drawing
            .bind_vertex_array_object_ref(&self.vertex_array_object)
                .draw_elements_instanced_with_i32(
                    // Mode of render
                    WebGl2RenderingContext::TRIANGLES,
                    //WebGl2RenderingContext::LINES,
                    // Number of elements, by default None
                    0,
                    self.num_letters as i32
                );
    }
}

use crate::shader::SendUniforms;
use crate::shader::ShaderBound;

impl SendUniforms for TextManager {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a> {
        shader
    }
}