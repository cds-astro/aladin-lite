use super::array_buffer::VertexAttribPointerType;

pub trait BufferDataStorage<'a, T: VertexAttribPointerType> {
    fn get_slice(&self) -> &[T];
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn len(&self) -> usize;
    fn ptr(&self) -> *const T;
}
pub struct VecData<'a, T: VertexAttribPointerType>(pub &'a Vec<T>);
impl<'a, T> BufferDataStorage<'a, T> for VecData<'a, T>
where
    T: VertexAttribPointerType,
{
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
where
    T: VertexAttribPointerType,
{
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

impl<'a, T> BufferDataStorage<'a, T> for &'a [T]
where
    T: VertexAttribPointerType,
{
    fn get_slice(&self) -> &[T] {
        self
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn ptr(&self) -> *const T {
        self.as_ptr()
    }
}
