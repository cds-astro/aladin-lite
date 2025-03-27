use cgmath::BaseFloat;
use crate::Abort;
// ArcDeg wrapper structure
#[derive(Clone, Copy)]
pub struct ArcDeg<T: BaseFloat>(pub T);

//pub const TWICE_PI: f64 = 6.28318530718;
pub const PI: f64 = std::f64::consts::PI;

use cgmath::{Deg, Rad};
use serde::Deserialize;
// Convert a Rad<T> to an ArcDeg<T>
impl<T> From<Rad<T>> for ArcDeg<T>
where
    T: BaseFloat,
{
    fn from(angle: Rad<T>) -> Self {
        let deg: Deg<T> = angle.into();
        ArcDeg(deg.0)
    }
}
// Convert an ArcMin<T> to a Rad<T>
impl<T> From<ArcDeg<T>> for Rad<T>
where
    T: BaseFloat,
{
    fn from(degrees: ArcDeg<T>) -> Self {
        let deg = Deg(*degrees);
        deg.into()
    }
}

use core::ops::Deref;
impl<T> Deref for ArcDeg<T>
where
    T: BaseFloat,
{
    type Target = T;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T> ToString for ArcDeg<T>
where
    T: BaseFloat + ToString,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// ArcHour wrapper structure
#[derive(Clone, Copy)]
pub struct ArcHour<T: BaseFloat>(pub T);


impl<T> From<Rad<T>> for ArcHour<T>
where
    T: BaseFloat,
{
    fn from(angle: Rad<T>) -> Self {
        let deg: Deg<T> = angle.into();

        let degrees_per_hour = T::from(15.0).unwrap_abort();
        let hours = deg.0 / degrees_per_hour;

        ArcHour(hours)
    }
}

impl<T> Deref for ArcHour<T>
where
    T: BaseFloat,
{
    type Target = T;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T> ToString for ArcHour<T>
where
    T: BaseFloat + ToString,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// ArcMin wrapper structure
#[derive(Clone, Copy)]
pub struct ArcMin<T: BaseFloat>(pub T);

// Convert a Rad<T> to an ArcMin<T>
impl<T> From<Rad<T>> for ArcMin<T>
where
    T: BaseFloat,
{
    fn from(angle: Rad<T>) -> Self {
        let deg: Deg<T> = angle.into();

        // There is 60 minutes in one degree
        let minutes_per_degree = T::from(60_f64).unwrap_abort();
        let minutes = deg.0 * minutes_per_degree;
        ArcMin(minutes)
    }
}
// Convert an ArcMin<T> to a Rad<T>
impl<T> From<ArcMin<T>> for Rad<T>
where
    T: BaseFloat,
{
    fn from(minutes: ArcMin<T>) -> Self {
        let minutes_per_degree = T::from(60_f64).unwrap_abort();
        let deg: Deg<T> = Deg(*minutes / minutes_per_degree);

        deg.into()
    }
}

impl<T> Deref for ArcMin<T>
where
    T: BaseFloat,
{
    type Target = T;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T> ToString for ArcMin<T>
where
    T: BaseFloat + ToString,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// ArcSec wrapper structure
#[derive(Clone, Copy)]
pub struct ArcSec<T: BaseFloat>(pub T);

impl<T> From<Rad<T>> for ArcSec<T>
where
    T: BaseFloat,
{
    fn from(angle: Rad<T>) -> Self {
        let deg: Deg<T> = angle.into();

        // There is 3600 seconds in one degree
        let seconds_per_degree = T::from(3600_f32).unwrap_abort();
        let seconds = deg.0 * seconds_per_degree;
        ArcSec(seconds)
    }
}
// Convert an ArcMin<T> to a Rad<T>
impl<T> From<ArcSec<T>> for Rad<T>
where
    T: BaseFloat,
{
    fn from(seconds: ArcSec<T>) -> Self {
        let seconds_per_degree = T::from(3600_f32).unwrap_abort();
        let deg: Deg<T> = Deg(seconds.0 / seconds_per_degree);

        deg.into()
    }
}

impl<T> ToString for ArcSec<T>
where
    T: BaseFloat + ToString,
{
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl<T> Deref for ArcSec<T>
where
    T: BaseFloat,
{
    type Target = T;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

use al_api::angle::Format;
/*
pub enum SerializeFmt {
    DMS,
    HMS,
    DMM,
    DD,
}

use al_api::angle_fmt::AngleSerializeFmt;
impl From<AngleSerializeFmt> for SerializeFmt {
    fn from(value: AngleSerializeFmt) -> Self {
        match value {
            AngleSerializeFmt::DMS => SerializeFmt::DMS,
            AngleSerializeFmt::HMS => SerializeFmt::HMS,
            AngleSerializeFmt::DMM => SerializeFmt::DMM,
            AngleSerializeFmt::DD => SerializeFmt::DD,
        }
    }
}

impl SerializeFmt {
    pub fn to_string<S: BaseFloat + ToString>(&self, angle: Angle<S>) -> String {
        match &self {
            Self::DMS => DMS::to_string(angle),
            Self::HMS => HMS::to_string(angle),
            Self::DMM => DMM::to_string(angle),
            Self::DD => DD::to_string(angle),
        }
    }
}*/

/*pub trait SerializeToString {
    fn to_string(&self) -> String;
}

impl<S> SerializeToString for Angle<S>
where
    S: BaseFloat + ToString,
{
    fn to_string<F: FormatType>(&self) -> String {
        F::to_string(*self)
    }
}*/

/*
pub struct DMS;
pub struct HMS;
pub struct DMM;
pub struct DD;
pub trait FormatType {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String;
}

impl FormatType for DD {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String {
        let angle = Rad(angle.0);
        let degrees: ArcDeg<S> = angle.into();

        degrees.to_string()
    }
}
impl FormatType for DMM {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String {
        let angle = Rad(angle.0);

        let mut degrees: ArcDeg<S> = angle.into();
        let minutes = degrees.get_frac_minutes();

        degrees.truncate();

        let mut result = degrees.to_string() + " ";
        result += &minutes.to_string();

        result
    }
}

impl FormatType for DMS {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String {
        let angle = Rad(angle.0);
        let degrees: ArcDeg<S> = angle.into();
        let minutes = degrees.get_frac_minutes();
        let seconds = minutes.get_frac_seconds();

        let num_sec_per_minutes = S::from(60).unwrap_abort();

        let degrees = degrees.trunc();
        let minutes = minutes.trunc() % num_sec_per_minutes;
        let seconds = seconds.trunc() % num_sec_per_minutes;

        let mut result = degrees.to_string() + "°";
        result += &minutes.to_string();
        result += "\'";
        result += &seconds.to_string();
        result += "\'\'";

        result
    }
}

impl FormatType for HMS {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String {
        let angle = Rad(angle.0);

        let hours: ArcHour<S> = angle.into();
        let minutes = hours.get_frac_minutes();
        let seconds = minutes.get_frac_seconds();

        let num_sec_per_minutes = S::from(60).unwrap_abort();

        let hours = hours.trunc();
        let minutes = minutes.trunc() % num_sec_per_minutes;
        let seconds = seconds.trunc() % num_sec_per_minutes;

        let mut result = hours.to_string() + "h";
        result += &minutes.to_string();
        result += "\'";
        result += &seconds.to_string();
        result += "\'\'";

        result
    }
}*/


#[derive(Clone, Copy, Debug, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct Angle<S: BaseFloat> {
    pub rad: S,
    fmt: AngleFormatter,
}
impl<S> Angle<S>
where
    S: BaseFloat,
{
    pub fn new<T: Into<Rad<S>>>(angle: T) -> Angle<S> {
        let radians: Rad<S> = angle.into();
        Angle { rad: radians.0, fmt: AngleFormatter::default() }
    }

    pub fn cos(&self) -> S {
        self.rad.cos()
    }

    pub fn sin(&self) -> S {
        self.rad.sin()
    }

    pub fn tan(&self) -> S {
        self.rad.tan()
    }

    pub fn asin(self) -> S {
        self.rad.asin()
    }

    pub fn acos(self) -> S {
        self.rad.acos()
    }

    pub fn atan(self) -> S {
        self.rad.atan()
    }

    pub fn atan2(self, other: Self) -> S {
        self.rad.atan2(other.rad)
    }

    pub fn floor(self) -> Self {
        self.rad.floor().to_angle()
    }

    pub fn ceil(self) -> Self {
        self.rad.ceil().to_angle()
    }

    pub fn round(self) -> Self {
        self.rad.round().to_angle()
    }

    pub fn trunc(self) -> Self {
        self.rad.trunc().to_angle()
    }

    pub fn fract(self) -> S {
        self.rad.fract()
    }

    pub fn abs(self) -> Self {
        self.rad.abs().to_angle()
    }

    pub fn max(self, other: Self) -> Self {
        self.rad.max(other.rad).to_angle()
    }

    pub fn min(self, other: Self) -> Self {
        self.rad.min(other.rad).to_angle()
    }

    pub fn min_value() -> Self {
        S::min_value().to_angle()
    }
    pub fn max_value() -> Self {
        S::max_value().to_angle()
    }

    pub const fn to_radians(&self) -> S {
        self.rad
    }

    pub fn to_degrees(&self) -> S {
        self.rad.to_degrees()
    }

    pub fn to_hours(&self) -> S {
        self.to_degrees() / S::from(15.0).unwrap()
    }

    pub fn set_format(&mut self, fmt: AngleFormatter) {
        self.fmt = fmt;
    }
}

pub trait ToAngle<S>
where
    S: BaseFloat,
{
    fn to_angle(self) -> Angle<S>;
}

impl<S> ToAngle<S> for S
where
    S: BaseFloat,
{
    fn to_angle(self) -> Angle<S> {
        Angle { rad: self, fmt: Default::default() } 
    }
}

// Convert from and to Rad<S>
impl<S> From<Rad<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(rad: Rad<S>) -> Self {
        rad.0.to_angle()
    }
}
impl<S> From<Angle<S>> for Rad<S>
where
    S: BaseFloat,
{
    fn from(angle: Angle<S>) -> Self {
        Rad(angle.rad)
    }
}

/*
trait AngleUnit<S>: Into<Angle<S>>
where
    S: BaseFloat
{}

impl<S> AngleUnit<S> for ArcSec<S> {}
*/
impl<S, T> PartialEq<T> for Angle<S>
where
    S: BaseFloat,
    T: Into<Angle<S>> + Clone + Copy,
{
    fn eq(&self, other: &T) -> bool {
        let angle: Angle<S> = (*other).into();
        angle.rad == self.rad
    }
}

use std::cmp::PartialOrd;
impl<S, T> PartialOrd<T> for Angle<S>
where
    S: BaseFloat,
    T: Into<Angle<S>> + Clone + Copy,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        let angle: Angle<S> = (*other).into();
        self.rad.partial_cmp(&angle.rad)
    }
}

// Convert from and to ArcDeg<S>
impl<S> From<ArcDeg<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(deg: ArcDeg<S>) -> Self {
        let rad: Rad<S> = deg.into();
        rad.0.to_angle()
    }
}
impl<S> From<Angle<S>> for ArcDeg<S>
where
    S: BaseFloat,
{
    fn from(angle: Angle<S>) -> Self {
        let rad: Rad<S> = angle.into();
        let deg: Deg<S> = rad.into();
        ArcDeg(deg.0)
    }
}

