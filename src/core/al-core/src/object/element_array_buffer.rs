use crate::webgl_ctx::WebGlRenderingCtx;
use web_sys::WebGlBuffer;

use super::array_buffer::VertexBufferObject;

use crate::webgl_ctx::WebGlContext;

#[derive(Clone)]
pub struct ElementArrayBuffer {
    buffer: WebGlBuffer,
    // Size of the buffer in number of elements
    len: usize,

    gl: WebGlContext,
}

impl VertexBufferObject for ElementArrayBuffer {
    fn bind(&self) {
        self.gl.bind_buffer(
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            Some(self.buffer.as_ref()),
        );
    }
    fn unbind(&self) {
        self.gl
            .bind_buffer(WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER, None);
    }
}

use super::array_buffer::VertexAttribPointerType;
use super::buffer_data::BufferDataStorage;
use crate::Abort;

impl ElementArrayBuffer {
    pub fn new<'a, T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        gl: &WebGlContext,
        usage: u32,
        data: B,
    ) -> ElementArrayBuffer {
        let buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap_abort();
        // Bind the buffer
        gl.bind_buffer(
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            Some(buffer.as_ref()),
        );
        // Total length
        let len = data.len();
        // Pass the vertices data to the buffer
        T::buffer_data_with_array_buffer_view(
            gl,
            data,
            WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            usage,
        );
        // Returns an instance that keeps only the buffer
        let gl = gl.clone();
        ElementArrayBuffer { buffer, len, gl }
    }

    // Returns the number of vertices stored in the array buffer
    pub fn num_elements(&self) -> usize {
        self.len
    }

    pub fn update<'a, T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        usage: u32,
        data: B,
    ) {
        self.bind();
        if self.len >= data.len() {
            T::buffer_sub_data_with_i32_and_array_buffer_view(
                &self.gl,
                data,
                WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
            );
        } else {
            // Reallocation if the new data size exceeds the size of the buffer
            self.len = data.len();
            T::buffer_data_with_array_buffer_view(
                &self.gl,
                data,
                WebGlRenderingCtx::ELEMENT_ARRAY_BUFFER,
                usage,
            );
        }
    }
}

impl Drop for ElementArrayBuffer {
    fn drop(&mut self) {
        self.gl.delete_buffer(Some(self.buffer.as_ref()));
    }
}
