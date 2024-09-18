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
//extern crate console_error_panic_hook;
//extern crate egui;
//extern crate epi;
//extern crate fontdue;
//extern crate image_decoder;
//extern crate itertools_num;
//extern crate num;
//extern crate num_traits;
//use crate::time::Time;
#[cfg(feature = "dbg")]
use std::panic;

#[macro_export]
macro_rules! log {
    // The pattern for a single `eval`
    ($arg:tt) => {
        let s = format!("{:?}", $arg);
        web_sys::console::log_1(&s.into());
    };
}

pub trait Abort {
    type Item;
    fn unwrap_abort(self) -> Self::Item
    where
        Self: Sized;
}

impl<T> Abort for Option<T> {
    type Item = T;

    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Some(t) => t,
            None => process::abort(),
        }
    }
}
impl<T, E> Abort for Result<T, E> {
    type Item = T;

    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Ok(t) => t,
            Err(_) => process::abort(),
        }
    }
}

extern crate serde_json;
#[macro_use]
extern crate enum_dispatch;

#[inline]
pub fn unwrap_abort<T>(o: Option<T>) -> T {
    use std::process;
    match o {
        Some(t) => t,
        None => process::abort(),
    }
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    pub type Catalog;
}

#[macro_use]
mod utils;

use math::projection::*;

use moclib::moc::RangeMOCIntoIterator;
//use votable::votable::VOTableWrapper;
use crate::tile_fetcher::HiPSLocalFiles;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

use crate::math::angle::ToAngle;

mod app;
pub mod async_task;
mod camera;
mod shaders;

mod coosys;
mod downloader;
mod fifo_cache;
mod healpix;
mod inertia;
pub mod math;
pub mod renderable;
mod shader;
mod survey;
mod tile_fetcher;
mod time;

use crate::downloader::request::moc::from_fits_hpx;
use crate::{
    camera::CameraViewPort, healpix::coverage::HEALPixCoverage, math::lonlat::LonLatT,
    shader::ShaderManager, time::DeltaTime,
};
use moclib::deser::fits;
use moclib::deser::fits::MocIdxType;
use moclib::deser::fits::MocQtyType;

use std::io::Cursor;

use al_api::color::{Color, ColorRGBA};
use al_api::coo_system::CooSystem;
use al_api::hips::HiPSProperties;

use al_core::colormap::Colormaps;
use al_core::Colormap;
use al_core::WebGlContext;

use app::App;
use cgmath::{Vector2, Vector4};

use crate::healpix::cell::HEALPixCell;
use math::angle::ArcDeg;
use moclib::{
    moc::{CellMOCIntoIterator, CellMOCIterator, RangeMOCIterator},
    qty::Hpx,
};

#[wasm_bindgen]
pub struct WebClient {
    // The app
    app: App,

    // The time between the previous and the current
    // frame
    dt: DeltaTime,
}

use al_api::hips::ImageMetadata;
use std::convert::TryInto;
#[wasm_bindgen]
impl WebClient {
    /// Create the Aladin Lite webgl backend
    ///
    /// # Arguments
    ///
    /// * `aladin_div_name` - The name of the div where aladin is created
    /// * `shaders` - The list of shader objects containing the GLSL code source
    /// * `resources` - Image resource files
    #[wasm_bindgen(constructor)]
    pub fn new(aladin_div: &HtmlElement, resources: JsValue) -> Result<WebClient, JsValue> {
        #[cfg(feature = "dbg")]
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        //let shaders = serde_wasm_bindgen::from_value(shaders)?;
        let resources = serde_wasm_bindgen::from_value(resources)?;
        let gl = WebGlContext::new(aladin_div)?;

        let shaders = ShaderManager::new().unwrap_abort();

        let app = App::new(&gl, aladin_div, shaders, resources)?;

        let dt = DeltaTime::zero();

        let webclient = WebClient { app, dt };

        Ok(webclient)
    }

    /*#[wasm_bindgen(js_name = setCallbackPositionChanged)]
    pub fn set_callback_position_changed(&mut self, callback: js_sys::Function) {
        self.app.set_callback_position_changed(callback);
    }*/

    #[wasm_bindgen(js_name = isInerting)]
    pub fn is_inerting(&self) -> bool {
        return self.app.is_inerting();
    }

