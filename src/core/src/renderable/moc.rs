use moclib::moc::CellMOCIntoIterator;

use crate::{healpix::{
    coverage::HEALPixCoverage,
    cell::HEALPixCell
}, camera, Projection, shader::ShaderId, math::angle::Angle, CameraViewPort, ShaderManager};
use al_api::color::Color;
use al_core::{WebGlContext, VertexArrayObject, VecData};
use moclib::{moc::{RangeMOCIterator, RangeMOCIntoIterator}, elem::cell::Cell};
use std::{borrow::Cow, collections::HashMap};
use crate::survey::ImageSurveys;
use web_sys::WebGl2RenderingContext;

use al_api::coo_system::CooSystem;

pub struct MOCParams {
    filled: bool,
    color: Color,
}

type MOCIdx = String;

pub struct MOC {
    vao: VertexArrayObject,
    num_indices: Vec<usize>,
    position: Vec<f32>,
    indices: Vec<u16>,

    mocs: HashMap<MOCIdx, HEALPixCoverage>,

    adaptative_mocs: HashMap<MOCIdx, HEALPixCoverage>,
    params: HashMap<MOCIdx, al_api::moc::MOC>,

    layers: Vec<MOCIdx>,

    gl: WebGlContext,
}

use crate::survey::view::HEALPixCellsInView;
use cgmath::Vector2;
use al_core::{log, info, inforec};

