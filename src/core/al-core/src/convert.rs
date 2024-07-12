pub trait Cast<T>: Clone + Copy {
    fn cast(self) -> T;
}

impl Cast<f32> for u8 {
    fn cast(self) -> f32 {
        self as f32
    }
}
impl Cast<f32> for i16 {
    fn cast(self) -> f32 {
        self as f32
    }
}

impl Cast<f32> for i32 {
    fn cast(self) -> f32 {
        self as f32
    }
}

impl Cast<f32> for f32 {
    fn cast(self) -> f32 {
        self
    }
}

impl Cast<f32> for f64 {
    fn cast(self) -> f32 {
        self as f32
    }
}
