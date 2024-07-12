extern crate futures;
extern crate jpeg_decoder as jpeg;
extern crate png;
extern crate serde_json;
extern crate wasm_streams;

pub mod convert;
pub mod image;
mod object;
pub mod shader;
pub mod texture;
pub mod webgl_ctx;
#[macro_use]
pub mod log;
pub use log::log;

pub mod colormap;
pub use colormap::{Colormap, Colormaps};

pub use texture::pixel;
pub use texture::Texture2DArray;
pub use texture::{Texture2D, Texture2DBound};

pub use webgl_ctx::WebGlContext;

pub use object::array_buffer::ArrayBuffer;
pub use object::array_buffer_instanced::ArrayBufferInstanced;
pub use object::buffer_data::{BufferDataStorage, SliceData, VecData};
pub use object::element_array_buffer::ElementArrayBuffer;
pub use object::framebuffer::FrameBufferObject;

use object::array_buffer::VertexAttribPointerType;
pub use object::vertex_array_object::vao::{
    ShaderVertexArrayObjectBound, ShaderVertexArrayObjectBoundRef, VertexArrayObject,
    VertexArrayObjectBound,
};

pub trait Abort {
    type Item;
    fn unwrap_abort(self) -> Self::Item
    where
        Self: Sized;
}

impl<T> Abort for Option<T> {
    type Item = T;

    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Some(t) => t,
            None => process::abort(),
        }
    }
}
impl<T, E> Abort for Result<T, E> {
    type Item = T;

    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Ok(t) => t,
            Err(_) => process::abort(),
        }
    }
}
