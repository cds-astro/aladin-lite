use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

fn compile_shader(
    gl: &WebGl2Context,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
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

fn link_program(
    gl: &WebGl2Context,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
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

fn get_active_uniform_locations(gl: &WebGl2Context, program: &WebGlProgram) -> UniformLocations {
    let num_uniforms = gl
        .get_program_parameter(program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
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

type UniformLocations = HashMap<String, Option<WebGlUniformLocation>>;

use std::collections::HashMap;
pub struct Shader {
    pub program: WebGlProgram,
    uniform_locations: UniformLocations,
}

use crate::WebGl2Context;
impl Shader {
    pub fn new(gl: &WebGl2Context, vert_src: &str, frag_src: &str) -> Result<Shader, String> {
        let vert_shader = compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, &vert_src)?;
        let frag_shader = compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, &frag_src)?;

        let program = link_program(gl, &vert_shader, &frag_shader)?;
        // Get the active uniforms
        let uniform_locations = get_active_uniform_locations(gl, &program);

        Ok(Shader {
            program,
            uniform_locations,
        })
    }

    pub fn bind<'a>(&'a self, gl: &WebGl2Context) -> ShaderBound<'a> {
        gl.use_program(Some(&self.program));
        let gl = gl.clone();
        ShaderBound { shader: self, gl }
    }
}

pub trait UniformType {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self);

    fn attach_uniform<'a>(name: &str, value: &Self, shader: &ShaderBound<'a>) {
        let location = shader.get_uniform_location(name);
        Self::uniform(&shader.gl, location, value);
    }
}

impl UniformType for i32 {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1i(location, *value);
    }
}
use crate::transfert_function::TransferFunction;
impl UniformType for TransferFunction {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1i(location, *value as i32);
    }
}
use crate::shaders::Colormap;
impl UniformType for Colormap {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1i(location, *value as i32);
    }
}

impl UniformType for f32 {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, *value);
    }
}
impl UniformType for f64 {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, *value as f32);
    }
}
use crate::time::Time;
impl UniformType for Time {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0);
    }
}

impl UniformType for &[f32] {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1fv_with_f32_array(location, value);
    }
}
impl UniformType for &[f64] {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        let values_f32 = value.iter().map(|i| *i as f32).collect::<Vec<_>>();
        gl.uniform1fv_with_f32_array(location, values_f32.as_slice());
    }
}
impl UniformType for &[i32] {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1iv_with_i32_array(location, value);
    }
}
use crate::Angle;
impl UniformType for Angle<f32> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0);
    }
}
impl UniformType for Angle<f64> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0 as f32);
    }
}

use cgmath::Vector2;
impl UniformType for Vector2<f32> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform2f(location, value.x, value.y);
    }
}
impl UniformType for Vector2<f64> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform2f(location, value.x as f32, value.y as f32);
    }
}

use cgmath::Vector3;
impl UniformType for Vector3<f32> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value.x, value.y, value.z);
    }
}
impl UniformType for Vector3<f64> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value.x as f32, value.y as f32, value.z as f32);
    }
}

impl UniformType for [f32; 3] {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform3f(location, value[0], value[1], value[2]);
    }
}

use cgmath::Vector4;
impl UniformType for Vector4<f32> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.x, value.y, value.z, value.w);
    }
}
impl UniformType for Vector4<f64> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(
            location,
            value.x as f32,
            value.y as f32,
            value.z as f32,
            value.w as f32,
        );
    }
}

use crate::color::Color;
impl UniformType for Color {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.red, value.green, value.blue, value.alpha);
    }
}
impl<'a> UniformType for &'a Color {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform4f(location, value.red, value.green, value.blue, value.alpha);
    }
}

use cgmath::Matrix4;
impl UniformType for Matrix4<f32> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform_matrix4fv_with_f32_array(location, false, value.as_ref() as &[f32; 16]);
    }
}
impl UniformType for Matrix4<f64> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        // Cast the matrix
        let mat_f32 = value.cast::<f32>().unwrap();
        gl.uniform_matrix4fv_with_f32_array(location, false, mat_f32.as_ref() as &[f32; 16]);
    }
}

