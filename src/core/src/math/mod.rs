pub const TWICE_PI: f64 = std::f64::consts::TAU;
pub const PI: f64 = std::f64::consts::PI;

pub const HALF_PI: f64 = std::f64::consts::PI * 0.5;
pub const MINUS_HALF_PI: f64 = -std::f64::consts::PI * 0.5;

pub const TWO_SQRT_TWO: f64 = 2.82842712475;
pub const SQRT_TWO: f64 = 1.41421356237;

pub const ZERO: f64 = 0.0;

pub mod angle;
pub mod lonlat;
pub mod projection;
pub mod rotation;
pub mod sph_geom;
pub mod utils;
pub mod vector;
