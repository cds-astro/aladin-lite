





/*
use super::super::graph::G;

pub struct Fill;

impl RenderMode for Fill {
    fn build(moc: &HEALPixCoverage) -> Vec<Node> {
        let g = G::new(moc);

        let n_seg_from_dir = |n: &NodeEdgeNeigs, dir: Ordinal| -> u32 {
            if let Some(neigs) = g.get_neigs(n, dir) {
                if let Some(neig_side) = g.get_neig_dir(neigs[0], n) {
                    n.compute_n_seg_with_neig_info(neigs[0], dir, neig_side)
                } else {
                    1
                }
            } else {
                1
            }
        };

        g.nodes_iter()
            .flat_map(|n| {
                let cell = n.cell;

                // Draw all of the node's edges
                let n_seg_nw = n_seg_from_dir(n, Ordinal::NW);
                let n_seg_ne = n_seg_from_dir(n, Ordinal::NE);
                let n_seg_sw = n_seg_from_dir(n, Ordinal::SW);
                let n_seg_se = n_seg_from_dir(n, Ordinal::SE);

                match cell.depth() {
                    0 => {
                        let n_seg_sw = (n_seg_sw >> 2).max(1);
                        let n_seg_se = (n_seg_se >> 2).max(1);
                        let n_seg_nw = (n_seg_nw >> 2).max(1);
                        let n_seg_ne = (n_seg_ne >> 2).max(1);

                        cell.get_children_cells(2)
                            .map(|child_cell| {
                                let mut edges = OrdinalMap::new();

                                let off = child_cell.idx() - (cell.idx() << 4);

                                match off {
                                    // S
                                    0 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, n_seg_sw);
                                        edges.put(Ordinal::SE, n_seg_se);
                                    }
                                    // W
                                    10 => {
                                        edges.put(Ordinal::NW, n_seg_nw);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, n_seg_sw);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    // E
                                    5 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, n_seg_ne);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, n_seg_se);
                                    }
                                    // N
                                    15 => {
                                        edges.put(Ordinal::NW, n_seg_nw);
                                        edges.put(Ordinal::NE, n_seg_ne);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    // SE
                                    1 | 4 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, n_seg_se);
                                    }
                                    // SW
                                    2 | 8 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, n_seg_sw);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    // NW
                                    11 | 14 => {
                                        edges.put(Ordinal::NW, n_seg_nw);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    // NE
                                    7 | 13 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, n_seg_ne);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    _ => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                }

                                Node {
                                    vertices: child_cell.path_along_sides(&edges),
                                    cell: child_cell,
                                }
                            })
                            .collect()
                    }
                    1 => {
                        let n_seg_sw = (n_seg_sw >> 1).max(1);
                        let n_seg_se = (n_seg_se >> 1).max(1);
                        let n_seg_nw = (n_seg_nw >> 1).max(1);
                        let n_seg_ne = (n_seg_ne >> 1).max(1);

                        cell.get_children_cells(1)
                            .map(|child_cell| {
                                let mut edges = OrdinalMap::new();

                                let off = child_cell.idx() - (cell.idx() << 2);
                                match off {
                                    // S
                                    0 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, n_seg_sw);
                                        edges.put(Ordinal::SE, n_seg_se);
                                    }
                                    // W
                                    2 => {
                                        edges.put(Ordinal::NW, n_seg_nw);
                                        edges.put(Ordinal::NE, 1);
                                        edges.put(Ordinal::SW, n_seg_sw);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    // E
                                    1 => {
                                        edges.put(Ordinal::NW, 1);
                                        edges.put(Ordinal::NE, n_seg_ne);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, n_seg_se);
                                    }
                                    // N
                                    3 => {
                                        edges.put(Ordinal::NW, n_seg_nw);
                                        edges.put(Ordinal::NE, n_seg_ne);
                                        edges.put(Ordinal::SW, 1);
                                        edges.put(Ordinal::SE, 1);
                                    }
                                    _ => {
                                        unimplemented!();
                                    }
                                }

                                Node {
                                    vertices: child_cell.path_along_sides(&edges),
                                    cell: child_cell,
                                }
                            })
                            .collect()
                    }
                    _ => {
                        let mut edges = OrdinalMap::new();

                        edges.put(Ordinal::NW, n_seg_nw);
                        edges.put(Ordinal::NE, n_seg_ne);
                        edges.put(Ordinal::SW, n_seg_sw);
                        edges.put(Ordinal::SE, n_seg_se);

                        vec![Node {
                            vertices: cell.path_along_sides(&edges),
                            cell,
                        }]
                    }
                }
            })
            .collect()
    }
}
*/
