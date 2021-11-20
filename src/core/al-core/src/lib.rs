extern crate fontdue;
extern crate lazy_static;
pub mod text;

mod object;
pub mod shader;
pub mod texture;
pub mod webgl_ctx;
#[macro_use]
pub mod log;

pub use texture::format;
pub use texture::image;
pub use texture::pixel;
pub use texture::texture::{IdxTextureUnit, Texture2D, Texture2DBound};
pub use texture::Texture2DArray;

pub use webgl_ctx::WebGl2Context;

pub use object::array_buffer::ArrayBuffer;
pub use object::array_buffer_instanced::ArrayBufferInstanced;
pub use object::buffer_data::{BufferDataStorage, SliceData, VecData};
pub use object::element_array_buffer::ElementArrayBuffer;
pub use object::framebuffer::FrameBufferObject;
pub use object::vertex_array_object::{
    ShaderVertexArrayObjectBound, ShaderVertexArrayObjectBoundRef, VertexArrayObject,
    VertexArrayObjectBound,
};

use object::array_buffer::VertexAttribPointerType;
