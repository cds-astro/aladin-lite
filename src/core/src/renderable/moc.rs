use crate::{healpix::{
    coverage::HEALPixCoverage,
    cell::HEALPixCell
}, Projection, shader::ShaderId, math::angle::Angle, CameraViewPort, ShaderManager, survey::view::HEALPixCellProjection};
use al_core::{WebGlContext, VertexArrayObject, VecData};
use moclib::{moc::{RangeMOCIterator, RangeMOCIntoIterator}, elem::cell::Cell};
use std::{borrow::Cow, collections::HashMap};
use web_sys::WebGl2RenderingContext;

use al_api::coo_system::CooSystem;

type MOCIdx = String;

pub struct MOC {
    vao: VertexArrayObject,
    num_indices: Vec<usize>,
    first_idx: Vec<usize>,
    position: Vec<f32>,
    indices: Vec<u32>,

    mocs: HashMap<MOCIdx, HierarchicalHpxCoverage>,

    adaptative_mocs: HashMap<MOCIdx, Option<HEALPixCoverage>>,
    params: HashMap<MOCIdx, al_api::moc::MOC>,

    layers: Vec<MOCIdx>,
    view: HEALPixCellsInView,

    gl: WebGlContext,
}

use crate::survey::view::HEALPixCellsInView;
use cgmath::Vector2;

