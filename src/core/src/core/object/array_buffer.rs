use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;

use crate::core::WebGl2Context;

use crate::core::BufferDataStorage;

pub trait VertexBufferObject {
    fn bind(&self);
    fn unbind(&self);
}

pub trait VertexAttribPointerType: std::marker::Sized {
    type ArrayBufferView;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView;
    /// Link the vertex attrib to the shader
    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    );

    /// Pass the vertices data to the buffer
    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    );

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    );

    // Initialize the VBO
    fn initialize_buffer<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        offset_idx: u32,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> WebGlBuffer {
        let buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer.as_ref()));

        Self::buffer_data_with_array_buffer_view(
            gl,
            data,
            WebGl2RenderingContext::ARRAY_BUFFER,
            usage,
        );
        // Attrib pointer to the shader
        for (idx, (size, offset)) in sizes.iter().zip(offsets.iter()).enumerate() {
            let idx = (idx as u32) + offset_idx;

            Self::vertex_attrib_pointer_with_i32(
                gl,
                idx,
                *size as i32,
                stride as i32,
                *offset as i32,
            );
        }

        buffer
    }
}

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
impl VertexAttribPointerType for u8 {
    type ArrayBufferView = js_sys::Uint8Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        unsafe { Self::ArrayBufferView::view(&data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    ) {
        /*let mem = wasm_bindgen::memory()
            .unchecked_ref::<WebAssembly::Memory>()
            .buffer();
        let mem = Self::ArrayBufferView::new(&mem);
        let ptr = (data.ptr() as u32) / (std::mem::size_of::<Self>() as u32);
        gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
            target,
            0 as i32,
            &mem,
            ptr,
            data.len() as u32
        );*/
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0 as i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(idx);
    }
}

impl VertexAttribPointerType for u16 {
    type ArrayBufferView = js_sys::Uint16Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        unsafe { Self::ArrayBufferView::view(&data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0 as i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(idx);
    }
}

impl VertexAttribPointerType for u32 {
    type ArrayBufferView = js_sys::Uint32Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        unsafe { Self::ArrayBufferView::view(&data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0 as i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGl2RenderingContext::UNSIGNED_INT,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(idx);
    }
}

impl VertexAttribPointerType for i32 {
    type ArrayBufferView = js_sys::Int32Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        unsafe { Self::ArrayBufferView::view(&data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0 as i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_i_pointer_with_i32(idx, size, WebGl2RenderingContext::INT, stride, offset);
        gl.enable_vertex_attrib_array(idx);
    }
}

use js_sys::Float32Array;

impl VertexAttribPointerType for f32 {
    type ArrayBufferView = Float32Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        //unsafe { Self::ArrayBufferView::view(&data) }
        let memory_buffer = wasm_bindgen::memory()
            .unchecked_ref::<WebAssembly::Memory>()
            .buffer();

        let len = data.len();
        let ptr = data.as_ptr() as u32 / 4;
        let data = Float32Array::new(&memory_buffer).subarray(ptr, ptr + len as u32);
        data
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
    ) {
        /*let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(
            target,
            0,
            &data,
        );*/

        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0 as i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGl2Context,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGl2Context,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_pointer_with_i32(
            idx,
            size,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset,
        );
        gl.enable_vertex_attrib_array(idx);
    }
}

pub struct ArrayBuffer {
    buffer: WebGlBuffer,
    // The size of the buffer in number of elements
    len: usize,
    num_packed_data: usize,

    offset_idx: u32,

    gl: WebGl2Context,
}

use web_sys::console;
impl ArrayBuffer {
    pub fn new<'a, T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        gl: &WebGl2Context,
        offset_idx: u32,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> ArrayBuffer {
        let len = data.len();
        let buffer = T::initialize_buffer(gl, offset_idx, stride, sizes, offsets, usage, data);

        let num_packed_data = sizes.len();

        let gl = gl.clone();
        // Returns an instance that keeps only the buffer
        ArrayBuffer {
            buffer,
            len,

            num_packed_data,
            offset_idx,

            gl,
        }
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
                WebGl2RenderingContext::ARRAY_BUFFER,
            );
        } else {
            console::log_1(
                &format!(
                    "array buffer reallocation! old/new size: {:?}/{:?}",
                    self.len,
                    data.len()
                )
                .into(),
            );
            self.len = data.len();

            T::buffer_data_with_array_buffer_view(
                &self.gl,
                data,
                WebGl2RenderingContext::ARRAY_BUFFER,
                usage,
            );
        }
    }
}

impl VertexBufferObject for ArrayBuffer {
    fn bind(&self) {
        self.gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(self.buffer.as_ref()),
        );
    }
    fn unbind(&self) {
        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    }
}

impl Drop for ArrayBuffer {
    fn drop(&mut self) {
        for idx in 0..self.num_packed_data {
            let idx = (idx as u32) + self.offset_idx;
            self.gl.disable_vertex_attrib_array(idx);
        }

        self.gl.delete_buffer(Some(self.buffer.as_ref()));
    }
}
