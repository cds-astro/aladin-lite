use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HEALPixCell(pub u8, pub u64);

#[derive(Debug)]
pub struct CellVertices {
    pub vertices: Vec<Box<[(f64, f64)]>>,
}

const BIT_MASK_ALL_ONE_EXCEPT_FIRST: u32 = !0x1;

use healpix::compass_point::Cardinal;
use healpix::compass_point::MainWind;
use healpix::compass_point::Ordinal;
use healpix::compass_point::OrdinalMap;

use crate::utils;
use crate::Abort;
impl HEALPixCell {
    // Build the parent cell
    #[inline(always)]
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

    #[inline(always)]
    pub fn ancestor(self, delta_depth: u8) -> HEALPixCell {
        let HEALPixCell(depth, idx) = self;
        let delta_depth = std::cmp::min(delta_depth, depth);

        HEALPixCell(depth - delta_depth, idx >> (2 * delta_depth))
    }

    // Get the texture cell in which the tile is
    #[inline(always)]
    pub fn get_texture_cell(&self, delta_depth_to_texture: u8) -> HEALPixCell {
        self.ancestor(delta_depth_to_texture)
    }

    #[inline(always)]
    pub fn get_offset_in_texture_cell(&self, delta_depth_to_texture: u8) -> (u32, u32) {
        let texture_cell = self.get_texture_cell(delta_depth_to_texture);
        self.offset_in_parent(&texture_cell)
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn uniq(&self) -> i32 {
        let HEALPixCell(depth, idx) = *self;
        ((16 << (depth << 1)) | idx) as i32
    }

    #[inline(always)]
    pub fn idx(&self) -> u64 {
        self.1
    }

    #[inline(always)]
    pub fn depth(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn is_root(&self) -> bool {
        self.depth() == 0
    }

    // Find the smallest HEALPix cell containing self and another cells
    // Returns None if the 2 HEALPix cell are not located in the same base HEALPix cell
    #[inline]
    pub fn smallest_common_ancestor(&self, other: &HEALPixCell) -> Option<HEALPixCell> {
        // We want the common smallest ancestor between self and another HEALPix cell
        // For this, we should find the number of bits to shift both the 29 order ipix so that
        // they are equal

        // First we compute both cells ipix number at order 29
        let mut c1 = *self;
        let mut c2 = *other;

        if c1.depth() > c2.depth() {
            std::mem::swap(&mut c1, &mut c2);
        }

        let HEALPixCell(d1, idx1) = c1;
        let HEALPixCell(d2, idx2) = c2.ancestor(c2.depth() - d1);

        // idx1 and idx2 belongs to the same order
        // c1 and c2 does not belong to the same HEALPix 0 order cell
        if idx1 >> (2 * d1) != idx2 >> (2 * d2) {
            None
        } else {
            // Find all the equal bits
            let xor = idx1 ^ idx2;

            // Then we retrieve the position of the bit where the value ipixs values change. This is the number of bits
            // we must right shift the ipix 29 order to find the common ipix value
            let xor_lz = xor.leading_zeros() & BIT_MASK_ALL_ONE_EXCEPT_FIRST;
            let msb = ((std::mem::size_of::<u64>() * 8) as u32 - xor_lz) as u8;
            // There is a common ancestor
            Some(HEALPixCell(d1 - (msb >> 1), idx1 >> msb))
        }
    }

    #[inline]
    pub fn smallest_common_ancestors<'a>(
        mut cells: impl Iterator<Item = &'a HEALPixCell>,
    ) -> Option<HEALPixCell> {
        let (first_cell, second_cell) = (cells.next(), cells.next());
        match (first_cell, second_cell) {
            (Some(c1), Some(c2)) => {
                let mut smallest_ancestor = c1.smallest_common_ancestor(c2);

                while let (Some(ancestor), Some(cell)) = (smallest_ancestor, cells.next()) {
                    smallest_ancestor = ancestor.smallest_common_ancestor(&cell);
                }

                smallest_ancestor
            }
            (None, Some(_c2)) => {
                // cannot happen as there must be a first cell before any second one
                // property of iterator
                unreachable!();
            }
            (Some(c1), None) => Some(*c1),
            (None, None) => None,
        }
    }

    // Returns the tile cells being contained into self
    // delta depth between texture stored and tile cells
    #[inline]
    pub fn get_tile_cells(&self, delta_depth: u8) -> impl Iterator<Item = HEALPixCell> {
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

    #[inline(always)]
    pub fn center(&self) -> (f64, f64) {
        healpix::nested::center(self.0, self.1)
    }

    #[inline(always)]
    pub fn vertices(&self) -> [(f64, f64); 4] {
        healpix::nested::vertices(self.0, self.1)
    }

    #[inline(always)]
    pub fn neighbor(&self, wind: MainWind) -> Option<HEALPixCell> {
        let HEALPixCell(d, idx) = *self;
        healpix::nested::neighbours(d, idx, false)
            .get(wind)
            .map(|idx| HEALPixCell(d, *idx))
    }

    #[inline(always)]
    pub fn is_on_pole(&self) -> bool {
        let HEALPixCell(depth, idx) = *self;

        let two_times_depth = 2 * depth;
        let idx_d0 = idx >> two_times_depth;

        match idx_d0 {
            0..=3 => (((idx_d0 + 1) << two_times_depth) - 1) == idx,
            4..=7 => false,
            8..=11 => (idx_d0 << two_times_depth) == idx,
            _ => unreachable!(),
        }
    }

    // Given in ICRS(J2000)
    #[inline]
    pub fn new(depth: u8, theta: f64, delta: f64) -> Self {
        let pix = healpix::nested::hash(depth, theta, delta);

        HEALPixCell(depth, pix)
    }

    #[inline]
    pub fn path_along_cell_edge(&self, n_segments_by_side: u32) -> Box<[(f64, f64)]> {
        healpix::nested::path_along_cell_edge(
            self.depth(),
            self.idx(),
            &healpix::compass_point::Cardinal::S,
            false,
            n_segments_by_side,
        )
    }

    #[inline]
    pub fn path_along_cell_side(
        &self,
        from_vertex: Cardinal,
        to_vertex: Cardinal,
        include_to_vertex: bool,
        n_segments: u32,
    ) -> Box<[(f64, f64)]> {
        healpix::nested::path_along_cell_side(
            self.depth(),
            self.idx(),
            &from_vertex,
            &to_vertex,
            include_to_vertex,
            n_segments,
        )
    }

    pub fn path_along_sides(&self, sides: &OrdinalMap<u32>) -> Option<CellVertices> {
        let se = sides.get(Ordinal::SE);
        let sw = sides.get(Ordinal::SW);
        let ne = sides.get(Ordinal::NE);
        let nw = sides.get(Ordinal::NW);

        let chain_edge_vertices = |card: &[Cardinal], n_segments: &[u32]| -> Box<[(f64, f64)]> {
            let mut vertices = vec![];
            let num_edges = card.len() - 1;
            for (idx, (from_vertex, to_vertex)) in
                (card.iter().zip(card.iter().skip(1))).enumerate()
            {
                let mut edge_vertices = self
                    .path_along_cell_side(
                        *from_vertex,
                        *to_vertex,
                        num_edges - 1 == idx,
                        n_segments[idx],
                    )
                    .into_vec();
                vertices.append(&mut edge_vertices);
            }

            vertices.into_boxed_slice()
        };

        // N -> W, W -> S, S -> E, E -> N
        match (nw, sw, se, ne) {
            // all edges case
            (Some(nw), Some(sw), Some(se), Some(ne)) => Some(CellVertices {
                vertices: vec![
                    self.path_along_cell_side(Cardinal::N, Cardinal::W, false, *nw),
                    self.path_along_cell_side(Cardinal::W, Cardinal::S, false, *sw),
                    self.path_along_cell_side(Cardinal::S, Cardinal::E, false, *se),
                    self.path_along_cell_side(Cardinal::E, Cardinal::N, true, *ne),
                ],
            }),
            // no edges
            (None, None, None, None) => None,
            // 1 edge found
            (Some(s), None, None, None) => Some(CellVertices {
                vertices: vec![self.path_along_cell_side(Cardinal::N, Cardinal::W, true, *s)],
            }),
            (None, Some(s), None, None) => Some(CellVertices {
                vertices: vec![self.path_along_cell_side(Cardinal::W, Cardinal::S, true, *s)],
            }),
            (None, None, Some(s), None) => Some(CellVertices {
                vertices: vec![self.path_along_cell_side(Cardinal::S, Cardinal::E, true, *s)],
            }),
            (None, None, None, Some(s)) => Some(CellVertices {
                vertices: vec![self.path_along_cell_side(Cardinal::E, Cardinal::N, true, *s)],
            }),
            // 2 edges cases
            (Some(nw), Some(sw), None, None) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::N, Cardinal::W, Cardinal::S],
                    &[*nw, *sw],
                )],
            }),
            (Some(nw), None, Some(se), None) => Some(CellVertices {
                vertices: vec![
                    self.path_along_cell_side(Cardinal::N, Cardinal::W, true, *nw),
                    self.path_along_cell_side(Cardinal::S, Cardinal::E, true, *se),
                ],
            }),
            (Some(nw), None, None, Some(ne)) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::E, Cardinal::N, Cardinal::W],
                    &[*ne, *nw],
                )],
            }),
            (None, Some(sw), Some(se), None) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::W, Cardinal::S, Cardinal::E],
                    &[*sw, *se],
                )],
            }),
            (None, Some(sw), None, Some(ne)) => Some(CellVertices {
                vertices: vec![
                    self.path_along_cell_side(Cardinal::W, Cardinal::S, true, *sw),
                    self.path_along_cell_side(Cardinal::E, Cardinal::N, true, *ne),
                ],
            }),
            (None, None, Some(se), Some(ne)) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::S, Cardinal::E, Cardinal::N],
                    &[*se, *ne],
                )],
            }),
            // 3 edges cases
            (Some(nw), Some(sw), Some(se), None) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::N, Cardinal::W, Cardinal::S, Cardinal::E],
                    &[*nw, *sw, *se],
                )],
            }),
            (Some(nw), Some(sw), None, Some(ne)) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::E, Cardinal::N, Cardinal::W, Cardinal::S],
                    &[*ne, *nw, *sw],
                )],
            }),
            (Some(nw), None, Some(se), Some(ne)) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::S, Cardinal::E, Cardinal::N, Cardinal::W],
                    &[*se, *ne, *nw],
                )],
            }),
            (None, Some(sw), Some(se), Some(ne)) => Some(CellVertices {
                vertices: vec![chain_edge_vertices(
                    &[Cardinal::W, Cardinal::S, Cardinal::E, Cardinal::N],
                    &[*sw, *se, *ne],
                )],
            }),
        }
    }

    #[inline]
    pub fn grid(&self, n_segments_by_side: u32) -> Box<[(f64, f64)]> {
        healpix::nested::grid(self.depth(), self.idx(), n_segments_by_side as u16)
    }

    #[inline(always)]
    pub fn z_29(&self) -> u64 {
        self.1 << ((29 - self.0) << 1)
    }

    #[inline(always)]
    pub fn z_29_rng(&self) -> Range<u64> {
        let start = self.1 << ((29 - self.0) << 1);
        let end = (self.1 + 1) << ((29 - self.0) << 1);

        start..end
    }
}

