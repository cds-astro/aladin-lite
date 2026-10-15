pub mod frequency;
pub mod time; 

pub trait Qty: Clone + Copy {
    /// The type of the quantity
    type T;

    /// Min & Max values
    const MIN_VALUE: Self::T;
    const MAX_VALUE: Self::T;

    const MAX_DEPTH: u8;

    fn hash(&self, order: u8) -> u64 {
        let f_hash_max_order = self.hash_max_order();
        f_hash_max_order >> (Self::MAX_DEPTH - order)
    }

    fn from_hash_with_order(hash: u64, order: u8) -> Self {
        let hash_max_order = hash << (Self::MAX_DEPTH - order);
        Self::from_hash(hash_max_order)
    }

    fn hash_max_order(&self) -> u64;

    fn from_hash(hash: u64) -> Self;

    fn num_max_cells(order: u8) -> u64;

    fn new(value: Self::T) -> Self;

    fn order(&self, dq: Self) -> u8;

    /// The quantity coverage, at a dedicated order, between to consecutive index
    fn dq(&self, order: u8) -> Self;
}

use frequency::Freq;
use time::Time;
/// This is a wrapper of all the quantity types (impl Qty)
#[derive(Clone, Copy)]
#[derive(Debug)]
pub(crate) enum QtyType {
    Freq(Freq),
    Time(Time)
}

impl From<QtyType> for f64 {
    fn from(item: QtyType) -> Self {
        match item {
            QtyType::Freq(freq) => freq.0,
            QtyType::Time(time) => time.0 as f64, 
        }
    }
}

impl QtyType {
    pub(crate) fn get_max_order(&self) -> u8 {
        match self {
            QtyType::Freq(_) => Freq::MAX_DEPTH,
            QtyType::Time(_) => Time::MAX_DEPTH,
        }
    }

    pub(crate) fn order(&self, dq: Self) -> u8 {
        match (self, dq) {
            (QtyType::Freq(f), QtyType::Freq(df)) => f.order(df),
            (QtyType::Time(t), QtyType::Time(dt)) => t.order(dt), 
            _ => unreachable!()
        }
    }

    pub(crate) fn dq(&self, order: u8) -> QtyType {
        match self {
            QtyType::Freq(freq) => QtyType::Freq(freq.dq(order)),
            QtyType::Time(time) => QtyType::Time(time.dq(order)), 
        }
    }

    pub(crate) fn hash(&self, order: u8) -> u64 {
        match self {
            QtyType::Freq(freq) => freq.hash(order),
            QtyType::Time(time) => time.hash(order), 
        }
    }

    pub(crate) fn hash_max_order(&self) -> u64 {
        match self {
            QtyType::Freq(freq) => freq.hash_max_order(),
            QtyType::Time(time) => time.hash_max_order(), 
        }
    }
}
