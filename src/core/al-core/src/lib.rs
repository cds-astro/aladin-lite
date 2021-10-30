mod object;
pub mod texture;
pub mod shader;
pub mod webgl_ctx;

pub use texture::texture::{Texture2D, Texture2DBound, IdxTextureUnit};
pub use texture::Texture2DArray;
pub use texture::image;
pub use texture::format;
pub use texture::pixel;

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