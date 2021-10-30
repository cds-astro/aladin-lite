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
extern crate img_pixel;
extern crate itertools_num;
extern crate num;
extern crate num_traits;
extern crate rand;
extern crate serde_derive;
extern crate serde_json;
extern crate task_async_executor;
extern crate egui;
extern crate epi;
extern crate egui_web;
use al_core;

use std::panic;


#[macro_use]
mod utils;

#[macro_use]
mod log;

use wasm_bindgen::prelude::*;

mod app;
mod async_task;
mod buffer;
mod camera;
mod cdshealpix;
mod color;
mod coo_conversion;

mod healpix_cell;
pub mod hips;
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
mod ui;

use al_core::format::ImageFormatType;
use crate::{
    camera::CameraViewPort,
    hips::{HiPSColor, HiPSFormat, HiPSProperties, SimpleHiPS},
    math::LonLatT,
    renderable::{image_survey::ImageSurveys, projection::Projection, Angle, ArcDeg},
    resources::Resources,
    shader::{ShaderManager},
    shaders::Colormaps,
    time::DeltaTime,
};
use al_core::{
    WebGl2Context,
    shader::Shader
};
pub use coo_conversion::CooSystem;

use app::App;
use cgmath::{Vector2, Vector4, VectorSpace};
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

use crate::shader::FileSrc;
use crate::transfert_function::TransferFunction;