fn path_along_edge<P: Projection>(cell: &HEALPixCell, n_segment_by_side: usize, camera: &CameraViewPort, idx_off: &mut u32) -> Option<(Vec<f32>, Vec<u32>)> {
    let vertices = cell
        .path_along_cell_edge(n_segment_by_side as u32)
        .into_iter()
        .filter_map(|(lon, lat)| {
            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            let xyzw = crate::coosys::apply_coo_system(&CooSystem::ICRSJ2000, camera.get_system(), &xyzw);
            
            P::model_to_ndc_space(&xyzw, camera)
                .and_then(|v| Some([v.x as f32, v.y as f32]))
        })
        .flatten()
        .collect::<Vec<_>>();

    let cell_inside = vertices.len() == 2*4*n_segment_by_side;

    let invalid_tri = |tri_ccw: bool, reversed_longitude: bool| -> bool {
        (!reversed_longitude && !tri_ccw) || (reversed_longitude && tri_ccw)
    };
    let reversed_longitude = camera.get_longitude_reversed();

    if cell_inside {
        let c0 = Vector2::new(vertices[0], vertices[1]);
        let c1 = Vector2::new(vertices[2*n_segment_by_side], vertices[2*n_segment_by_side + 1]);
        let c2 = Vector2::new(vertices[2*2*n_segment_by_side], vertices[2*2*n_segment_by_side + 1]);
        let c3 = Vector2::new(vertices[3*2*n_segment_by_side], vertices[3*2*n_segment_by_side + 1]);

        let first_tri_ccw = !crate::math::vector::ccw_tri(&c0, &c1, &c2);
        let second_tri_ccw = !crate::math::vector::ccw_tri(&c1, &c2, &c3);
        let third_tri_ccw = !crate::math::vector::ccw_tri(&c2, &c3, &c0);
        let fourth_tri_ccw = !crate::math::vector::ccw_tri(&c3, &c0, &c1);

        let invalid_cell = invalid_tri(first_tri_ccw, reversed_longitude) || invalid_tri(second_tri_ccw, reversed_longitude) || invalid_tri(third_tri_ccw, reversed_longitude) || invalid_tri(fourth_tri_ccw, reversed_longitude);

        if !invalid_cell {
            // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
            let num_vertices = 4 * n_segment_by_side as u32;
            let indices = std::iter::once(*idx_off as u32)
                .chain((2..2*num_vertices).map(|idx| idx / 2 + *idx_off))
                .chain(std::iter::once(*idx_off as u32))
                .collect();
            *idx_off += num_vertices;

            Some((vertices, indices))
        } else {
            None
        }
    } else {
        None
    }
}
use al_api::cell::HEALPixCellProjeted;
fn rasterize_hpx_cell<P: Projection + HEALPixCellProjection>(cell: &HEALPixCell, n_segment_by_side: usize, camera: &CameraViewPort, idx_off: &mut u32) -> Option<(Vec<f32>, Vec<u32>)> {
    let n_vertices_per_segment = n_segment_by_side + 1;

    let vertices = cell
        .grid(n_segment_by_side as u32)
        .into_iter()
        .filter_map(|(lon, lat)| {
            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            let xyzw = crate::coosys::apply_coo_system(&CooSystem::ICRSJ2000, camera.get_system(), &xyzw);

            P::model_to_ndc_space(&xyzw, camera)
                .and_then(|v| Some([v.x as f32, v.y as f32]))
        })
        .flatten()
        .collect::<Vec<_>>();

    let cell_inside = vertices.len() == 2*(n_segment_by_side+1)*(n_segment_by_side+1);

    if cell_inside {
        // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
        let mut indices = Vec::with_capacity(n_segment_by_side * n_segment_by_side * 6);
        let num_vertices = (n_segment_by_side+1)*(n_segment_by_side+1);

        let longitude_reversed = camera.get_longitude_reversed();
        let invalid_tri = |tri_ccw: bool, reversed_longitude: bool| -> bool {
            (!reversed_longitude && !tri_ccw) || (reversed_longitude && tri_ccw)
        };

        for i in 0..n_segment_by_side {
            for j in 0..n_segment_by_side {
                let idx_0 = j + i * n_vertices_per_segment;
                let idx_1 = j + 1 + i * n_vertices_per_segment;
                let idx_2 = j + (i + 1) * n_vertices_per_segment;
                let idx_3 = j + 1 + (i + 1) * n_vertices_per_segment;

                let c0 = Vector2::new(vertices[2*idx_0], vertices[2*idx_0 + 1]);
                let c1 = Vector2::new(vertices[2*idx_1], vertices[2*idx_1 + 1]);
                let c2 = Vector2::new(vertices[2*idx_2], vertices[2*idx_2 + 1]);
                let c3 = Vector2::new(vertices[2*idx_3], vertices[2*idx_3 + 1]);

                let first_tri_ccw = crate::math::vector::ccw_tri(&c0, &c1, &c2);
                let second_tri_ccw = crate::math::vector::ccw_tri(&c1, &c3, &c2);

                if invalid_tri(first_tri_ccw, longitude_reversed) || invalid_tri(second_tri_ccw, longitude_reversed) {
                    return None;
                }

                /*let vx = [vertices[2*idx_0] as f64, vertices[2*idx_1] as f64, vertices[2*idx_2] as f64, vertices[2*idx_3] as f64];
                let vy = [vertices[2*idx_0 + 1] as f64, vertices[2*idx_1 + 1] as f64, vertices[2*idx_2 + 1] as f64, vertices[2*idx_3 + 1] as f64];

                let projeted_cell = HEALPixCellProjeted {
                    ipix: cell.idx(),
                    vx,
                    vy
                };
                if P::project(projeted_cell, &camera).is_none() {
                    return None;
                }*/

                indices.push(*idx_off + idx_0 as u32);
                indices.push(*idx_off + idx_1 as u32);
                indices.push(*idx_off + idx_2 as u32);

                indices.push(*idx_off + idx_1 as u32);
                indices.push(*idx_off + idx_3 as u32);
                indices.push(*idx_off + idx_2 as u32);
            }
        }

        *idx_off += num_vertices as u32;

        Some((vertices, indices))
    } else {
        None
    }
}

struct HierarchicalHpxCoverage {
    full_moc: HEALPixCoverage,
    partially_degraded_moc: HEALPixCoverage,
}

impl HierarchicalHpxCoverage {
    fn new(full_moc: HEALPixCoverage) -> Self {
        let partially_degraded_moc = HEALPixCoverage(full_moc.degraded(full_moc.depth_max() >> 1));

        Self {
            full_moc,
            partially_degraded_moc
        }
    }

    fn get(&self, depth: u8) -> &HEALPixCoverage {
        if depth <= self.partially_degraded_moc.depth_max() {
            &self.partially_degraded_moc
        } else {
            &self.full_moc
        }
    }

    fn get_full_moc(&self) -> &HEALPixCoverage {
        &self.full_moc
    }
}

