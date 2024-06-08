use crate::{
    healpix::{cell::HEALPixCell, coverage::HEALPixCoverage},
    math::angle::Angle,
    CameraViewPort, ShaderManager,
};
mod graph;
pub mod mode;

pub mod hierarchy;
pub mod moc;

use crate::renderable::line::RasterizedLineRenderer;

use wasm_bindgen::JsValue;

use hierarchy::MOCHierarchy;

use al_api::coo_system::CooSystem;

use al_api::moc::MOC as Cfg;

pub struct MOCRenderer {
    mocs: Vec<MOCHierarchy>,
    cfgs: Vec<Cfg>,
}

/*
use cgmath::Vector2;
use super::utils::triangle::Triangle;
fn is_crossing_projection(
    cell: &HEALPixCell,
    camera: &CameraViewPort,
    projection: &ProjectionType,
) -> bool {
    let vertices = cell
        .vertices()
        .iter()
        .filter_map(|(lon, lat)| {
            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            let xyzw =
                crate::coosys::apply_coo_system(CooSystem::ICRS, camera.get_coo_system(), &xyzw);

            projection
                .model_to_normalized_device_space(&xyzw, camera)
                .map(|v| [v.x as f32, v.y as f32])
        })
        .collect::<Vec<_>>();

    let cell_inside = vertices.len() == 4;

    if cell_inside {
        let c0 = &vertices[0];
        let c1 = &vertices[1];
        let c2 = &vertices[2];
        let c3 = &vertices[3];

        let t0 = Triangle::new(c0, c1, c2);
        let t2 = Triangle::new(c2, c3, c0);

        t0.is_invalid(camera) || t2.is_invalid(camera)
    } else {
        true
    }
}

use al_api::cell::HEALPixCellProjeted;
fn rasterize_hpx_cell(
    cell: &HEALPixCell,
    n_segment_by_side: usize,
    camera: &CameraViewPort,
    idx_off: &mut u32,
    proj: &ProjectionType,
) -> Option<(Vec<f32>, Vec<u32>)> {
    let n_vertices_per_segment = n_segment_by_side + 1;

    let vertices = cell
        .grid(n_segment_by_side as u32)
        .iter()
        .filter_map(|(lon, lat)| {
            let xyzw = crate::math::lonlat::radec_to_xyzw(Angle(*lon), Angle(*lat));
            let xyzw =
                crate::coosys::apply_coo_system(CooSystem::ICRS, camera.get_coo_system(), &xyzw);

            proj.model_to_normalized_device_space(&xyzw, camera)
                .map(|v| [v.x as f32, v.y as f32])
        })
        .flatten()
        .collect::<Vec<_>>();

    let cell_inside = vertices.len() == 2 * (n_segment_by_side + 1) * (n_segment_by_side + 1);

    if cell_inside {
        // Generate the iterator: idx_off + 1, idx_off + 1, .., idx_off + 4*n_segment - 1, idx_off + 4*n_segment - 1
        let mut indices = Vec::with_capacity(n_segment_by_side * n_segment_by_side * 6);
        let num_vertices = (n_segment_by_side + 1) * (n_segment_by_side + 1);

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

                let c0 = crate::math::projection::ndc_to_screen_space(
                    &Vector2::new(vertices[2 * idx_0] as f64, vertices[2 * idx_0 + 1] as f64),
                    camera,
                );
                let c1 = crate::math::projection::ndc_to_screen_space(
                    &Vector2::new(vertices[2 * idx_1] as f64, vertices[2 * idx_1 + 1] as f64),
                    camera,
                );
                let c2 = crate::math::projection::ndc_to_screen_space(
                    &Vector2::new(vertices[2 * idx_2] as f64, vertices[2 * idx_2 + 1] as f64),
                    camera,
                );
                let c3 = crate::math::projection::ndc_to_screen_space(
                    &Vector2::new(vertices[2 * idx_3] as f64, vertices[2 * idx_3 + 1] as f64),
                    camera,
                );

                let first_tri_ccw = !crate::math::vector::ccw_tri(&c0, &c1, &c2);
                let second_tri_ccw = !crate::math::vector::ccw_tri(&c1, &c3, &c2);

                if invalid_tri(first_tri_ccw, longitude_reversed)
                    || invalid_tri(second_tri_ccw, longitude_reversed)
                {
                    return None;
                }

                let vx = [c0.x, c1.x, c2.x, c3.x];
                let vy = [c0.y, c1.y, c2.y, c3.y];

                let projeted_cell = HEALPixCellProjeted {
                    ipix: cell.idx(),
                    vx,
                    vy,
                };

                crate::camera::view_hpx_cells::project(projeted_cell, camera, proj)?;

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
}*/

