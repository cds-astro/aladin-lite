use web_sys::WebGl2RenderingContext;
use web_sys::WebGlBuffer;

use crate::WebGl2Context;

pub struct ArrayBufferInstanced {
    buffer: WebGlBuffer,

    num_packed_data: usize,
    offset_idx: u32,

    num_instances: i32,
    num_bytes_per_instance: i32,
    num_bytes_in_buf: i32,
    usage: u32,
    sizes: Vec<usize>,
    stride: usize,

    gl: WebGl2Context,
}

use crate::core::VertexBufferObject;

impl VertexBufferObject for ArrayBufferInstanced {
    fn bind(&self) {
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(self.buffer.as_ref()));
    }
    fn unbind(&self) {
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
    }
}

use crate::core::{BufferDataStorage, VertexAttribPointerType};

impl ArrayBufferInstanced {
    pub fn new<'a, B: BufferDataStorage<'a, f32>>(gl: &WebGl2Context, offset_idx: u32, stride: usize, sizes: &[usize], _offsets: &[usize], usage: u32, data: B) -> ArrayBufferInstanced {
        // Instance length
        let num_f32_per_instance = sizes.iter().sum::<usize>() as i32; 
        // Total length
        let num_f32_in_buf = data.len() as i32;
        let num_bytes_in_buf = num_f32_in_buf * (std::mem::size_of::<f32>() as i32);

        let num_instances = num_f32_in_buf / (num_f32_per_instance as i32);
        let num_bytes_per_instance = num_f32_per_instance * (std::mem::size_of::<f32>() as i32);

        let buffer = gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();

        // Bind the buffer
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer.as_ref()));
        // Pass the vertices data to the buffer
        f32::buffer_data_with_array_buffer_view(gl, data, WebGl2RenderingContext::ARRAY_BUFFER, usage);
        // Link to the shader
        for (idx, size) in sizes.iter().enumerate() {
            let idx = (idx as u32) + offset_idx;
            gl.vertex_attrib_pointer_with_i32(idx, *size as i32, WebGl2RenderingContext::FLOAT, false, stride as i32, 0);
            gl.enable_vertex_attrib_array(idx);
            gl.vertex_attrib_divisor(idx, 1);
        }

        let num_packed_data = sizes.len();

        let gl = gl.clone();
        // Returns an instance that keeps only the buffer
        ArrayBufferInstanced {
            buffer,
            num_packed_data,
            offset_idx,

            usage,
            sizes: sizes.to_vec(),
            stride,

            num_instances,
            num_bytes_per_instance,
            num_bytes_in_buf,
            gl,
        }
    }

    pub fn update<'a, B: BufferDataStorage<'a, f32>>(&self, buffer: B) {
        self.bind();
        f32::buffer_sub_data_with_i32_and_array_buffer_view(&self.gl, buffer, WebGl2RenderingContext::ARRAY_BUFFER);
        /*self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            &data,
        );*/
    }

    // Add some data at the end of the buffer
    pub fn append<'a, B: BufferDataStorage<'a, f32>>(&mut self, buffer: B) {
        // Bind the current buffer to another target.
        self.gl.bind_buffer(WebGl2RenderingContext::COPY_READ_BUFFER, Some(self.buffer.as_ref()));

        // Create the bigger buffer that will contain the new data appended to the old
        // Get the size of the new data to add
        let num_f32_in_appended_buf = buffer.len() as i32;
        let num_bytes_in_appended_buf = num_f32_in_appended_buf * (std::mem::size_of::<f32>() as i32);
        let dest_buf = self.gl.create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        // Set its size
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(dest_buf.as_ref()));
        let num_bytes_in_dest_buf = num_bytes_in_appended_buf + self.num_bytes_in_buf;
        self.gl.buffer_data_with_i32(WebGl2RenderingContext::ARRAY_BUFFER, num_bytes_in_dest_buf, self.usage);

        // Link to the shader
        for (idx, size) in self.sizes.iter().enumerate() {
            let idx = (idx as u32) + self.offset_idx;
            self.gl.vertex_attrib_pointer_with_i32(idx, *size as i32, WebGl2RenderingContext::FLOAT, false, self.stride as i32, 0);
            self.gl.enable_vertex_attrib_array(idx);
            self.gl.vertex_attrib_divisor(idx, 1);
        }

        // Copy the current buffer to the new one
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            WebGl2RenderingContext::COPY_READ_BUFFER, // src target
            WebGl2RenderingContext::ARRAY_BUFFER, // dest target
            0, // read offset
            0, // write offset
            self.num_bytes_in_buf // number of bytes to copy
        );

        // Copy the new data at the end of the buffer
        let buffer = f32::array_buffer_view(buffer);
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            self.num_bytes_in_buf, // offset in bytes
            &buffer
        );
        // unbind the buffer of origin
        self.gl.bind_buffer(WebGl2RenderingContext::COPY_READ_BUFFER, None);

        self.buffer = dest_buf;
        self.num_bytes_in_buf = num_bytes_in_dest_buf;
        self.num_instances = num_bytes_in_dest_buf / self.num_bytes_per_instance;
    }

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