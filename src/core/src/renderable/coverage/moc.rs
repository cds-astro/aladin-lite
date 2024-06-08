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

use moclib::elem::cell::Cell;
use moclib::moc::range::CellAndEdges;
use moclib::moc::RangeMOCIntoIterator;
use moclib::moc::RangeMOCIterator;

use super::mode::Node;

use crate::HEALPixCell;
use cgmath::Vector2;
use wasm_bindgen::prelude::*;

use healpix::compass_point::OrdinalMap;

pub struct MOC {
    pub sky_fraction: f32,
    pub max_order: u8,

    inner: [Option<MOCIntern>; 3],

    pub moc: HEALPixCoverage,
}

impl MOC {
    pub(super) fn new(moc: HEALPixCoverage, cfg: &Cfg) -> Self {
        let sky_fraction = moc.sky_fraction() as f32;
        let max_order = moc.depth_max();

        let inner = [
            if cfg.perimeter {
                // draw only perimeter
                Some(MOCIntern::new(RenderModeType::Perimeter {
                    thickness: cfg.line_width,
                    color: cfg.color,
                }))
            } else {
                None
            },
            if cfg.filled {
                // change color
                let fill_color = cfg.fill_color;
                // draw the edges
                Some(MOCIntern::new(RenderModeType::Filled { color: fill_color }))
            } else {
                None
            },
            if cfg.edges {
                Some(MOCIntern::new(RenderModeType::Edge {
                    thickness: cfg.line_width,
                    color: cfg.color,
                }))
            } else {
                None
            },
        ];

        Self {
            inner,
            max_order,
            sky_fraction,
            moc,
        }
    }

    /*pub(super) fn cell_indices_in_view(&mut self, camera: &mut CameraViewPort) {
        for render in &mut self.inner {
            if let Some(render) = render.as_mut() {
                render.cell_indices_in_view(camera);
            }
        }
    }*/

    /*pub(super) fn num_cells_in_view(&self, camera: &mut CameraViewPort) -> usize {
        self.inner
            .iter()
            .filter_map(|moc| moc.as_ref())
            .map(|moc| moc.num_cells_in_view(camera))
            .sum()
    }*/

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
                render.draw(&self.moc, camera, proj, rasterizer)
            }
        }
    }
}

struct MOCIntern {
    // HEALPix index vector
    // Used for fast HEALPix cell retrieval
    //hpx_idx_vec: IdxVec,

    // Node indices in view
    //indices: Vec<Range<usize>>,
    mode: RenderModeType,
}

#[derive(Clone)]
pub enum RenderModeType {
    Perimeter { thickness: f32, color: ColorRGBA },
    Edge { thickness: f32, color: ColorRGBA },
    Filled { color: ColorRGBA },
}
use healpix::compass_point::Ordinal;
impl MOCIntern {
    fn new(mode: RenderModeType) -> Self {
        /*let hpx_idx_vec =
        IdxVec::from_hpx_cells((&moc.0).into_range_moc_iter().cells().flat_map(|cell| {
            let cell = HEALPixCell(cell.depth, cell.idx);
            let dd = if 3 >= cell.depth() {
                3 - cell.depth()
            } else {
                0
            };
            cell.get_tile_cells(dd)
        }));
        */
        Self {
            //nodes,
            //moc,
            //hpx_idx_vec,
            //indices: vec![],
            mode,
        }
    }

    /*fn cell_indices_in_view(&mut self, moc: &HEALPixCoverage, camera: &mut CameraViewPort) {
        // Cache it for several reuse during the same frame
        let view_depth = camera.get_texture_depth();
        let cells_iter = camera.get_hpx_cells(view_depth, CooSystem::ICRS);

        if moc.is_empty() {
            self.indices = vec![0..0];
            return;
        }

        /*let indices: Vec<_> = if view_depth > 7 {
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
        };*/