use crate::color::Color;
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
        let gl = WebGl2Context::new(aladin_div_name)?;

        let shaders = ShaderManager::new(&gl, shaders).unwrap();
        let app = App::new(&gl, aladin_div_name, shaders, resources)?;

        let dt = DeltaTime::zero();
        let projection = ProjectionType::Ortho;

        //let a = TemplateApp::default();
        //eframe::start_web("aladin-guiCanvas", Box::new(a)).unwrap();
        //let mut backend = egui_web::WebBackend::new("aladin-guiCanvas").expect("Failed to make a web backend for egui");
        //let mut web_input: WebInput = Default::default();

        let webclient = WebClient {
            app,
            projection,

            dt,
        };

        Ok(webclient)
    }

    /// Update the view
    ///
    /// # Arguments
    ///
    /// * `dt` - The time elapsed from the last frame update
    /// * `force` - This parameter ensures to force the update of some elements
    ///   even if the camera has not moved
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
    ///
    /// # Arguments
    ///
    /// * `width` - The width in pixels of the view
    /// * `height` - The height in pixels of the view
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.projection.resize(&mut self.app, width, height);

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
        self.projection.render(&mut self.app, force)?;

        Ok(())
    }

    /// Set the type of projections
    ///
    /// # Arguments
    ///
    /// * `name` - Can be aitoff, mollweide, arc, sinus, tan or mercator
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(&mut self, name: String) -> Result<(), JsValue> {
        self.projection.set_projection(&mut self.app, name)?;

        Ok(())
    }

    /// Reverse the longitude axis
    ///
    /// # Arguments
    ///
    /// * `reversed` - set it to `false` for planetary surveys, `true` for spatial ones
    #[wasm_bindgen(js_name = setLongitudeReversed)]
    pub fn set_longitude_reversed(&mut self, reversed: bool) -> Result<(), JsValue> {
        self.projection
            .set_longitude_reversed(&mut self.app, reversed);

        Ok(())
    }

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
    ///     layer: 'base',
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
        let surveys = surveys?;
        self.app.set_image_surveys(surveys)?;

        Ok(())
    }

    /// Move a layer forward the other ones
    ///
    /// # Arguments
    ///
    /// * `layer_name` - The name of the layer to move
    ///
    /// # Panics
    ///
    /// * If the layer specified is not found
    #[wasm_bindgen(js_name = moveImageSurveysLayerForward)]
    pub fn move_image_surveys_layer_forward(&mut self, layer_name: &str) -> Result<(), JsValue> {
        self.app.move_image_surveys_layer_forward(layer_name)
    }

    /// Set the opacity of a layer
    ///
    /// # Arguments
    ///
    /// * `opacity` - Set an opacity value (between 0.0 and 1.0)
    /// * `layer_name` - The name of the layer to move
    ///
    /// # Panics
    ///
    /// * If the layer specified is not found
    #[wasm_bindgen(js_name = setOpacityLayer)]
    pub fn set_opacity_layer(&mut self, opacity: f32, layer_name: &str) -> Result<(), JsValue> {
        self.app.set_opacity_layer(layer_name, opacity)
    }

    /// Set the equatorial grid color
    ///
    /// # Arguments
    ///
    /// * `red` - Red amount (between 0.0 and 1.0)
    /// * `green` - Green amount (between 0.0 and 1.0)
    /// * `blue` - Blue amount (between 0.0 and 1.0)
    /// * `alpha` - Alpha amount (between 0.0 and 1.0)
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

    /// Enable the rendering of the equatorial grid
    #[wasm_bindgen(js_name = enableGrid)]
    pub fn enable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.enable_grid(&mut self.app);

        Ok(())
    }

    /// Disable the rendering of the equatorial grid
    #[wasm_bindgen(js_name = disableGrid)]
    pub fn disable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.disable_grid(&mut self.app);

        Ok(())
    }

    /// Enable the rendering of the equatorial grid labels
    #[wasm_bindgen(js_name = hideGridLabels)]
    pub fn enable_grid_labels(&mut self) -> Result<(), JsValue> {
        self.projection.hide_grid_labels(&mut self.app);

        Ok(())
    }

    /// Disable the rendering of the equatorial grid labels
    #[wasm_bindgen(js_name = showGridLabels)]
    pub fn disable_grid_labels(&mut self) -> Result<(), JsValue> {
        self.projection.show_grid_labels(&mut self.app);

        Ok(())
    }

    /// Get the coordinate system of the view
    ///
    /// Returns either ICRSJ2000 or GAL
    #[wasm_bindgen(js_name = cooSystem)]
    pub fn get_coo_system(&self) -> Result<CooSystem, JsValue> {
        Ok(self.app.system)
    }

    /// Set the coordinate system for the view
    ///
    /// # Arguments
    ///
    /// * `coo_system` - The coordinate system
    #[wasm_bindgen(js_name = setCooSystem)]
    pub fn set_coo_system(&mut self, coo_system: CooSystem) -> Result<(), JsValue> {
        self.projection.set_coo_system(&mut self.app, coo_system);

        Ok(())
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

        self.projection.start_zooming_to(&mut self.app, fov);
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
        self.projection.rotate_around_center(&mut self.app, theta);

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
        self.projection.get_max_fov(&mut self.app)
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

    /// Set the center of the view
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

        self.projection.set_center(&mut self.app, location);

        Ok(())
    }

    /// Get the center of the view
    ///
    /// This returns a javascript array of size 2.
    /// The first component is the longitude, the second one is the latitude.
    /// The angles are given in degrees.
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f64]>, JsValue> {
        let center = self.projection.get_center(&self.app);

        let lon_deg: ArcDeg<f64> = center.lon().into();
        let lat_deg: ArcDeg<f64> = center.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    /// Rest the north pole orientation to the top of the screen
    #[wasm_bindgen(js_name = resetNorthOrientation)]
    pub fn reset_north_orientation(&mut self) {
        self.projection.reset_north_orientation(&mut self.app);
    }

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
        self.projection.start_moving_to(&mut self.app, location);

        Ok(())
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
        self.projection
            .go_from_to(&mut self.app, s1x, s1y, s2x, s2y);

        Ok(())
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

        if let Some(screen_pos) = self.projection.world_to_screen(&self.app, &lonlat)? {
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
        let screen_positions = self.projection.world_to_screen_vec(&self.app, &sources)?;
        Ok(screen_positions.into_boxed_slice())
    }

    /// Screen to world unprojection
    ///
    /// # Arguments
    ///
    /// * `pos_x` - The x screen coordinate in pixels
    /// * `pos_y` - The y screen coordinate in pixels
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

    /// Signal the backend when a wheel event has been registered
    ///
    /// The field of view is changed accordingly
    ///
    /// # Arguments
    ///
    /// * `delta` - The delta coming from the wheel event. This is
    ///   used to know if we are zooming or not.
    #[wasm_bindgen(js_name = mouseOnUi)]
    pub fn mouse_on_ui(&mut self) -> bool {
        self.app.mouse_on_ui()
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
    pub fn screen_position_on_ui(&mut self, sx: f32, sy:f32) -> bool {
        self.app.pos_over_ui(sx, sy)
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
        self.projection
            .add_catalog(&mut self.app, name_catalog, data, colormap);

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
    #[wasm_bindgen(js_name = setCatalogColormap)]
    pub fn set_catalog_colormap(
        &mut self,
        name_catalog: String,
        colormap: String,
    ) -> Result<(), JsValue> {
        let colormap = self.app.colormaps.get(&colormap);
        self.projection
            .set_catalog_colormap(&mut self.app, name_catalog, colormap)?;

        Ok(())
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
    pub fn set_heatmap_opacity(
        &mut self,
        name_catalog: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        self.projection
            .set_heatmap_opacity(&mut self.app, name_catalog, opacity)?;

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
        self.projection
            .set_kernel_strength(&mut self.app, name_catalog, strength)?;

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
    /// * `layer` - The name of the layer to read the pixel from.
    #[wasm_bindgen(js_name = readPixel)]
    pub fn read_pixel(&self, x: f64, y: f64, layer: &str) -> Result<JsValue, JsValue> {
        let pixel = self.projection.read_pixel(&self.app, x, y, layer)?;
        Ok(pixel.into())
    }
}
