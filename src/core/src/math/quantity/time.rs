use std::ops::Add;
use moclib::qty::MocQty;

use super::Qty;
impl Qty for Time {
    /// The type of the quantity
    type T = u64;

    /// Min & Max values
    const MIN_VALUE: u64 = 0;
    const MAX_VALUE: u64 = (1_u64 << Self::MAX_DEPTH) - 1;

    const MAX_DEPTH: u8 = 61;

    fn hash_max_order(&self) -> u64 {
        self.0
    }

    fn from_hash(hash: u64) -> Self {
        Time(hash)
    }

    fn order(&self, dq: Self) -> u8 {
        61 - (dq.0.ilog2() as u8)
    }

    fn num_max_cells(order: u8) -> u64 {
        moclib::qty::Time::<u64>::n_cells_max() >> (Self::MAX_DEPTH - order)
    }

    fn new(value: Self::T) -> Self {
        Self(value.max(Self::MIN_VALUE).min(Self::MAX_VALUE))
    }

    fn dq(&self, order_pixel: u8) -> Self {
        let t = 1_u64 << (61 - order_pixel);

        Time(t)
    }
}

/// Time in micro-seconds since JD=0
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct Time(pub u64);

impl Time {
    pub(crate) fn from_mjd(mjd: f64) -> Self {
        let jd = mjd + 2400000.5;
        let us = jd * 86400000000.0;
        Self(us as u64)
    }

    pub fn max(&self, other: Self) -> Self {
        Time(self.0.max(other.0))
    }

    pub fn min(&self, other: Self) -> Self {
        Time(self.0.min(other.0))
    }
}

use std::ops::Sub;
impl Sub for Time {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Add for Time {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

use std::ops::Mul;
impl Mul<u64> for Time {
    type Output = Self;

    fn mul(self, other: u64) -> Self::Output {
        Self(self.0 * other)
    }
}

use std::ops::Div;
impl Div<u64> for Time {
    type Output = Self;

    fn div(self, other: u64) -> Self::Output {
        Self(self.0 / other)
    }
}
