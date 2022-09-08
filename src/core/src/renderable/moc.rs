use moclib::moc::CellMOCIntoIterator;

use crate::{healpix::{
    coverage::HEALPixCoverage,
    cell::HEALPixCell
}, camera, Projection, shader::ShaderId, math::angle::Angle, CameraViewPort, ShaderManager};
use al_api::color::Color;
use al_core::{WebGlContext, VertexArrayObject, VecData};
use moclib::{moc::{RangeMOCIterator, RangeMOCIntoIterator}, elem::cell::Cell};
use std::{borrow::Cow, f32::consts::E};

use web_sys::WebGl2RenderingContext;

pub struct MOC {
    vao: VertexArrayObject,
    num_indices: usize,
    position: Vec<f32>,
    indices: Vec<u16>,
    gl: WebGlContext,
}  
use cgmath::Vector2;
use al_core::{log, info, inforec};
impl MOC {
    pub fn new(gl: &WebGlContext) -> Self {
        let mut vao = VertexArrayObject::new(gl);

        // layout (location = 0) in vec2 lonlat;
        // layout (location = 1) in vec3 position;
        // layout (location = 2) in vec3 uv_start;
        // layout (location = 3) in vec3 uv_end;
        // layout (location = 4) in float time_tile_received;
        // layout (location = 5) in float m0;
        // layout (location = 6) in float m1;
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

        let num_indices = 0;
        let gl = gl.clone();
        Self {
            position,
            indices,
            num_indices,
            vao,
            gl
        }
    }

    pub fn update<P: Projection>(&mut self, mocs: &[HEALPixCoverage], camera: &CameraViewPort) {
        self.indices.clear();
        self.position.clear();

        let mut idx_off = 0;

        for moc in mocs {
            let depth_max = moc.depth();

            let positions_moc = (&moc.0).into_range_moc_iter()
                .cells()
                .filter_map(|Cell { depth, idx, .. }| {
                    let delta_depth = depth_max - depth;
                    let n_segment_by_side = (1 << delta_depth) as usize;
                    let vertices = HEALPixCell(depth, idx)
                        .path_along_cell_edge(n_segment_by_side as u32)
                        .into_iter()
                        .filter_map(|(lon, lat)| {
                            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
                            P::model_to_ndc_space(&xyzw, camera)
                                .and_then(|v| Some([v.x as f32, v.y as f32]))
                        })
                        .flatten()
                        .collect::<Vec<_>>();
                    
                    let cell_inside = vertices.len() == 2*4*n_segment_by_side;
                    
                    if cell_inside {
                        let cell_cross_screen = crate::math::vector::ccw_tri(
                            &Vector2::new(vertices[0], vertices[1]),
                            &Vector2::new(vertices[2*n_segment_by_side], vertices[2*n_segment_by_side + 1]),
                            &Vector2::new(vertices[2*2*n_segment_by_side], vertices[2*2*n_segment_by_side + 1]),
                        ) || crate::math::vector::ccw_tri(
                            &Vector2::new(vertices[2*2*n_segment_by_side], vertices[2*2*n_segment_by_side + 1]),
                            &Vector2::new(vertices[3*2*n_segment_by_side], vertices[3*2*n_segment_by_side + 1]),
                            &Vector2::new(vertices[0], vertices[1]),
                        );
    
                        if !cell_cross_screen {
                            // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
                            // it has (3 - 1 + 1) * 2 = 6
                            let num_vertices = (4 * n_segment_by_side) as u16;
                            let it = (2..2*num_vertices).map(|idx| idx / 2 + idx_off);
                            let mut indices_cell = vec![idx_off as u16];
                            indices_cell.extend(it);
                            indices_cell.push(idx_off);
    
                            self.indices.extend(indices_cell);
    
                            idx_off += num_vertices;
                            //indices.extend([idx_off, idx_off + 1, idx_off + 1, idx_off + 2, idx_off + 2, idx_off + 3, idx_off + 3, idx_off]);
                            //idx_off += 4*n_segment_by_side;
        
                            Some(vertices)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<_>>();

            self.position.extend(&positions_moc);
        }
        
        self.num_indices = self.indices.len();
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
    
    pub fn draw(
        &self,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &Color,
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
        let shader = shader.bind(&self.gl);
        shader
            .attach_uniforms_from(camera)
            .attach_uniform("color", color)
            .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(
                    WebGl2RenderingContext::LINES,
                    Some(self.num_indices as i32),
                    WebGl2RenderingContext::UNSIGNED_SHORT,
                    0
                );
    }
}