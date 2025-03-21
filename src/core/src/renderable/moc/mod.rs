pub mod hierarchy;
pub mod renderer;
pub use renderer::MOCRenderer;

use crate::camera::CameraViewPort;
use crate::healpix::coverage::HEALPixCoverage;
use crate::math::projection::ProjectionType;
use crate::renderable::WebGl2RenderingContext;
use crate::shader::ShaderManager;
use al_api::moc::MOCOptions;

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
    pub(super) fn new(gl: WebGlContext, moc: HEALPixCoverage, cfg: &MOCOptions) -> Self {
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

    pub fn set_options(&mut self, cfg: &MOCOptions, gl: WebGlContext) {
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

        self.inner = inner;
    }

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

        Self {
            vao,
            gl,
            mode,
        }
    }

    fn vertices_in_view<'a>(
        &self,
        moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
    ) -> impl Iterator<Item = [(f64, f64); 4]> + 'a {
        let view_moc = camera.get_cov(CooSystem::FK5J2000);

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
                    .overlapped_by_iter(&camera.get_cov(CooSystem::FK5J2000))
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

                let j20002view = CooSystem::FK5J2000.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let j20002world = view2world * j20002view;

                crate::shader::get_shader(
                    &self.gl,
                    shaders,
                    "line_inst_lonlat.vert",
                    "line_base.frag",
                )?
                .bind(&self.gl)
                .attach_uniforms_from(camera)
                .attach_uniform("u_2world", &j20002world)
                .attach_uniform("u_color", &color)
                .attach_uniform("u_width", &(camera.get_width()))
                .attach_uniform("u_height", &(camera.get_height()))
                .attach_uniform("u_thickness", &thickness)
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

                let j20002view = CooSystem::FK5J2000.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let j20002world = view2world * j20002view;

                crate::shader::get_shader(
                    &self.gl,
                    shaders,
                    "line_inst_lonlat.vert",
                    "line_base.frag",
                )?
                .bind(&self.gl)
                .attach_uniforms_from(camera)
                .attach_uniform("u_2world", &j20002world)
                .attach_uniform("u_color", &color)
                .attach_uniform("u_width", &(camera.get_width()))
                .attach_uniform("u_height", &(camera.get_height()))
                .attach_uniform("u_thickness", &thickness)
                .attach_uniform("u_proj", proj)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_instanced_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    num_instances as i32,
                );
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
                            off_idx + 0,
                            off_idx + 2,
                            off_idx + 1,
                            off_idx + 0,
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

                let j20002view = CooSystem::FK5J2000.to(camera.get_coo_system());
                let view2world = camera.get_m2w();
                let j20002world = view2world * j20002view;
                
                self.gl.enable(WebGl2RenderingContext::CULL_FACE);

                crate::shader::get_shader(&self.gl, shaders, "moc_base.vert", "moc_base.frag")?
                    .bind(&self.gl)
                    .attach_uniforms_from(camera)
                    .attach_uniform("u_2world", &j20002world)
                    .attach_uniform("u_color", &color)
                    .attach_uniform("u_proj", proj)
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(num_idx),
                        WebGl2RenderingContext::UNSIGNED_INT,
                        0,
                    );

                self.gl.disable(WebGl2RenderingContext::CULL_FACE);
            }
        }

        Ok(())
    }

    fn compute_edge_paths_iter<'a>(
        &self,
        moc: &'a HEALPixCoverage,
        camera: &'a mut CameraViewPort,
    ) -> impl Iterator<Item = f32> + 'a {
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
