use cgmath::BaseFloat;
// ArcDeg wrapper structure
#[derive(Clone, Copy)]
pub struct ArcDeg<T: BaseFloat>(pub T);

//pub const TWICE_PI: f64 = 6.28318530718;
pub const PI: f64 = std::f64::consts::PI;

impl<T> ArcDeg<T>
where
    T: BaseFloat,
{
    fn get_frac_minutes(&self) -> ArcMin<T> {
        let deg = *self;

        let frac = deg.fract();
        let minutes_per_degree = T::from(60_f32).unwrap_abort();
        ArcMin(frac * minutes_per_degree)
    }

    fn truncate(&mut self) {
        *self = Self((*self).trunc());
    }
    /*fn round(&mut self) {
        *self = Self((*self).round());
    }*/
}

use cgmath::{Deg, Rad};
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

// ArcMin wrapper structure
#[derive(Clone, Copy)]
pub struct ArcMin<T: BaseFloat>(pub T);

impl<T> ArcMin<T>
where
    T: BaseFloat,
{
    fn get_frac_seconds(&self) -> ArcSec<T> {
        let min = *self;

        let frac = min.fract();
        let seconds_per_minute = T::from(60_f64).unwrap_abort();
        ArcSec(seconds_per_minute * frac)
    }

    /*fn truncate(&mut self) {
        *self = Self((*self).trunc());
    }*/
}

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

impl<T> ArcSec<T>
where
    T: BaseFloat,
{
    fn _truncate(&mut self) {
        *self = Self((*self).trunc());
    }
}

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

pub trait SerializeToString {
    fn to_string<F: FormatType>(&self) -> String;
}

impl<S> SerializeToString for Angle<S>
where
    S: BaseFloat + ToString,
{
    fn to_string<F: FormatType>(&self) -> String {
        F::to_string(*self)
    }
}
pub struct DMS;
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
use crate::Abort;
impl FormatType for DMS {
    fn to_string<S: BaseFloat + ToString>(angle: Angle<S>) -> String {
        let angle = Rad(angle.0);
        let degrees: ArcDeg<S> = angle.into();
        let minutes = degrees.get_frac_minutes();
        let seconds = minutes.get_frac_seconds();

        let num_per_sec_per_minutes = S::from(60).unwrap_abort();

        let degrees = degrees.round();
        let minutes = minutes.round() % num_per_sec_per_minutes;
        let seconds = seconds.round() % num_per_sec_per_minutes;

        let mut result = degrees.to_string() + "°";
        result += &minutes.to_string();
        result += "\'";
        result += &seconds.to_string();
        result += "\'\'";

        result
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash)]
#[repr(C)]
pub struct Angle<S: BaseFloat>(pub S);
impl<S> Angle<S>
where
    S: BaseFloat,
{
    pub fn new<T: Into<Rad<S>>>(angle: T) -> Angle<S> {
        let radians: Rad<S> = angle.into();
        Angle(radians.0)
    }

    pub fn cos(&self) -> S {
        self.0.cos()
    }

    pub fn sin(&self) -> S {
        self.0.sin()
    }

    pub fn tan(&self) -> S {
        self.0.tan()
    }

    pub fn asin(self) -> S {
        self.0.asin()
    }

    pub fn acos(self) -> S {
        self.0.acos()
    }

    pub fn atan(self) -> S {
        self.0.atan()
    }

    pub fn atan2(self, other: Self) -> S {
        self.0.atan2(other.0)
    }

    pub fn floor(self) -> Self {
        Angle(self.0.floor())
    }

    pub fn ceil(self) -> Self {
        Angle(self.0.ceil())
    }

    pub fn round(self) -> Self {
        Angle(self.0.round())
    }

    pub fn trunc(self) -> Self {
        Angle(self.0.trunc())
    }

    pub fn fract(self) -> S {
        self.0.fract()
    }

    pub fn abs(self) -> Self {
        Angle(self.0.abs())
    }

    pub fn max(self, other: Self) -> Self {
        Angle(self.0.max(other.0))
    }

    pub fn min(self, other: Self) -> Self {
        Angle(self.0.min(other.0))
    }

    pub fn min_value() -> Self {
        Angle(S::min_value())
    }
    pub fn max_value() -> Self {
        Angle(S::max_value())
    }

    pub fn to_radians(&self) -> S {
        self.0
    }

    pub fn to_degrees(&self) -> S {
        self.0.to_degrees()
    }
}