    /// Update the view
    ///
    /// # Arguments
    ///
    /// * `dt` - The time elapsed from the last frame update
    /// * `force` - This parameter ensures to force the update of some elements
    ///   even if the camera has not moved
    ///
    /// # Return
    /// Whether the view is moving or not
    pub fn update(&mut self, dt: f32) -> Result<bool, JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get back the
        // world coordinates of the center of projection in (ra, dec)
        self.app.update(
            // Time of the previous frame rendering
            self.dt,
        )
    }

    /// Resize the window
    ///
    /// # Arguments
    ///
    /// * `width` - The width in pixels of the view
    /// * `height` - The height in pixels of the view
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.app.resize(width, height);
        Ok(())
    }

    /// Set the type of projections
    ///
    /// # Arguments
    ///
    /// * `name` - Can be aitoff, mollweide, arc, sinus, tan or mercator
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(&mut self, projection: &str) -> Result<(), JsValue> {
        match projection {
            // Zenithal
            "TAN" => self
                .app
                .set_projection(ProjectionType::Tan(mapproj::zenithal::tan::Tan::new())), /* Gnomonic projection      */
            "STG" => self
                .app
                .set_projection(ProjectionType::Stg(mapproj::zenithal::stg::Stg::new())), /* Stereographic projection */
            "SIN" => self
                .app
                .set_projection(ProjectionType::Sin(mapproj::zenithal::sin::Sin::new())), /* Orthographic		         */
            "ZEA" => self
                .app
                .set_projection(ProjectionType::Zea(mapproj::zenithal::zea::Zea::new())), /* Equal-area 		         */
            /*"FEYE" => self
                .app
                .set_projection(ProjectionType::Feye(mapproj::zenithal::feye::Feye::new())),
            "AIR" => {
                let air_proj = mapproj::zenithal::air::Air::new();
                //air_proj.set_n_iter(10);
                //air_proj.set_eps(1e-12);
                self.app.set_projection(ProjectionType::Air(air_proj))
            }*/
            //"AZP",
            /*"ARC" => self
                .app
                .set_projection(ProjectionType::Arc(mapproj::zenithal::arc::Arc::new())),
            "NCP" => self
                .app
                .set_projection(ProjectionType::Ncp(mapproj::zenithal::ncp::Ncp::new())),*/
            // Cylindrical
            "MER" => self
                .app
                .set_projection(ProjectionType::Mer(mapproj::cylindrical::mer::Mer::new())),
            /*"CAR" => self
                .app
                .set_projection(ProjectionType::Car(mapproj::cylindrical::car::Car::new())),
            "CEA" => self
                .app
                .set_projection(ProjectionType::Cea(mapproj::cylindrical::cea::Cea::new())),
            "CYP" => self
                .app
                .set_projection(ProjectionType::Cyp(mapproj::cylindrical::cyp::Cyp::new())),*/
            // Pseudo-cylindrical
            "AIT" => self
                .app
                .set_projection(ProjectionType::Ait(mapproj::pseudocyl::ait::Ait::new())),
            /*"PAR" => self
                .app
                .set_projection(ProjectionType::Par(mapproj::pseudocyl::par::Par::new())),
            "SFL" => self
                .app
                .set_projection(ProjectionType::Sfl(mapproj::pseudocyl::sfl::Sfl::new())),*/
            "MOL" => {
                let mut mol_proj = mapproj::pseudocyl::mol::Mol::new();
                mol_proj.set_n_iter(10);
                mol_proj.set_epsilon(1e-12);

                self.app.set_projection(ProjectionType::Mol(mol_proj))
            } // Conic
            /*"COD" => self
                .app
                .set_projection(ProjectionType::Cod(mapproj::conic::cod::Cod::new())),
            // Hybrid
            "HPX" => self
                .app
                .set_projection(ProjectionType::Hpx(mapproj::hybrid::hpx::Hpx::new())),*/
            _ => Err(JsValue::from_str(
                "Not a valid projection name. AIT, ZEA, SIN, STG, TAN, MOL and MER are accepted",
            )),
        }
    }

    /*
    /// Check whether the app is ready
    ///
    /// Aladin Lite is in a good state when the root tiles of the
    /// HiPS chosen have all been retrieved and accessible for the GPU
    ///
    /// Surveys can be changed only if Aladin Lite is ready
    #[wasm_bindgen(js_name = isReady)]
    pub fn is_ready(&mut self) -> Result<bool, JsValue> {
        self.app.is_ready()
    }
    */

    #[wasm_bindgen(js_name = getNOrder)]
    pub fn get_norder(&mut self) -> Result<i32, JsValue> {
        Ok(self.app.get_norder())
    }

    /// Set new image surveys
    ///
    /// Send the image surveys to render inside the Aladin Lite view
    ///
    /// # Arguments
    ///
    /// * `surveys` - A list/array of survey. A survey is a javascript object
    /// having the specific form. Please check the file in core/src/hips.rs to see
    /// the different semantics accepted.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// let al = new Aladin.wasmLibs.core.WebClient(...);
    /// const panstarrs = {
    ///     properties: {
    ///         url: "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/r",
    ///
    ///         maxOrder: 11,
    ///         frame: { label: "J2000", system: "J2000" },
    ///         tileSize: 512,
    ///         format: {
    ///             FITSImage: {
    ///                 bitpix: 16,
    ///             }
    ///         },
    ///         minCutout: -0.15,
    ///         maxCutout: 5,
    ///     },
    ///     color: {
    ///         Grayscale2Colormap: {
    ///             colormap: "RedTemperature",
    ///             transfer: "asinh",
    ///             reversed: false,
    ///         }
    ///     },
    /// };
    /// al.setImageSurveys([panstarrs]);
    /// ```
    ///
    /// # Panics
    ///
    /// * If the surveys do not match SimpleHiPS type
    /// * If the number of surveys is greater than 4. For the moment, due to the limitations
    ///   of WebGL2 texture units on some architectures, the total number of surveys rendered is
    ///   limited to 4.
    #[wasm_bindgen(js_name = addHiPS)]
    pub fn add_image_hips(
        &mut self,
        hips: JsValue,
        files: Option<HiPSLocalFiles>,
    ) -> Result<(), JsValue> {
        // Deserialize the survey objects that compose the survey
        let hips = serde_wasm_bindgen::from_value(hips)?;
        self.app.add_image_hips(hips, files)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = addImageFITS)]
    pub fn add_image_fits(
        &mut self,
        stream: web_sys::ReadableStream,
        cfg: JsValue,
        layer: String,
    ) -> Result<js_sys::Promise, JsValue> {
        let cfg: ImageMetadata = serde_wasm_bindgen::from_value(cfg)?;

        self.app.add_image_fits(stream, cfg, layer)
    }

    #[wasm_bindgen(js_name = addImageWithWCS)]
    pub fn add_image_with_wcs(
        &mut self,
        stream: web_sys::ReadableStream,
        wcs: JsValue,
        cfg: JsValue,
        layer: String,
    ) -> Result<js_sys::Promise, JsValue> {
        use wcs::{WCSParams, WCS};
        let cfg: ImageMetadata = serde_wasm_bindgen::from_value(cfg)?;
        let wcs_params: WCSParams = serde_wasm_bindgen::from_value(wcs)?;
        let wcs = WCS::new(&wcs_params).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        self.app
            .add_image_from_blob_and_wcs(layer, stream, wcs, cfg)
    }

    #[wasm_bindgen(js_name = removeLayer)]
    pub fn remove_layer(&mut self, layer: String) -> Result<(), JsValue> {
        // Deserialize the survey objects that compose the survey
        self.app.remove_layer(&layer)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = renameLayer)]
    pub fn rename_layer(&mut self, layer: String, new_layer: String) -> Result<(), JsValue> {
        // Deserialize the survey objects that compose the survey
        self.app.rename_layer(&layer, &new_layer)
    }

    #[wasm_bindgen(js_name = swapLayers)]
    pub fn swap_layers(
        &mut self,
        first_layer: String,
        second_layer: String,
    ) -> Result<(), JsValue> {
        // Deserialize the survey objects that compose the survey
        self.app.swap_layers(&first_layer, &second_layer)
    }

    #[wasm_bindgen(js_name = setHiPSUrl)]
    pub fn set_hips_url(&mut self, cdid: String, new_url: String) -> Result<(), JsValue> {
        self.app.set_hips_url(&cdid, new_url)
    }

    #[wasm_bindgen(js_name = getImageMetadata)]
    pub fn get_layer_cfg(&self, layer: String) -> Result<ImageMetadata, JsValue> {
        self.app.get_layer_cfg(&layer)
    }

    // Set a new color associated with a layer
    #[wasm_bindgen(js_name = setImageMetadata)]
    pub fn set_survey_color_cfg(&mut self, layer: String, meta: JsValue) -> Result<(), JsValue> {
        let meta = serde_wasm_bindgen::from_value(meta)?;

        self.app.set_image_survey_color_cfg(layer, meta)
    }

    #[wasm_bindgen(js_name = setImageSurveyUrl)]
    pub fn set_survey_url(&mut self, cdid: String, new_url: String) -> Result<(), JsValue> {
        self.app.set_survey_url(&cdid, new_url)
    }

    #[wasm_bindgen(js_name = setBackgroundColor)]
    pub fn set_background_color(&mut self, color: JsValue) -> Result<(), JsValue> {
        let color = color.try_into()?;
        self.app.set_background_color(color);

        Ok(())
    }

    /// Set the equatorial grid color
    ///
    /// # Arguments
    ///
    /// * `red` - Red amount (between 0.0 and 1.0)
    /// * `green` - Green amount (between 0.0 and 1.0)
    /// * `blue` - Blue amount (between 0.0 and 1.0)
    /// * `alpha` - Alpha amount (between 0.0 and 1.0)
    #[wasm_bindgen(js_name = setGridOptions)]
    pub fn set_grid_cfg(&mut self, cfg: JsValue) -> Result<(), JsValue> {
        let cfg = serde_wasm_bindgen::from_value(cfg)?;

        self.app.set_grid_cfg(cfg)
    }

    /// Set the coordinate system for the view
    ///
    /// # Arguments
    ///
    /// * `coo_system` - The coordinate system
    #[wasm_bindgen(js_name = setCooSystem)]
    pub fn set_coo_system(&mut self, coo_system: CooSystem) -> Result<(), JsValue> {
        self.app.set_coo_system(coo_system);

        Ok(())
    }

    /// Get the field of the view in degrees
    #[wasm_bindgen(js_name = getFieldOfView)]
    pub fn get_fov(&self) -> Result<f64, JsValue> {
        let fov = self.app.get_fov();
        Ok(fov)
    }

    /// Set the field of view
    ///
    /// # Arguments
    ///
    /// * `fov` - The field of view in degrees
    #[wasm_bindgen(js_name = setFieldOfView)]
    pub fn set_fov(&mut self, fov: f64) -> Result<(), JsValue> {
        //let fov = fov as f32;
        let fov = ArcDeg(fov).into();

        self.app.set_fov(fov);
        //self.projection.set_fov(&mut self.app, ArcDeg(fov).into());

        Ok(())
    }

    #[wasm_bindgen(js_name = setInertia)]
    pub fn set_inertia(&mut self, inertia: bool) -> Result<(), JsValue> {
        self.app.set_inertia(inertia);

        Ok(())
    }

    /// Set the absolute orientation of the view
    ///
    /// # Arguments
    ///
    /// * `theta` - The rotation angle in degrees
    #[wasm_bindgen(js_name = setViewCenter2NorthPoleAngle)]
    pub fn set_view_center_pos_angle(&mut self, theta: f64) -> Result<(), JsValue> {
        let theta = ArcDeg(theta);
        self.app.set_view_center_pos_angle(theta);

        Ok(())
    }

    /// Get the absolute orientation angle of the view
    #[wasm_bindgen(js_name = getViewCenter2NorthPoleAngle)]
    pub fn get_north_shift_angle(&mut self) -> Result<f64, JsValue> {
        let phi = self.app.get_north_shift_angle();
        Ok(phi.to_degrees())
    }

    #[wasm_bindgen(js_name = getNorthPoleCelestialPosition)]
    pub fn get_north_pole_celestial_position(&mut self) -> Result<Box<[f64]>, JsValue> {
        let np = self
            .app
            .projection
            .north_pole_celestial_space(&self.app.camera);

        let (lon, lat) = (np.lon().to_degrees(), np.lat().to_degrees());
        Ok(Box::new([lon, lat]))
    }

    /// Get if the longitude axis is reversed
    #[wasm_bindgen(js_name = getLongitudeReversed)]
    pub fn get_longitude_reversed(&mut self) -> bool {
        self.app.get_longitude_reversed()
    }

    /// Get the field of view angle value when the view is zoomed out to its maximum
    ///
    /// This method is dependent of the projection currently set.
    /// All sky projections should return 360 degrees whereas
    /// the sinus would be 180 degrees.
    #[wasm_bindgen(js_name = getMaxFieldOfView)]
    pub fn get_max_fov(&mut self) -> f64 {
        self.app.get_max_fov()
    }

    /// Get the clip zoom factor of the view
    ///
    /// This factor is deduced from the field of view angle.
    /// It is a constant which when multiplied to the screen coordinates
    /// gives the coordinates in clipping space.
    #[wasm_bindgen(js_name = getClipZoomFactor)]
    pub fn get_clip_zoom_factor(&self) -> Result<f64, JsValue> {
        Ok(self.app.get_clip_zoom_factor())
    }

    /// Set the center of the view in ICRS coosys
    ///
    /// The core works in ICRS system so
    /// the location must be given in this system
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = setCenter)]
    pub fn set_center(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        self.app.set_center(&location);

        Ok(())
    }

    /// Get the center of the view
    ///
    /// This returns a javascript array of size 2.
    /// The first component is the longitude, the second one is the latitude.
    /// The angles are given in degrees.
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f64]>, JsValue> {
        let center = self.app.get_center();

        let (lon, lat) = (center.lon(), center.lat());

        let lon_deg: ArcDeg<f64> = lon.into();
        let lat_deg: ArcDeg<f64> = lat.into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    /// Go from a location to another one
    ///
    /// # Arguments
    ///
    /// * `s1x` - The x screen coordinate in pixels of the starting point
    /// * `s1y` - The y screen coordinate in pixels of the starting point
    /// * `s2x` - The x screen coordinate in pixels of the goal point
    /// * `s2y` - The y screen coordinate in pixels of the goal point
    #[wasm_bindgen(js_name = goFromTo)]
    pub fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) -> Result<(), JsValue> {
        self.app.go_from_to(s1x, s1y, s2x, s2y);

        Ok(())
    }

    /// View frame to ICRS/J2000 coosys conversion
    ///
    /// Coordinates must be given in the ICRS coo system
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = viewToICRSCooSys)]
    pub fn view_to_icrs_coosys(&self, lon: f64, lat: f64) -> Box<[f64]> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let res = self.app.view_to_icrs_coosys(&lonlat);

        let lon_deg: ArcDeg<f64> = res.lon().into();
        let lat_deg: ArcDeg<f64> = res.lat().into();

        Box::new([lon_deg.0, lat_deg.0])
    }

    /// World to screen projection
    ///
    /// Coordinates must be given in the ICRS coo system
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = world2pix)]
    pub fn world_to_pixel(
        &self,
        mut lon: f64,
        mut lat: f64,
        frame: Option<CooSystem>,
    ) -> Option<Box<[f64]>> {
        if let Some(frame) = frame {
            // first convert the coo to the view frame
            use crate::math::lonlat::LonLat;
            let xyz =
                LonLatT::new(lon.to_radians().to_angle(), lat.to_radians().to_angle()).vector();
            let lonlat = coosys::apply_coo_system(frame, CooSystem::ICRS, &xyz).lonlat();
            lon = lonlat.lon().to_degrees();
            lat = lonlat.lat().to_degrees();
        }

        self.app
            .world_to_screen(lon, lat)
            .map(|v| Box::new([v.x, v.y]) as Box<[f64]>)
    }

    #[wasm_bindgen(js_name = angularDist)]
    pub fn ang_dist(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
        crate::math::lonlat::ang_between_lonlat(
            LonLatT::new(lon1.to_radians().to_angle(), lat1.to_radians().to_angle()),
            LonLatT::new(lon2.to_radians().to_angle(), lat2.to_radians().to_angle()),
        )
        .to_degrees()
    }

    #[wasm_bindgen(js_name = screenToClip)]
    pub fn screen_to_clip(&self, x: f64, y: f64) -> Box<[f64]> {
        let v = self.app.screen_to_clip(&Vector2::new(x, y));

        Box::new([v.x, v.y]) as Box<[f64]>
    }

    #[wasm_bindgen(js_name = worldToScreenVec)]
    pub fn world_to_screen_vec(&self, lon: &[f64], lat: &[f64]) -> Box<[f64]> {
        let vertices = lon
            .iter()
            .zip(lat.iter())
            .map(|(&lon, &lat)| {
                let xy = self
                    .app
                    .world_to_screen(lon, lat)
                    .map(|v| [v.x, v.y])
                    .unwrap_or([0.0, 0.0]);

                xy
            })
            .flatten()
            .collect::<Vec<_>>();

        vertices.into_boxed_slice()
    }

    /*#[wasm_bindgen(js_name = drawCatalog)]
    pub fn world_to_screen_vec(&self, lon: &[f64], lat: &[f64], shape: &'static str, pixel_size) -> Box<[f64]> {
        let vertices = Time::measure_perf("projection rust side", || {
            Ok(lon
                .iter()
                .zip(lat.iter())
                .map(|(&lon, &lat)| {
                    let xy = self
                        .app
                        .world_to_screen(lon, lat)
                        .map(|v| [v.x, v.y])
                        .unwrap_or([0.0, 0.0]);

                    xy
                })
                .flatten()
                .collect::<Vec<_>>())
        })
        .unwrap();

        vertices.into_boxed_slice()
    }*/

    #[wasm_bindgen(js_name = setCatalog)]
    pub fn set_catalog(&self, _catalog: &Catalog) {}

    /// Screen to world unprojection
    ///
    /// # Arguments
    ///
    /// * `pos_x` - The x screen coordinate in pixels
    /// * `pos_y` - The y screen coordinate in pixels
    /// * `frame` - If not given, the coo given will be in the current view frame
    #[wasm_bindgen(js_name = pix2world)]
    pub fn pixel_to_world(
        &self,
        pos_x: f64,
        pos_y: f64,
        frame: Option<CooSystem>,
    ) -> Option<Box<[f64]>> {
        self.app
            .screen_to_world(&Vector2::new(pos_x, pos_y))
            .map(|mut lonlat| {
                if let Some(frame) = frame {
                    use crate::math::lonlat::LonLat;
                    let xyz = lonlat.vector();
                    lonlat =
                        coosys::apply_coo_system(self.app.get_coo_system(), frame, &xyz).lonlat();
                }

                let lon_deg: ArcDeg<f64> = lonlat.lon().into();
                let lat_deg: ArcDeg<f64> = lonlat.lat().into();

                Box::new([lon_deg.0, lat_deg.0]) as Box<[f64]>
            })
    }

    /// Signal the backend when the left mouse button has been released.
    ///
    /// This is useful for beginning inerting.
    #[wasm_bindgen(js_name = releaseLeftButtonMouse)]
    pub fn release_left_button_mouse(&mut self) -> Result<(), JsValue> {
        self.app.release_left_button_mouse();

        Ok(())
    }

    /// Signal the backend when the left mouse button has been pressed.
    #[wasm_bindgen(js_name = pressLeftMouseButton)]
    pub fn press_left_button_mouse(&mut self) -> Result<(), JsValue> {
        self.app.press_left_button_mouse();

        Ok(())
    }

    /// Signal the backend when the left mouse button has been pressed.
    #[wasm_bindgen(js_name = moveMouse)]
    pub fn move_mouse(&mut self, s1x: f32, s1y: f32, s2x: f32, s2y: f32) -> Result<(), JsValue> {
        self.app.move_mouse(s1x, s1y, s2x, s2y);

        Ok(())
    }

    /// Add a catalog rendered as a heatmap.
    ///
    /// # Arguments
    ///
    /// * `name_catalog` - The name of the catalog
    /// * `data` - The list of the catalog sources.
    /// * `colormap` - The name of the colormap. Check out the list of possible colormaps names `getAvailableColormapList`.
    #[wasm_bindgen(js_name = addCatalog)]
    pub fn add_catalog(
        &mut self,
        name_catalog: String,
        data: JsValue,
        colormap: String,
    ) -> Result<(), JsValue> {
        self.app.add_catalog(name_catalog, data, colormap);

        Ok(())
    }

    /// Set the catalog heatmap colormap
    ///
    /// # Arguments
    ///
    /// * `name_catalog` - The name of the catalog to apply this change to
    /// * `colormap` - The name of the colormap. Check out the list of possible colormaps names `getAvailableColormapList`.
    ///
    /// # Panics
    ///
    /// If the catalog has not been found
    #[wasm_bindgen(js_name = isCatalogLoaded)]
    pub fn is_catalog_loaded(&mut self) -> Result<bool, JsValue> {
        let cat_loaded = self.app.is_catalog_loaded();
        Ok(cat_loaded)
    }

    /// Set the catalog heatmap opacity
    ///
    /// # Arguments
    ///
    /// * `name_catalog` - The name of the catalog to apply this change to
    /// * `opacity` - The opacity factor (between 0.0 and 1.0)
    ///
    /// # Panics
    ///
    /// If the catalog has not been found
    #[wasm_bindgen(js_name = setCatalogOpacity)]
    pub fn set_catalog_opacity(
        &mut self,
        name_catalog: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        self.app.set_catalog_opacity(name_catalog, opacity)?;

        Ok(())
    }

    /// Set the kernel strength for the catalog heatmap rendering
    ///
    /// # Arguments
    ///
    /// * `name_catalog` - The name of the catalog to apply this change to
    /// * `strength` - The strength of the kernel
    ///
    /// # Panics
    ///
    /// If the catalog has not been found
    #[wasm_bindgen(js_name = setCatalogKernelStrength)]
    pub fn set_kernel_strength(
        &mut self,
        name_catalog: String,
        strength: f32,
    ) -> Result<(), JsValue> {
        self.app.set_kernel_strength(name_catalog, strength)?;

        Ok(())
    }

    /// Project a line to the screen
    ///
    /// # Returns
    ///
    /// A list of xy screen coordinates defining the projected line.
    /// The algorithm involved is recursive and can return different number of
    /// control points depending on the projection used and therefore
    /// the deformation of the line.
    ///
    /// # Arguments
    ///
    /// * `lon1` - The longitude in degrees of the starting line point
    /// * `lat1` - The latitude in degrees of the starting line point
    /// * `lon2` - The longitude in degrees of the ending line point
    /// * `lat2` - The latitude in degrees of the ending line point
    /*#[wasm_bindgen(js_name = projectLine)]
    pub fn project_line(
        &self,
        lon1: f64,
        lat1: f64,
        lon2: f64,
        lat2: f64,
    ) -> Result<Box<[f64]>, JsValue> {
        let vertices = self.app.project_line(lon1, lat1, lon2, lat2);

        let vertices = vertices
            .into_iter()
            .flat_map(|v| vec![v.x, v.y])
            .collect::<Vec<_>>();

        Ok(vertices.into_boxed_slice())
    }*/

    /// Get the list of colormap supported
    ///
    /// This list must be updated whenever a new colormap is added
    /// in core/img/colormaps/colormaps.png
    #[wasm_bindgen(js_name = getAvailableColormapList)]
    pub fn get_available_colormap_list(&self) -> Result<Vec<JsValue>, JsValue> {
        let colormaps = self
            .app
            .get_colormaps()
            .get_list_available_colormaps()
            .iter()
            .map(|s| JsValue::from_str(s))
            .collect::<Vec<_>>();

        Ok(colormaps)
    }

    #[wasm_bindgen(js_name = createCustomColormap)]
    pub fn add_custom_colormap(
        &mut self,
        label: String,
        hex_colors: Vec<JsValue>,
    ) -> Result<(), JsValue> {
        let rgba_colors: Result<Vec<_>, JsValue> = hex_colors
            .into_iter()
            .map(|hex_color| {
                let hex_color = serde_wasm_bindgen::from_value(hex_color)?;
                let color = Color::hexToRgba(hex_color);
                let color_rgba: ColorRGBA = color.try_into()?;

                Ok(colorgrad::Color::new(
                    color_rgba.r as f64,
                    color_rgba.g as f64,
                    color_rgba.b as f64,
                    color_rgba.a as f64,
                ))
            })
            .collect();

        let grad = colorgrad::CustomGradient::new()
            .colors(&rgba_colors?)
            .build()
            .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?;

        let cmap = Colormap::new(&label, grad);
        self.app.add_cmap(label, cmap)?;
        Ok(())
    }

    /// Get the image canvas where the webgl rendering is done
    #[wasm_bindgen(js_name = canvas)]
    pub fn get_gl_canvas(&mut self) -> Result<Option<js_sys::Object>, JsValue> {
        let canvas = self.app.get_gl_canvas();
        Ok(canvas)
    }

    /// Read the pixel value
    ///
    /// The current implementation only returns the pixel value
    /// of the first survey of the `layer` specified.
    ///
    /// # Returns
    ///
    /// - An array of 3 items (rgb) for JPG tiles
    /// - An array of 4 items (rgba) for PNG tiles
    /// - A single value for FITS tiles
    ///
    /// # Arguments
    ///
    /// * `x` - The x screen coordinate in pixels
    /// * `y` - The y screen coordinate in pixels
    /// * `base_url` - The base url of the survey identifying it
    #[wasm_bindgen(js_name = readPixel)]
    pub fn read_pixel(&self, x: f64, y: f64, layer: String) -> Result<JsValue, JsValue> {
        let pixel = self.app.read_pixel(&Vector2::new(x, y), layer.as_str())?;
        Ok(pixel)
    }

    #[wasm_bindgen(js_name = getVisibleCells)]
    pub fn get_visible_cells(&self, depth: u8) -> Result<JsValue, JsValue> {
        let cells = self.app.get_visible_cells(depth);
        Ok(serde_wasm_bindgen::to_value(&cells)?)
    }

    #[wasm_bindgen(js_name = isRendering)]
    pub fn is_rendering(&self) -> bool {
        self.app.is_rendering()
    }

    #[wasm_bindgen(js_name = drawGridLabels)]
    pub fn draw_grid_labels(&mut self) -> Result<(), JsValue> {
        self.app.draw_grid_labels()
    }

    #[wasm_bindgen(js_name = parseVOTable)]
    pub fn parse_votable(&mut self, _s: &str) -> Result<JsValue, JsValue> {
        /*let votable: VOTableWrapper<votable::impls::mem::InMemTableDataRows> =
            votable::votable::VOTableWrapper::from_ivoa_xml_str(s)
                .map_err(|err| JsValue::from_str(&format!("Error parsing votable: {:?}", err)))?;

        let votable = serde_wasm_bindgen::to_value(&votable)
            .map_err(|_| JsValue::from_str("cannot convert votable to js type"))?;

        Ok(votable)*/
        Ok(JsValue::null())
    }

    #[wasm_bindgen(js_name = addJSONMoc)]
    pub fn add_json_moc(
        &mut self,
        params: &al_api::moc::MOC,
        data: &JsValue,
    ) -> Result<(), JsValue> {
        let str: String = js_sys::JSON::stringify(data)?.into();

        let moc = moclib::deser::json::from_json_aladin::<u64, Hpx<u64>>(&str)
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?
            .into_cell_moc_iter()
            .ranges()
            .into_range_moc();

        self.app.add_moc(params.clone(), HEALPixCoverage(moc))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = addFITSMOC)]
    pub fn add_fits_moc(&mut self, params: &al_api::moc::MOC, data: &[u8]) -> Result<(), JsValue> {
        //let bytes = js_sys::Uint8Array::new(array_buffer).to_vec();
        let moc = match fits::from_fits_ivoa_custom(Cursor::new(&data[..]), false)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
        {
            MocIdxType::U16(MocQtyType::<u16, _>::Hpx(moc)) => {
                Ok(crate::downloader::request::moc::from_fits_hpx(moc))
            }
            MocIdxType::U32(MocQtyType::<u32, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
            MocIdxType::U64(MocQtyType::<u64, _>::Hpx(moc)) => Ok(from_fits_hpx(moc)),
            _ => Err(JsValue::from_str("MOC not supported. Must be a HPX MOC")),
        }?;

        self.app.add_moc(params.clone(), HEALPixCoverage(moc))?;

        Ok(())
    }

    #[wasm_bindgen(js_name = addConeMOC)]
    pub fn add_cone_moc(
        &mut self,
        params: &al_api::moc::MOC,
        ra_deg: f64,
        dec_deg: f64,
        rad_deg: f64,
    ) -> Result<(), JsValue> {
        let tile_d = self.app.get_norder();
        let pixel_d = tile_d + 9;
        let moc = HEALPixCoverage::from_cone(
            &LonLatT::new(
                ra_deg.to_radians().to_angle(),
                dec_deg.to_radians().to_angle(),
            ),
            rad_deg.to_radians(),
            pixel_d as u8 - 1,
        );

        self.app.add_moc(params.clone(), moc)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = addPolyMOC)]
    pub fn add_poly_moc(
        &mut self,
        params: &al_api::moc::MOC,
        ra_deg: &[f64],
        dec_deg: &[f64],
    ) -> Result<(), JsValue> {
        let tile_d = self.app.get_norder();
        let pixel_d = tile_d + 9;

        let vertex_it = ra_deg
            .iter()
            .zip(dec_deg.iter())
            .map(|(ra, dec)| -> Vector4<f64> {
                let lonlat = LonLatT(ra.to_radians().to_angle(), dec.to_radians().to_angle());
                lonlat.vector()
            });

        let v_in = &Vector4::new(1.0, 0.0, 0.0, 1.0);

        let mut moc = HEALPixCoverage::from_3d_coos(pixel_d as u8 - 1, vertex_it, &v_in);
        if moc.sky_fraction() > 0.5 {
            moc = moc.not();
        }

        self.app.add_moc(params.clone(), moc)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = removeMoc)]
    pub fn remove_moc(&mut self, params: &al_api::moc::MOC) -> Result<(), JsValue> {
        self.app.remove_moc(params)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setMocParams)]
    pub fn set_moc_cfg(&mut self, cfg: &al_api::moc::MOC) -> Result<(), JsValue> {
        self.app.set_moc_cfg(cfg.clone())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = mocContains)]
    pub fn moc_contains(
        &mut self,
        params: &al_api::moc::MOC,
        lon: f64,
        lat: f64,
    ) -> Result<bool, JsValue> {
        let moc = self
            .app
            .get_moc(params)
            .ok_or_else(|| JsValue::from(js_sys::Error::new("MOC not found")))?;
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        Ok(moc.contains_lonlat(&location))
    }

    #[wasm_bindgen(js_name = mocSerialize)]
    pub fn moc_serialize(
        &mut self,
        params: &al_api::moc::MOC,
        _format: String, // todo support the fits/ascii serialization
    ) -> Result<JsValue, JsValue> {
        let moc = self
            .app
            .get_moc(params)
            .ok_or_else(|| JsValue::from(js_sys::Error::new("MOC not found")))?;

        let mut buf: Vec<u8> = Default::default();
        let json = (&moc.0)
            .into_range_moc_iter()
            .cells()
            .to_json_aladin(None, &mut buf)
            .map(|()| unsafe { String::from_utf8_unchecked(buf) })
            .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?;

        serde_wasm_bindgen::to_value(&json).map_err(|err| JsValue::from_str(&format!("{:?}", err)))
    }

    #[wasm_bindgen(js_name = getMOCSkyFraction)]
    pub fn get_moc_sky_fraction(&mut self, params: &al_api::moc::MOC) -> f32 {
        if let Some(moc) = self.app.get_moc(params) {
            moc.sky_fraction() as f32
        } else {
            0.0
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct LonLat {
    pub lon: f64,
    pub lat: f64,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct HPXVertices {
    pub v1: LonLat,
    pub v2: LonLat,
    pub v3: LonLat,
    pub v4: LonLat,
}

/* HEALPix utils functions */
#[wasm_bindgen(js_name = HEALPixVertices)]
pub fn hpx_vertices(nside: u32, ipix: &[u64]) -> Result<Box<[HPXVertices]>, JsValue> {
    let depth = crate::healpix::cell::nside2depth(nside);
    let vertices = ipix
        .iter()
        .map(|i| {
            let cell = HEALPixCell(depth, *i);
            let vertices = cell.vertices();
            HPXVertices {
                v1: LonLat {
                    lon: vertices[0].0,
                    lat: vertices[0].1,
                },
                v2: LonLat {
                    lon: vertices[1].0,
                    lat: vertices[1].1,
                },
                v3: LonLat {
                    lon: vertices[2].0,
                    lat: vertices[2].1,
                },
                v4: LonLat {
                    lon: vertices[3].0,
                    lat: vertices[3].1,
                },
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    Ok(vertices)
}

#[wasm_bindgen(js_name = HEALPixPix2Ang)]
pub fn hpx_pix2ang(nside: u32, ipix: &[u64]) -> Result<Box<[LonLat]>, JsValue> {
    let depth = crate::healpix::cell::nside2depth(nside);
    let vertices = ipix
        .iter()
        .map(|i| {
            let cell = HEALPixCell(depth, *i);
            let (lon, lat) = cell.center();

            LonLat { lon, lat }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    Ok(vertices)
}

#[wasm_bindgen(js_name = HEALPixAng2Pix)]
pub fn hpx_ang2pix(nside: u32, lon: &[f64], lat: &[f64]) -> Result<Box<[u64]>, JsValue> {
    let depth = crate::healpix::cell::nside2depth(nside);
    let vertices = lon
        .iter()
        .zip(lat.iter())
        .map(|(&lon, &lat)| {
            let cell = HEALPixCell::new(depth, lon, lat);

            cell.idx()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    Ok(vertices)
}
