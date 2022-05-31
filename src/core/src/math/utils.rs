//use num_traits::Float;
#[inline]
pub fn asinc_positive(x: f64) -> f64 {
    debug_assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.asin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        let x2 = x * x;
        1.0 + x2 / 6.0 + x2 * x2 * 0.075
    }
}

#[inline]
pub fn sinc_positive(x: f64) -> f64 {
    debug_assert!(x >= 0.0);
    if x > 1.0e-4 {
        x.sin() / x
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        let x2 = x * x;
        1.0 - x2 / 6.0 + x2 * x2 * 0.075
    }
}

#[inline]
const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

use num::traits::PrimInt;
use num::traits::Zero;
#[inline]
pub fn log_2_checked<T>(x: T) -> u32
where
    T: PrimInt + Zero,
{
    debug_assert!(x > T::zero());
    num_bits::<T>() as u32 - x.leading_zeros() - 1
}

#[inline]
pub fn log_2_unchecked<T>(x: T) -> u32
where
    T: PrimInt,
{
    num_bits::<T>() as u32 - x.leading_zeros() - 1
}

use num::One;
use std::cmp::PartialEq;
use std::ops::BitAnd;
use std::ops::Sub;
#[inline]
pub fn is_power_of_two<T>(x: T) -> bool
where
    T: BitAnd<Output = T> + One + Zero + Sub<Output = T> + PartialEq + Copy,
{
    x.bitand(x - T::one()) == T::zero()
}

/// Compute the negative branch of the lambert fonction (W_{-1})
/// defined for x in [-1/e; 0[
/// This paper: https://doi.org/10.1016/S0378-4754(00)00172-5
/// gives an analytical approximation with a relative error of 0.025%
#[inline]
#[allow(dead_code)]
pub fn lambert_wm1(x: f32) -> f32 {
    debug_assert!((-1.0 / std::f32::consts::E..0.0).contains(&x));
    let m1 = 0.3361;
    let m2 = -0.0042;
    let m3 = -0.0201;

    let s = -1.0 - (-x).ln();
    let s_root = s.sqrt();
    let s_div_2_root = (s * 0.5).sqrt();

    -1.0 - s
        - (2.0 / m1)
            * (1.0 - 1.0 / (1.0 + ((m1 * s_div_2_root) / (1.0 + m2 * s * (m3 * s_root).exp()))))
}
