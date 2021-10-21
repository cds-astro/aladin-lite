mod object;
mod texture;
mod shader;
mod webgl_ctx;

pub use texture::{IdxTextureUnit, Pixel, Texture2D, Texture2DBound};
pub use texture::Texture2DArray;

pub use shader::{Shader, ShaderBound, SendUniforms};
pub use webgl_ctx::WebGl2Context;

pub use object::array_buffer::ArrayBuffer;
pub use object::array_buffer_instanced::ArrayBufferInstanced;
pub use object::buffer_data::{BufferDataStorage, SliceData, VecData};
pub use object::element_array_buffer::ElementArrayBuffer;
pub use object::vertex_array_object::{
    ShaderVertexArrayObjectBound, ShaderVertexArrayObjectBoundRef, VertexArrayObject,
    VertexArrayObjectBound,
};

use object::array_buffer::VertexAttribPointerType;
use object::array_buffer::VertexBufferObject;