impl MOC {
    pub fn new(gl: &WebGlContext) -> Self {
        let mut vao = VertexArrayObject::new(gl);

        // layout (location = 0) in vec2 ndc_pos;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];

        //let vertices = vec![];
        let position = vec![];
        let indices = vec![];
        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer_single(
                2,
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&indices),
            )
            .unbind();
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            .add_array_buffer(
                2,
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&indices),
            )
            .unbind();

        let num_indices = vec![0];
        let first_idx = vec![0];

        let gl = gl.clone();
        let mocs = HashMap::new();
        let adaptative_mocs = HashMap::new();
        let layers = vec![];
        let params = HashMap::new();
        let view = HEALPixCellsInView::new();
        
        Self {
            position,
            indices,

            mocs,
            adaptative_mocs,
            params,

            layers,

            num_indices,
            first_idx,

            vao,
            gl,

            view,
        }
    }

    pub fn reset_frame(&mut self) {
        self.view.reset_frame();
    }

    fn recompute_draw_mocs(&mut self, camera: &CameraViewPort) {
        let view_depth = self.view.get_depth();
        let depth = view_depth + 5;

        let fov_moc = crate::survey::view::compute_view_coverage(camera, view_depth, &CooSystem::ICRSJ2000);
        self.adaptative_mocs = self.layers.iter()
            .map(|layer| {
                let params = self.params.get(layer).unwrap();
                let coverage = self.mocs.get(layer).unwrap();

                let moc = if !params.is_showing() {
                    None
                } else {
                    let partially_degraded_moc = coverage.get(depth);
                    let moc = HEALPixCoverage(fov_moc.intersection(&partially_degraded_moc).degraded(depth));
                    Some(moc)
                };

                (layer.clone(), moc)
            }).collect();
        
    }

    pub fn insert<P: Projection + HEALPixCellProjection>(&mut self, moc: HEALPixCoverage, params: al_api::moc::MOC, camera: &CameraViewPort) {
        let key = params.get_uuid().to_string();

        self.mocs.insert(key.clone(), HierarchicalHpxCoverage::new(moc));
        self.params.insert(key.clone(), params);
        self.layers.push(key);

        self.recompute_draw_mocs(camera);
        self.update_buffers::<P>(camera);
        // Compute or retrieve the mocs to render
    }

    pub fn remove(&mut self, params: &al_api::moc::MOC, camera: &CameraViewPort) -> Option<al_api::moc::MOC> {
        let key = params.get_uuid();

        self.mocs.remove(key);
        let moc = self.params.remove(key);

        if let Some(index) = self.layers.iter().position(|x| x == key) {
            self.layers.remove(index);
            self.num_indices.remove(index);
            self.first_idx.remove(index);

            self.recompute_draw_mocs(camera);
            moc
        } else {
            None
        }
    }

    pub fn set_params<P: Projection + HEALPixCellProjection>(&mut self, params: al_api::moc::MOC, camera: &CameraViewPort) -> Option<al_api::moc::MOC> {
        let key = params.get_uuid().to_string();
        let old_params = self.params.insert(key, params);

        self.recompute_draw_mocs(camera);
        self.update_buffers::<P>(camera);

        old_params
    }

    pub fn get(&self, params: &al_api::moc::MOC) -> Option<&HEALPixCoverage> {
        let key = params.get_uuid();
        self.mocs.get(key).and_then(|coverage| Some(coverage.get_full_moc()) )
    }

    fn update_buffers<P: Projection + HEALPixCellProjection>(&mut self, camera: &CameraViewPort) {
        self.indices.clear();
        self.position.clear();
        self.num_indices.clear();
        self.first_idx.clear();

        let mut idx_off = 0;

        for layer in self.layers.iter() {
            let moc = self.adaptative_mocs.get(layer).unwrap();
            let params = self.params.get(layer).unwrap();

            if let Some(moc) = moc {
                let depth_max = moc.depth();
                let mut indices_moc = vec![];
                if params.get_opacity() == 1.0 {
                    let positions_moc = (&(moc.0)).into_range_moc_iter()
                        .cells()
                        .filter_map(|Cell { depth, idx, .. }| {
                            let delta_depth = depth_max - depth;
                            let n_segment_by_side = (1 << delta_depth) as usize;
    
                            let cell = HEALPixCell(depth, idx);
                            if let Some((vertices_cell, indices_cell)) = path_along_edge::<P>(
                                &cell,
                                n_segment_by_side,
                                camera,
                                &mut idx_off,
                            ) {
                                // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
                                indices_moc.extend(indices_cell);
    
                                Some(vertices_cell)
                            } else if depth < 3 {
                                let mut vertices = vec![];
    
                                let depth_sub_cell = 3;
                                let delta_depth_sub_cell = depth_max - depth_sub_cell;
                                let n_segment_by_side_sub_cell = (1 << delta_depth_sub_cell) as usize;
    
                                for sub_cell in cell.get_children_cells(3 - depth) {
                                    if let Some((vertices_sub_cell, indices_sub_cell)) = path_along_edge::<P>(
                                        &sub_cell,
                                        n_segment_by_side_sub_cell,
                                        camera,
                                        &mut idx_off
                                    ) {
                                        indices_moc.extend(indices_sub_cell);
                                        vertices.extend(vertices_sub_cell);
                                    }
                                }
    
                                Some(vertices)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect::<Vec<_>>();
    
                    self.first_idx.push(self.indices.len());
                    self.num_indices.push(indices_moc.len());
    
                    self.position.extend(&positions_moc);
                    self.indices.extend(&indices_moc);
                } else {
                    let positions_moc = (&(moc.0)).into_range_moc_iter()
                        .cells()
                        .filter_map(|Cell { depth, idx, .. }| {
                            let delta_depth = (depth_max as i32 - depth as i32).max(0);
                            let n_segment_by_side = (1 << delta_depth) as usize;
    
                            let cell = HEALPixCell(depth, idx);
                            if let Some((vertices_cell, indices_cell)) = rasterize_hpx_cell::<P>(
                                &cell,
                                n_segment_by_side,
                                camera,
                                &mut idx_off,
                            ) {
                                // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
                                indices_moc.extend(indices_cell);
    
                                Some(vertices_cell)
                            } else if depth < 3 {
                                let mut vertices = vec![];
    
                                let depth_sub_cell = 3;
                                let delta_depth_sub_cell = depth_max - depth_sub_cell;
                                let n_segment_by_side_sub_cell = (1 << delta_depth_sub_cell) as usize;
    
                                for sub_cell in cell.get_children_cells(3 - depth) {
                                    if let Some((vertices_sub_cell, indices_sub_cell)) = rasterize_hpx_cell::<P>(
                                        &sub_cell,
                                        n_segment_by_side_sub_cell,
                                        camera,
                                        &mut idx_off
                                    ) {
                                        indices_moc.extend(indices_sub_cell);
                                        vertices.extend(vertices_sub_cell);
                                    }
                                }
    
                                Some(vertices)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect::<Vec<_>>();
    
                    self.first_idx.push(self.indices.len());
                    self.num_indices.push(indices_moc.len());
    
                    self.position.extend(&positions_moc);
                    self.indices.extend(&indices_moc);
                } 
            } else {
                self.first_idx.push(self.indices.len());
                self.num_indices.push(0);
            }
        }

        self.vao.bind_for_update()
            .update_array(
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.position),
            )
            .update_element_array(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&self.indices),
            );
    }

    pub fn update<P: Projection + HEALPixCellProjection>(&mut self, camera: &CameraViewPort) {
        // Compute or retrieve the mocs to render
        let new_depth = crate::survey::view::depth_from_pixels_on_screen(camera, 512);
        self.view.refresh(new_depth, CooSystem::ICRSJ2000, camera);

        if self.view.has_view_changed() {
            self.recompute_draw_mocs(camera);
        }

        self.update_buffers::<P>(camera);
    }
    
    pub fn draw(
        &self,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
    ) {
        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        self.gl.enable(WebGl2RenderingContext::BLEND);

        let shader = shaders
            .get(
                &self.gl,
                &ShaderId(Cow::Borrowed("GridVS_CPU"), Cow::Borrowed("GridFS_CPU")),
            )
            .unwrap();
        let shaderbound = shader.bind(&self.gl);
        for (idx, layer) in self.layers.iter().enumerate() {
            let moc = self.params.get(layer).unwrap();
            //if moc.is_showing() {
                let mode = if moc.get_opacity() == 1.0 {
                    WebGl2RenderingContext::LINES
                } else {
                    WebGl2RenderingContext::TRIANGLES
                };

                let color = moc.get_color();
                shaderbound
                    .attach_uniforms_from(camera)
                    .attach_uniform("color", color)
                    .attach_uniform("opacity", &moc.get_opacity())
                    .bind_vertex_array_object_ref(&self.vao)
                        .draw_elements_with_i32(
                            mode,
                            Some(self.num_indices[idx] as i32),
                            WebGl2RenderingContext::UNSIGNED_INT,
                            (self.first_idx[idx] * std::mem::size_of::<u32>()) as i32
                        );
            //}
        }

        self.gl.disable(WebGl2RenderingContext::BLEND);
    }
}