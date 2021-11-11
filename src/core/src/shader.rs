use al_core::shader::Shader;
use al_core::WebGl2Context;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

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

use wasm_bindgen::JsValue;
impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        match e {
            Error::ShaderAlreadyInserted { message } => {
                JsValue::from_str(&format!("Shader already inserted: {:?}", message))
            }
            Error::ShaderNotFound { message } => {
                JsValue::from_str(&format!("Shader not found: {:?}", message))
            }
            Error::FileNotFound { message } => {
                JsValue::from_str(&format!("Shader not found: {:?}", message))
            }
        }
    }
}

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct FileSrc {
    pub id: String,
    pub content: String,
}

use std::collections::hash_map::Entry;
use std::collections::HashMap;
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

use crate::projection::*;
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader;
    fn get_raster_shader_gray2color_unsigned<'a>(
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
    fn get_raytracer_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raytracer_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RayTracerVS"),
                    Cow::Borrowed("RayTracerGrayscale2ColorUnsignedFS"),
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerAitoffVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerMollVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerArcVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
                ),
            )
            .unwrap()
    }
}

use crate::projection::*;
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
    fn get_raster_shader_gray2colormap_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColormapUnsignedFS"),
                ),
            )
            .unwrap()
    }
    fn get_raster_shader_gray2color_unsigned<'a>(
        gl: &WebGl2Context,
        shaders: &'a mut ShaderManager,
    ) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("RasterizerGnomonicVS"),
                    Cow::Borrowed("RasterizerGrayscale2ColorUnsignedFS"),
                ),
            )
            .unwrap()
    }
}
