#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Time(pub f32);

use crate::utils;
use wasm_bindgen::JsValue;
impl Time {
    pub fn measure_perf<T>(
        label: &str,
        f: impl FnOnce() -> Result<T, JsValue>,
    ) -> Result<T, JsValue> {
        let start_time = Time::now();
        let r = f()?;
        let duration = Time::now() - start_time;
        // print the duration in the console
        al_core::log(&format!("{:?} time: {:?}", label, duration));

        Ok(r)
    }

    pub fn now() -> Self {
        Time(utils::get_current_time())
    }

    pub fn as_millis(&self) -> f32 {
        self.0
    }

    pub fn as_secs(&self) -> f32 {
        self.as_millis() / 1000.0
    }
}

impl From<f32> for DeltaTime {
    fn from(x: f32) -> Self {
        DeltaTime(x)
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
    pub const fn from_millis(millis: f32) -> Self {
        DeltaTime(millis)
    }

    pub fn zero() -> Self {
        DeltaTime(0.0)
    }

    pub fn as_millis(&self) -> f32 {
        self.0
    }

    pub fn as_secs(&self) -> f32 {
        self.as_millis() / 1000.0
    }
}

use std::ops::{Add, Mul};
impl Add<DeltaTime> for Time {
    type Output = Self;

    fn add(self, duration: DeltaTime) -> Self {
        Time(self.0 + duration.0)
    }
}

impl Sub<DeltaTime> for Time {
    type Output = Self;

    fn sub(self, duration: DeltaTime) -> Self {
        Time(self.0 - duration.0)
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
