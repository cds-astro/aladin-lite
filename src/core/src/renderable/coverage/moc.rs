use al_api::moc::MOC as Cfg;
use std::cmp::Ordering;

use std::ops::Range;
use std::vec;

use crate::camera::CameraViewPort;
use crate::healpix::cell::CellVertices;
use crate::healpix::coverage::HEALPixCoverage;
use crate::math::projection::ProjectionType;
use crate::renderable::coverage::mode::RenderMode;
use crate::renderable::coverage::Angle;

use crate::renderable::coverage::IdxVec;
use crate::renderable::line::PathVertices;
use crate::renderable::line::RasterizedLineRenderer;
use al_api::color::ColorRGBA;
use al_api::coo_system::CooSystem;

use super::mode::Node;

use cgmath::Vector2;
use wasm_bindgen::prelude::*;

pub struct MOC {
    pub sky_fraction: f32,
    pub max_order: u8,

    inner: [Option<MOCIntern>; 3],
}

impl MOC {
    pub(super) fn new(moc: &HEALPixCoverage, cfg: &Cfg) -> Self {
        let sky_fraction = moc.sky_fraction() as f32;
        let max_order = moc.depth_max();

        let inner = [
            if cfg.perimeter {
                // draw only perimeter
                Some(MOCIntern::new(
                    moc,
                    RenderModeType::Perimeter {
                        thickness: cfg.line_width,
                        color: cfg.color,
                    },
                ))
            } else {
                None
            },
            if cfg.filled {
                // change color
                let fill_color = cfg.fill_color;
                // draw the edges
                Some(MOCIntern::new(
                    moc,
                    RenderModeType::Filled { color: fill_color },
                ))
            } else {
                None
            },
            if cfg.edges {
                Some(MOCIntern::new(
                    moc,
                    RenderModeType::Edge {
                        thickness: cfg.line_width,
                        color: cfg.color,
                    },
                ))
            } else {
                None
            },
        ];

        Self {
            inner,
            max_order,
            sky_fraction,
        }
    }

    pub(super) fn cell_indices_in_view(&mut self, camera: &mut CameraViewPort) {
        for render in &mut self.inner {
            if let Some(render) = render.as_mut() {
                render.cell_indices_in_view(camera);
            }
        }
    }

    pub(super) fn num_cells_in_view(&self, camera: &mut CameraViewPort) -> usize {
        self.inner
            .iter()
            .filter_map(|moc| moc.as_ref())
            .map(|moc| moc.num_cells_in_view(camera))
            .sum()
    }

    /*pub(super) fn num_vertices_in_view(&self, camera: &mut CameraViewPort) -> usize {
        let mut num_vertices = 0;
        for render in &self.0 {
            if let Some(render) = render.as_ref() {
                num_vertices += render.num_vertices_in_view(camera);
            }
        }

        num_vertices
    }*/

    pub fn sky_fraction(&self) -> f32 {
        self.sky_fraction
    }

    pub fn max_order(&self) -> u8 {
        self.max_order
    }

    pub(super) fn draw(
        &self,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        rasterizer: &mut RasterizedLineRenderer,
    ) {
        for render in &self.inner {
            if let Some(render) = render.as_ref() {
                render.draw(camera, proj, rasterizer)
            }
        }
    }
}

struct MOCIntern {
    // HEALPix index vector
    // Used for fast HEALPix cell retrieval
    hpx_idx_vec: IdxVec,

    // Node indices in view
    indices: Vec<Range<usize>>,

    nodes: Vec<Node>,

    mode: RenderModeType,
}

#[derive(Clone)]
pub enum RenderModeType {
    Perimeter { thickness: f32, color: ColorRGBA },
    Edge { thickness: f32, color: ColorRGBA },
    Filled { color: ColorRGBA },
}

impl MOCIntern {
    fn new(moc: &HEALPixCoverage, mode: RenderModeType) -> Self {
        let nodes = match mode {
            RenderModeType::Edge { .. } => super::mode::edge::Edge::build(moc),
            RenderModeType::Filled { .. } => super::mode::filled::Fill::build(moc),
            RenderModeType::Perimeter { .. } => super::mode::perimeter::Perimeter::build(moc),
        };

        let hpx_idx_vec = IdxVec::from_hpx_cells(nodes.iter().map(|n| &n.cell));

        Self {
            nodes,
            hpx_idx_vec,
            indices: vec![],
            mode,
        }
    }

