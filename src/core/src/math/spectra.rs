pub trait SpectralUnit: Into<Freq> + Clone + Copy {
    fn hash(&self, order_f: u8) -> u64 {
        let f: Freq = (*self).into();
        let f_hash_max_order = Frequency::<u64>::freq2hash(f.0);

        f_hash_max_order >> (Frequency::<u64>::MAX_DEPTH - order_f)
    }

    /// The frequency coverage, at a dedicated order, between to consecutive index is provided by this expression:
    fn delta_freq(&self, order_f: u8) -> Freq {
        let f: Freq = (*self).into();
        let exponent = 56.0 / ((1u64 << order_f) as f64);

        Freq(f.0 * (10.0_f64.powf(exponent) - 1.0))
    }
}

use std::ops::Add;

/// The appropriate order required to map a ∆ftarget frequency at frequency f
//  at a resolution is approximated by this formulae:
pub(crate) fn order_f<S: SpectralUnit, V: SpectralUnit>(freq: S, delta_f_target: V) -> u8 {
    let freq: Freq = freq.into();
    let delta_f_target: Freq = delta_f_target.into();

    ((56.0_f64 * freq.0 / delta_f_target.0).log2())
        .round() as u8
}

use moclib::qty::{Frequency, MocQty};

/// Frequency in Hz unit
pub(crate) const FREQ_MIN: f64 = 1e-18;
pub(crate) const FREQ_MAX: f64 = 1e+38;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Freq(pub f64);

impl Freq {
    pub fn from_hash(hash: u64) -> Self {
        let f = Frequency::hash2freq(hash);

        Freq(f)
    }

    pub fn from_hash_with_order(hash: u64, order: u8) -> Self {
        let hash_max_order = hash << (Frequency::<u64>::MAX_DEPTH - order);
        let f = Frequency::hash2freq(hash_max_order);

        Freq(f)
    }

    pub fn max(&self, other: Self) -> Self {
        Freq(self.0.max(other.0))
    }

    pub fn min(&self, other: Self) -> Self {
        Freq(self.0.min(other.0))
    }

    pub fn num_max_cells(order: u8) -> usize {
        (Frequency::<u64>::n_cells_max() >> (Frequency::<u64>::MAX_DEPTH - order)) as usize
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

impl SpectralUnit for Freq {}
impl SpectralUnit for Wavelength {}
impl SpectralUnit for Velocity {}