pub const MAX_HPX_DEPTH: u8 = 29;
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

// Follow the z-order curve
impl PartialOrd for HEALPixCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.z_29().partial_cmp(&other.z_29())
    }
}
impl Ord for HEALPixCell {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

// Utils
#[inline(always)]
pub fn nside2depth(nside: u32) -> u8 {
    crate::math::utils::log_2_unchecked(nside) as u8
}

#[cfg(test)]
mod tests {
    use super::HEALPixCell;

    fn test_ancestor(c1: HEALPixCell, c2: HEALPixCell) {
        let test = dbg!(c1.smallest_common_ancestor(&c2));
        let gnd_true = dbg!(get_common_ancestor(c1, c2));

        assert_eq!(test, gnd_true);
    }

    fn get_common_ancestor(mut c1: HEALPixCell, mut c2: HEALPixCell) -> Option<HEALPixCell> {
        if c1.depth() > c2.depth() {
            std::mem::swap(&mut c1, &mut c2);
        }

        c2 = c2.ancestor(c2.depth() - c1.depth());

        while c2 != c1 && c1.depth() > 0 {
            c2 = c2.parent();
            c1 = c1.parent();
        }

        if c1 == c2 {
            Some(c1)
        } else {
            None
        }
    }

    #[test]
    fn test_smallest_common_ancestor() {
        test_ancestor(HEALPixCell(1, 2), HEALPixCell(1, 3));
        test_ancestor(HEALPixCell(3, 0), HEALPixCell(3, 192));
        test_ancestor(HEALPixCell(5, 6814), HEALPixCell(11, 27910909));

        test_ancestor(HEALPixCell(2, 41), HEALPixCell(2, 37));
        assert_eq!(
            HEALPixCell(2, 159).smallest_common_ancestor(&HEALPixCell(2, 144)),
            Some(HEALPixCell(0, 9))
        );
        assert_eq!(
            HEALPixCell(2, 144).smallest_common_ancestor(&HEALPixCell(2, 159)),
            Some(HEALPixCell(0, 9))
        );

        assert_eq!(
            HEALPixCell(3, 0).smallest_common_ancestor(&HEALPixCell(3, 192)),
            None
        );
        test_ancestor(HEALPixCell(3, 0), HEALPixCell(3, 15));
        test_ancestor(HEALPixCell(6, 27247), HEALPixCell(11, 27912704));
        assert_eq!(
            HEALPixCell(9, 1048575).smallest_common_ancestor(&HEALPixCell(9, 786432)),
            Some(HEALPixCell(0, 3))
        );
        assert_eq!(
            HEALPixCell(9, 786432).smallest_common_ancestor(&HEALPixCell(9, 1048575)),
            Some(HEALPixCell(0, 3))
        );

        assert_eq!(
            HEALPixCell(1, 0).smallest_common_ancestor(&HEALPixCell(1, 0)),
            Some(HEALPixCell(1, 0))
        );
    }
}
