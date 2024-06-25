use super::Node;
use super::RenderMode;
use crate::healpix::cell::HEALPixCell;
use healpix::compass_point::{Ordinal, OrdinalMap};
use moclib::elem::cell::Cell;

use crate::HEALPixCoverage;
use moclib::moc::range::CellAndEdges;

pub struct Perimeter;

impl RenderMode for Perimeter {
    fn build(moc: &HEALPixCoverage) -> impl Iterator<Item = Node> {
        moc.0
            .border_elementary_edges()
            .map(|CellAndEdges { uniq, edges }| {
                let c = Cell::from_uniq_hpx(uniq);
                let cell = HEALPixCell(c.depth, c.idx);

                let mut map = OrdinalMap::new();
                if edges.get(moclib::moc::range::Ordinal::SE) {
                    map.put(Ordinal::SE, 1);
                }
                if edges.get(moclib::moc::range::Ordinal::SW) {
                    map.put(Ordinal::SW, 1);
                }
                if edges.get(moclib::moc::range::Ordinal::NE) {
                    map.put(Ordinal::NE, 1);
                }
                if edges.get(moclib::moc::range::Ordinal::NW) {
                    map.put(Ordinal::NW, 1);
                }

                let vertices = cell.path_along_sides(&map);

                Node { cell, vertices }
            })
    }
}
