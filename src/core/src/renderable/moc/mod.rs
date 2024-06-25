mod graph;
mod mode;

pub mod hierarchy;
pub mod renderer;
pub use renderer::MOCRenderer;

use crate::camera::CameraViewPort;
use crate::healpix::coverage::HEALPixCoverage;
use crate::math::projection::ProjectionType;
use crate::renderable::WebGl2RenderingContext;
use crate::shader::ShaderManager;
use al_api::moc::MOC as Cfg;

use wasm_bindgen::JsValue;

use crate::WebGlContext;
use al_core::VertexArrayObject;

use al_api::color::ColorRGBA;
use al_api::coo_system::CooSystem;

use moclib::elem::cell::Cell;
use moclib::moc::range::CellAndEdges;

use moclib::moc::RangeMOCIterator;

use crate::HEALPixCell;

use al_core::VecData;

pub struct MOC {
    pub sky_fraction: f32,
    pub max_order: u8,

    inner: [Option<MOCIntern>; 3],

    pub moc: HEALPixCoverage,
}

impl MOC {
    pub(super) fn new(gl: WebGlContext, moc: HEALPixCoverage, cfg: &Cfg) -> Self {
        let sky_fraction = moc.sky_fraction() as f32;
        let max_order = moc.depth_max();

        let inner = [
            if cfg.perimeter {
                // draw only perimeter
                Some(MOCIntern::new(
                    gl.clone(),
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
                    gl.clone(),
                    RenderModeType::Filled { color: fill_color },
                ))
            } else {
                None
            },
            if cfg.edges {
                Some(MOCIntern::new(
                    gl,
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
        &mut self,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        for render in &mut self.inner {
            if let Some(render) = render.as_mut() {
                render.draw(&self.moc, camera, proj, shaders)?
            }
        }

        Ok(())
    }
}

struct MOCIntern {
    // HEALPix index vector
    // Used for fast HEALPix cell retrieval
    //hpx_idx_vec: IdxVec,

    // Node indices in view
    //indices: Vec<Range<usize>>,
    mode: RenderModeType,

    gl: WebGlContext,
    vao: VertexArrayObject,
}

#[derive(Clone)]
pub enum RenderModeType {
    Perimeter { thickness: f32, color: ColorRGBA },
    Edge { thickness: f32, color: ColorRGBA },
    Filled { color: ColorRGBA },
}
impl MOCIntern {
    fn new(gl: WebGlContext, mode: RenderModeType) -> Self {
        let lonlat = vec![];
        let vertices = [
            0_f32, -0.5_f32, 1_f32, -0.5_f32, 1_f32, 0.5_f32, 0_f32, 0.5_f32,
        ];
        let indices = [0_u16, 1_u16, 2_u16, 0_u16, 2_u16, 3_u16];
        let vao = match mode {
            RenderModeType::Perimeter { .. } | RenderModeType::Edge { .. } => {
                let mut vao = VertexArrayObject::new(&gl);
                vao.bind_for_update()
                    // Store the cartesian position of the center of the source in the a instanced VBO
                    .add_instanced_array_buffer(
                        "lonlat",
                        4 * std::mem::size_of::<f32>(),
                        &[2, 2],
                        &[0, 2 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        VecData::<f32>(&lonlat),
                    )
                    .add_array_buffer(
                        "vertices",
                        2 * std::mem::size_of::<f32>(),
                        &[2],
                        &[0],
                        WebGl2RenderingContext::STATIC_DRAW,
                        &vertices as &[f32],
                    )
                    // Set the element buffer
                    .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, &indices as &[u16])
                    // Unbind the buffer
                    .unbind();

                vao
            }
            RenderModeType::Filled { .. } => {
                let mut vao = VertexArrayObject::new(&gl);
                let indices = vec![];
                vao.bind_for_update()
                    // Store the cartesian position of the center of the source in the a instanced VBO
                    .add_array_buffer(
                        "lonlat",
                        2 * std::mem::size_of::<f32>(),
                        &[2],
                        &[0],
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        VecData::<f32>(&lonlat),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        VecData::<u32>(&indices),
                    )
                    // Unbind the buffer
                    .unbind();

                vao
            }
        };

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
            vao,
            gl,
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
        moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
    ) -> impl Iterator<Item = [(f64, f64); 4]> + 'a {
        let view_moc = camera.get_cov(CooSystem::ICRS);
        //self.cells_in_view(camera)
        //    .filter_map(move |node| node.vertices.as_ref())
        moc.overlapped_by_iter(view_moc)
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
    }

    fn draw(
        &mut self,
        moc: &HEALPixCoverage,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        //let _ = crate::Time::measure_perf("rasterize moc", move || {
        match self.mode {
            RenderModeType::Perimeter { thickness, color } => {
                let moc_in_view = moc
                    .overlapped_by_iter(&camera.get_cov(CooSystem::ICRS))
                    .into_range_moc();
                let perimeter_vertices_iter = moc_in_view
                    .border_elementary_edges()
                    .filter_map(|CellAndEdges { uniq, edges }| {
                        if edges.is_empty() {
                            None
                        } else {
                            let mut paths = vec![];

                            let c = Cell::from_uniq_hpx(uniq);
                            let cell = HEALPixCell(c.depth, c.idx);
                            let v = cell.vertices();

                            if edges.get(moclib::moc::range::Ordinal::SE) {
                                paths.extend([
                                    v[0].0 as f32,
                                    v[0].1 as f32,
                                    v[1].0 as f32,
                                    v[1].1 as f32,
                                ]);
                            }
                            if edges.get(moclib::moc::range::Ordinal::NE) {
                                paths.extend([
                                    v[1].0 as f32,
                                    v[1].1 as f32,
                                    v[2].0 as f32,
                                    v[2].1 as f32,
                                ]);
                            }
                            if edges.get(moclib::moc::range::Ordinal::NW) {
                                paths.extend([
                                    v[2].0 as f32,
                                    v[2].1 as f32,
                                    v[3].0 as f32,
                                    v[3].1 as f32,
                                ]);
                            }
                            if edges.get(moclib::moc::range::Ordinal::SW) {
                                paths.extend([
                                    v[3].0 as f32,
                                    v[3].1 as f32,
                                    v[0].0 as f32,
                                    v[0].1 as f32,
                                ])
                            }

                            Some(paths)
                        }
                    })
                    .flatten();

                let mut buf: Vec<_> = vec![];
                buf.extend(perimeter_vertices_iter);

                self.vao.bind_for_update().update_instanced_array(
                    "lonlat",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&buf),
                );

                let num_instances = buf.len() / 4;

                let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let icrs2world = view2world * icrs2view;

                crate::shader::get_shader(
                    &self.gl,
                    shaders,
                    "line_inst_lonlat.vert",
                    "line_base.frag",
                )?
                .bind(&self.gl)
                .attach_uniforms_from(camera)
                .attach_uniform("u_2world", &icrs2world)
                .attach_uniform("u_color", &color)
                .attach_uniform("u_width", &thickness)
                .attach_uniform("u_proj", proj)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_instanced_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    num_instances as i32,
                );
            }
            RenderModeType::Edge { thickness, color } => {
                let mut buf: Vec<_> = vec![];
                buf.extend(self.compute_edge_paths_iter(moc, camera));
                //let mut buf = self.compute_edge_paths_iter(moc, camera).collect();

                self.vao.bind_for_update().update_instanced_array(
                    "lonlat",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&buf),
                );

                let num_instances = buf.len() / 4;

                let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let icrs2world = view2world * icrs2view;

                crate::shader::get_shader(
                    &self.gl,
                    shaders,
                    "line_inst_lonlat.vert",
                    "line_base.frag",
                )?
                .bind(&self.gl)
                .attach_uniforms_from(camera)
                .attach_uniform("u_2world", &icrs2world)
                .attach_uniform("u_color", &color)
                .attach_uniform("u_width", &thickness)
                .attach_uniform("u_proj", proj)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_instanced_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    num_instances as i32,
                );

                /*rasterizer.add_stroke_paths(
                    ,
                    thickness,
                    &color,
                    &super::line::Style::None,
                    CooSpace::LonLat,
                );*/
            }
            RenderModeType::Filled { color } => {
                let mut off_idx = 0;
                let mut indices: Vec<u32> = vec![];
                let vertices = self
                    .vertices_in_view(moc, camera)
                    .map(|v| {
                        let vertices = [
                            v[0].0 as f32,
                            v[0].1 as f32,
                            v[1].0 as f32,
                            v[1].1 as f32,
                            v[2].0 as f32,
                            v[2].1 as f32,
                            v[3].0 as f32,
                            v[3].1 as f32,
                        ];

                        indices.extend_from_slice(&[
                            off_idx + 1,
                            off_idx + 0,
                            off_idx + 3,
                            off_idx + 1,
                            off_idx + 3,
                            off_idx + 2,
                        ]);

                        off_idx += 4;

                        vertices
                    })
                    .flatten()
                    .collect();

                let num_idx = indices.len() as i32;

                self.vao
                    .bind_for_update()
                    .update_array(
                        "lonlat",
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        VecData(&vertices),
                    )
                    .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&indices));

