use al_core::shader::Shader;
use al_core::WebGlContext;

pub type VertId = &'static str;
pub type FragId = &'static str;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct ShaderId(pub VertId, pub FragId);

pub struct ShaderManager {
    // Compiled shaders stored in an HashMap
    shaders: HashMap<ShaderId, Shader>,
    src: HashMap<&'static str, &'static str>,
}

#[derive(Debug)]
pub enum Error {
    ShaderAlreadyInserted { message: &'static str },
    ShaderNotFound { message: &'static str },
    ShaderCompilingLinking { message: JsValue },
    FileNotFound { message: &'static str },
    Io { message: String },
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
            Error::ShaderCompilingLinking { message } => message,
            Error::Io { message } => message.into(),
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
    pub fn new() -> Result<ShaderManager, Error> {
        let src = crate::shaders::get_all();
        // Loop over the entries in the directory
        /*let _src = std::fs::read_dir("./shaders")
        .map_err(|e| Error::Io {
            message: e.to_string(),
        })?
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            console_log(&format!("aaa"));

            if path.is_file() {
                let file_name = path.to_str()?;

                console_log(&format!("{}", file_name));

                // read the file into a bufreader
                let file = File::open(file_name).ok()?;
                let mut reader = std::io::BufReader::new(file);
                let mut content = String::new();

                reader.read_to_string(&mut content).ok()?;

                Some((Cow::Owned(file_name.to_owned()), content))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();*/

        Ok(ShaderManager {
            shaders: HashMap::new(),
            src,
        })
    }

    pub fn get(&mut self, gl: &WebGlContext, id: ShaderId) -> Result<&Shader, Error> {
        let shader = match self.shaders.entry(id.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let ShaderId(vert_id, frag_id) = id;

                let &vert_src = self
                    .src
                    .get(vert_id)
                    .ok_or(Error::FileNotFound { message: vert_id })?;
                let &frag_src = self
                    .src
                    .get(frag_id)
                    .ok_or(Error::FileNotFound { message: frag_id })?;

                let shader = Shader::new(gl, vert_src, frag_src)
                    .map_err(|err| Error::ShaderCompilingLinking { message: err })?;
                v.insert(shader)
            }
        };

        Ok(shader)
    }
}

pub(crate) fn get_shader<'a>(
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
    vert: &'static str,
    frag: &'static str,
) -> Result<&'a Shader, JsValue> {
    shaders
        .get(gl, ShaderId(vert, frag))
        .map_err(|err| err.into())
}
