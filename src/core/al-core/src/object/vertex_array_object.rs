#[cfg(feature = "webgl2")]
pub mod vao {
    use web_sys::WebGlVertexArrayObject;

    use crate::object::array_buffer::ArrayBuffer;
    use crate::object::array_buffer_instanced::ArrayBufferInstanced;
    use crate::object::buffer_data::BufferDataStorage;
    use crate::object::element_array_buffer::ElementArrayBuffer;

    use crate::webgl_ctx::WebGlContext;
    pub struct VertexArrayObject {
        array_buffer: Vec<ArrayBuffer>,
        array_buffer_instanced: Vec<ArrayBufferInstanced>,
        element_array_buffer: Option<ElementArrayBuffer>,
    
        idx: u32, // Number of vertex attributes
    
        vao: WebGlVertexArrayObject,
    
        gl: WebGlContext,
    }
    
    impl VertexArrayObject {
        pub fn new(gl: &WebGlContext) -> VertexArrayObject {
            let vao = gl
                .create_vertex_array()
                .ok_or("failed to create the vertex array buffer")
                .unwrap();
    
            let array_buffer = vec![];
            let array_buffer_instanced = vec![];
    
            let element_array_buffer = None;
    
            let idx = 0;
    
            let gl = gl.clone();
            VertexArrayObject {
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
        ) -> ShaderVertexArrayObjectBound<'a, 'b> {
            self.gl.bind_vertex_array(Some(self.vao.as_ref()));
    
            ShaderVertexArrayObjectBound { vao: self, _shader }
        }
        // Shader has to be already bound before calling this
        // This returns a ShaderVertexArrayObjectBound for which it is possible
        // to add some buffers and or draw the buffers
        pub fn bind_ref<'a, 'b>(
            &'a self,
            _shader: &'b ShaderBound<'b>,
        ) -> ShaderVertexArrayObjectBoundRef<'a, 'b> {
            self.gl.bind_vertex_array(Some(self.vao.as_ref()));
    
            ShaderVertexArrayObjectBoundRef { vao: self, _shader }
        }
    
        // No need to bind a shader here
        // This returns a VertexArrayObjectBound for which it is only possible to
        // update the buffers
        pub fn bind_for_update<'a>(&'a mut self) -> VertexArrayObjectBound<'a> {
            self.gl.bind_vertex_array(Some(self.vao.as_ref()));
    
            VertexArrayObjectBound { vao: self }
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
    
    impl Drop for VertexArrayObject {
        fn drop(&mut self) {
            //self.unbind();
            self.gl.delete_vertex_array(Some(self.vao.as_ref()));
        }
    }
    
    use crate::shader::ShaderBound;
    pub struct ShaderVertexArrayObjectBound<'a, 'b> {
        vao: &'a mut VertexArrayObject,
        _shader: &'b ShaderBound<'b>,
    }
    
    use web_sys::WebGl2RenderingContext;
    impl<'a, 'b> ShaderVertexArrayObjectBound<'a, 'b> {
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
    
    impl<'a, 'b> Drop for ShaderVertexArrayObjectBound<'a, 'b> {
        fn drop(&mut self) {
            self.unbind();
        }
    }
    
    pub struct ShaderVertexArrayObjectBoundRef<'a, 'b> {
        vao: &'a VertexArrayObject,
        _shader: &'b ShaderBound<'b>,
    }
    
    impl<'a, 'b> ShaderVertexArrayObjectBoundRef<'a, 'b> {
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
    
    impl<'a, 'b> Drop for ShaderVertexArrayObjectBoundRef<'a, 'b> {
        fn drop(&mut self) {
            self.unbind();
        }
    }
    
    
    // Struct defined when only the Vertex Array Object is
    // defined
    pub struct VertexArrayObjectBound<'a> {
        vao: &'a mut VertexArrayObject,
    }
    
    impl<'a> VertexArrayObjectBound<'a> {
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
    
    impl<'a> Drop for VertexArrayObjectBound<'a> {
        fn drop(&mut self) {
            self.unbind();
        }
    }    
}

#[cfg(not(feature = "webgl2"))]
pub mod vao {
    use crate::object::array_buffer::ArrayBuffer;
    use crate::object::array_buffer_instanced::ArrayBufferInstanced;
    use crate::object::buffer_data::BufferDataStorage;
    use crate::object::element_array_buffer::ElementArrayBuffer;
    
    use crate::webgl_ctx::WebGlContext;    

    pub struct VertexArrayObject {
        array_buffer: Vec<ArrayBuffer>,
        array_buffer_instanced: Vec<ArrayBufferInstanced>,
        element_array_buffer: Option<ElementArrayBuffer>,

        idx: u32, // Number of vertex attributes

        gl: WebGlContext,
    }

    impl VertexArrayObject {
        pub fn new(gl: &WebGlContext) -> VertexArrayObject {
            let array_buffer = vec![];
            let array_buffer_instanced = vec![];

            let element_array_buffer = None;

            let idx = 0;

            let gl = gl.clone();
            VertexArrayObject {
                array_buffer,
                array_buffer_instanced,
                element_array_buffer,

                idx,

                gl,
            }
        }