fn path_along_edge<P: Projection>(cell: &HEALPixCell, n_segment_by_side: usize, view_frame: &CooSystem, camera: &CameraViewPort) -> Option<Vec<f32>> {
    let vertices = cell
        .path_along_cell_edge(n_segment_by_side as u32)
        .into_iter()
        .filter_map(|(lon, lat)| {
            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            let xyzw = crate::coosys::apply_coo_system(view_frame, camera.get_system(), &xyzw);
            
            P::model_to_ndc_space(&xyzw, camera)
                .and_then(|v| Some([v.x as f32, v.y as f32]))
        })
        .flatten()
        .collect::<Vec<_>>();

    let cell_inside = vertices.len() == 2*4*n_segment_by_side;

    if cell_inside {
        let c0 = Vector2::new(vertices[0], vertices[1]);
        let c1 = Vector2::new(vertices[2*n_segment_by_side], vertices[2*n_segment_by_side + 1]);
        let c2 = Vector2::new(vertices[2*2*n_segment_by_side], vertices[2*2*n_segment_by_side + 1]);
        let c3 = Vector2::new(vertices[3*2*n_segment_by_side], vertices[3*2*n_segment_by_side + 1]);

        let cell_cross_screen = crate::math::vector::ccw_tri(&c0, &c1, &c2) || crate::math::vector::ccw_tri(&c1, &c2, &c3) || crate::math::vector::ccw_tri(&c2, &c3, &c0) || crate::math::vector::ccw_tri(&c3, &c0, &c1);

        if !cell_cross_screen {
            Some(vertices)
        } else {
            None
        }
    } else {
        None
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
                VecData::<u16>(&indices),
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
                VecData::<u16>(&indices),
            )
            .unbind();

        let num_indices = vec![0];
        let gl = gl.clone();
        let mocs = HashMap::new();
        let adaptative_mocs = HashMap::new();
        let layers = vec![];
        let params = HashMap::new();
        Self {
            position,
            indices,

            mocs,
            adaptative_mocs,
            params,

            layers,

            num_indices,
            vao,
            gl
        }
    }

    fn recompute_draw_mocs(&mut self, view: &HEALPixCellsInView) {
        let depth = view.get_depth() + 5;
        let view_moc = view.get_coverage();

        self.adaptative_mocs = self.mocs.iter()
            .map(|(key, moc)| {
                let render_moc = view_moc.intersection(&moc).degraded(depth);
                (key.clone(), crate::healpix::coverage::HEALPixCoverage(render_moc))
            }).collect::<HashMap<_, _>>();
    }

    pub fn insert(&mut self, moc: HEALPixCoverage, params: al_api::moc::MOC, surveys: &ImageSurveys) {
        let key = params.get_uuid().to_string();
        self.mocs.insert(key.clone(), moc);
        self.params.insert(key.clone(), params);

        self.layers.push(key);
        let num_indices = if let Some(&off_idx) = self.num_indices.last() {
            off_idx
        } else {
            0
        };
        self.num_indices.push(num_indices);

        if let Some(view) = surveys.get_view() {
            self.recompute_draw_mocs(view);
        }
    }

    pub fn remove(&mut self, key: &MOCIdx, surveys: &ImageSurveys) {
        self.mocs.remove(key);
        self.params.remove(key);

        let index = self.layers.iter().position(|x| x == key).unwrap();
        self.layers.remove(index);

        if let Some(view) = surveys.get_view() {
            self.recompute_draw_mocs(view);
        }
    }

    pub fn set_params(&mut self, params: al_api::moc::MOC) -> Option<al_api::moc::MOC> {
        let key = params.get_uuid().to_string();
        self.params.insert(key, params)
    }

    pub fn update<P: Projection>(&mut self, surveys: &ImageSurveys, camera: &CameraViewPort) {
        if let Some(view) = surveys.get_view() {
            // Compute or retrieve the mocs to render
            if view.has_view_changed() {
                self.recompute_draw_mocs(view);
            }

            self.indices.clear();
            self.position.clear();
            self.num_indices.clear();

            let mut idx_off = 0;

            let view_frame = view.get_frame();
            for layer in self.layers.iter() {
                let moc = self.adaptative_mocs.get(layer).unwrap();
                let params = self.params.get(layer).unwrap();

                let depth_max = moc.depth();
                let mut indices_moc = vec![];
                let positions_moc = (&(moc.0)).into_range_moc_iter()
                    .cells()
                    .filter_map(|Cell { depth, idx, .. }| {
                        let delta_depth = depth_max - depth;
                        let n_segment_by_side = (1 << delta_depth) as usize;
                        let cell = HEALPixCell(depth, idx);
                        if let Some(vertices) = path_along_edge::<P>(
                            &cell,
                            n_segment_by_side,
                            view_frame,
                            camera
                        ) {
                            // HEALPix projection special case
                            /*//if (this.projection.PROJECTION == ProjectionEnum.HPX) {
                            const triIdxInCollignonZone = ((p) => {
                                const x = ((p.vx / this.catalogCanvas.clientWidth) - 0.5) * this.zoomFactor;
                                const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;

                                const xZone = Math.floor((x + 0.5) * 4);
                                return xZone + 4 * (y > 0.0);
                            });

                            const isInCollignon = ((p) => {
                                const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;

                                return y < -0.25 || y > 0.25;
                            });

                            if (isInCollignon(cornersXYView[0]) && isInCollignon(cornersXYView[1]) && isInCollignon(cornersXYView[2]) && isInCollignon(cornersXYView[3])) {
                                const allVerticesInSameCollignonRegion = (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[1])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[2])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[3]));
                                if (!allVerticesInSameCollignonRegion) {
                                    continue;
                                }
                            }
                            //}*/

                            // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
                            let num_vertices = (4 * n_segment_by_side) as u16;

                            let indices_cell = std::iter::once(idx_off as u16)
                                .chain((2..2*num_vertices).map(|idx| idx / 2 + idx_off))
                                .chain(std::iter::once(idx_off as u16));
                            indices_moc.extend(indices_cell);
                            idx_off += num_vertices;

                            Some(vertices)
                        } else if depth < 3 {
                            let mut vertices = vec![];

                            let depth_sub_cell = 3;
                            let delta_depth_sub_cell = depth_max - depth_sub_cell;
                            let n_segment_by_side_sub_cell = (1 << delta_depth_sub_cell) as usize;
                            let num_vertices = (4 * n_segment_by_side_sub_cell) as u16;

                            for sub_cell in cell.get_children_cells(3 - depth) {
                                if let Some(v) = path_along_edge::<P>(
                                    &sub_cell,
                                    n_segment_by_side_sub_cell,
                                    view_frame,
                                    camera
                                ) {
                                    // HEALPix projection special case
                                    /*//if (this.projection.PROJECTION == ProjectionEnum.HPX) {
                                    const triIdxInCollignonZone = ((p) => {
                                        const x = ((p.vx / this.catalogCanvas.clientWidth) - 0.5) * this.zoomFactor;
                                        const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;
        
                                        const xZone = Math.floor((x + 0.5) * 4);
                                        return xZone + 4 * (y > 0.0);
                                    });
        
                                    const isInCollignon = ((p) => {
                                        const y = ((p.vy / this.catalogCanvas.clientHeight) - 0.5) * this.zoomFactor;
        
                                        return y < -0.25 || y > 0.25;
                                    });
        
                                    if (isInCollignon(cornersXYView[0]) && isInCollignon(cornersXYView[1]) && isInCollignon(cornersXYView[2]) && isInCollignon(cornersXYView[3])) {
                                        const allVerticesInSameCollignonRegion = (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[1])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[2])) && (triIdxInCollignonZone(cornersXYView[0]) == triIdxInCollignonZone(cornersXYView[3]));
                                        if (!allVerticesInSameCollignonRegion) {
                                            continue;
                                        }
                                    }
                                    //}*/
        
                                    // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
        
                                    let indices_cell = std::iter::once(idx_off as u16)
                                        .chain((2..2*num_vertices).map(|idx| idx / 2 + idx_off))
                                        .chain(std::iter::once(idx_off as u16));
                                    indices_moc.extend(indices_cell);
                                    idx_off += num_vertices;
        
                                    vertices.extend(v);
                                }
                            }

                            Some(vertices)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                self.num_indices.push(indices_moc.len());

                self.position.extend(&positions_moc);
                self.indices.extend(&indices_moc);

            }

            self.vao.bind_for_update()
                .update_array(
                    "ndc_pos",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData(&self.position),
                )
                .update_element_array(
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<u16>(&self.indices),
                );
        }
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

        let shader = shaders
            .get(
                &self.gl,
                &ShaderId(Cow::Borrowed("GridVS_CPU"), Cow::Borrowed("GridFS_CPU")),
            )
            .unwrap();
        let shaderbound = shader.bind(&self.gl);
        let mut off_indices = 0;
        for (idx, layer) in self.layers.iter().enumerate() {
            let params = self.params.get(layer).unwrap();

            shaderbound
                .attach_uniforms_from(camera)
                .attach_uniform("color", params.get_color())
                .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::LINES,
                        Some(self.num_indices[idx] as i32),
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        off_indices
                    );
            
            off_indices += self.num_indices[idx] as i32;
        }
            
    }
}