                let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let icrs2world = view2world * icrs2view;

                self.gl.enable(WebGl2RenderingContext::BLEND);

                crate::shader::get_shader(&self.gl, shaders, "moc_base.vert", "moc_base.frag")?
                    .bind(&self.gl)
                    .attach_uniforms_from(camera)
                    .attach_uniform("u_2world", &icrs2world)
                    .attach_uniform("u_color", &color)
                    .attach_uniform("u_proj", proj)
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(num_idx),
                        WebGl2RenderingContext::UNSIGNED_INT,
                        0,
                    );

                self.gl.disable(WebGl2RenderingContext::BLEND);
            }
        }
        Ok(())
        //});
    }

    fn compute_edge_paths_iter<'a>(
        &self,
        moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
    ) -> impl Iterator<Item = f32> + 'a {
        /*self.vertices_in_view(view_moc, moc, camera)
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
        })*/
        self.vertices_in_view(moc, camera)
            .map(|v| {
                let vertices = [
                    v[0].0 as f32,
                    v[0].1 as f32,
                    v[1].0 as f32,
                    v[1].1 as f32,
                    v[1].0 as f32,
                    v[1].1 as f32,
                    v[2].0 as f32,
                    v[2].1 as f32,
                    v[2].0 as f32,
                    v[2].1 as f32,
                    v[3].0 as f32,
                    v[3].1 as f32,
                    v[3].0 as f32,
                    v[3].1 as f32,
                    v[0].0 as f32,
                    v[0].1 as f32,
                ];

                vertices
            })
            .flatten()
    }
}
