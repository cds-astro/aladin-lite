use al_core::shader::Shader;
use al_core::WebGlContext;

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
    ShaderAlreadyInserted { message: &'static str },
    ShaderNotFound { message: &'static str },
    ShaderCompilingLinking { message: JsValue },
    FileNotFound { message: &'static str },
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
            Error::ShaderCompilingLinking { message } => message
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
    pub fn new(_gl: &WebGlContext, files: Vec<FileSrc>) -> Result<ShaderManager, Error> {
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

    pub fn get(&mut self, gl: &WebGlContext, id: &ShaderId) -> Result<&Shader, Error> {
        let shader = match self.shaders.entry(id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let ShaderId(vert_id, frag_id) = id;
                let vert_src = self.src.get(vert_id).ok_or(Error::FileNotFound {
                    message: "Vert not found",
                })?;
                let frag_src = self.src.get(frag_id).ok_or(Error::FileNotFound {
                    message: "Frag not found",
                })?;

                let shader = Shader::new(gl, vert_src, frag_src).map_err(|err| Error::ShaderCompilingLinking {
                    message: err,
                })?;
                v.insert(shader)
            }
        };

        Ok(shader)
    }
}
use crate::Abort;
use std::borrow::Cow;
/*use paste::paste;
macro_rules! define_shader_getter {
    ($renderer_type:ident, $shader_type:ident, $vert_key:tt, $frag_key:tt) => {
        paste! {
            pub fn [< get_ $renderer_type _shader_ $shader_type >]<'a>(
                gl: &WebGlContext,
                shaders: &'a mut ShaderManager
            ) -> &'a Shader {
                shaders.get(
                    gl,
                    &ShaderId(
                        Cow::Borrowed($vert_key),
                        Cow::Borrowed($frag_key),
                    ),
                )
                .unwrap_abort()
            }
        }
    }
}

/* Raytracer shaders */
define_shader_getter!(raytracer, color, "RayTracerVS", "RayTracerColorFS");
define_shader_getter!(raytracer, gray2colormap, "RayTracerVS", "RayTracerGrayscale2ColormapFS");
define_shader_getter!(raytracer, gray2color, "RayTracerVS", "RayTracerGrayscale2ColorFS");
define_shader_getter!(raytracer, gray2colormap_integer, "RayTracerVS", "RayTracerGrayscale2ColormapIntegerFS");
define_shader_getter!(raytracer, gray2color_integer, "RayTracerVS", "RayTracerGrayscale2ColorIntegerFS");
define_shader_getter!(raytracer, gray2colormap_unsigned, "RayTracerVS", "RayTracerGrayscale2ColormapUnsignedFS");
define_shader_getter!(raytracer, gray2color_unsigned, "RayTracerVS", "RayTracerGrayscale2ColorUnsignedFS");

/* Rasterizer shaders */
define_shader_getter!(raster, color, "RasterizerVS", "RasterizerColorFS");
define_shader_getter!(raster, gray2colormap, "RasterizerVS", "RasterizerGrayscale2ColormapFS");
define_shader_getter!(raster, gray2color, "RasterizerVS", "RasterizerGrayscale2ColorFS");
define_shader_getter!(raster, gray2colormap_integer, "RasterizerVS", "RasterizerGrayscale2ColormapIntegerFS");
define_shader_getter!(raster, gray2color_integer, "RasterizerVS", "RasterizerGrayscale2ColorIntegerFS");
define_shader_getter!(raster, gray2colormap_unsigned, "RasterizerVS", "RasterizerGrayscale2ColormapUnsignedFS");
define_shader_getter!(raster, gray2color_unsigned, "RasterizerVS", "RasterizerGrayscale2ColorUnsignedFS");

/* Pass shaders */
define_shader_getter!(pass, post, "PostVS", "PostFS");

/* Catalog shaders */
define_shader_getter!(catalog, ait, "CatalogAitoffVS", "CatalogFS");
define_shader_getter!(catalog, mol, "CatalogMollVS", "CatalogFS");
define_shader_getter!(catalog, arc, "CatalogArcVS", "CatalogFS");
define_shader_getter!(catalog, hpx, "CatalogHEALPixVS", "CatalogFS");
define_shader_getter!(catalog, mer, "CatalogMercatVS", "CatalogFS");
define_shader_getter!(catalog, ort, "CatalogOrthoVS", "CatalogOrthoFS");
define_shader_getter!(catalog, tan, "CatalogTanVS", "CatalogFS");*/
pub(crate) fn get_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager, vert: &'static str, frag: &'static str) -> Result<&'a Shader, JsValue> {
    shaders.get(
        gl,
        &ShaderId(Cow::Borrowed(vert), Cow::Borrowed(frag)),
    ).map_err(|err| err.into())
}