pub trait ToAngle<S>
where
    S: BaseFloat
{
    fn to_angle(self) -> Angle<S>;
}

impl<S> ToAngle<S> for S
where
    S: BaseFloat
{
    fn to_angle(self) -> Angle<S> {
        Angle(self)
    }
}

// Convert from and to Rad<S>
impl<S> From<Rad<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(rad: Rad<S>) -> Self {
        Angle(rad.0)
    }
}
impl<S> From<Angle<S>> for Rad<S>
where
    S: BaseFloat,
{
    fn from(angle: Angle<S>) -> Self {
        Rad(angle.0)
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
        angle.0 == self.0
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
        self.0.partial_cmp(&angle.0)
    }
}

// Convert from and to ArcDeg<S>
impl<S> From<ArcDeg<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(deg: ArcDeg<S>) -> Self {
        let rad: Rad<S> = deg.into();
        Angle(rad.0)
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
        Angle(rad.0)
    }
}
// Convert from ArcSec<S>
impl<S> From<ArcSec<S>> for Angle<S>
where
    S: BaseFloat,
{
    fn from(sec: ArcSec<S>) -> Self {
        let rad: Rad<S> = sec.into();
        Angle(rad.0)
    }
}
/*
impl<S> PartialEq<S> for Angle<S>
where
    S: BaseFloat + !AngleUnit<S>,
{
    fn eq(&self, other: &S) -> bool {
        self.0 == *other
    }
}
*/
use std::cmp::Ordering;
/*impl<S> PartialOrd<S> for Angle<S>
where
    S: BaseFloat,
{
    fn partial_cmp(&self, other: &S) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}*/

use std::ops::Div;
impl<S> Div for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let angle = self.0 / rhs.0;
        Angle(angle)
    }
}
impl<S> Div<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn div(self, rhs: S) -> Self::Output {
        let angle = self.0 / rhs;
        Angle(angle)
    }
}

use std::ops::Mul;
impl<S> Mul for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let angle = self.0 * rhs.0;
        Angle(angle)
    }
}
impl<S> Mul<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn mul(self, rhs: S) -> Self::Output {
        let angle = self.0 * rhs;
        Angle(angle)
    }
}

use std::ops::Sub;
impl<S> Sub for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let angle = self.0 - other.0;
        Angle(angle)
    }
}
impl<S> Sub<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn sub(self, other: S) -> Self::Output {
        let angle = self.0 - other;
        Angle(angle)
    }
}

use std::ops::Add;
impl<S> Add for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let angle = self.0 + other.0;
        Angle(angle)
    }
}
impl<S> Add<S> for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;

    fn add(self, other: S) -> Self::Output {
        let angle = self.0 + other;
        Angle(angle)
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
        let angle = self.0 % other.0;
        Angle(angle)
    }
}

use std::ops::Neg;
impl<S> Neg for Angle<S>
where
    S: BaseFloat,
{
    type Output = Self;
    fn neg(self) -> Self::Output {
        Angle(-self.0)
    }
}
use al_core::{shader::UniformType, WebGlContext};
use web_sys::WebGlUniformLocation;
impl UniformType for Angle<f32> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0);
    }
}
impl UniformType for Angle<f64> {
    fn uniform(gl: &WebGlContext, location: Option<&WebGlUniformLocation>, value: &Self) {
        gl.uniform1f(location, value.0 as f32);
    }
}