        let indices = cells_iter
            .map(|cell| self.hpx_idx_vec.get_item_indices_inside_hpx_cell(&cell))
            .collect();
        let indices = crate::utils::merge_overlapping_intervals(indices);
        self.indices = indices;
    }*/

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

    /*fn num_cells_in_view(&self, _camera: &CameraViewPort) -> usize {
        self.indices
            .iter()
            .map(|range| range.end - range.start)
            .sum()
    }*/

    /*fn cells_in_view<'a>(&'a self, _camera: &CameraViewPort) -> impl Iterator<Item = Node> {
        let nodes = &self.nodes;
        self.indices
            .iter()
            .map(move |indices| nodes[indices.start..indices.end].iter())
            .flatten()
    }*/

    fn vertices_in_view<'a>(
        &self,
        view_moc: &'a HEALPixCoverage,
        moc: &'a HEALPixCoverage,
        camera: &mut CameraViewPort,
    ) -> impl Iterator<Item = [(f64, f64); 4]> + 'a {
        //self.cells_in_view(camera)
        //    .filter_map(move |node| node.vertices.as_ref())
        moc.overlapped_by_iter(&view_moc)
            .cells()
            .flat_map(|cell| {
                let Cell { idx, depth } = cell;
                let cell = HEALPixCell(depth, idx);
                let dd = if 3 >= cell.depth() {
                    3 - cell.depth()
                } else {
                    0
                };
                cell.get_tile_cells(dd)
            })
            .map(|hpx_cell| hpx_cell.vertices())

        //.map(|Cell { idx, depth }| HEALPixCell(depth, idx).vertices())
    }

    fn draw(
        &self,
        moc: &HEALPixCoverage,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        rasterizer: &mut RasterizedLineRenderer,
    ) {
        let view_depth = camera.get_texture_depth();

        let view_moc = HEALPixCoverage::from_fixed_hpx_cells(
            view_depth,
            camera
                .get_hpx_cells(view_depth, CooSystem::ICRS)
                .map(|c| c.idx()),
            None,
        );

        crate::Time::measure_perf("rasterize moc", move || {
            match self.mode {
                RenderModeType::Perimeter { thickness, color } => {
                    let moc_in_view =
                        HEALPixCoverage(moc.overlapped_by_iter(&view_moc).into_range_moc());
                    rasterizer.add_stroke_paths(
                        self.compute_perimeter_paths_iter(&moc_in_view, &view_moc, camera, proj),
                        thickness,
                        &color,
                        &super::line::Style::None,
                    );
                }
                RenderModeType::Edge { thickness, color } => {
                    rasterizer.add_stroke_paths(
                        self.compute_edge_paths_iter(moc, &view_moc, camera, proj),
                        thickness,
                        &color,
                        &super::line::Style::None,
                    );
                }
                RenderModeType::Filled { color } => {
                    rasterizer.add_fill_paths(
                        self.compute_edge_paths_iter(moc, &view_moc, camera, proj),
                        &color,
                    );
                }
            }
            Ok(())
        });
    }

    fn compute_edge_paths_iter<'a>(
        &self,
        moc: &'a HEALPixCoverage,
        view_moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
        proj: &'a ProjectionType,
    ) -> impl Iterator<Item = PathVertices<[[f32; 2]; 5]>> + 'a {
        let camera_coosys = camera.get_coo_system();
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

        self.vertices_in_view(view_moc, moc, camera)
            .filter_map(move |cell_vertices| {
                let mut ndc: [[f32; 2]; 5] =
                    [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]];

                let vertices = cell_vertices;

                for i in 0..4 {
                    let line_vertices = vertices[i];

                    //for k in 0..line_vertices.len() {
                    let (lon, lat) = line_vertices;

                    let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(lon), Angle(lat));
                    let xyzw =
                        crate::coosys::apply_coo_system(CooSystem::ICRS, camera_coosys, &xyzw);

                    if let Some(p) = proj.model_to_normalized_device_space(&xyzw, camera) {
                        if i > 0 && crossing_edges_testing {
                            let mag2 = crate::math::vector::dist2(
                                crate::math::projection::ndc_to_clip_space(&p, camera).as_ref(),
                                crate::math::projection::ndc_to_clip_space(
                                    &Vector2::new(ndc[i - 1][0] as f64, ndc[i - 1][1] as f64),
                                    camera,
                                )
                                .as_ref(),
                            );
                            //al_core::info!("mag", i, mag2);
                            if mag2 > 0.1 {
                                return None;
                            }
                        }

                        ndc[i] = [p.x as f32, p.y as f32];
                    } else {
                        return None;
                    }

                    //ndc[i] = [xyzw.x as f32, xyzw.y as f32];
                    //ndc[i] = [lon as f32, lat as f32];
                }

                ndc[4] = ndc[0].clone();

                Some(PathVertices { vertices: ndc })
            })
    }

    fn compute_perimeter_paths_iter<'a>(
        &self,
        moc: &'a HEALPixCoverage,
        view_moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
        proj: &'a ProjectionType,
    ) -> impl Iterator<Item = PathVertices<Vec<[f32; 2]>>> + 'a {
        let camera_coosys = camera.get_coo_system();
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

        moc.border_elementary_edges()
            .filter_map(|CellAndEdges { uniq, edges }| {
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

                cell.path_along_sides(&map)
            })
            .filter_map(move |CellVertices { vertices }| {
                let mut ndc = Vec::<[f32; 2]>::with_capacity(vertices.len() * 2);

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

                Some(PathVertices { vertices: ndc })
            })
    }
}
