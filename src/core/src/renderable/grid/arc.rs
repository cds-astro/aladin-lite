#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Arc {
    DMS {
        deg: i32,
        minute: u32,
        second: f64,
    },
    HMS {
        hour: i32,
        minute: u32,
        second: f64,
    },
    D(f64)
}

impl Arc {
    pub const fn from_degrees(deg: f64) -> Self {
        Arc::D(deg)
    }

    pub const fn from_hms(h: i32, m: u32, s: f64) -> Self {
        Arc::HMS {
            hour: h,
            minute: m,
            second: s
        }
    }

    pub const fn second(value: f64) -> Self {
        Self::DMS {
            deg: 0,
            minute: 0,
            second: value
        }
    }

    pub const fn minute(value: u32) -> Self {
        Self::DMS {
            deg: 0,
            minute: value,
            second: 0.0
        }
    }

    pub const fn deg(value: i32) -> Self {
        Self::DMS {
            deg: value,
            minute: 0,
            second: 0.0
        }
    }

    pub fn to_degrees(&self) -> f64 {
        match self {
            Self::DMS { deg, minute, second } => {
                let sign = if *deg < 0 { -1.0 } else { 1.0 };
                sign * (deg.unsigned_abs() as f64 + (*minute as f64) / 60.0 + second / 3600.0)
            }
            Self::HMS { hour, minute, second } => {
                let sign = if *hour < 0 { -1.0 } else { 1.0 };
                sign * (hour.unsigned_abs() as f64 * 15.0 + (*minute as f64) * 0.25 + second / 240.0)
            }
            Self::D(deg) => *deg,
        }
    }

    pub fn to_decimals(self) -> Self {
        Self::D(self.to_degrees())
    }

    pub fn to_hms(self) -> Self {
        match self {
            // Already HMS, nothing to do
            Self::HMS { .. } => self,

            // DMS or D: convert via total time-seconds
            _ => {
                let total_tsec = self.to_degrees() * 240.0; // deg * (3600 / 15)

                let sign = if total_tsec < 0.0 { -1 } else { 1 };
                let total_abs = total_tsec.abs();

                let hour   = ((total_abs / 3600.0) as i32 * sign) % 24;
                let minute = ((total_abs % 3600.0) / 60.0) as u32;
                let second = total_abs % 60.0;

                Self::HMS { hour, minute, second }
            }
        }
    }
}

impl Mul<i32> for Arc {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        if rhs < 0 {
            -(self * (rhs.unsigned_abs()))
        } else {
            self * (rhs as u32)
        }
    }
}

use std::ops::Neg;
impl Neg for Arc {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::DMS { deg, minute, second } => {
                Self::DMS {
                    deg: -deg,
                    minute,
                    second,
                }
            },
            Self::HMS { hour, minute, second } => {
                let mut hour = -hour;
                if hour < 0 {
                    hour += 24;
                }
                Self::HMS {
                    hour,
                    minute,
                    second,
                }
            },
            Self::D(deg) => Self::D(-deg)
        }
    }
}


impl Add for Arc {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::DMS { deg: d1, minute: m1, second: s1 }, Self::DMS { deg: d2, minute: m2, second: s2 }) => {
                let sign1 = if d1 < 0 { -1.0 } else { 1.0 };
                let arcsec1 = sign1 * (d1.unsigned_abs() as f64 * 3600.0 + m1 as f64 * 60.0 + s1);

                let sign2 = if d2 < 0 { -1.0 } else { 1.0 };
                let arcsec2 = sign2 * (d2.unsigned_abs() as f64 * 3600.0 + m2 as f64 * 60.0 + s2);

                let total = arcsec1 + arcsec2;

                let sign = if total < 0.0 { -1 } else { 1 };
                let total_abs = total.abs();

                let deg    = (total_abs / 3600.0) as i32 * sign;
                let minute = ((total_abs % 3600.0) / 60.0) as u32;
                let second = total_abs % 60.0;

