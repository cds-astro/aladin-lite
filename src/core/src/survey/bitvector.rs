struct BitVector<const N: usize>([u64; N]);

impl<const N: usize> BitVector<N> {
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