    fn cell_indices_in_view(&mut self, camera: &mut CameraViewPort) {
        // Cache it for several reuse during the same frame
        let view_depth = camera.get_tile_depth();
        let cells_iter = camera.get_hpx_cells(view_depth, CooSystem::ICRS);

        if self.nodes.is_empty() {
            self.indices = vec![0..0];
            return;
        }

        let indices: Vec<_> = if view_depth > 7 {
            // Binary search version, we are using this alternative for retrieving
            // MOC's cells to render for deep fields of view
            let first_cell_rng = &self.nodes[0].cell.z_29_rng();
            let last_cell_rng = &self.nodes[self.nodes.len() - 1].cell.z_29_rng();

            cells_iter
                .filter_map(|cell| {
                    let cell_rng = cell.z_29_rng();
                    // Quick rejection test
                    if cell_rng.end <= first_cell_rng.start || cell_rng.start >= last_cell_rng.end {
                        None
                    } else {
                        let contains_val = |hash_z29: u64| -> Result<usize, usize> {
                            self.nodes.binary_search_by(|node| {
                                let node_cell_rng = node.cell.z_29_rng();

                                if hash_z29 < node_cell_rng.start {
                                    // the node cell range contains hash_z29
                                    Ordering::Greater
                                } else if hash_z29 >= node_cell_rng.end {
                                    Ordering::Less
                                } else {
                                    Ordering::Equal
                                }
                            })
                        };

                        let start_idx = contains_val(cell_rng.start);
                        let end_idx = contains_val(cell_rng.end);

                        let cell_indices = match (start_idx, end_idx) {
                            (Ok(l), Ok(r)) => {
                                if l == r {
                                    l..(r + 1)
                                } else {
                                    l..r
                                }
                            }
                            (Err(l), Ok(r)) => l..r,
                            (Ok(l), Err(r)) => l..r,
                            (Err(l), Err(r)) => l..r,
                        };

                        Some(cell_indices)
                    }
                })
                .collect()
        } else {
            // Index Vector 7 order version
            cells_iter
                .map(|cell| self.hpx_idx_vec.get_item_indices_inside_hpx_cell(&cell))
                .collect()
        };

        let indices = crate::utils::merge_overlapping_intervals(indices);
        self.indices = indices;
    }

    /*fn num_vertices_in_view(&self, camera: &CameraViewPort) -> usize {
        self.cells_in_view(camera)
            .filter_map(|n| n.vertices.as_ref())
            .map(|n_vertices| {
                n_vertices
                    .vertices
                    .iter()
                    .map(|edge| edge.len())
                    .sum::<usize>()
            })
            .sum()
    }*/

    fn num_cells_in_view(&self, _camera: &CameraViewPort) -> usize {
        self.indices
            .iter()
            .map(|range| range.end - range.start)
            .sum()
    }

    fn cells_in_view<'a>(&'a self, _camera: &CameraViewPort) -> impl Iterator<Item = &'a Node> {
        let nodes = &self.nodes;
        self.indices
            .iter()
            .map(move |indices| nodes[indices.start..indices.end].iter())
            .flatten()
    }

    fn vertices_in_view<'a>(
        &'a self,
        camera: &mut CameraViewPort,
        _projection: &ProjectionType,
    ) -> impl Iterator<Item = &'a CellVertices> {
        self.cells_in_view(camera)
            .filter_map(move |node| node.vertices.as_ref())
    }

    fn draw(
        &self,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        rasterizer: &mut RasterizedLineRenderer,
    ) {
        // Determine if the view may lead to crossing edges/triangles
        // This is dependant on the projection used
        let crossing_edges_testing = if proj.is_allsky() {
            let sky_percent_covered = camera.get_cov(CooSystem::ICRS).sky_fraction();
            //al_core::info!("sky covered: ", sky_percent_covered);
            sky_percent_covered > 0.80
        } else {
            // The projection is not allsky.
            false
        };

        let camera_coosys = camera.get_coo_system();

        let paths_iter = self
            .vertices_in_view(camera, proj)
            .filter_map(|cell_vertices| {
                let vertices = &cell_vertices.vertices[..];
                let mut ndc: Vec<[f32; 2]> = vec![];

                for i in 0..vertices.len() {
                    let line_vertices = &vertices[i];

                    for k in 0..line_vertices.len() {
                        let (lon, lat) = line_vertices[k];

                        let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(lon), Angle(lat));
                        let xyzw =
                            crate::coosys::apply_coo_system(CooSystem::ICRS, camera_coosys, &xyzw);

                        if let Some(p) = proj.model_to_normalized_device_space(&xyzw, camera) {
                            if ndc.len() > 0 && crossing_edges_testing {
                                let mag2 = crate::math::vector::dist2(
                                    crate::math::projection::ndc_to_clip_space(&p, camera).as_ref(),
                                    crate::math::projection::ndc_to_clip_space(
                                        &Vector2::new(
                                            ndc[ndc.len() - 1][0] as f64,
                                            ndc[ndc.len() - 1][1] as f64,
                                        ),
                                        camera,
                                    )
                                    .as_ref(),
                                );
                                //al_core::info!("mag", i, mag2);
                                if mag2 > 0.1 {
                                    return None;
                                }
                            }

                            ndc.push([p.x as f32, p.y as f32]);
                        } else {
                            return None;
                        }
                    }
                }

                // Check the last
                if cell_vertices.closed && crossing_edges_testing {
                    let mag2 = crate::math::vector::dist2(
                        crate::math::projection::ndc_to_clip_space(
                            &Vector2::new(ndc[0][0] as f64, ndc[0][1] as f64),
                            camera,
                        )
                        .as_ref(),
                        crate::math::projection::ndc_to_clip_space(
                            &Vector2::new(
                                ndc[ndc.len() - 1][0] as f64,
                                ndc[ndc.len() - 1][1] as f64,
                            ),
                            camera,
                        )
                        .as_ref(),
                    );
                    if mag2 > 0.1 {
                        return None;
                    }
                }

                Some(PathVertices {
                    vertices: ndc,
                    closed: cell_vertices.closed,
                })
            });

        match self.mode {
            RenderModeType::Perimeter { thickness, color }
            | RenderModeType::Edge { thickness, color } => {
                rasterizer.add_stroke_paths(
                    paths_iter,
                    thickness,
                    &color,
                    &super::line::Style::None,
                );
            }
            RenderModeType::Filled { color } => rasterizer.add_fill_paths(paths_iter, &color),
        }
    }
}