use crate::core::Texture2D;
impl UniformType for Texture2D {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, value: &Self) {
        // 1. Bind the texture
        let tex = value.bind();

        // 2. Get its sampler idx and send it
        // to the the GPU as a i32 uniform
        let idx_sampler = tex.get_idx_sampler();
        gl.uniform1i(location, idx_sampler);
    }
}

/*use crate::core::Texture2DArrayBound;
impl<'a> UniformType for Texture2DArrayBound<'a> {
    fn uniform(gl: &WebGl2Context, location: Option<&WebGlUniformLocation>, tex: &Self) {
        // 1. Bind the texture array
        //let tex = value.bind();

        // 2. Get its sampler idx and send it
        // to the the GPU as a i32 uniform

    }
}*/

pub struct ShaderBound<'a> {
    pub shader: &'a Shader,
    gl: WebGl2Context,
}

use crate::core::{
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

    pub fn unbind(&'a self, gl: &WebGl2Context) -> &'a Shader {
        gl.use_program(None);
        self.shader
    }
}

pub trait SendUniforms {
    fn attach_uniforms<'a>(&self, shader: &'a ShaderBound<'a>) -> &'a ShaderBound<'a>;
}

pub type VertId = Cow<'static, str>;
pub type FragId = Cow<'static, str>;
type FileId = Cow<'static, str>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ShaderId(pub VertId, pub FragId);

pub struct ShaderManager {
    // Compiled shaders stored in an HashMap
    shaders: HashMap<ShaderId, Shader>,
    // Shaders sources coming from the javascript
    src: HashMap<FileId, String>,
}

#[derive(Debug)]
pub enum Error {
    ShaderAlreadyInserted { message: String },
    ShaderNotFound { message: String },
    FileNotFound { message: String },
}

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct FileSrc {
    pub id: String,
    pub content: String,
}

use std::collections::hash_map::Entry;
impl ShaderManager {
    pub fn new(_gl: &WebGl2Context, files: Vec<FileSrc>) -> Result<ShaderManager, Error> {
        let src = files
            .into_iter()
            .map(|file| {
                let FileSrc { id, content } = file;
                (Cow::Owned(id), content)
            })
            .collect::<HashMap<_, _>>();

        Ok(ShaderManager {
            shaders: HashMap::new(),
            src,
        })
    }

    pub fn get(&mut self, gl: &WebGl2Context, id: &ShaderId) -> Result<&Shader, Error> {
        let shader = match self.shaders.entry(id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let ShaderId(vert_id, frag_id) = id;
                let vert_src = self.src.get(vert_id).ok_or(Error::FileNotFound {
                    message: format!("Vert id {} not found", vert_id),
                })?;
                let frag_src = self.src.get(frag_id).ok_or(Error::FileNotFound {
                    message: format!("Frag id {} not found", frag_id),
                })?;

                let shader = Shader::new(&gl, &vert_src, &frag_src).unwrap();

                v.insert(shader)
            }
        };

        Ok(shader)
    }
}

use crate::renderable::projection::*;
use std::borrow::Cow;
pub trait GetShader {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;

    fn get_raytracer_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raytracer_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raytracer_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }

    fn get_raytracer_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raytracer_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}

impl GetShader for Aitoff {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
impl GetShader for Mollweide {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
impl GetShader for AzimuthalEquidistant {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
impl GetShader for Gnomonic {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
impl GetShader for Mercator {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMercatorVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMercatorVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMercatorVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMercatorVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMercatorVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
impl GetShader for Orthographic {
    fn get_raster_shader_color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerOrthoVS"),
                    Cow::Borrowed("RasterizerColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerOrthoVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerOrthoVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2colormap_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerOrthoVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapIntegerFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_integer<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerOrthoVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorIntegerFS"),
                ),
            )
            .unwrap()
    }
}
