use crate::webgl_ctx::WebGlRenderingCtx;
use web_sys::WebGlBuffer;

use crate::webgl_ctx::WebGlContext;

pub struct ArrayBufferInstanced {
    buffer: WebGlBuffer,

    len: usize,
    num_packed_data: usize,
    offset_idx: u32,

    num_instances: i32,
    sizes: Vec<usize>,
    stride: usize,

    gl: WebGlContext,
}

use super::array_buffer::VertexBufferObject;

impl VertexBufferObject for ArrayBufferInstanced {
    fn bind(&self) {
        self.gl
            .bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(self.buffer.as_ref()));
    }
    fn unbind(&self) {
        self.gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, None);
    }
}

use super::array_buffer::VertexAttribPointerType;
use super::buffer_data::BufferDataStorage;
use crate::shader::ShaderBound;
use crate::Abort;

impl ArrayBufferInstanced {
    pub fn new<'a, B: BufferDataStorage<'a, f32>>(
        gl: &WebGlContext,
        offset_idx: u32,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> ArrayBufferInstanced {
        // Instance length
        let num_f32_per_instance = sizes.iter().sum::<usize>() as i32;
        // Total length
        let num_f32_in_buf = data.len() as i32;

        let num_instances = num_f32_in_buf / (num_f32_per_instance as i32);
        let len = data.len();

        let buffer = gl
            .create_buffer()
            .ok_or("failed to create buffer")
            .unwrap_abort();

        // Bind the buffer
        gl.bind_buffer(WebGlRenderingCtx::ARRAY_BUFFER, Some(buffer.as_ref()));
        // Pass the vertices data to the buffer
        f32::buffer_data_with_array_buffer_view(gl, data, WebGlRenderingCtx::ARRAY_BUFFER, usage);
        // Link to the shader
        for (idx, (size, offset)) in sizes.iter().zip(offsets.iter()).enumerate() {
            let idx = (idx as u32) + offset_idx;

            f32::vertex_attrib_pointer_with_i32(
                gl,
                idx,
                *size as i32,
                stride as i32,
                *offset as i32,
            );

            gl.enable_vertex_attrib_array(idx);

            #[cfg(feature = "webgl2")]
            gl.vertex_attrib_divisor(idx, 1);
            #[cfg(feature = "webgl1")]
            gl.ext.angles.vertex_attrib_divisor_angle(idx, 1);
        }

        let num_packed_data = sizes.len();
        let gl = gl.clone();
        // Returns an instance that keeps only the buffer
        ArrayBufferInstanced {
            buffer,
            len,
            num_packed_data,
            offset_idx,

            sizes: sizes.to_vec(),
            stride,

            num_instances,
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
        self.gl.vertex_attrib_pointer_with_i32(
            loc as u32,
            *self.sizes.first().unwrap_abort() as i32,
            WebGlRenderingCtx::FLOAT,
            false,
            self.stride as i32,
            0,
        );
        self.gl.enable_vertex_attrib_array(loc as u32);

        #[cfg(feature = "webgl2")]
        self.gl.vertex_attrib_divisor(loc as u32, 1);
        #[cfg(feature = "webgl1")]
        self.gl
            .ext
            .angles
            .vertex_attrib_divisor_angle(loc as u32, 1);
    }

    pub fn disable_vertex_attrib_pointer_by_name<'a>(
        &self,
        shader: &ShaderBound<'a>,
        location: &str,
    ) {
        let loc = shader.get_attrib_location(&self.gl, location);

        self.gl.disable_vertex_attrib_array(loc as u32);
    }

    pub fn update<'a, B: BufferDataStorage<'a, f32>>(&mut self, usage: u32, data: B) {
        self.bind();
        if self.len >= data.len() {
            f32::buffer_sub_data_with_i32_and_array_buffer_view(
                &self.gl,
                data,
                WebGlRenderingCtx::ARRAY_BUFFER,
            );
        } else {
            self.len = data.len();

            f32::buffer_data_with_array_buffer_view(
                &self.gl,
                data,
                WebGlRenderingCtx::ARRAY_BUFFER,
                usage,
            );
        }

        /*f32::buffer_sub_data_with_i32_and_array_buffer_view(
            &self.gl,
            buffer,
            WebGlRenderingCtx::ARRAY_BUFFER,
        );*/
        /*self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGlRenderingCtx::ARRAY_BUFFER,
            0,
            &data,
        );*/
    }

    // Add some data at the end of the buffer
    /*pub fn append<'a, B: BufferDataStorage<'a, f32>>(&mut self, buffer: B) {
        // Create the bigger buffer that will contain the new data appended to the old
        // Get the size of the new data to add
        let num_f32_in_appended_buf = buffer.len() as i32;
        let num_bytes_in_appended_buf =
            num_f32_in_appended_buf * (std::mem::size_of::<f32>() as i32);
        let dest_buf = self
            .gl
            .create_buffer()
            .ok_or("failed to create buffer")
            .unwrap_abort();
        // Set its size
        self.gl.bind_buffer(
            WebGlRenderingCtx::ARRAY_BUFFER,
            Some(dest_buf.as_ref()),
        );
        let num_bytes_in_dest_buf = num_bytes_in_appended_buf + self.num_bytes_in_buf;
        self.gl.buffer_data_with_i32(
            WebGlRenderingCtx::ARRAY_BUFFER,
            num_bytes_in_dest_buf,
            self.usage,
        );

        #[cfg(feature = "webgl2")]
        {
            // Bind the current buffer to another target.
            self.gl.bind_buffer(
                WebGlRenderingCtx::COPY_READ_BUFFER,
                Some(self.buffer.as_ref()),
            );

            // Link to the shader
            self.set_vertex_attrib_pointers();

            // Copy the current buffer to the new one
            self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                WebGlRenderingCtx::COPY_READ_BUFFER, // src target
                WebGlRenderingCtx::ARRAY_BUFFER,     // dest target
                0,                                        // read offset
                0,                                        // write offset
                self.num_bytes_in_buf,                    // number of bytes to copy
            );

            // Copy the new data at the end of the buffer
            let buffer = f32::array_buffer_view(buffer);
            self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGlRenderingCtx::ARRAY_BUFFER,
                self.num_bytes_in_buf, // offset in bytes
                &buffer,
            );
            // unbind the buffer of origin
            self.gl
                .bind_buffer(WebGlRenderingCtx::COPY_READ_BUFFER, None);
        }

        #[cfg(feature = "webgl1")]


        self.buffer = dest_buf;
        self.num_bytes_in_buf = num_bytes_in_dest_buf;
        self.num_instances = num_bytes_in_dest_buf / self.num_bytes_per_instance;
    }*/

    // Returns the number of vertices stored in the array buffer
    pub fn num_instances(&self) -> i32 {
        self.num_instances
    }
}

impl Drop for ArrayBufferInstanced {
    fn drop(&mut self) {
        for idx in 0..self.num_packed_data {
            let idx = (idx as u32) + self.offset_idx;
            self.gl.disable_vertex_attrib_array(idx);
        }

        self.gl.delete_buffer(Some(self.buffer.as_ref()));
    }
}
