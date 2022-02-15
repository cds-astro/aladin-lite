use web_sys::{WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::webgl_ctx::WebGlRenderingCtx;
fn compile_shader(
    gl: &WebGlContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingCtx::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    gl: &WebGlContext,
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
        .get_program_parameter(&program, WebGlRenderingCtx::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

type UniformLocations = HashMap<String, Option<WebGlUniformLocation>>;
fn get_active_uniform_locations(gl: &WebGlContext, program: &WebGlProgram) -> UniformLocations {
    let num_uniforms = gl
        .get_program_parameter(program, WebGlRenderingCtx::ACTIVE_UNIFORMS)
        .as_f64()
        .unwrap();

    let uniforms = (0..num_uniforms as u32)
        .map(|idx_uniform| {
            let active_uniform = gl.get_active_uniform(program, idx_uniform).unwrap();
            let name_uniform = active_uniform.name();
            // Get the location by the name of the active uniform
            let location_uniform = gl.get_uniform_location(&program, &name_uniform);
            (name_uniform, location_uniform)
        })
        .collect::<HashMap<_, _>>();
    uniforms
}

use std::collections::HashMap;
pub struct Shader {
    pub program: WebGlProgram,
    uniform_locations: UniformLocations,
}

use crate::webgl_ctx::WebGlContext;
impl Shader {
    pub fn new(gl: &WebGlContext, vert_src: &str, frag_src: &str) -> Result<Shader, String> {
        crate::log::log(&format!("Compiling shader: {:?}\n{:?}", vert_src, frag_src));

        let vert_shader = compile_shader(gl, WebGlRenderingCtx::VERTEX_SHADER, &vert_src)?;
        let frag_shader = compile_shader(gl, WebGlRenderingCtx::FRAGMENT_SHADER, &frag_src)?;

        let program = link_program(gl, &[vert_shader, frag_shader])?;
        // Get the active uniforms
        let uniform_locations = get_active_uniform_locations(gl, &program);

        Ok(Shader {
            program,
            uniform_locations,
        })
    }

    pub fn bind<'a>(&'a self, gl: &WebGlContext) -> ShaderBound<'a> {
        unsafe { CUR_IDX_TEX_UNIT = 0 };
        gl.use_program(Some(&self.program));
        let gl = gl.clone();
        ShaderBound { shader: self, gl }
    }

    pub fn get_attrib_location(&self, gl: &WebGlContext, name: &str) -> i32 {
        gl.get_attrib_location(&self.program, name)
    }
}

pub trait UniformType {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self);

    fn attach_uniform<'a>(name: &str, value: &Self, shader: &ShaderBound<'a>) {
        let location = shader.get_uniform_location(name);
        Self::uniform(&shader.gl, location, value);
    }
}
impl UniformType for bool {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1i(location, *value as i32);
    }
}
impl UniformType for i32 {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1i(location, *value);
    }
}

impl UniformType for f32 {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, *value);
    }
}
impl UniformType for f64 {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, *value as f32);
    }
}

impl UniformType for &[f32] {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1fv_with_f32_array(location, value);
    }
}
impl UniformType for &[f64] {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        let values_f32 = value.iter().map(|i| *i as f32).collect::<Vec<_>>();
        gl.uniform1fv_with_f32_array(location, values_f32.as_slice());
    }
}
impl UniformType for &[i32] {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1iv_with_i32_array(location, value);
    }
}

use cgmath::Vector2;
impl UniformType for Vector2<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform2f(location, value.x, value.y);
    }
}
impl UniformType for Vector2<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform2f(location, value.x as f32, value.y as f32);
    }
}

use cgmath::Vector3;
impl UniformType for Vector3<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value.x, value.y, value.z);
    }
}
impl UniformType for Vector3<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value.x as f32, value.y as f32, value.z as f32);
    }
}

impl UniformType for [f32; 3] {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value[0], value[1], value[2]);
    }
}

use cgmath::Vector4;
impl UniformType for Vector4<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.x, value.y, value.z, value.w);
    }
}
impl UniformType for Vector4<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(
            location,
            value.x as f32,
            value.y as f32,
            value.z as f32,
            value.w as f32,
        );
    }
}

use cgmath::Matrix2;
impl UniformType for Matrix2<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform_matrix2fv_with_f32_array(location, false, value.as_ref() as &[f32; 4]);
    }
}

use cgmath::Matrix4;
impl UniformType for Matrix4<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform_matrix4fv_with_f32_array(location, false, value.as_ref() as &[f32; 16]);
    }
}
impl UniformType for Matrix4<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        // Cast the matrix
        let mat_f32 = value.cast::<f32>().unwrap();
        gl.uniform_matrix4fv_with_f32_array(location, false, mat_f32.as_ref() as &[f32; 16]);
    }
}
use super::texture::CUR_IDX_TEX_UNIT;
use super::texture::Texture2D;
impl UniformType for Texture2D {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, tex: &Self) {
        // 1. Active the texture unit of the texture
        unsafe {
            let tex = tex
                .active_texture(CUR_IDX_TEX_UNIT)
                // 2. Bind the texture to that texture unit
                .bind();

            gl.uniform1i(location, CUR_IDX_TEX_UNIT as i32);

            CUR_IDX_TEX_UNIT += 1;
            crate::log(&format!("idx tex unit {:?}", CUR_IDX_TEX_UNIT));
        };


    }
}

pub struct ShaderBound<'a> {
    pub shader: &'a Shader,
    gl: WebGlContext,
}

use crate::object::{
    ShaderVertexArrayObjectBound, ShaderVertexArrayObjectBoundRef, VertexArrayObject,
};
impl<'a> ShaderBound<'a> {
    fn get_uniform_location(&self, name: &str) -> Option<&WebGlUniformLocation> {
        if let Some(location) = self.shader.uniform_locations.get(name) {
            location.as_ref()
        } else {
            None
        }
    }

    pub fn attach_uniform<T: UniformType>(&self, name: &str, value: &T) -> &Self {
        T::attach_uniform(name, value, self);

        self
    }

    pub fn attach_uniforms_from<T: SendUniforms>(&'a self, t: &T) -> &'a Self {
        t.attach_uniforms(self);

        self
    }

    pub fn bind_vertex_array_object<'b>(
        &'a self,
        vao: &'b mut VertexArrayObject,
    ) -> ShaderVertexArrayObjectBound<'b, 'a> {
        vao.bind(self)
    }

    pub fn bind_vertex_array_object_ref<'b>(
        &'a self,
        vao: &'b VertexArrayObject,
    ) -> ShaderVertexArrayObjectBoundRef<'b, 'a> {
        vao.bind_ref(self)
    }

    pub fn unbind(&'a self, gl: &WebGlContext) -> &'a Shader {
        gl.use_program(None);
        self.shader
    }

    pub fn get_attrib_location(&self, gl: &WebGlContext, name: &str) -> i32 {
        self.shader.get_attrib_location(gl, name)
    }
}

impl<'a> Drop for ShaderBound<'a> {
    fn drop(&mut self) {
        self.unbind(&self.gl);
    }
}

pub trait SendUniforms {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a>;
}