        // Shader has to be already bound before calling this
        // This returns a ShaderVertexArrayObjectBound for which it is possible
        // to add some buffers and or draw the buffers
        pub fn bind<'a, 'b>(
            &'a mut self,
            _shader: &'b ShaderBound<'b>,
        ) -> ShaderVertexArrayObjectBound<'a, 'b> {
            //self.gl.bind_vertex_array(Some(self.vao.as_ref()));

            ShaderVertexArrayObjectBound { vao: self, _shader }
        }
        // Shader has to be already bound before calling this
        // This returns a ShaderVertexArrayObjectBound for which it is possible
        // to add some buffers and or draw the buffers
        pub fn bind_ref<'a, 'b>(
            &'a self,
            _shader: &'b ShaderBound<'b>,
        ) -> ShaderVertexArrayObjectBoundRef<'a, 'b> {
            //self.gl.bind_vertex_array(Some(self.vao.as_ref()));

            ShaderVertexArrayObjectBoundRef { vao: self, _shader }
        }

        // No need to bind a shader here
        // This returns a VertexArrayObjectBound for which it is only possible to
        // update the buffers
        pub fn bind_for_update<'a>(&'a mut self) -> VertexArrayObjectBound<'a> {
            //self.gl.bind_vertex_array(Some(self.vao.as_ref()));

            VertexArrayObjectBound { vao: self }
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

    impl Drop for VertexArrayObject {
        fn drop(&mut self) {
            //self.unbind();
            //self.gl.delete_vertex_array(Some(self.vao.as_ref()));
        }
    }

    use crate::shader::ShaderBound;
    pub struct ShaderVertexArrayObjectBound<'a, 'b> {
        vao: &'a mut VertexArrayObject,
        _shader: &'b ShaderBound<'b>,
    }

    use crate::VertexAttribPointerType;
    impl<'a, 'b> ShaderVertexArrayObjectBound<'a, 'b> {
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
            //self.vao.gl.bind_vertex_array(None);
        }
    }

    impl<'a, 'b> Drop for ShaderVertexArrayObjectBound<'a, 'b> {
        fn drop(&mut self) {
            self.unbind();
        }
    }

    use crate::webgl_ctx::WebGlRenderingCtx;
    pub struct ShaderVertexArrayObjectBoundRef<'a, 'b> {
        vao: &'a VertexArrayObject,
        _shader: &'b ShaderBound<'b>,
    }
    use crate::object::array_buffer::VertexBufferObject;
    impl<'a, 'b> ShaderVertexArrayObjectBoundRef<'a, 'b> {
        pub fn draw_arrays<T: VertexAttribPointerType>(&self, mode: u32, byte_offset: i32, size: i32) {
            for buf in &self.vao.array_buffer {
                buf.bind();
                buf.set_vertex_attrib_pointers::<T>();
            }

            self.vao
                .gl
                .draw_arrays(mode, byte_offset, size);
        }

        pub fn draw_elements_with_i32<T: VertexAttribPointerType>(&self, mode: u32, num_elements: Option<i32>, type_: u32, byte_offset: i32) {
            for buf in &self.vao.array_buffer {
                buf.bind();
                buf.set_vertex_attrib_pointers::<T>();
            }

            let num_elements = num_elements.unwrap_or(self.vao.num_elements() as i32);
            self.vao
                .gl
                .draw_elements_with_i32(mode, num_elements, type_, byte_offset);
        }

        pub fn draw_elements_instanced_with_i32<T: VertexAttribPointerType>(
            &self,
            mode: u32,
            offset_instance_idx: i32,
            num_instances: i32,
        ) {
            for buf in &self.vao.array_buffer {
                buf.bind();
                buf.set_vertex_attrib_pointers::<T>();
            }

            
            for inst_buf in &self.vao.array_buffer_instanced {
                inst_buf.bind();
                inst_buf.set_vertex_attrib_pointers::<T>();
            }

            #[cfg(feature = "webgl2")]
            self.vao.gl.draw_elements_instanced_with_i32(
                mode,
                self.vao.num_elements() as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                offset_instance_idx,
                num_instances,
            );
            #[cfg(not(feature = "webgl2"))]
            self.vao.gl.ext.angles.draw_elements_instanced_angle_with_i32(
                mode,
                self.vao.num_elements() as i32,
                WebGlRenderingCtx::UNSIGNED_SHORT,
                offset_instance_idx,
                num_instances,
            );
        }

        pub fn unbind(&self) {
            //self.vao.gl.bind_vertex_array(None);
        }
    }

    impl<'a, 'b> Drop for ShaderVertexArrayObjectBoundRef<'a, 'b> {
        fn drop(&mut self) {
            self.unbind();
        }
    }


    // Struct defined when only the Vertex Array Object is
    // defined
    pub struct VertexArrayObjectBound<'a> {
        vao: &'a mut VertexArrayObject,
    }

    impl<'a> VertexArrayObjectBound<'a> {
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
            //self.vao.gl.bind_vertex_array(None);
        }
    }

    impl<'a> Drop for VertexArrayObjectBound<'a> {
        fn drop(&mut self) {
            self.unbind();
        }
    }

}
