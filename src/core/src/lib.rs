//! Rust core WebGL entry point
//!
//! This is the starting point of the Rust core backend
//! of Aladin Lite v3. It features the code that handles:
//!
//! - The call to the WebGL API, the GPU shaders, and the
//!   definition of Vertex/Index buffer to send to the GPU.
//! - The HEALPix tiles retrieving heuristic.
//! - All the spherical geometry maths for the computation
//!   of the equatorial/galactic coordinates grid.
extern crate console_error_panic_hook;
extern crate fitsrs;
extern crate itertools_num;
extern crate num;
extern crate num_traits;
extern crate rand;
extern crate serde_derive;
extern crate serde_json;
extern crate task_async_executor;
use std::panic;

#[macro_use]
mod utils;

use wasm_bindgen::prelude::*;

mod app;
mod async_task;
mod buffer;
mod camera;
mod cdshealpix;
mod color;
mod coo_conversion;
mod core;
mod healpix_cell;
mod hips;
mod image_fmt;
mod line;
mod math;
mod projection_type;
mod renderable;
mod resources;
mod rotation;
mod shader;
mod shaders;
mod sphere_geometry;
mod time;
mod transfert_function;
mod webgl_ctx;

use crate::{
    camera::CameraViewPort,
    coo_conversion::CooSystem,
    hips::{HiPSColor, HiPSFormat, HiPSProperties, SimpleHiPS},
    image_fmt::FormatImageType,
    math::LonLatT,
    renderable::{image_survey::ImageSurveys, projection::Projection, Angle, ArcDeg},
    resources::Resources,
    shader::{Shader, ShaderManager},
    shaders::Colormap,
    time::DeltaTime,
    webgl_ctx::WebGl2Context,
};

use app::App;
use cgmath::{Vector2, Vector4};
use projection_type::ProjectionType;

#[wasm_bindgen]
pub struct WebClient {
    // The app
    app: App,
    projection: ProjectionType,

    // The time between the previous and the current
    // frame
    dt: DeltaTime,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
use crate::shader::FileSrc;
use crate::transfert_function::TransferFunction;

use crate::color::Color;
#[wasm_bindgen]
impl WebClient {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new(shaders: &JsValue, resources: &JsValue) -> Result<WebClient, JsValue> {
        let shaders = shaders.into_serde::<Vec<FileSrc>>().unwrap();
        let resources = resources.into_serde::<Resources>().unwrap();
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let gl = WebGl2Context::new();

        let shaders = ShaderManager::new(&gl, shaders).unwrap();
        let app = App::new(&gl, shaders, resources)?;

        //let appconfig = AppConfig::Ortho(app);
        let dt = DeltaTime::zero();
        let projection = ProjectionType::Ortho;

        let webclient = WebClient {
            app,
            projection,

            dt,
        };

        Ok(webclient)
    }

    /// Main update method
    /// The force parameter ensures to force the update of some elements
    /// even if the camera has not moved
    pub fn update(&mut self, dt: f32, force: bool) -> Result<(), JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get back the
        // world coordinates of the center of projection in (ra, dec)
        self.projection.update(
            &mut self.app,
            // Time of the previous frame rendering
            self.dt,
            // Force the update of some elements:
            // i.e. the grid
            force,
        )?;

        Ok(())
    }

    /// Resize the window
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.projection.resize(&mut self.app, width, height);