                Self::DMS { deg, minute, second }
            },
            (Self::HMS { hour: h1, minute: m1, second: s1 }, Self::HMS { hour: h2, minute: m2, second: s2 }) => {
                let sign1 = if h1 < 0 { -1.0 } else { 1.0 };
                let tsec1 = sign1 * (h1.unsigned_abs() as f64 * 3600.0 + m1 as f64 * 60.0 + s1);

                let sign2 = if h2 < 0 { -1.0 } else { 1.0 };
                let tsec2 = sign2 * (h2.unsigned_abs() as f64 * 3600.0 + m2 as f64 * 60.0 + s2);

                let total = tsec1 + tsec2;

                let sign = if total < 0.0 { -1 } else { 1 };
                let total_abs = total.abs();

                let mut hour = (total_abs / 3600.0) as i32 * sign;
                if hour < 0 {
                    hour += 24;
                }
                let minute = ((total_abs % 3600.0) / 60.0) as u32;
                let second = total_abs % 60.0;

                Self::HMS { hour, minute, second }
            }
            (Self::D(d1), Self::D(d2)) => Self::D(d1 + d2),
            (lhs, rhs) => Self::D(lhs.to_degrees() + rhs.to_degrees())
        }
    }
}

use std::ops::Sub;
impl Sub for Arc {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

use std::ops::{Add, Mul};
impl Mul<u32> for Arc {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        match self {
            Self::DMS { deg, minute, second } => {
                let new_second = second * (rhs as f64);
                let dminute = new_second.div_euclid(60.0) as u32;

                let new_minute = minute * rhs + dminute;
                let ddeg = new_minute.div_euclid(60) as i32;

                Self::DMS {
                    deg: deg * (rhs as i32) + ddeg,
                    minute: new_minute.rem_euclid(60),
                    second: new_second.rem_euclid(60.0),
                }
            }
            Self::HMS { hour, minute, second } => {
                let new_second = second * (rhs as f64);
                let dminute = new_second.div_euclid(60.0) as u32;

                let new_minute = minute * rhs + dminute;
                let dhour = new_minute.div_euclid(60) as i32;

                let mut hour = hour * (rhs as i32) + dhour;
                if hour < 0 {
                    hour += 24;
                }

                Self::HMS {
                    hour,
                    minute: new_minute.rem_euclid(60),
                    second: new_second.rem_euclid(60.0),
                }
            }
            Self::D(deg) => Self::D(deg * (rhs as f64)),
        }
    }
}

use std::fmt;

pub struct ArcDisplay<'a> {
    arc: &'a Arc,
    precision: usize,
    plus: bool,
}

impl Arc {
    pub fn display(&self, precision: usize, plus: bool) -> ArcDisplay<'_> {
        ArcDisplay { arc: self, precision, plus }
    }
}

impl fmt::Display for ArcDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let p = self.precision;
        // seconds field: always 2 integer digits + dot + p decimal digits
        let sec_width = 3 + p;

        match self.arc {
            Arc::DMS { deg, minute, second } => {
                let sign = if *deg < 0 { "-" } else if self.plus { "+" } else { "" };
                write!(
                    f, "{}{:02}°{:02}′{:0>width$.prec$}″",
                    sign, deg.unsigned_abs(), minute, second,
                    width = sec_width, prec = p
                )
            }
            Arc::HMS { hour, minute, second } => {
                let sign = if *hour < 0 { "-" } else if self.plus { "+" } else { "" };
                write!(
                    f, "{}{}h{:02}m{:0>width$.prec$}s",
                    sign, hour.unsigned_abs(), minute, second,
                    width = sec_width, prec = p
                )
            }
            Arc::D(deg) => {
                let sign = if *deg < 0.0 { "" } else if self.plus { "+" } else { "" };
                write!(f, "{}{:.prec$}°", sign, deg, prec = p)
            }
        }
    }
}