// Convert from ArcMin<S>
impl<S> From<ArcMin<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(min: ArcMin<S>) -> Self {
        let rad: Rad<S> = min.into();
        rad.0.to_angle()
    }
}
// Convert from ArcSec<S>
impl<S> From<ArcSec<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(sec: ArcSec<S>) -> Self {
        let rad: Rad<S> = sec.into();
        rad.0.to_angle()
    }
}

use std::cmp::Ordering;

use std::ops::Div;
impl<S> Div for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let rad = self.rad / rhs.rad;
        rad.to_angle()
    }
}
impl<S> Div<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn div(self, rhs: S) -> Self::Output {
        let rad = self.rad / rhs;
        rad.to_angle()
    }
}

use std::ops::Mul;
impl<S> Mul for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let angle = self.rad * rhs.rad;
        angle.to_angle()
    }
}
impl<S> Mul<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn mul(self, rhs: S) -> Self::Output {
        let angle = self.rad * rhs;
        angle.to_angle()
    }
}

use std::ops::Sub;
impl<S> Sub for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let angle = self.rad - other.rad;
        angle.to_angle()
    }
}
impl<S> Sub<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn sub(self, other: S) -> Self::Output {
        let angle = self.rad - other;
        angle.to_angle()
    }
}

use std::ops::Add;
impl<S> Add for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let angle = self.rad + other.rad;
        angle.to_angle()
    }
}
impl<S> Add<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn add(self, other: S) -> Self::Output {
        let angle = self.rad + other;
        angle.to_angle()
    }
}

