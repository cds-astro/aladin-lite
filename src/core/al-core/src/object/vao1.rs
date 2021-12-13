use web_sys::WebGlVertexArrayObject;

use super::array_buffer::ArrayBuffer;
use super::array_buffer_instanced::ArrayBufferInstanced;
use super::buffer_data::BufferDataStorage;
use super::element_array_buffer::ElementArrayBuffer;

use crate::webgl_ctx::WebGl2Context;

pub struct VertexArrayObject1 {
    array_buffer: Vec<ArrayBuffer>,
    array_buffer_instanced: Vec<ArrayBufferInstanced>,
    element_array_buffer: Option<ElementArrayBuffer>,

    idx: u32, // Number of vertex attributes

    vao: WebGlVertexArrayObject,

    gl: WebGl2Context,
}

impl VertexArrayObject1 {
    pub fn new(gl: &WebGl2Context) -> VertexArrayObject1 {
        let vao = gl
            .create_vertex_array()
            .ok_or("failed to create the vertex array buffer")
            .unwrap();

        let array_buffer = vec![];
        let array_buffer_instanced = vec![];

        let element_array_buffer = None;

        let idx = 0;

        let gl = gl.clone();
        VertexArrayObject1 {
            array_buffer,
            array_buffer_instanced,
            element_array_buffer,

            idx,

            vao,
            gl,
        }
    }

    // Shader has to be already bound before calling this
    // This returns a ShaderVertexArrayObjectBound for which it is possible
    // to add some buffers and or draw the buffers
    pub fn bind<'a, 'b>(
        &'a mut self,
        _shader: &'b ShaderBound<'b>,
    ) -> ShaderVertexArrayObject1Bound<'a, 'b> {
        self.gl.bind_vertex_array(Some(self.vao.as_ref()));

        ShaderVertexArrayObject1Bound { vao: self, _shader }
    }
    // Shader has to be already bound before calling this
    // This returns a ShaderVertexArrayObjectBound for which it is possible
    // to add some buffers and or draw the buffers
    pub fn bind_ref<'a, 'b>(
        &'a self,
        _shader: &'b ShaderBound<'b>,
    ) -> ShaderVertexArrayObject1BoundRef<'a, 'b> {
        self.gl.bind_vertex_array(Some(self.vao.as_ref()));

        ShaderVertexArrayObject1BoundRef { vao: self, _shader }
    }

    // No need to bind a shader here
    // This returns a VertexArrayObjectBound for which it is only possible to
    // update the buffers
    pub fn bind_for_update<'a>(&'a mut self) -> VertexArrayObject1Bound<'a> {
        self.gl.bind_vertex_array(Some(self.vao.as_ref()));

        VertexArrayObject1Bound { vao: self }
    }

    /*pub fn bind_ref(&self) {
        self.gl.bind_vertex_array(Some(self.vao.as_ref()));
    }*/

    pub fn num_elements(&self) -> usize {
        self.element_array_buffer.as_ref().unwrap().num_elements()
    }

    pub fn num_instances(&self) -> i32 {
        self.array_buffer_instanced[0].num_instances()
    }
}

impl Drop for VertexArrayObject1 {
    fn drop(&mut self) {
        //self.unbind();
        self.gl.delete_vertex_array(Some(self.vao.as_ref()));
    }
}

use crate::shader::ShaderBound;
pub struct ShaderVertexArrayObject1Bound<'a, 'b> {
    vao: &'a mut VertexArrayObject1,
    _shader: &'b ShaderBound<'b>,
}

use crate::VertexAttribPointerType;
use web_sys::WebGl2RenderingContext;
impl<'a, 'b> ShaderVertexArrayObject1Bound<'a, 'b> {
    pub fn update_array<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        idx: usize,
        usage: u32,
        array_data: B,
    ) -> &mut Self {
        self.vao.array_buffer[idx].update(usage, array_data);
        self
    }

    pub fn update_element_array<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        usage: u32,
        element_data: B,
    ) -> &mut Self {
        if let Some(ref mut element_array_buffer) = self.vao.element_array_buffer {
            element_array_buffer.update(usage, element_data);
        }
        self
    }

    pub fn update_instanced_array<B: BufferDataStorage<'a, f32>>(
        &mut self,
        idx: usize,
        array_data: B,
    ) -> &mut Self {
        self.vao.array_buffer_instanced[idx].update(array_data);
        self
    }

    pub fn unbind(&self) {
        self.vao.gl.bind_vertex_array(None);
    }
}

