







/*

pub struct Edge;

impl RenderMode for Edge {
    fn build(moc: &HEALPixCoverage) -> Vec<Node> {
        let g = graph::G::new(moc);

        // 2. Precompute the vertices from the graph structure
        g.nodes_iter()
            .flat_map(|n| {
                let mut edges = OrdinalMap::new();
                let cell = n.cell;

                if let Some(edge_neigs) = &n.edge_neigs[Ordinal::NW as u8 as usize] {
                    // if the smallest neig for this edge is smaller than self
                    let _smallest_neig_depth = edge_neigs.max_depth_neig;

                    let first_neig_idx = edge_neigs.neig_idx[0];
                    let neig_cell = &g.nodes()[first_neig_idx].cell;

                    let draw_side =
                            // the current node has several (smaller) neig
                            edge_neigs.neig_idx.len() > 1
                            // or it has only one neig and if so
                            // we draw the side either if the node's idx is < to the neig's idx
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() == neig_cell.depth()
                                && neig_cell.idx() > cell.idx())
                            // or we draw the side if the neig is smaller than the node
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() < neig_cell.depth());

                    if draw_side {
                        debug_assert!(edge_neigs.max_depth_neig >= cell.depth());

                        // draw the NW edge
                        edges.put(Ordinal::NW, n.compute_n_seg(Ordinal::NW));
                    }
                } else {
                    // draw the NW edge because there are no neig along that edge
                    edges.put(Ordinal::NW, n.compute_n_seg(Ordinal::NW));
                }

                if let Some(edge_neigs) = &n.edge_neigs[Ordinal::SW as u8 as usize] {
                    // if the smallest neig for this edge is smaller than self
                    let _smallest_neig_depth = edge_neigs.max_depth_neig;

                    let first_neig_idx = edge_neigs.neig_idx[0];
                    let neig_cell = &g.nodes()[first_neig_idx].cell;

                    let draw_side =
                            // the current node has several (smaller) neig
                            edge_neigs.neig_idx.len() > 1
                            // or it has only one neig and if so
                            // we draw the side either if the node's idx is < to the neig's idx
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() == neig_cell.depth()
                                && neig_cell.idx() > cell.idx())
                            // or we draw the side if the neig is smaller than the node
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() < neig_cell.depth());

                    if draw_side {
                        debug_assert!(edge_neigs.max_depth_neig >= cell.depth());

                        // draw the NW edge
                        edges.put(Ordinal::SW, n.compute_n_seg(Ordinal::SW));
                    }
                } else {
                    // draw the NW edge because there are no neig along that edge
                    edges.put(Ordinal::SW, n.compute_n_seg(Ordinal::SW));
                }

                if let Some(edge_neigs) = &n.edge_neigs[Ordinal::SE as u8 as usize] {
                    // if the smallest neig for this edge is smaller than self
                    let _smallest_neig_depth = edge_neigs.max_depth_neig;

                    let first_neig_idx = edge_neigs.neig_idx[0];
                    let neig_cell = &g.nodes()[first_neig_idx].cell;

                    let draw_side =
                            // the current node has several (smaller) neig
                            edge_neigs.neig_idx.len() > 1
                            // or it has only one neig and if so
                            // we draw the side either if the node's idx is < to the neig's idx
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() == neig_cell.depth()
                                && neig_cell.idx() > cell.idx())
                            // or we draw the side if the neig is smaller than the node
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() < neig_cell.depth());

                    if draw_side {
                        debug_assert!(edge_neigs.max_depth_neig >= cell.depth());

                        edges.put(Ordinal::SE, n.compute_n_seg(Ordinal::SE));
                    }
                } else {
                    // draw the NW edge because there are no neig along that edge
                    edges.put(Ordinal::SE, n.compute_n_seg(Ordinal::SE));
                }

                if let Some(edge_neigs) = &n.edge_neigs[Ordinal::NE as u8 as usize] {
                    // if the smallest neig for this edge is smaller than self
                    let _smallest_neig_depth = edge_neigs.max_depth_neig;

                    let first_neig_idx = edge_neigs.neig_idx[0];
                    let neig_cell = &g.nodes()[first_neig_idx].cell;

                    let draw_side =
                            // the current node has several (smaller) neig
                            edge_neigs.neig_idx.len() > 1
                            // or it has only one neig and if so
                            // we draw the side either if the node's idx is < to the neig's idx
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() == neig_cell.depth()
                                && neig_cell.idx() > cell.idx())
                            // or we draw the side if the neig is smaller than the node
                            || (edge_neigs.neig_idx.len() == 1
                                && cell.depth() < neig_cell.depth());

                    if draw_side {
                        debug_assert!(edge_neigs.max_depth_neig >= cell.depth());

                        // draw the NW edge
                        edges.put(Ordinal::NE, n.compute_n_seg(Ordinal::NE));
                    }
                } else {
                    // draw the NE edge because there are no neig along that edge
                    edges.put(Ordinal::NE, n.compute_n_seg(Ordinal::NE));
                }

                /*let delta_depth = (3 - (cell.depth() as usize)).max(0) as u8;

                cell.get_children_cells(delta_depth).map(move |child_cell| {
                    let mut edges = OrdinalMap::new();
                    edges.put(Ordinal::NW, 1);
                    edges.put(Ordinal::SW, 1);
                    edges.put(Ordinal::SE, 1);
                    edges.put(Ordinal::NE, 1);
                    Node {
                        vertices: child_cell.path_along_sides(&edges),
                        cell: child_cell,
                    }
                })*/

                if cell.depth() < 2 {
                    /*let max_depth = crate::math::utils::log_2_unchecked(n_seg_nw)
                        .max(crate::math::utils::log_2_unchecked(n_seg_se))
                        .max(crate::math::utils::log_2_unchecked(n_seg_ne))
                        .max(crate::math::utils::log_2_unchecked(n_seg_sw))
                        + cell.depth() as u32;
                    let n_seg = if max_depth > 3 {
                        1 << (max_depth - 3)
                    } else {
                        1
                    };*/

                    cell.get_children_cells(2 - cell.depth())
                        .map(|child_cell| {
                            let mut edges = OrdinalMap::new();
                            edges.put(Ordinal::NW, 1);
                            edges.put(Ordinal::NE, 1);
                            edges.put(Ordinal::SW, 1);
                            edges.put(Ordinal::SE, 1);

                            Node {
                                vertices: child_cell.path_along_sides(&edges),
                                cell: child_cell,
                            }
                        })
                        .collect()
                } else {
                    vec![Node {
                        vertices: cell.path_along_sides(&edges),
                        cell,
                    }]
                }
            })
            .collect()
    }
}
*/
