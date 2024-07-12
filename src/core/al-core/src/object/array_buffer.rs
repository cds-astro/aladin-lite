use crate::webgl_ctx::WebGlContext;
use web_sys::WebGlBuffer;

use super::buffer_data::BufferDataStorage;

pub trait VertexBufferObject {
    fn bind(&self);

    #[allow(dead_code)]
    fn unbind(&self);
}

pub trait VertexAttribPointerType: std::marker::Sized {
    type ArrayBufferView;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView;
    /// Link the vertex attrib to the shader
    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    );

    /// Pass the vertices data to the buffer
    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    );

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
    );

    // Initialize the VBO
    fn initialize_buffer<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        offset_idx: u32,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> WebGlBuffer {
        let buffer = gl
            .create_buffer()
            .ok_or("failed to create buffer")
            .unwrap_abort();
        // Bind the buffer
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(buffer.as_ref()));

        Self::buffer_data_with_array_buffer_view(gl, data, WebGlRenderingCtx::ARRAY_BUFFER, usage);
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

    fn set_vertex_attrib_pointers(
        gl: &WebGlContext,
        offset_idx: u32,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
    ) {
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
    }
}
use crate::webgl_ctx::WebGlRenderingCtx;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
impl VertexAttribPointerType for u8 {
    type ArrayBufferView = js_sys::Uint8Array;

    fn array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(data: B) -> Self::ArrayBufferView {
        let data = data.get_slice();
        unsafe { Self::ArrayBufferView::view(data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0_i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        #[cfg(feature = "webgl2")]
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            stride,
            offset,
        );
        #[cfg(feature = "webgl1")]
        gl.vertex_attrib_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_BYTE,
            false,
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
        unsafe { Self::ArrayBufferView::view(data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0_i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        #[cfg(feature = "webgl2")]
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_SHORT,
            stride,
            offset,
        );
        #[cfg(feature = "webgl1")]
        gl.vertex_attrib_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_SHORT,
            false,
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
        unsafe { Self::ArrayBufferView::view(data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0_i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        #[cfg(feature = "webgl2")]
        gl.vertex_attrib_i_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_INT,
            stride,
            offset,
        );
        #[cfg(feature = "webgl1")]
        gl.vertex_attrib_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::UNSIGNED_INT,
            false,
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
        unsafe { Self::ArrayBufferView::view(data) }
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0_i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        #[cfg(feature = "webgl2")]
        gl.vertex_attrib_i_pointer_with_i32(idx, size, WebGlRenderingCtx::INT, stride, offset);
        #[cfg(feature = "webgl1")]
        gl.vertex_attrib_pointer_with_i32(idx, size, WebGlRenderingCtx::INT, false, stride, offset);
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
        Float32Array::new(&memory_buffer).subarray(ptr, ptr + len as u32)
    }

    fn buffer_sub_data_with_i32_and_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
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
        gl.buffer_sub_data_with_i32_and_array_buffer_view(target, 0_i32, &data);
    }

    fn buffer_data_with_array_buffer_view<'a, B: BufferDataStorage<'a, Self>>(
        gl: &WebGlContext,
        data: B,
        target: u32,
        usage: u32,
    ) {
        let data = Self::array_buffer_view(data);
        gl.buffer_data_with_array_buffer_view(target, &data, usage);
    }

    fn vertex_attrib_pointer_with_i32(
        gl: &WebGlContext,
        idx: u32,
        size: i32,
        stride: i32,
        offset: i32,
    ) {
        gl.vertex_attrib_pointer_with_i32(
            idx,
            size,
            WebGlRenderingCtx::FLOAT,
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
    sizes: Box<[usize]>,

    gl: WebGlContext,
}
use crate::shader::ShaderBound;

use crate::Abort;

impl ArrayBuffer {
    pub fn new<'a, T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        gl: &WebGlContext,
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
            sizes: sizes.into(),

            gl,
        }
    }

    pub fn set_vertex_attrib_pointer_by_name<'a, T: VertexAttribPointerType>(
        &self,
        shader: &ShaderBound<'a>,
        location: &str,
    ) {
        let loc = shader.get_attrib_location(&self.gl, location);

        assert_eq!(self.sizes.len(), 1);
        T::vertex_attrib_pointer_with_i32(
            &self.gl,
            loc as u32,
            *self.sizes.first().unwrap_abort() as i32,
            0,
            0,
        );

        #[cfg(feature = "webgl2")]
        self.gl.vertex_attrib_divisor(loc as u32, 0);
        #[cfg(feature = "webgl1")]
        self.gl
            .ext
            .angles
            .vertex_attrib_divisor_angle(loc as u32, 0);
    }

    pub fn disable_vertex_attrib_pointer_by_name<'a>(
        &self,
        shader: &ShaderBound<'a>,
        location: &str,
    ) {
        let loc = shader.get_attrib_location(&self.gl, location);
        self.gl.disable_vertex_attrib_array(loc as u32);
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
                WebGlRenderingCtx::ARRAY_BUFFER,
            );
        } else {
            self.len = data.len();

            T::buffer_data_with_array_buffer_view(
                &self.gl,
                data,
                WebGlRenderingCtx::ARRAY_BUFFER,
                usage,
            );
        }
    }
}

impl VertexBufferObject for ArrayBuffer {
    fn bind(&self) {
        self.gl
            .bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(self.buffer.as_ref()));
    }
    fn unbind(&self) {
        self.gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, None);
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
