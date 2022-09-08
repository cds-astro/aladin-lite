use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HEALPixCell(pub u8, pub u64);

use crate::survey::config::HiPSConfig;
use crate::utils;
impl HEALPixCell {
    // Build the parent cell
    #[inline]
    pub fn parent(self) -> HEALPixCell {
        let depth = self.depth();
        if depth == 0 {
            // If cell belongs to a root cell
            // we return it as a root cell do not have any parent
            self
        } else {
            HEALPixCell(depth - 1, self.1 >> 2)
        }
    }

    pub fn ancestor(self, delta_depth: u8) -> HEALPixCell {
        let HEALPixCell(depth, idx) = self;
        let delta_depth = std::cmp::min(delta_depth, depth);

        HEALPixCell(depth - delta_depth, idx >> (2 * delta_depth))
    }

    // Get the texture cell in which the tile is
    pub fn get_texture_cell(&self, config: &HiPSConfig) -> HEALPixCell {
        let delta_depth_to_texture = config.delta_depth();

        self.ancestor(delta_depth_to_texture)
    }
    pub fn get_offset_in_texture_cell(&self, config: &HiPSConfig) -> (u32, u32) {
        let texture_cell = self.get_texture_cell(config);
        self.offset_in_parent(&texture_cell)
    }

    pub fn offset_in_parent(&self, parent_cell: &HEALPixCell) -> (u32, u32) {
        let HEALPixCell(depth, idx) = *self;
        let HEALPixCell(parent_depth, parent_idx) = *parent_cell;

        let idx_off = parent_idx << (2 * (depth - parent_depth));

        debug_assert!(idx >= idx_off);
        debug_assert!(depth >= parent_depth);
        let nside = 1 << (depth - parent_depth);

        let (x, y) = utils::unmortonize(idx - idx_off);
        debug_assert!(x < nside);
        debug_assert!(y < nside);

        (x, y)
    }

    #[inline]
    pub fn uniq(&self) -> i32 {
        let HEALPixCell(depth, idx) = *self;
        ((16 << (depth << 1)) | idx) as i32
    }

    #[inline]
    pub fn idx(&self) -> u64 {
        self.1
    }

    #[inline]
    pub fn depth(&self) -> u8 {
        self.0
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.depth() == 0
    }

    // Returns the tile cells being contained into self
    #[inline]
    pub fn get_tile_cells(&self, config: &HiPSConfig) -> impl Iterator<Item = HEALPixCell> {
        let delta_depth = config.delta_depth();
        self.get_children_cells(delta_depth)
    }

    #[inline]
    pub fn get_children_cells(&self, delta_depth: u8) -> HEALPixTilesIter {
        let HEALPixCell(depth, idx) = *self;
        let first_idx = idx << (2 * delta_depth);
        let last_idx = (idx + 1) << (2 * delta_depth);

        let depth_children = depth + delta_depth;
        HEALPixTilesIter::new(depth_children, first_idx..last_idx)
    }

    #[inline]
    pub fn allsky(depth: u8) -> impl Iterator<Item = HEALPixCell> {
        let npix = 12 << ((depth as usize) << 1);
        (0_u64..(npix as u64)).map(move |pix| HEALPixCell(depth, pix))
    }

    #[inline]
    pub fn center(&self) -> (f64, f64) {
        cdshealpix::nested::center(self.0, self.1)
    }

    #[inline]
    pub fn vertices(&self) -> [(f64, f64); 4] {
        cdshealpix::nested::vertices(self.0, self.1)
    }

    #[inline]
    pub fn is_on_pole(&self) -> bool {
        let idx_d0 = self.idx() >> (2*self.depth());

        match idx_d0 {
            0..=3 => {
                (((idx_d0 + 1) << (2*self.depth())) - 1) == self.idx()
            },
            8..=11 => {
                (idx_d0 << (2*self.depth())) == self.idx()
            },
            4..=7 => false,
            _ => unreachable!()
        }
    }

    // Given in ICRS(J2000)
    #[inline]
    pub fn new(&self, depth: u8, theta: f64, delta: f64) -> Self {
        let pix = cdshealpix::nested::hash(depth, theta, delta);

        HEALPixCell(depth, pix)
    }

    #[inline]
    pub fn path_along_cell_edge(
        &self,
        n_segments_by_side: u32
    ) -> Box<[(f64, f64)]> {
        cdshealpix::nested::path_along_cell_edge(
            self.depth(),
            self.idx(),
            &cdshealpix::compass_point::Cardinal::S,
            false,
            n_segments_by_side
        )
    }
}

pub const NUM_HPX_TILES_DEPTH_ZERO: usize = 12;
pub const ALLSKY_HPX_CELLS_D0: &[HEALPixCell; NUM_HPX_TILES_DEPTH_ZERO] = &[
    HEALPixCell(0, 0),
    HEALPixCell(0, 1),
    HEALPixCell(0, 2),
    HEALPixCell(0, 3),
    HEALPixCell(0, 4),
    HEALPixCell(0, 5),
    HEALPixCell(0, 6),
    HEALPixCell(0, 7),
    HEALPixCell(0, 8),
    HEALPixCell(0, 9),
    HEALPixCell(0, 10),
    HEALPixCell(0, 11),
];

use std::ops::Range;

pub struct HEALPixTilesIter {
    depth: u8,
    idx: Range<u64>,
    off: u64,
    num_tiles: u64,
}

impl HEALPixTilesIter {
    fn new(depth: u8, idx: Range<u64>) -> Self {
        let off = 0;
        let num_tiles = idx.end - idx.start;
        Self {
            depth,
            idx,
            off,
            num_tiles,
        }
    }
}

impl Iterator for HEALPixTilesIter {
    type Item = HEALPixCell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.off < self.num_tiles {
            let d = self.depth;
            let idx = self.idx.start + self.off;
            self.off += 1;
            Some(HEALPixCell(d, idx))
        } else {
            None
        }
    }
}

impl PartialOrd for HEALPixCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let n1 = self.1 << ((29 - self.0) << 1);
        let n2 = other.1 << ((29 - other.0) << 1);

        n1.partial_cmp(&n2)
    }
}
impl Ord for HEALPixCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
