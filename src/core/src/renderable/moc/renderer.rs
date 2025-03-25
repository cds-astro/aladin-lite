use crate::{healpix::coverage::HEALPixCoverage, CameraViewPort, ShaderManager};
use al_core::WebGlContext;
use wasm_bindgen::JsValue;

use super::hierarchy::MOCHierarchy;

use al_api::coo_system::CooSystem;

use al_api::moc::MOCOptions;

pub struct MOCRenderer {
    mocs: Vec<MOCHierarchy>,
    cfgs: Vec<MOCOptions>,
    gl: WebGlContext,
}

use crate::ProjectionType;

impl MOCRenderer {
    pub fn new(gl: &WebGlContext) -> Result<Self, JsValue> {
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

        Ok(Self {
            gl: gl.clone(),
            mocs,
            cfgs,
        })
    }

    pub fn push_back(
        &mut self,
        moc: HEALPixCoverage,
        cfg: MOCOptions,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) {
        self.mocs
            .push(MOCHierarchy::from_full_res_moc(self.gl.clone(), moc, &cfg));
        self.cfgs.push(cfg);

        camera.register_view_frame(CooSystem::ICRS, proj);
        //self.layers.push(key);
    }

    pub fn get_hpx_coverage(&self, moc_uuid: &str) -> Option<&HEALPixCoverage> {
        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == moc_uuid) {
            Some(&self.mocs[idx].get_full_moc())
        } else {
            None
        }
    }

    pub fn remove(
        &mut self,
        moc_uuid: &str,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) -> Option<MOCOptions> {
        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == moc_uuid) {
            self.mocs.remove(idx);
            camera.unregister_view_frame(CooSystem::ICRS, proj);

            Some(self.cfgs.remove(idx))
        } else {
            None
        }
    }

    pub fn set_options(
        &mut self,
        options: MOCOptions,
    ) -> Option<MOCOptions> {
        let name = options.get_uuid();

        if let Some(idx) = self.cfgs.iter().position(|cfg| cfg.get_uuid() == name) {
            let old_cfg = self.cfgs[idx].clone();
            self.mocs[idx].set_options(&options);
            self.cfgs[idx] = options;

            //let _ = self.draw(camera, projection, shaders);

            Some(old_cfg)
        } else {
            // the cfg has not been found
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mocs.is_empty()
    }

    pub fn draw(
        &mut self,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        if !self.is_empty() {
            for (hmoc, cfg) in self.mocs.iter_mut().zip(self.cfgs.iter()) {
                if cfg.show {
                    let moc = hmoc.select_moc_from_view(camera);
                    moc.draw(camera, proj, shaders)?;
                }
            }
        }

        Ok(())
    }
}
