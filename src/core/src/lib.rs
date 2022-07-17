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
//extern crate egui;
//extern crate epi;
extern crate fontdue;
//extern crate image_decoder;
//extern crate itertools_num;
extern crate num;
//extern crate num_traits;
extern crate serde_json;
#[macro_use]
extern crate enum_dispatch;

use std::panic;

#[macro_use]
mod utils;

use math::projection::*;
use wasm_bindgen::prelude::*;

mod app;
pub mod async_task;
mod camera;

mod colormap;
mod coosys;
mod downloader;
mod healpix;
pub mod line;
pub mod math;
pub mod renderable;
mod shader;
mod survey;
mod tile_fetcher;
mod time;
mod fifo_cache;

use crate::{
    camera::CameraViewPort, colormap::Colormaps, math::lonlat::LonLatT, shader::ShaderManager, time::DeltaTime,
};
use al_api::grid::GridCfg;
use al_api::hips::{HiPSColor, HiPSProperties, SimpleHiPS};
use al_api::resources::Resources;
use al_core::{WebGlContext};

use al_api::coo_system::CooSystem;

use app::App;
use cgmath::{Vector2};

use math::angle::ArcDeg;

#[wasm_bindgen]
pub struct WebClient {
    // The app
    app: AppType,

    // The time between the previous and the current
    // frame
    dt: DeltaTime,
}

use crate::shader::FileSrc;

