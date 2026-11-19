use moclib::qty::{Frequency, MocQty};
use crate::math::quantity::Qty;
use std::convert::From;
impl Qty for Freq {
    /// Frequency is expressed in double floating point
    type T = f64;

    /// Frequency in Hz unit
    const MIN_VALUE: Self::T = 1e-18;
    const MAX_VALUE: Self::T = 1e+38;

    const MAX_DEPTH: u8 = Frequency::<u64>::MAX_DEPTH;

    fn hash_max_order(&self) -> u64 {
        let f: Freq = (*self).into();
        Frequency::<u64>::freq2hash(f.0)
    }

    fn from_hash(hash: u64) -> Self {
        let f = Frequency::hash2freq(hash);
        Freq(f).into()
    }

    fn num_max_cells(order: u8) -> u64 {
        Frequency::<u64>::n_cells_max() >> (Self::MAX_DEPTH - order)
    }

    fn new(value: Self::T) -> Self {
        Freq(value.max(Self::MIN_VALUE).min(Self::MAX_VALUE))
    }

    fn order(&self, dq: Self) -> u8 {
        (56.0 / (1.0 + dq.0 / self.0).log10()).log2().floor() as u8
    }

    fn dq(&self, order: u8) -> Self {
        let df = self.0 * (10.0_f64.powf(56.0 / ((1 << order) as f64)) - 1.0);

        Freq(df)
    }
}

use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Freq(pub f64);

impl Freq {
    pub fn max(&self, other: Self) -> Self {
        Freq(self.0.max(other.0))
    }

    pub fn min(&self, other: Self) -> Self {
        Freq(self.0.min(other.0))
    }
}

use std::ops::Sub;
impl Sub for Freq {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Add for Freq {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

use std::ops::Mul;
impl Mul<f64> for Freq {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self(self.0 * other)
    }
}

use std::ops::Div;
impl Div<f64> for Freq {
    type Output = Self;

    fn div(self, other: f64) -> Self::Output {
        Self(self.0 / other)
    }
}

/// Wavelength in meter unit
#[derive(Clone, Copy)]
pub struct Wavelength(pub f64);

/// Velocity in meter/sec unit
#[derive(Clone, Copy)]
pub struct Velocity {
    /// A rest frequency to compute the velocity from
    /// given by the obs_restfreq HiPS property
    rest_freq: Freq,
    /// The velocity in m/s
    velocity: f64,
}

const SPEED_OF_LIGHT: f64 = 299792458.0;

impl From<Velocity> for Freq {
    fn from(v: Velocity) -> Self {
        let Velocity {
            rest_freq,
            velocity,
        } = v;

        // v = c * (of - f) / of
        // v * of = c * (of - f)
        // c * f = c * of - v * of = of * (c - v)
        // f = of * (c - v) / c = of * (1 - v / c)

        Freq(rest_freq.0 * (1.0 - velocity / SPEED_OF_LIGHT))
    }
}

impl From<Wavelength> for Freq {
    fn from(lambda: Wavelength) -> Self {
        Freq(SPEED_OF_LIGHT / lambda.0)
    }
}
