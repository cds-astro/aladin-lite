use crate::core::VertexAttribPointerType;

pub trait BufferDataStorage<'a, T: VertexAttribPointerType> {
    fn get_slice(&self) -> &[T];

    fn len(&self) -> usize;
    fn ptr(&self) -> *const T;
}
pub struct VecData<'a, T: VertexAttribPointerType>(pub &'a Vec<T>);
impl<'a, T> BufferDataStorage<'a, T> for VecData<'a, T>
where T: VertexAttribPointerType {
    fn get_slice(&self) -> &[T] {
        self.0
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}
pub struct SliceData<'a, T: VertexAttribPointerType>(pub &'a [T]);
impl<'a, T> BufferDataStorage<'a, T> for SliceData<'a, T>
where T: VertexAttribPointerType {
    fn get_slice(&self) -> &[T] {
        self.0
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}