use crate::app::AppTrait;
use crate::app::AppType;
use al_api::color::Color;
use al_api::hips::ImageSurveyMeta;

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
    pub fn new(
        aladin_div_name: &str,
        shaders: &JsValue,
        resources: &JsValue,
    ) -> Result<WebClient, JsValue> {
        let shaders = shaders.into_serde::<Vec<FileSrc>>().unwrap();
        let resources = resources.into_serde::<Resources>().unwrap();
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let gl = WebGlContext::new(aladin_div_name)?;

        let shaders = ShaderManager::new(&gl, shaders).unwrap();
        let app = AppType::OrthoApp(App::<Orthographic>::new(
            &gl,
            aladin_div_name,
            shaders,
            resources,
        )?);

        let dt = DeltaTime::zero();

        let webclient = WebClient { app, dt };

        Ok(webclient)
    }

    /// Update the view
    ///
    /// # Arguments
    ///
    /// * `dt` - The time elapsed from the last frame update
    /// * `force` - This parameter ensures to force the update of some elements
    ///   even if the camera has not moved
    pub fn update(&mut self, dt: f32) -> Result<(), JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get ba    ck the
        // world coordinates of the center of projection in (ra, dec)
        self.app.update(
            // Time of the previous frame rendering
            self.dt, // Force the update of some elements:
                    // i.e. the grid
        )?;

        Ok(())
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

    /// Render the frame to the canvas
    ///
    /// The rendering does not redraw the scene if nothing has changed
    ///
    /// # Arguments
    ///
    /// * `force` - Force the rendering of the frame
    pub fn render(&mut self, force: bool) -> Result<(), JsValue> {
        self.app.draw(force)?;

        Ok(())
    }

    /// Set the type of projections
    ///
    /// # Arguments
    ///
    /// * `name` - Can be aitoff, mollweide, arc, sinus, tan or mercator
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(
        mut self,
        projection: String,
        width: f32,
        height: f32,
    ) -> Result<WebClient, JsValue> {
        match projection.as_str() {
            "AIT" => {
                self.app = AppType::AitoffApp(self.app.set_projection::<Aitoff>(width, height));
            },
            "SIN" => {
                self.app = AppType::OrthoApp(self.app.set_projection::<Orthographic>(width, height));
            },
            "MOL" => {
                self.app = AppType::MollweideApp(self.app.set_projection::<Mollweide>(width, height));
            },
            "ARC" => {
                self.app = AppType::ArcApp(self.app.set_projection::<AzimuthalEquidistant>(width, height));
            },
            "TAN" => {
                self.app = AppType::TanApp(self.app.set_projection::<Gnomonic>(width, height));
            },
            "MER" => {
                self.app = AppType::MercatorApp(self.app.set_projection::<Mercator>(width, height));
            },
            "HPX" => {
                self.app = AppType::HEALPixApp(self.app.set_projection::<HEALPix>(width, height));
            },
            _ => return Err(format!("{} is not a valid projection name. AIT, ARC, SIN, TAN, MOL, HPX and MER are accepted", projection).into()),
        }

        Ok(self)
    }

    /*
    /// Reverse the longitude axis
    ///
    /// # Arguments
    ///
    /// * `reversed` - set it to `false` for planetary surveys, `true` for spatial ones
    #[wasm_bindgen(js_name = setLongitudeReversed)]
    pub fn set_longitude_reversed(&mut self, reversed: bool) -> Result<(), JsValue> {
        self.app.set_longitude_reversed(reversed);

        Ok(())
    }
    */

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
    /// let al = new Aladin.wasmLibs.webgl.WebClient(...);
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

        //let surveys: Vec<SimpleHiPS> = surveys.iter().map(SimpleHiPS::from).collect::<Vec<_>>();
        let surveys = surveys?;
        self.app.set_image_surveys(surveys)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = getImageSurveyMeta)]
    pub fn get_survey_color_cfg(&self, layer: String) -> Result<ImageSurveyMeta, JsValue> {
        self.app.get_image_survey_color_cfg(&layer)
    }

    // Set a new color associated with a layer
    #[wasm_bindgen(js_name = setImageSurveyMeta)]
    pub fn set_survey_color_cfg(&mut self, layer: String, meta: JsValue) -> Result<(), JsValue> {
        let meta = meta
            .into_serde()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.app.set_image_survey_color_cfg(layer, meta)
    }

    #[wasm_bindgen(js_name = setImageSurveyImageFormat)]
    pub fn set_image_survey_img_format(&mut self, layer: String, format: JsValue) -> Result<(), JsValue> {
        let format = format.into_serde()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.app.set_image_survey_img_format(layer, format)
    }

    #[wasm_bindgen(js_name = setImageSurveyUrl)]
    pub fn set_survey_url(&mut self, past_url: String, new_url: String) -> Result<(), JsValue> {
        self.app.set_survey_url(&past_url, &new_url)
    }

    /// Set the equatorial grid color
    ///
    /// # Arguments
    ///
    /// * `red` - Red amount (between 0.0 and 1.0)
    /// * `green` - Green amount (between 0.0 and 1.0)
    /// * `blue` - Blue amount (between 0.0 and 1.0)
    /// * `alpha` - Alpha amount (between 0.0 and 1.0)
    #[wasm_bindgen(js_name = setGridConfig)]
    pub fn set_grid_cfg(&mut self, cfg: &JsValue) -> Result<(), JsValue> {
        let cfg = cfg
            .into_serde::<GridCfg>()
            .map_err(|e| JsValue::from(js_sys::Error::new(&e.to_string())))?;

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

    /*/// Get the coordinate system of the view
    ///
    /// Returns either ICRSJ2000 or GAL
    #[wasm_bindgen(js_name = cooSystem)]
    pub fn get_coo_system(&self) -> Result<CooSystem, JsValue> {
        Ok(*self.app.get_coo_system())
    }

    /// Convert a j2000 lonlat to a galactic one
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = J20002Gal)]
    pub fn j2000_to_gal(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let gal_lonlat = coo_conversion::to_galactic(lonlat);

        Ok(Some(Box::new([
            gal_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI),
            gal_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI),
        ])))
    }

    /// Convert a galactic lonlat to a j2000 one
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = Gal2J2000)]
    pub fn gal_to_j2000(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let icrsj2000_lonlat = coo_conversion::to_icrs_j2000(lonlat);

        Ok(Some(Box::new([
            icrsj2000_lonlat.lon().0 * 360.0 / (2.0 * std::f64::consts::PI),
            icrsj2000_lonlat.lat().0 * 360.0 / (2.0 * std::f64::consts::PI),
        ])))
    }
    */
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

    /// Set the absolute orientation of the view
    ///
    /// # Arguments
    ///
    /// * `theta` - The rotation angle in degrees
    #[wasm_bindgen(js_name = setRotationAroundCenter)]
    pub fn rotate_around_center(&mut self, theta: f64) -> Result<(), JsValue> {
        let theta = ArcDeg(theta);
        self.app.rotate_around_center(theta);

        Ok(())
    }

    /// Get the absolute orientation angle of the view
    #[wasm_bindgen(js_name = getRotationAroundCenter)]
    pub fn get_rotation_around_center(&mut self) -> Result<f64, JsValue> {
        let theta = self.app.get_rotation_around_center();

        Ok(theta.0 * 360.0 / (2.0 * std::f64::consts::PI))
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

    /// Set the center of the view in ICRSJ2000 coosys
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

    /// Rest the north pole orientation to the top of the screen
    #[wasm_bindgen(js_name = resetNorthOrientation)]
    pub fn reset_north_orientation(&mut self) {
        self.app.reset_north_orientation();
    }
    /*
    /// Move to a location
    ///
    /// The core works in ICRS system so
    /// the location must be given in this system
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = moveToLocation)]
    pub fn move_to_location(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        // The core works in ICRS_J2000 coordinates
        // Check if the user is giving galactic coordinates
        // so that we can convert them to icrs
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        self.app.start_moving_to(&location);

        Ok(())
    }
    */
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
    #[wasm_bindgen(js_name = viewToICRSJ2000CooSys)]
    pub fn view_to_icrsj2000_coosys(&self, lon: f64, lat: f64) -> Box<[f64]> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let res = self.app.view_to_icrsj2000_coosys(&lonlat);

        let lon_deg: ArcDeg<f64> = res.lon().into();
        let lat_deg: ArcDeg<f64> = res.lat().into();

        Box::new([lon_deg.0, lat_deg.0])
    }

    /// ICRS/J2000 to view frame coosys conversion
    ///
    /// Coordinates must be given in the ICRS coo system
    ///
    /// # Arguments
    ///
    /// * `lon` - A longitude in degrees
    /// * `lat` - A latitude in degrees
    #[wasm_bindgen(js_name = ICRSJ2000ToViewCooSys)]
    pub fn icrsj2000_to_view_coosys(&self, lon: f64, lat: f64) -> Box<[f64]> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        let res = self.app.icrsj2000_to_view_coosys(&lonlat);

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
    #[wasm_bindgen(js_name = worldToScreen)]
    pub fn world_to_screen(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());

        if let Some(screen_pos) = self.app.world_to_screen(&lonlat)? {
            Ok(Some(Box::new([screen_pos.x, screen_pos.y])))
        } else {
            Ok(None)
        }
    }

    /// World to screen projection of a list of sources
    ///
    /// Coordinates must be given in the ICRS coo system
    ///
    /// # Arguments
    ///
    /// * `sources` - An array of sources
    #[wasm_bindgen(js_name = worldToScreenVec)]
    pub fn world_to_screen_vec(&self, sources: Vec<JsValue>) -> Result<Box<[f64]>, JsValue> {
        self.app.world_to_screen_vec(&sources)
    }

    /// Screen to world unprojection
    ///
    /// # Arguments
    ///
    /// * `pos_x` - The x screen coordinate in pixels
    /// * `pos_y` - The y screen coordinate in pixels
    #[wasm_bindgen(js_name = screenToWorld)]
    pub fn screen_to_world(&self, pos_x: f64, pos_y: f64) -> Option<Box<[f64]>> {
        if let Some(lonlat) = self.app.screen_to_world(&Vector2::new(pos_x, pos_y)) {
            let lon_deg: ArcDeg<f64> = lonlat.lon().into();
            let lat_deg: ArcDeg<f64> = lonlat.lat().into();

            Some(Box::new([lon_deg.0, lat_deg.0]))
        } else {
            None
        }
    }

    /// Signal the backend when the left mouse button has been released.
    ///
    /// This is useful for beginning inerting.
    #[wasm_bindgen(js_name = releaseLeftButtonMouse)]
    pub fn release_left_button_mouse(&mut self, sx: f32, sy: f32) -> Result<(), JsValue> {
        self.app.release_left_button_mouse(sx, sy);

        Ok(())
    }

    /// Signal the backend when the left mouse button has been pressed.
    #[wasm_bindgen(js_name = pressLeftMouseButton)]
    pub fn press_left_button_mouse(&mut self, sx: f32, sy: f32) -> Result<(), JsValue> {
        self.app.press_left_button_mouse(sx, sy);

        Ok(())
    }

    /// Signal the backend when a wheel event has been registered
    ///
    /// The field of view is changed accordingly
    ///
    /// # Arguments
    ///
    /// * `delta` - The delta coming from the wheel event. This is
    ///   used to know if we are zooming or not.
    #[wasm_bindgen(js_name = posOnUi)]
    pub fn screen_position_on_ui(&mut self) -> bool {
        self.app.over_ui()
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
    #[wasm_bindgen(js_name = projectLine)]
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
    }

    /// Get the list of colormap supported
    ///
    /// This list must be updated whenever a new colormap is added
    /// in core/img/colormaps/colormaps.png
    #[wasm_bindgen(js_name = getAvailableColormapList)]
    pub fn get_available_colormap_list(&self) -> Result<JsValue, JsValue> {
        let colormaps = Colormaps::get_list_available_colormaps()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        JsValue::from_serde(&colormaps).map_err(|e| JsValue::from_str(&e.to_string()))
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
        Ok(pixel.into())
    }

    /// TODO! This will be removed when integrating the MOC code in wasm because
    /// this method is only called in MOC.js
    /// Computes the location on the unit sphere of the 4 vertices of the given HEALPix cell
    /// (define by its depth and number).
    /// # Inputs
    /// - `order` the order of the cell we look for the vertices
    /// - `icell`: the cell number value of the cell we look for the unprojected center, in the NESTED scheme
    /// # Output
    /// - array containing the longitudes and latitudes (in degrees) of the vertices in the following order:
    ///   `[SouthLon, SouthLat, EastLon, EastLat, NoethLon, NorthLat, WestLon, WestLat]`
    #[wasm_bindgen(js_name = hpxNestedVertices)]
    pub fn hpx_nested_vertices(&self, depth: u8, hash: f64) -> Box<[f64]> {
        let [(s_lon, s_lat), (e_lon, e_lat), (n_lon, n_lat), (w_lon, w_lat)] =
            cdshealpix::nested::vertices(depth, hash as u64);
        Box::new([
            s_lon.to_degrees(),
            s_lat.to_degrees(),
            e_lon.to_degrees(),
            e_lat.to_degrees(),
            n_lon.to_degrees(),
            n_lat.to_degrees(),
            w_lon.to_degrees(),
            w_lat.to_degrees(),
        ])
    }

    #[wasm_bindgen(js_name = queryDisc)]
    pub fn query_disc(
        &self,
        depth: u8,
        lon_degrees: f64,
        lat_degrees: f64,
        radius_degress: f64,
    ) -> Box<[f64]> {
        cdshealpix::nested::cone_coverage_approx(
            depth,
            lon_degrees.to_radians(),
            lat_degrees.to_radians(),
            radius_degress.to_radians(),
        )
        .to_flat_array()
        .iter()
        .map(|&v| v as f64)
        .collect::<Vec<_>>()
        .into_boxed_slice()
    }

    #[wasm_bindgen(js_name = isRendering)]
    pub fn is_rendering(&self) -> bool {
        self.app.is_rendering()
    }
}
