use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HEALPixCell(pub u8, pub u64);

use crate::renderable::projection::Projection;

use crate::buffer::HiPSConfig;
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

        let ancestor = HEALPixCell(depth - delta_depth, idx >> (2 * delta_depth));
        ancestor
    }

    // Get the texture cell in which the tile is
    pub fn get_texture_cell(&self, config: &HiPSConfig) -> HEALPixCell {
        let delta_depth_to_texture = config.delta_depth();
        let texture_cell = self.ancestor(delta_depth_to_texture);
        texture_cell
    }
    pub fn get_offset_in_texture_cell(&self, config: &HiPSConfig) -> (u32, u32) {
        let texture_cell = self.get_texture_cell(config);
        self.offset_in_parent(&texture_cell)
    }

    pub fn offset_in_parent(&self, parent_cell: &HEALPixCell) -> (u32, u32) {
        let HEALPixCell(depth, idx) = *self;
        let HEALPixCell(parent_depth, parent_idx) = *parent_cell;

        let idx_off = parent_idx << (2 * (depth - parent_depth));

        assert!(idx >= idx_off);
        assert!(depth >= parent_depth);
        let nside = 1 << (depth - parent_depth);

        let (x, y) = utils::unmortonize(idx - idx_off);
        assert!(x < nside);
        assert!(y < nside);

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
    pub fn get_tile_cells(&self, config: &HiPSConfig) -> HEALPixTilesIter {
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
}

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
        self.partial_cmp(&other).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct HEALPixCellUniqOrd<'a> {
    cell: &'a HEALPixCell,
}

impl<'a> PartialOrd for HEALPixCellUniqOrd<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let u1 = self.cell.uniq();
        let u2 = other.cell.uniq();

        u1.partial_cmp(&u2)
    }
}
impl<'a> Ord for HEALPixCellUniqOrd<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
pub struct SphereSubdivided;

impl SphereSubdivided {
    // Get the number of subdivision necessary for the given cell
    pub fn get_num_subdivide<P: Projection>(&self, cell: &HEALPixCell) -> u8 {
        let HEALPixCell(depth, _idx) = *cell;
        let num_sub = if depth < 5 {
            // Get the 3 depth cells contained in it and add
            // each of them individually to the buffer
            /*let idx_off = (idx << (2*(5 - depth))) as usize;
            let idx_off2 = ((idx + 1) << (2*(5 - depth))) as usize;

            let num_sub_d3_cells = self.0[idx_off..idx_off2].iter().max();
            *num_sub_d3_cells.unwrap() + (5 - depth)*/
            std::cmp::min(5 - depth, 3)
        } else {
            /*let idx_d3 = cell.idx() >> (2*(depth - 6));
            let num_sub_d3 = self[idx_d3 as usize];

            if depth > num_sub_d3 + 6 {
                0
            } else {
                num_sub_d3 - (depth - 6)
            }*/
            0
        };
        num_sub

        //std::cmp::min(num_sub, 3)
    }
}