use crate::ProjectionType;

use super::line;
impl MOCRenderer {
    pub fn new() -> Result<Self, JsValue> {
        // layout (location = 0) in vec2 ndc_pos;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];

        //let vertices = vec![];
        /*let position = vec![];
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
        */

        let mocs = Vec::new();
        let cfgs = Vec::new();

        Ok(Self { mocs, cfgs })
    }

    pub fn push_back(
        &mut self,
        moc: HEALPixCoverage,
        cfg: Cfg,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) {
        self.mocs.push(MOCHierarchy::from_full_res_moc(moc, &cfg));
        self.cfgs.push(cfg);

        camera.register_view_frame(CooSystem::ICRS, proj);
        //self.layers.push(key);
    }

    pub fn get_hpx_coverage(&self, cfg: &Cfg) -> Option<&HEALPixCoverage> {
        let name = cfg.get_uuid();

        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == name) {
            Some(&self.mocs[idx].get_full_moc())
        } else {
            None
        }
    }

    pub fn remove(
        &mut self,
        cfg: &Cfg,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) -> Option<Cfg> {
        let name = cfg.get_uuid();

        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == name) {
            self.mocs.remove(idx);
            camera.unregister_view_frame(CooSystem::ICRS, proj);

            Some(self.cfgs.remove(idx))
        } else {
            None
        }
    }

    pub fn set_cfg(
        &mut self,
        cfg: Cfg,
        camera: &mut CameraViewPort,
        projection: &ProjectionType,
        line_renderer: &mut RasterizedLineRenderer,
    ) -> Option<Cfg> {
        let name = cfg.get_uuid();

        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == name) {
            let old_cfg = self.cfgs[idx].clone();
            self.cfgs[idx] = cfg;

            self.update(camera, projection, line_renderer);

            Some(old_cfg)
        } else {
            // the cfg has not been found
            None
        }
    }

    /*pub fn get(&self, cfg: &Cfg) -> Option<&HEALPixCoverage> {
        let key = cfg.get_uuid();
        self.mocs.get(key).map(|coverage| coverage.get_full_moc())
    }*/

    fn update(
        &mut self,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        line_renderer: &mut RasterizedLineRenderer,
    ) {
        for (hmoc, cfg) in self.mocs.iter_mut().zip(self.cfgs.iter()) {
            if cfg.show {
                let moc = hmoc.select_moc_from_view(camera);
                moc.draw(camera, proj, line_renderer);
            }
        }

        /*self.vao.bind_for_update()
        .update_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&self.position),
        )
        .update_element_array(
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData::<u32>(&self.indices),
        );*/
    }

    pub fn is_empty(&self) -> bool {
        self.cfgs.is_empty()
    }

    pub fn draw(
        &mut self,
        _shaders: &mut ShaderManager,
        camera: &mut CameraViewPort,
        projection: &ProjectionType,
        line_renderer: &mut RasterizedLineRenderer,
    ) {
        if self.is_empty() {
            return;
        }

        self.update(camera, projection, line_renderer);
    }
}
