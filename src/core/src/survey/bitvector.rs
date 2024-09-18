struct BitVector<const N: usize>([u64; N]);

impl<const N: usize> Default for BitVector<N> {
    fn default() -> Self {
        Self([0_u64; N])
    }
}

impl<const N: usize> BitVector<N> {
    pub fn new_empty() -> Self {
        Self::default()
    }
    pub fn set(&mut self, i: usize) {
        debug_assert!(i < (N << 6));
        let j = i >> 6;
        let k = i & 0x3f;

        self.0[j] |= 1 << k;
    }

    pub fn get(&self, i: usize) -> bool {
        debug_assert!(i < (N << 6));
        let j = i >> 6;
        let k = i & 0x3f;

        (self.0[j] >> k) & 0x1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::BitVector;

    #[test]
    fn test_bitvector_basic_op() {
        let mut bv: BitVector<32> = BitVector::new_empty();
        bv.set(64);
        assert!(bv.get(64));
    }
}
