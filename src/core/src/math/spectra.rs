pub trait SpectralUnit: Into<Freq> + Clone + Copy {
    fn hash(&self, depth: u8) -> u64 {
        let f: Freq = (*self).into();
        let f_hash_max_order = Frequency::<u64>::freq2hash(f.0);

        f_hash_max_order >> (Frequency::<u64>::MAX_DEPTH - depth)
    }
}

use moclib::qty::{Frequency, MocQty};

/// Frequency in Hz unit
#[derive(Clone, Copy, Debug)]
pub struct Freq(pub f64);

impl Freq {
    pub fn from_hash(hash: u64) -> Self {
        let f = Frequency::hash2freq(hash);

        Freq(f)
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