impl<'a, 'b> Drop for ShaderVertexArrayObject1Bound<'a, 'b> {
    fn drop(&mut self) {
        self.unbind();
    }
}

pub struct ShaderVertexArrayObject1BoundRef<'a, 'b> {
    vao: &'a VertexArrayObject1,
    _shader: &'b ShaderBound<'b>,
}

impl<'a, 'b> ShaderVertexArrayObject1BoundRef<'a, 'b> {
    pub fn draw_arrays(&self, mode: u32, byte_offset: i32, size: i32) {
        self.vao
            .gl
            .draw_arrays(mode, byte_offset, size);
    }

    pub fn draw_elements_with_i32(&self, mode: u32, num_elements: Option<i32>, type_: u32, byte_offset: i32) {
        let num_elements = num_elements.unwrap_or(self.vao.num_elements() as i32);
        self.vao
            .gl
            .draw_elements_with_i32(mode, num_elements, type_, byte_offset);
    }

    pub fn draw_elements_instanced_with_i32(
        &self,
        mode: u32,
        offset_instance_idx: i32,
        num_instances: i32,
    ) {
        self.vao.gl.draw_elements_instanced_with_i32(
            mode,
            self.vao.num_elements() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            offset_instance_idx,
            num_instances,
        );
    }

    pub fn unbind(&self) {
        self.vao.gl.bind_vertex_array(None);
    }
}

impl<'a, 'b> Drop for ShaderVertexArrayObject1BoundRef<'a, 'b> {
    fn drop(&mut self) {
        self.unbind();
    }
}


// Struct defined when only the Vertex Array Object is
// defined
pub struct VertexArrayObject1Bound<'a> {
    vao: &'a mut VertexArrayObject1,
}

impl<'a> VertexArrayObject1Bound<'a> {
    /// Precondition: self must be bound
    pub fn add_array_buffer<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> &mut Self {
        let array_buffer = ArrayBuffer::new(
            &self.vao.gl,
            self.vao.idx,
            stride,
            sizes,
            offsets,
            usage,
            data,
        );

        // Update the number of vertex attrib
        self.vao.idx += sizes.len() as u32;

        self.vao.array_buffer.push(array_buffer);

        self
    }

    /// Precondition: self must be bound
    pub fn add_instanced_array_buffer<B: BufferDataStorage<'a, f32>>(
        &mut self,
        stride: usize,
        sizes: &[usize],
        offsets: &[usize],
        usage: u32,
        data: B,
    ) -> &mut Self {
        let array_buffer = ArrayBufferInstanced::new(
            &self.vao.gl,
            self.vao.idx,
            stride,
            sizes,
            offsets,
            usage,
            data,
        );

        // Update the number of vertex attrib
        self.vao.idx += sizes.len() as u32;

        self.vao.array_buffer_instanced.push(array_buffer);

        self
    }

    /// Precondition: self must be bound
    pub fn add_element_buffer<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        usage: u32,
        data: B,
    ) -> &mut Self {
        let element_buffer = ElementArrayBuffer::new(&self.vao.gl, usage, data);

        self.vao.element_array_buffer = Some(element_buffer);

        self
    }

    pub fn update_array<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        idx: usize,
        usage: u32,
        array_data: B,
    ) -> &mut Self {
        self.vao.array_buffer[idx].update(usage, array_data);
        self
    }

    pub fn update_element_array<T: VertexAttribPointerType, B: BufferDataStorage<'a, T>>(
        &mut self,
        usage: u32,
        element_data: B,
    ) -> &mut Self {
        if let Some(ref mut element_array_buffer) = self.vao.element_array_buffer {
            element_array_buffer.update(usage, element_data);
        }
        self
    }

    pub fn update_instanced_array<B: BufferDataStorage<'a, f32>>(
        &mut self,
        idx: usize,
        array_data: B,
    ) -> &mut Self {
        self.vao.array_buffer_instanced[idx].update(array_data);
        self
    }

    pub fn append_to_instanced_array<B: BufferDataStorage<'a, f32>>(
        &mut self,
        idx: usize,
        buffer: B,
    ) -> &mut Self {
        self.vao.array_buffer_instanced[idx].append(buffer);
        self
    }

    pub fn unbind(&self) {
        self.vao.gl.bind_vertex_array(None);
    }
}

impl<'a> Drop for VertexArrayObject1Bound<'a> {
    fn drop(&mut self) {
        self.unbind();
    }
}