        Ok(())
    }

    /// Update our WebGL Water application.
    pub fn render(&mut self, force: bool) -> Result<(), JsValue> {
        self.projection.render(&mut self.app, force)?;

        Ok(())
    }

    /// Change the current projection of the HiPS
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(&mut self, name: String) -> Result<(), JsValue> {
        self.projection.set_projection(&mut self.app, name)?;

        Ok(())
    }

    /// Change the current projection of the HiPS
    #[wasm_bindgen(js_name = setLongitudeReversed)]
    pub fn set_longitude_reversed(&mut self, reversed: bool) -> Result<(), JsValue> {
        self.projection
            .set_longitude_reversed(&mut self.app, reversed);

        Ok(())
    }

    /// Image surveys

    /// Check whether the app is ready
    ///
    /// Aladin Lite is in a good state when the root tiles of the
    /// HiPS chosen have all been retrieved and accessible for the GPU
    ///
    /// The javascript can change the HiPSes only if aladin lite is ready
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&mut self) -> Result<bool, JsValue> {
        self.app.is_ready()
    }

    #[wasm_bindgen(js_name = setImageSurveys)]
    pub fn set_image_surveys(&mut self, surveys: Vec<JsValue>) -> Result<(), JsValue> {
        // Deserialize the survey objects that compose the survey
        let surveys: Result<Vec<SimpleHiPS>, JsValue> = surveys
            .into_iter()
            .map(|h| {
                h.into_serde()
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>();
        let surveys = surveys?;
        self.app.set_image_surveys(surveys)?;

        Ok(())
    }

    /// Move a layer forward
    ///
    /// # Panics
    ///
    /// If the layer specified is not found
    #[wasm_bindgen(js_name = moveImageSurveysLayerForward)]
    pub fn move_image_surveys_layer_forward(&mut self, layer_name: &str) -> Result<(), JsValue> {
        self.app.move_image_surveys_layer_forward(layer_name)
    }

    #[wasm_bindgen(js_name = setOpacityLayer)]
    pub fn set_opacity_layer(&mut self, opacity: f32, layer_name: &str) -> Result<(), JsValue> {
        self.app.set_opacity_layer(layer_name, opacity)?;

        Ok(())
    }

    /// Grid

    /// Change grid color
    #[wasm_bindgen(js_name = setGridColor)]
    pub fn set_grid_color(
        &mut self,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
    ) -> Result<(), JsValue> {
        let color = Color::new(red, green, blue, alpha);
        self.projection.set_grid_color(&mut self.app, color);

        Ok(())
    }

    /// Enable the draw of the grid
    #[wasm_bindgen(js_name = enableGrid)]
    pub fn enable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.enable_grid(&mut self.app);

        Ok(())
    }

    /// Disable the draw of the grid
    #[wasm_bindgen(js_name = disableGrid)]
    pub fn disable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.disable_grid(&mut self.app);

        Ok(())
    }

    #[wasm_bindgen(js_name = hideGridLabels)]
    pub fn hide_grid_labels(&mut self) -> Result<(), JsValue> {
        self.projection.hide_grid_labels(&mut self.app);

        Ok(())
    }

    #[wasm_bindgen(js_name = showGridLabels)]
    pub fn show_grid_labels(&mut self) -> Result<(), JsValue> {
        self.projection.show_grid_labels(&mut self.app);

        Ok(())
    }

    /// ICRS in J2000 to galactic conversion functions

    #[wasm_bindgen(js_name = cooSystem)]
    pub fn get_coo_system(&self) -> Result<CooSystem, JsValue> {
        Ok(self.app.system)
    }

    #[wasm_bindgen(js_name = setCooSystem)]
    pub fn set_coo_system(&mut self, coo_system: CooSystem) -> Result<(), JsValue> {
        self.projection.set_coo_system(&mut self.app, coo_system);

        Ok(())
    }

    #[wasm_bindgen(js_name = J20002Gal)]
    pub fn j2000_to_gal(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let gal_lonlat = coo_conversion::to_galactic(lonlat);

        Ok(Some(Box::new([
            gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI),
            gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI),
        ])))
    }

    #[wasm_bindgen(js_name = Gal2J2000)]
    pub fn gal_to_j2000(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let icrsj2000_lonlat = coo_conversion::to_icrs_j2000(lonlat);

        Ok(Some(Box::new([
            icrsj2000_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI),
            icrsj2000_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI),
        ])))
    }

    /// Camera moving functions

    #[wasm_bindgen(js_name = getFieldOfView)]
    pub fn get_fov(&self) -> Result<f64, JsValue> {
        let fov = self.app.get_fov();
        Ok(fov)
    }

    /// Set directly the field of view (for pinch zooming)
    #[wasm_bindgen(js_name = setFieldOfView)]
    pub fn set_fov(&mut self, fov: f64) -> Result<(), JsValue> {
        //let fov = fov as f32;
        let fov = ArcDeg(fov).into();

        self.projection.start_zooming_to(&mut self.app, fov);
        //self.projection.set_fov(&mut self.app, ArcDeg(fov).into());

        Ok(())
    }

    #[wasm_bindgen(js_name = setRotationAroundCenter)]
    pub fn rotate_around_center(&mut self, theta: f64) -> Result<(), JsValue> {
        let theta = ArcDeg(theta);
        self.projection.rotate_around_center(&mut self.app, theta);

        Ok(())
    }

    #[wasm_bindgen(js_name = getRotationAroundCenter)]
    pub fn get_rotation_around_center(&mut self) -> Result<f64, JsValue> {
        let theta = self.app.get_rotation_around_center();

        Ok(theta.0 * 360.0 / (2.0 * std::f64::consts::PI))
    }

    #[wasm_bindgen(js_name = getMaxFieldOfView)]
    pub fn get_max_fov(&mut self) -> f64 {
        self.projection.get_max_fov(&mut self.app)
    }

    #[wasm_bindgen(js_name = getClipZoomFactor)]
    pub fn get_clip_zoom_factor(&self) -> Result<f64, JsValue> {
        Ok(self.app.get_clip_zoom_factor())
    }

    /// Set directly the center position
    #[wasm_bindgen(js_name = setCenter)]
    pub fn set_center(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        //let location = self.app.system.to_icrs_j2000(lonlat);

        self.projection.set_center(&mut self.app, location);

        Ok(())
    }

    /// Set directly the center position
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f64]>, JsValue> {
        let center = self.projection.get_center(&self.app);

        let lon_deg: ArcDeg<f64> = center.lon().into();
        let lat_deg: ArcDeg<f64> = center.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    /// Initiate a finite state machine that will move to a specific location
    #[wasm_bindgen(js_name = moveToLocation)]
    pub fn start_moving_to(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        // The core works in ICRS_J2000 coordinates
        // Check if the user is giving galactic coordinates
        // so that we can convert them to icrs
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        //let location = self.app.system.to_icrs_j2000(location);

        self.projection.start_moving_to(&mut self.app, location);

        Ok(())
    }

    #[wasm_bindgen(js_name = goFromTo)]
    pub fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) -> Result<(), JsValue> {
        self.projection
            .go_from_to(&mut self.app, s1x, s1y, s2x, s2y);

        Ok(())
    }

    /// World to screen projection
    ///
    /// Coordinates must be given in ICRS J2000
    /// They will be converted accordingly to the current frame of Aladin Lite
    #[wasm_bindgen(js_name = worldToScreen)]
    pub fn world_to_screen(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        if let Some(screen_pos) = self.projection.world_to_screen(&self.app, &lonlat)? {
            Ok(Some(Box::new([screen_pos.x, screen_pos.y])))
        } else {
            Ok(None)
        }
    }

    #[wasm_bindgen(js_name = worldToScreenVec)]
    pub fn world_to_screen_vec(&self, sources: Vec<JsValue>) -> Result<Box<[f64]>, JsValue> {
        let screen_positions = self.projection.world_to_screen_vec(&self.app, &sources)?;
        Ok(screen_positions.into_boxed_slice())
    }

    #[wasm_bindgen(js_name = screenToWorld)]
    pub fn screen_to_world(&self, pos_x: f64, pos_y: f64) -> Option<Box<[f64]>> {
        if let Some(lonlat) = self
            .projection
            .screen_to_world(&self.app, &Vector2::new(pos_x, pos_y))
        {
            let lon_deg: ArcDeg<f64> = lonlat.lon().into();
            let lat_deg: ArcDeg<f64> = lonlat.lat().into();

            Some(Box::new([lon_deg.0, lat_deg.0]))
        } else {
            None
        }
    }

    /// Tell the backend when the left mouse button has been
    /// released. This is useful for beginning inerting
    #[wasm_bindgen(js_name = releaseLeftButtonMouse)]
    pub fn release_left_button_mouse(&mut self) -> Result<(), JsValue> {
        self.app.release_left_button_mouse();

        Ok(())
    }

    /// Tell the backend when the left mouse button has been pressed
    #[wasm_bindgen(js_name = pressLeftMouseButton)]
    pub fn press_left_button_mouse(&mut self) -> Result<(), JsValue> {
        self.app.press_left_button_mouse();

        Ok(())
    }

    #[wasm_bindgen(js_name = registerWheelEvent)]
    pub fn wheel_event_callback(&mut self, delta: f64) -> Result<(), JsValue> {
        let zooming = delta > 0.0;
        let cur_fov = self.app.get_fov();
        let target_fov = if zooming {
            let fov = cur_fov / 1.10;
            // max fov: 2e-10 deg = 2e-10*3600*10e6 µas = 0.72µas
            fov.max(2e-10 as f64)
        } else {
            let fov = cur_fov * 1.10;
            fov.min(1000.0)
        };

        let target_fov = ArcDeg(target_fov).into();
        self.projection.start_zooming_to(&mut self.app, target_fov);

        Ok(())
    }

    /// Catalogs
    #[wasm_bindgen(js_name = addCatalog)]
    pub fn add_catalog(
        &mut self,
        name_catalog: String,
        data: JsValue,
        colormap: String,
    ) -> Result<(), JsValue> {
        self.projection
            .add_catalog(&mut self.app, name_catalog, data, colormap);

        Ok(())
    }

    #[wasm_bindgen(js_name = isCatalogLoaded)]
    pub fn is_catalog_loaded(&mut self) -> Result<bool, JsValue> {
        let cat_loaded = self.app.is_catalog_loaded();
        Ok(cat_loaded)
    }

    #[wasm_bindgen(js_name = setCatalogColormap)]
    pub fn set_catalog_colormap(
        &mut self,
        name_catalog: String,
        colormap: String,
    ) -> Result<(), JsValue> {
        let colormap: Colormap = colormap.into();
        self.projection
            .set_catalog_colormap(&mut self.app, name_catalog, colormap)?;

        Ok(())
    }

    /// Set the heatmap global opacity
    #[wasm_bindgen(js_name = setCatalogOpacity)]
    pub fn set_heatmap_opacity(
        &mut self,
        name_catalog: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        self.projection
            .set_heatmap_opacity(&mut self.app, name_catalog, opacity)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setCatalogKernelStrength)]
    pub fn set_kernel_strength(
        &mut self,
        name_catalog: String,
        strength: f32,
    ) -> Result<(), JsValue> {
        self.projection
            .set_kernel_strength(&mut self.app, name_catalog, strength)?;

        Ok(())
    }

    /// Utilities

    #[wasm_bindgen(js_name = projectLine)]
    pub fn project_line(
        &self,
        lon1: f64,
        lat1: f64,
        lon2: f64,
        lat2: f64,
    ) -> Result<Box<[f64]>, JsValue> {
        let vertices = self
            .projection
            .project_line(&self.app, lon1, lat1, lon2, lat2);

        let vertices = vertices
            .into_iter()
            .map(|v| vec![v.x, v.y])
            .flatten()
            .collect::<Vec<_>>();

        Ok(vertices.into_boxed_slice())
    }

    #[wasm_bindgen(js_name = getAvailableColormapList)]
    pub fn get_available_colormap_list(&self) -> Result<JsValue, JsValue> {
        let colormaps = Colormap::get_list_available_colormaps()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        JsValue::from_serde(&colormaps).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

mod tests {
    #[test]
    fn lambert_wm1() {
        let d = 0.1;
        let x = -d / std::f32::consts::E;

        let w = super::math::lambert_wm1(x);

        println!("{}", w);
    }
}
