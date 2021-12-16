#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Time(pub f32);

use crate::utils;
impl Time {
    pub fn now() -> Self {
        Time(utils::get_current_time())
    }

    pub fn as_millis(&self) -> f32 {
        self.0
    }
}

use std::cmp::Ordering;
impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
impl Eq for Time {}

use core::ops::Sub;
impl Sub for Time {
    type Output = DeltaTime;
    fn sub(self, other: Self) -> Self::Output {
        DeltaTime(self.0 - other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct DeltaTime(pub f32);

impl DeltaTime {
    pub fn from_millis(millis: f32) -> Self {
        DeltaTime(millis)
    }

    pub fn zero() -> Self {
        DeltaTime(0.0)
    }
}

use std::ops::{Add, Mul};
impl Add<DeltaTime> for Time {
    type Output = Self;

    fn add(self, duration: DeltaTime) -> Self {
        Time(self.0 + duration.0)
    }
}

impl Mul<f32> for DeltaTime {
    type Output = Self;

    fn mul(self, factor: f32) -> Self {
        DeltaTime(self.0 * factor)
    }
}
use al_core::{shader::UniformType, WebGlContext};
use web_sys::WebGlUniformLocation;
impl UniformType for Time {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0);
    }
}