use std::ops::AddAssign;
impl<S> AddAssign<S> for Angle<S>
where
    S: BaseFloat,
{
    fn add_assign(&mut self, other: S) {
        *self = *self + other;
    }
}
impl<S> AddAssign<Angle<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn add_assign(&mut self, other: Angle<S>) {
        *self = *self + other;
    }
}

use std::ops::SubAssign;
impl<S> SubAssign<S> for Angle<S>
where
    S: BaseFloat,
{
    fn sub_assign(&mut self, other: S) {
        *self = *self - other;
    }
}
impl<S> SubAssign<Angle<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn sub_assign(&mut self, other: Angle<S>) {
        *self = *self - other;
    }
}

use std::ops::Rem;
impl<S> Rem for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn rem(self, other: Self) -> Self::Output {
        let angle = self.rad % other.rad;
        angle.to_angle()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Deserialize)]
pub enum AngleFormatter {
    Sexagesimal {
        /// Number of digit of precision for the unit value
        /// (interpreted as hours or degrees depending of the hours boolean field)
        prec: u8,
        /// Whether a '+' is added
        plus: bool,
        /// HMS or DMS
        hours: bool,
    },
    Decimal {
        /// Number of digit of precision
        prec: u8,
    }
}

impl Default for AngleFormatter {
    fn default() -> Self {
        AngleFormatter::Decimal { prec: 8 }
    }
}

use std::fmt::Display;
impl Display for Angle<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fmt {
            AngleFormatter::Sexagesimal { prec, plus, hours } => {
                let unit = if hours {
                    self.to_hours()
                } else {
                    self.to_degrees()
                };
                // Round at a specific number of digit of precision
                let pw = 10.0_f64.powi(prec as i32);
                let unit = (unit * pw).round() / pw;

                // Format the unit value to sexagesimal.
                // The precision 8 corresponds to the formatting: deg/hour min sec.ddd
                write!(f, "{}", Format::toSexagesimal(unit, 8, plus))
            },
            AngleFormatter::Decimal { prec } => {
                write!(f, "{:.1$}°", self.to_degrees(), prec as usize)
            }
        }
    }
}

use std::ops::Neg;
impl<S> Neg for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        (-self.rad).to_angle()
    }
}
use al_core::{shader::UniformType, WebGlContext};
use web_sys::WebGlUniformLocation;
impl UniformType for Angle<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.rad);
    }
}
impl UniformType for Angle<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.rad as f32);
    }
}
