//use moclib::moc::range::CellAndNeighs;

/*use crate::renderable::coverage::HEALPixCell;

use healpix::compass_point::Ordinal;

#[derive(Debug)]
pub(super) struct EdgeNeigs {
    // Indices of the neighbors in the stack
    pub neig_idx: Vec<usize>,
    // Smallest depth from the neighbor cells
    pub max_depth_neig: u8,
}

#[derive(Debug)]
pub(super) struct NodeEdgeNeigs {
    pub cell: HEALPixCell,
    pub edge_neigs: [Option<EdgeNeigs>; 4],
}

impl PartialEq for NodeEdgeNeigs {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}

impl NodeEdgeNeigs {
    pub(super) fn add_neig(&mut self, org: Ordinal, neig_idx: usize, neig_cell_depth: u8) {
        let org_idx = org as u8 as usize;
        if let Some(neigs) = &mut self.edge_neigs[org_idx] {
            neigs.neig_idx.push(neig_idx);
            neigs.max_depth_neig = neigs.max_depth_neig.max(neig_cell_depth);
        } else {
            self.edge_neigs[org_idx] = Some(EdgeNeigs {
                neig_idx: vec![neig_idx],
                max_depth_neig: neig_cell_depth,
            });
        }
    }

    pub(super) fn compute_n_seg(&self, side: Ordinal) -> u32 {
        let mut delta_depth =
            if let Some(edge_neigs) = self.edge_neigs[side as u8 as usize].as_ref() {
                edge_neigs.max_depth_neig.max(self.cell.depth()) - self.cell.depth()
            } else {
                0
            };

        if self.cell.depth() + delta_depth < 3 {
            delta_depth = 3 - self.cell.depth();
        }

        if self.cell.depth() >= 6 {
            delta_depth = 0
        }

        1 << delta_depth
    }

    pub(super) fn compute_n_seg_with_neig_info(
        &self,
        neig: &Self,
        side: Ordinal,
        side_neig: Ordinal,
    ) -> u32 {
        let mut delta_depth = if self.cell.depth() > 6 {
            0
        } else {
            if let (Some(edge_neigs), Some(edge_self)) = (
                self.edge_neigs[side as u8 as usize].as_ref(),
                neig.edge_neigs[side_neig as u8 as usize].as_ref(),
            ) {
                edge_neigs
                    .max_depth_neig
                    .max(edge_self.max_depth_neig)
                    .max(self.cell.depth())
                    - self.cell.depth()
            } else {
                0
            }
        };

        if self.cell.depth() + delta_depth < 3 {
            delta_depth = 3 - self.cell.depth();
        }

        1 << delta_depth
    }
}
*/
/*pub(super) struct G {
    nodes: Vec<NodeEdgeNeigs>,
}
use crate::renderable::coverage::mode::Node;
impl G {
    pub(super) fn new(moc: &HEALPixCoverage) -> Self {
        let mut nodes: Vec<_> = (&moc.0)
            .into_range_moc_iter()
            .cells()
            .map(|cell| {
                let cell = HEALPixCell(cell.depth, cell.idx);

                NodeEdgeNeigs {
                    cell,
                    edge_neigs: [None, None, None, None],
                }
            })
            .collect();

        let find_cell_node_idx = |nodes: &[NodeEdgeNeigs], cell: &Cell<u64>| -> usize {
            let hpx_cell = HEALPixCell(cell.depth, cell.idx);
            let result = nodes.binary_search_by(|n| n.cell.cmp(&hpx_cell));
            match result {
                Ok(i) => i,
                Err(_) => unreachable!(),
            }
        };

        // 1. Build the MOC graph structure
        for cell_and_neig in moc.0.all_cells_with_unidirectional_neigs() {
            let CellAndNeighs { cell, neigs } = cell_and_neig;
            // cells are given by uniq order so big cells at first and smaller cells after
            // neighbor information are also given for small cells towards bigger cells
            // Thus we are sure we have already processed the neighbor before as it is a bigger cell or equal order
            // cell having an idx inferior

            let small_node_idx = find_cell_node_idx(&nodes, &cell);

            if let Some(&nw_neig_cell_idx) = neigs.get(Ordinal::NW) {
                //al_core::log("nw neig");
                let nw_neig_cell_d = nodes[nw_neig_cell_idx].cell.depth();
                debug_assert!(nw_neig_cell_d <= cell.depth);

                if let Some(dir) =
                    find_neig_dir(nodes[nw_neig_cell_idx].cell, nodes[small_node_idx].cell)
                {
                    nodes[nw_neig_cell_idx].add_neig(dir, small_node_idx, cell.depth);
                }
                // Add the neig info from the big to the small node
                //nodes[nw_neig_cell_idx].add_neig(Ordinal::SE, small_node_idx, cell.depth);
                // Add the neig info from the small to the big node
                nodes[small_node_idx].add_neig(Ordinal::NW, nw_neig_cell_idx, nw_neig_cell_d);
            }

            if let Some(&ne_neig_cell_idx) = neigs.get(Ordinal::NE) {
                //al_core::log("ne neig");
                let ne_neig_cell_d = nodes[ne_neig_cell_idx].cell.depth();
                debug_assert!(ne_neig_cell_d <= cell.depth);

                if let Some(dir) =
                    find_neig_dir(nodes[ne_neig_cell_idx].cell, nodes[small_node_idx].cell)
                {
                    nodes[ne_neig_cell_idx].add_neig(dir, small_node_idx, cell.depth);
                }

                // Add the neig info from the big to the small node
                //nodes[ne_neig_cell_idx].add_neig(Ordinal::SW, small_node_idx, cell.depth);
                // Add the neig info from the small to the big node
                nodes[small_node_idx].add_neig(Ordinal::NE, ne_neig_cell_idx, ne_neig_cell_d);
            }

            if let Some(&se_neig_cell_idx) = neigs.get(Ordinal::SE) {
                //al_core::log("se neig");
                let se_neig_cell_d = nodes[se_neig_cell_idx].cell.depth();
                debug_assert!(se_neig_cell_d <= cell.depth);

                if let Some(dir) =
                    find_neig_dir(nodes[se_neig_cell_idx].cell, nodes[small_node_idx].cell)
                {
                    nodes[se_neig_cell_idx].add_neig(dir, small_node_idx, cell.depth);
                }

                // Add the neig info from the big to the small node
                //nodes[se_neig_cell_idx].add_neig(Ordinal::NW, small_node_idx, cell.depth);
                // Add the neig info from the small to the big node
                nodes[small_node_idx].add_neig(Ordinal::SE, se_neig_cell_idx, se_neig_cell_d);
            }

            if let Some(&sw_neig_cell_idx) = neigs.get(Ordinal::SW) {
                //al_core::log("sw neig");
                let sw_neig_cell_d = nodes[sw_neig_cell_idx].cell.depth();
                debug_assert!(sw_neig_cell_d <= cell.depth);

                if let Some(dir) =
                    find_neig_dir(nodes[sw_neig_cell_idx].cell, nodes[small_node_idx].cell)
                {
                    nodes[sw_neig_cell_idx].add_neig(dir, small_node_idx, cell.depth);
                }

                // Add the neig info from the big to the small node
                //nodes[sw_neig_cell_idx].add_neig(Ordinal::NE, small_node_idx, cell.depth);
                // Add the neig info from the small to the big node
                nodes[small_node_idx].add_neig(Ordinal::SW, sw_neig_cell_idx, sw_neig_cell_d);
            }
        }

        Self { nodes }
    }

    pub(super) fn get_neigs(
        &self,
        node: &NodeEdgeNeigs,
        dir: Ordinal,
    ) -> Option<Vec<&NodeEdgeNeigs>> {
        node.edge_neigs[dir as u8 as usize]
            .as_ref()
            .map(|edge| {
                if !edge.neig_idx.is_empty() {
                    Some(
                        edge.neig_idx
                            .iter()
                            .map(|idx| &self.nodes[*idx])
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
    }

    pub(super) fn get_neig_dir(
        &self,
        node: &NodeEdgeNeigs,
        neig: &NodeEdgeNeigs,
    ) -> Option<Ordinal> {
        if let Some(neigs) = self.get_neigs(node, Ordinal::NW) {
            if let Some(_) = neigs.iter().find(|&&n| n == neig) {
                return Some(Ordinal::NW);
            }
        }
        if let Some(neigs) = self.get_neigs(node, Ordinal::SW) {
            if let Some(_) = neigs.iter().find(|&&n| n == neig) {
                return Some(Ordinal::SW);
            }
        }
        if let Some(neigs) = self.get_neigs(node, Ordinal::SE) {
            if let Some(_) = neigs.iter().find(|&&n| n == neig) {
                return Some(Ordinal::SE);
            }
        }
        if let Some(neigs) = self.get_neigs(node, Ordinal::NE) {
            if let Some(_) = neigs.iter().find(|&&n| n == neig) {
                return Some(Ordinal::NE);
            }
        }

        None
    }

    pub(super) fn nodes_iter<'a>(&'a self) -> impl Iterator<Item = &'a NodeEdgeNeigs> {
        self.nodes.iter()
    }

    pub(super) fn nodes(&self) -> &[NodeEdgeNeigs] {
        &self.nodes[..]
    }
}

fn find_neig_dir(mut cell: HEALPixCell, mut neig: HEALPixCell) -> Option<Ordinal> {
    if cell.depth() > neig.depth() {
        cell = cell.ancestor(cell.depth() - neig.depth());
    } else if cell.depth() < neig.depth() {
        neig = neig.ancestor(neig.depth() - cell.depth());
    }

    if let Some(nw) = cell.neighbor(MainWind::NW) {
        if nw == neig {
            return Some(Ordinal::NW);
        }
    }

    if let Some(ne) = cell.neighbor(MainWind::NE) {
        if ne == neig {
            return Some(Ordinal::NE);
        }
    }

    if let Some(sw) = cell.neighbor(MainWind::SW) {
        if sw == neig {
            return Some(Ordinal::SW);
        }
    }

    if let Some(se) = cell.neighbor(MainWind::SE) {
        if se == neig {
            return Some(Ordinal::SE);
        }
    }

    None
}
*/
