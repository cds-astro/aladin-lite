use crate::{
    async_task::{BuildCatalogIndex, ParseTableTask, TaskExecutor, TaskResult, TaskType},
    camera::CameraViewPort,
    colormap::Colormaps,
    downloader::Downloader,
    line,
    math::{
        self,
        angle::{Angle, ArcDeg},
        lonlat::{LonLat, LonLatT},
        projection::{Orthographic, Projection},
    },
    renderable::{
        catalog::{Manager, Source},
        grid::ProjetedGrid,
    },
    shader::ShaderManager,
    survey::ImageSurveys,
    tile_fetcher::TileFetcherQueue,
    time::DeltaTime,
    utils,
};
//use al_core::resources::Resources;
use al_core::WebGlContext;

use al_api::{
    coo_system::CooSystem,
    grid::GridCfg,
    hips::{ImageSurveyMeta, SimpleHiPS},
};

use super::coosys;
use cgmath::Vector4;

use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashSet;

use serde::Deserialize;
#[derive(Deserialize, Debug)]
struct S {
    ra: f64,
    dec: f64,
}

use crate::renderable::final_pass::RenderPass;
use al_core::FrameBufferObject;

pub struct App<P>
where
    P: Projection,
{
    pub gl: WebGlContext,

    //ui: GuiRef,
    shaders: ShaderManager,
    camera: CameraViewPort,

    downloader: Downloader,
    tile_fetcher: TileFetcherQueue,
    surveys: ImageSurveys,

    time_start_blending: Time,
    request_redraw: bool,
    rendering: bool,

    // The grid renderable
    grid: ProjetedGrid,
    // Catalog manager
    manager: Manager,

    // Task executor
    exec: Rc<RefCell<TaskExecutor>>,
    resources: Resources,

    //move_animation: Option<MoveAnimation>,
    //zoom_animation: Option<ZoomAnimation>,
    inertial_move_animation: Option<InertiaAnimation>,
    prev_cam_position: Vector3<f64>,
    prev_center: Vector3<f64>,
    out_of_fov: bool,
    tasks_finished: bool,
    catalog_loaded: bool,

    final_rendering_pass: RenderPass,
    fbo_view: FrameBufferObject,
    pub fbo_ui: FrameBufferObject,

    pub colormaps: Colormaps,

    p: std::marker::PhantomData<P>,
}

use cgmath::{Vector2, Vector3};
use futures::stream::StreamExt; // for `next`

use crate::math::rotation::Rotation;

/*struct MoveAnimation {
    start_anim_rot: Rotation<f64>,
    goal_anim_rot: Rotation<f64>,
    time_start_anim: Time,
}*/

/// State for inertia
struct InertiaAnimation {
    // Initial angular distance
    d0: Angle<f64>,
    // Vector of rotation
    axis: Vector3<f64>,
    // The time when the inertia begins
    time_start_anim: Time,
}
/*
struct ZoomAnimation {
    time_start_anim: Time,
    start_fov: Angle<f64>,
    goal_fov: Angle<f64>,
    w0: f64,
}
*/
use crate::math::projection::*;
pub const BLENDING_ANIM_DURATION: f32 = 500.0; // in ms
                                               //use crate::buffer::Tile;
use crate::time::Time;
use cgmath::InnerSpace;



type OrthoApp = App<Orthographic>;
type AitoffApp = App<Aitoff>;
type MollweideApp = App<Mollweide>;
type ArcApp = App<AzimuthalEquidistant>;
type TanApp = App<Gnomonic>;
type MercatorApp = App<Mercator>;
type HEALPixApp = App<HEALPix>;

#[enum_dispatch]
pub enum AppType {
    OrthoApp,
    AitoffApp,
    MollweideApp,
    ArcApp,
    TanApp,
    HEALPixApp,
    MercatorApp,
}
use al_api::resources::Resources;
use crate::downloader::query;

use moclib::deser::fits::MocQtyType::Hpx;
use moclib::moc::range::RangeMOC;
use moclib::elemset::range::MocRanges;
use moclib::ranges::Ranges;
use crate::healpix::coverage::HEALPixCoverage;

use crate::downloader::request::moc::MOC;
impl<P> App<P>
where
    P: Projection,
{
    pub fn new(
        gl: &WebGlContext,
        _aladin_div_name: &str,
        mut shaders: ShaderManager,
        resources: Resources,
    ) -> Result<Self, JsValue> {
        let gl = gl.clone();
        let exec = Rc::new(RefCell::new(TaskExecutor::new()));

        gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        gl.enable(WebGl2RenderingContext::SCISSOR_TEST);
        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        // The tile buffer responsible for the tile requests
        let downloader = Downloader::new();

        let camera = CameraViewPort::new::<Orthographic>(&gl, CooSystem::ICRSJ2000);
        let screen_size = &camera.get_screen_size();

        let fbo_view = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;
        let fbo_ui = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;

        // The surveys storing the textures of the resolved tiles
        let surveys = ImageSurveys::new::<Orthographic>(&gl);

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources)?;

        // Grid definition
        let grid = ProjetedGrid::new::<Orthographic>(&gl, &camera)?;

        // Variable storing the location to move to
        let inertial_move_animation = None;
        let tasks_finished = false;
        let request_redraw = false;
        let rendering = true;
        let prev_cam_position = camera.get_center().truncate();
        let prev_center = Vector3::new(0.0, 1.0, 0.0);
        let out_of_fov = false;
        let catalog_loaded = false;

        let colormaps = Colormaps::new(&gl, &resources)?;

        let final_rendering_pass =
            RenderPass::new(&gl, screen_size.x as i32, screen_size.y as i32)?;
        let tile_fetcher = TileFetcherQueue::new();

        //let ui = Gui::new(aladin_div_name, &gl)?;
        Ok(App {
            gl,
            //ui,
            shaders,

            camera,

            downloader,
            surveys,

            time_start_blending,
            rendering,
            request_redraw,
            // The grid renderable
            grid,
            // The catalog renderable
            manager,
            exec,
            resources,
            prev_center,

            fbo_view,
            fbo_ui,
            final_rendering_pass,

            inertial_move_animation,
            prev_cam_position,
            out_of_fov,

            tasks_finished,
            catalog_loaded,

            tile_fetcher,

            colormaps,
            p: std::marker::PhantomData,
        })
    }

    fn look_for_new_tiles(&mut self) {
        // Move the views of the different active surveys
        self.surveys.refresh_views(&mut self.camera);
        self.tile_fetcher.clear();
        // Loop over the surveys
        for (_, survey) in self.surveys.iter_mut() {
            // do not add tiles if the view is already at depth 0
            let view = survey.get_view();
            let depth_tile = view.get_depth();
            if depth_tile > survey.get_min_depth() {
                let mut tile_cells = survey
                    .get_view()
                    .get_cells()
                    //.flat_map(|texture_cell| texture_cell.get_tile_cells(survey.get_config()))
                    .flat_map(|cell| {
                        let texture_cell = cell.get_texture_cell(survey.get_config());
                        texture_cell.get_tile_cells(survey.get_config())
                    })
                    .collect::<HashSet<_>>();

                if depth_tile > 3 {
                    // Retrieve the grand-grand parent cells but not if it is root ones as it may interfere with already done requests
                    let tile_cells_ancestor = tile_cells
                        .iter()
                        .map(|tile_cell| tile_cell.ancestor(3))
                        .collect::<HashSet<_>>();

                    tile_cells.extend(tile_cells_ancestor);
                }

                // Do not request the cells where we know from its moc that there is no data
                {
                    if let Some(coverage) = *survey.get_footprint_moc().lock().unwrap() {
                        let tile_cells = tile_cells.into_iter()
                            .filter(|tile_cell| {
                                let start_idx = tile_cell.idx() << (2*(29 - tile_cell.depth()));
                                let end_idx = (tile_cell.idx() + 1) << (2*(29 - tile_cell.depth()));

                                let mut moc = RangeMOC::new(
                                    tile_cell.depth(),
                                    MocRanges::<u64, moclib::qty::Hpx<u64>>(
                                        Ranges::<u64>(Box::new([tile_cell.idx()..(tile_cell.idx() + 1)])),
                                        std::marker::PhantomData
                                    )
                                );
                                coverage.intersection(&HEALPixCoverage(moc))
                            });
                    }
                }

                for tile_cell in tile_cells {
                    let tile_found = survey.update_priority_tile(&tile_cell);
                    if !tile_found {
                        // Submit the request to the buffer
                        let cfg = survey.get_config();
                        // Launch the new tile requests
                        //self.downloader.fetch(query::Tile::new(&tile_cell, cfg));
                        self.tile_fetcher
                            .append(query::Tile::new(&tile_cell, cfg), &mut self.downloader);
                    }
                }
            }
        }
    }

    // Run async tasks:
    // - parsing catalogs
    // - copying textures to GPU
    // Return true when a task is complete. This always lead
    // to a redraw of aladin lite
    /*fn run_tasks(&mut self, dt: DeltaTime) -> Result<HashSet<Tile>, JsValue> {
        let tasks_time = (dt.0 * 0.5).min(8.3);
        let results = self.exec.borrow_mut().run(tasks_time);
        self.tasks_finished = !results.is_empty();

        // Retrieve back all the tiles that have been
        // copied to the GPU
        // This is important for the tile buffer to know which
        // requests can be reused to query more tiles
        let mut tiles_available = HashSet::new();
        for result in results {
            match result {
                TaskResult::TableParsed {
                    name,
                    sources,
                    colormap,
                } => {
                    self.manager.add_catalog::<P>(
                        name,
                        sources,
                        colormap,
                        &mut self.shaders,
                        &self.camera,
                        self.surveys.get_view().unwrap(),
                    );
                    self.catalog_loaded = true;
                    self.request_redraw = true;
                }
                TaskResult::TileSentToGPU { tile } => {
                    tiles_available.insert(tile);
                }
            }
        }

        Ok(tiles_available)
    }*/
    /*fn run_tasks(&mut self, dt: DeltaTime) -> Result<(), JsValue> {
        let tasks_time = (dt.0 * 0.5).min(8.3);
        let results = self.exec.borrow_mut().run(tasks_time);
        self.tasks_finished = !results.is_empty();

        // Retrieve back all the tiles that have been
        // copied to the GPU
        // This is important for the tile buffer to know which
        // requests can be reused to query more tiles
        for result in results {
            match result {
                TaskResult::TableParsed {
                    name,
                    sources,
                    colormap,
                } => {
                    self.manager.add_catalog::<P>(
                        name,
                        sources,
                        colormap,
                        &mut self.shaders,
                        &self.camera,
                        self.surveys.get_view().unwrap(),
                    );
                    self.catalog_loaded = true;
                    self.request_redraw = true;
                } //TaskResult::TileSentToGPU { tile } => todo!()
            }
        }

        Ok(())
    }*/
}
use al_api::hips::HiPSTileFormat;
#[enum_dispatch(AppType)]
pub trait AppTrait {
    /// View
    fn is_ready(&self) -> Result<bool, JsValue>;
    fn is_rendering(&self) -> bool;

    // Called whenever the canvas changed its dimensions
    fn resize(&mut self, width: f32, height: f32);
    // Low level method for updating the view
    fn update(&mut self, dt: DeltaTime) -> Result<(), JsValue>;
    // Low level method for rendering the view (layers + ui) on the render
    fn draw(&mut self, force_render: bool) -> Result<(), JsValue>;

    /// Survey
    // Low level methods that must be called whenever a change happen in the group of layers to render
    // - A layer moves in the stack
    // - A layer is added
    // - A layer is removed
    fn set_image_surveys(&mut self, hipses: Vec<SimpleHiPS>) -> Result<(), JsValue>;
    // Getter to access the meta data of a layer
    fn get_image_survey_color_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue>;
    // Setter of the meta data of a layer
    fn set_image_survey_color_cfg(
        &mut self,
        layer: String,
        meta: ImageSurveyMeta,
    ) -> Result<(), JsValue>;
    fn set_image_survey_img_format(&mut self, layer: String, format: HiPSTileFormat) -> Result<(), JsValue>;
    // This method is used to change the root url when a better mirror has been found
    fn set_survey_url(&mut self, past_url: &str, new_url: &str) -> Result<(), JsValue>;

    fn read_pixel(&self, pos: &Vector2<f64>, base_url: &str) -> Result<JsValue, JsValue>;
    fn set_projection<Q: Projection>(self, width: f32, height: f32) -> App<Q>;

    // Catalog
    fn add_catalog(&mut self, name: String, table: JsValue, colormap: String);
    fn is_catalog_loaded(&mut self) -> bool;
    fn set_catalog_colormap(&mut self, name: String, colormap: String) -> Result<(), JsValue>;
    fn set_catalog_opacity(&mut self, name: String, opacity: f32) -> Result<(), JsValue>;
    fn set_kernel_strength(&mut self, name: String, strength: f32) -> Result<(), JsValue>;

    // Grid
    fn set_grid_cfg(&mut self, cfg: GridCfg) -> Result<(), JsValue>;

    // Coo System
    fn set_coo_system(&mut self, coo_system: CooSystem);

    // Localization
    fn press_left_button_mouse(&mut self, sx: f32, sy: f32);
    fn release_left_button_mouse(&mut self, sx: f32, sy: f32);

    fn set_center(&mut self, lonlat: &LonLatT<f64>);
    fn set_fov(&mut self, fov: Angle<f64>);

    //fn start_moving_to(&mut self, lonlat: &LonLatT<f64>);
    fn rotate_around_center(&mut self, theta: ArcDeg<f64>);
    fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64);
    fn reset_north_orientation(&mut self);

    // Accessors
    fn get_center(&self) -> LonLatT<f64>;
    fn get_clip_zoom_factor(&self) -> f64;
    fn get_fov(&self) -> f64;
    fn get_rotation_around_center(&self) -> &Angle<f64>;
    fn get_gl_canvas(&self) -> Option<js_sys::Object>;
    fn get_max_fov(&self) -> f64;
    fn get_coo_system(&self) -> &CooSystem;
    fn get_norder(&self) -> i32;

    // Utils
    fn project_line(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>>;
    fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>>;
    fn world_to_screen(&self, lonlat: &LonLatT<f64>) -> Result<Option<Vector2<f64>>, String>;
    fn world_to_screen_vec(&self, sources: &Vec<JsValue>) -> Result<Box<[f64]>, JsValue>;

    fn view_to_icrsj2000_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64>;
    fn icrsj2000_to_view_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64>;

    // UI
    fn over_ui(&self) -> bool;
}

use crate::downloader::request::Resource;
use crate::downloader::request::{allsky::Allsky, tile::Tile};
use crate::healpix::cell::HEALPixCell;

impl<P> AppTrait for App<P>
where
    P: Projection,
{
    fn over_ui(&self) -> bool {
        //self.ui.lock().pos_over_ui()
        false
    }

    fn is_catalog_loaded(&mut self) -> bool {
        if self.catalog_loaded {
            self.catalog_loaded = false;

            true
        } else {
            false
        }
    }

    fn is_ready(&self) -> Result<bool, JsValue> {
        let res = self.surveys.is_ready();

        Ok(res)
    }

    fn update(&mut self, _dt: DeltaTime) -> Result<(), JsValue> {
        //let available_tiles = self.run_tasks(dt)?;

        if let Some(InertiaAnimation {
            time_start_anim,
            d0,
            axis,
        }) = self.inertial_move_animation
        {
            let t = ((utils::get_current_time() - time_start_anim.as_millis()) / 1000.0) as f64;

            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where:
            // * k is the stiffness of the ressort
            // * m is its mass
            let w0 = 5.0;
            let d1 = Angle(0.0);
            // The angular distance goes from d0 to 0.0
            let d = d1 + (d0 - d1) * (w0 * t + 1.0) * ((-w0 * t).exp());
            /*let alpha = 1_f32 + (0_f32 - 1_f32) * (10_f32 * t + 1_f32) * (-10_f32 * t).exp();
            let alpha = alpha * alpha;
            let fov = start_fov * (1_f32 - alpha) + goal_fov * alpha;*/

            self.camera.rotate::<P>(&axis, d);
            self.look_for_new_tiles();
            // The threshold stopping criteria must be dependant
            // of the zoom level, in this case the initial angular distance
            // speed
            let thresh: Angle<f64> = d0 * 1e-3;
            if d < thresh {
                self.inertial_move_animation = None;
            }
        }

        // The rendering is done following these different situations:
        // - the camera has moved
        let has_camera_moved = self.camera.has_moved();

        {
            // Newly available tiles must lead to
            //al_core::log(&format!("{:?}, ready {:?}", self.surveys.urls, self.is_ready()));
            // 1. Surveys must be aware of the new available tiles
            //self.surveys.set_available_tiles(&available_tiles);
            // 2. Get the resolved tiles and push them to the image surveys
            /*let is_there_new_available_tiles = self
            .downloader
            .get_resolved_tiles(/*&available_tiles, */&mut self.surveys);*/
            let rscs = self.downloader.get_received_resources();
            let mut num_tile_received = 0;
            for rsc in rscs.into_iter() {
                match rsc {
                    Resource::Tile(tile) => {
                        let is_tile_root = tile.is_root;

                        if let Some(survey) = self.surveys.get_mut(&tile.get_hips_url()) {
                            if is_tile_root && tile.missing() {
                                al_core::log("tile root missing!");
                                survey.set_min_depth(survey.get_config().delta_depth());
                                // If at least one tile root is missing, query the allsky!
                                self.downloader.fetch(query::Allsky::new(survey.get_config()));

                            } else {
                                if is_tile_root {
                                    let is_missing = tile.missing();
                                    let Tile {
                                        cell,
                                        image,
                                        time_req,
                                        ..
                                    } = tile;
                                    let image = if is_missing {
                                        None
                                    } else {
                                        Some(image)
                                    };
                                    survey.add_tile(&cell, image, time_req);

                                    self.request_redraw = true;
                                } else {
                                    let cfg = survey.get_config();
                                    let coverage = survey.get_coverage();
                                    let texture_cell = tile.cell.get_texture_cell(cfg);
                                    let included_or_near_coverage = texture_cell.get_tile_cells(cfg)
                                        .any(|neighbor_tile_cell| {
                                            coverage.contains_tile(&neighbor_tile_cell)
                                        });
        
                                    if included_or_near_coverage {
                                        let is_missing = tile.missing();
                                        let Tile {
                                            cell,
                                            image,
                                            time_req,
                                            ..
                                        } = tile;

                                        let image = if is_missing {
                                            None
                                        } else {
                                            Some(image)
                                        };
                                        survey.add_tile(&cell, image, time_req);
        
                                        self.request_redraw = true;
                                    } else {
                                        self.downloader.cache_rsc(Resource::Tile(tile));
                                    }
                                }  
                            }
                        } else {
                            self.downloader.cache_rsc(Resource::Tile(tile));
                        }

                        num_tile_received += 1;
                    }
                    Resource::Allsky(allsky) => {
                        let hips_url = allsky.get_hips_url();

                        if let Some(survey) = self.surveys.get_mut(&hips_url) {
                            let is_missing = allsky.missing();
                            if is_missing {
                                survey.set_min_depth(survey.get_config().delta_depth());
                                // The allsky image is missing so we donwload all the tiles contained into
                                // the 0's cell
                                let cfg = survey.get_config();
                                for texture_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                                    for cell in texture_cell.get_tile_cells(cfg) {
                                        let query = query::Tile::new(&cell, cfg);
                                        self.tile_fetcher
                                            .append_base_tile(query, &mut self.downloader);
                                    }
                                }
                            } else {
                                al_core::log("received allsky2");

                                // tell the survey to not download tiles which order is <= 3 because the allsky
                                // give them already
                                survey.add_allsky(allsky);
                                // Once received ask for redraw
                                self.request_redraw = true;
                            }
                        }
                    },
                    Resource::PixelMetadata(metadata) => {
                        if let Some(survey) = self.surveys.get_mut(&metadata.hips_url) {
                            let mut cfg = survey.get_config_mut();
                            cfg.blank = metadata.value.blank;
                            cfg.offset = metadata.value.offset;
                            cfg.scale = metadata.value.scale;
                        }
                    }
                    Resource::MOC(moc) => {
                        let hips_url = moc.get_hips_url();
                        if let Some(survey) = self.surveys.get_mut(&hips_url) {
                            let MOC {
                                moc,
                                ..
                            } = moc;

                            survey.set_footprint_moc(moc);
                        }
                    }
                }
            }

            if num_tile_received > 0 {
                self.tile_fetcher
                    .notify(num_tile_received, &mut self.downloader);
                self.time_start_blending = Time::now();
            }
            //self.surveys.add_resolved_tiles(resolved_tiles);
            // 3. Try sending new tile requests after
            //self.downloader.try_sending_tile_requests()?;
        }

        // - there is at least one tile in its blending phase
        let blending_anim_occuring =
            (Time::now().0 - self.time_start_blending.0) < BLENDING_ANIM_DURATION;

        let mut start_fading = false;
        for survey in self.surveys.values() {
            if let Some(start_time) = survey.get_ready_time() {
                start_fading |= Time::now().0 - start_time.0 < BLENDING_ANIM_DURATION;
                if start_fading {
                    break;
                }
            }
        }

        self.rendering =
            blending_anim_occuring | has_camera_moved | self.request_redraw | start_fading;
        self.request_redraw = false;

        // Finally update the camera that reset the flag camera changed
        if has_camera_moved {
            if let Some(view) = self.surveys.get_view() {
                self.manager.update::<P>(&self.camera, view);
            }

            self.grid.update::<P>(&self.camera);
        }

        /*{
            let events = self.ui.lock().update();
            let mut events = events.lock().unwrap();

            for event in events.drain(..) {
                match event {
                    al_ui::Event::ImageSurveys(surveys) => self.set_image_surveys(surveys)?,
                    _ => { todo!() }
                    //al_ui::Event::ReverseLongitude(longitude_reversed) => { self.set_longitude_reversed(longitude_reversed)? }
                }
            }
        }*/

        Ok(())
    }

    fn reset_north_orientation(&mut self) {
        // Reset the rotation around the center if there is one
        self.camera.set_rotation_around_center::<P>(Angle(0.0));
        // Reset the camera position to its current position
        // this will keep the current position but reset the orientation
        // so that the north pole is at the top of the center.
        self.set_center(&self.get_center());
    }

    fn read_pixel(&self, pos: &Vector2<f64>, layer_id: &str) -> Result<JsValue, JsValue> {
        if let Some(lonlat) = self.screen_to_world(pos) {
            let survey = self
                .surveys
                .get_from_layer(layer_id)
                .ok_or(JsValue::from_str(&format!(
                    "Did not found the survey {:?}",
                    layer_id
                )))?;

            survey.read_pixel(&lonlat, &self.camera)
        } else {
            Err(JsValue::from_str(&format!(
                "{:?} is out of projection",
                pos
            )))
        }
    }

    fn draw(&mut self, force_render: bool) -> Result<(), JsValue> {
        /*let scene_redraw = self.rendering | force_render;
        let mut ui = self.ui.lock();
        //al_core::log(&format!("dpi {:?}", dpi));

        if scene_redraw {
            let shaders = &mut self.shaders;
            let gl = self.gl.clone();
            let camera = &self.camera;

            let grid = &mut self.grid;
            let surveys = &mut self.surveys;
            let catalogs = &self.manager;
            let colormaps = &self.colormaps;
            let fbo_view = &self.fbo_view;

            fbo_view.draw_onto(
                move || {
                    // Render the scene
                    gl.clear_color(0.00, 0.00, 0.00, 1.0);
                    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                    surveys.draw::<P>(camera, shaders, colormaps);

                    // Draw the catalog
                    catalogs.draw::<P>(&gl, shaders, camera, colormaps, fbo_view)?;

                    grid.draw::<P>(camera, shaders)?;

                    Ok(())
                },
                None,
            )?;

            // Reset the flags about the user action
            self.camera.reset();
        }

        let gl = self.gl.clone();

        let ui_redraw = ui.redraw_needed();
        if ui_redraw {
            let dpi  = self.camera.get_dpi();

            self.fbo_ui.draw_onto(move || {
                ui.draw(&gl, dpi)?;

                Ok(())
            }, None)?;
        }

        // If neither of the scene or the ui has been redraw then do nothing
        // otherwise, redraw both fbos on the screen
        if scene_redraw || ui_redraw {
            self.final_rendering_pass.draw_on_screen(&self.fbo_view);
            self.final_rendering_pass.draw_on_screen(&self.fbo_ui);
        }

        self.surveys.reset_frame();*/

        let scene_redraw = self.rendering | force_render;
        //let mut ui = self.ui.lock();
        //let ui_redraw = ui.redraw_needed();
        //if scene_redraw || ui_redraw {
        if scene_redraw {
            let shaders = &mut self.shaders;
            let gl = self.gl.clone();
            let camera = &mut self.camera;

            let grid = &mut self.grid;
            let surveys = &mut self.surveys;
            let catalogs = &self.manager;
            let colormaps = &self.colormaps;
            // Render the scene
            gl.clear_color(0.08, 0.08, 0.08, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            surveys.draw::<P>(camera, shaders, colormaps);

            // Draw the catalog
            //let fbo_view = &self.fbo_view;
            //catalogs.draw::<P>(&gl, shaders, camera, colormaps, fbo_view)?;
            catalogs.draw::<P>(&gl, shaders, camera, colormaps, None)?;
            grid.draw::<P>(camera, shaders)?;

            //let dpi  = self.camera.get_dpi();
            //ui.draw(&gl, dpi)?;

            // Reset the flags about the user action
            self.camera.reset();

            if self.rendering {
                self.surveys.reset_frame();
            }
        }

        Ok(())
    }

    fn set_image_surveys(&mut self, hipses: Vec<SimpleHiPS>) -> Result<(), JsValue> {
        self.surveys.set_image_surveys(hipses, &self.gl, &mut self.camera)?;

        for survey in self.surveys.surveys.values_mut() {
            // Request for the allsky first
            // The allsky is not mandatory present in a HiPS service but it is better to first try to search for it
            let tile_size = survey.get_config().get_tile_size();

            //Request the allsky for the small tile size
            if tile_size <= 128 {
                // Request the allsky
                self.downloader.fetch(query::Allsky::new(survey.get_config()));
                // tell the survey to not download tiles which order is <= 3 because the allsky
                // give them already
                survey.set_min_depth(survey.get_config().delta_depth());
            } else {
                let cfg = survey.get_config();
                for texture_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                    for cell in texture_cell.get_tile_cells(survey.get_config()) {
                        let query = query::Tile::new(&cell, survey.get_config());
                        self.tile_fetcher
                            .append_base_tile(query, &mut self.downloader);
                    }
                }
            }
            self.downloader.fetch(query::PixelMetadata::new(survey.get_config()));
        }

        // Once its added, request the tiles in the view (unless the viewer is at depth 0)
        self.look_for_new_tiles();
        self.request_redraw = true;
        self.grid.update::<P>(&self.camera);

        Ok(())
    }

    fn get_image_survey_color_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue> {
        self.surveys.get_image_survey_color_cfg(layer)
    }

    fn set_image_survey_color_cfg(
        &mut self,
        layer: String,
        meta: ImageSurveyMeta,
    ) -> Result<(), JsValue> {
        self.request_redraw = true;

        self.surveys.set_image_survey_color_cfg(layer, meta)
    }

    fn set_image_survey_img_format(&mut self, layer: String, format: HiPSTileFormat) -> Result<(), JsValue> {
        let survey = self.surveys.get_mut_from_layer(&layer)
            .ok_or(JsValue::from_str("Layer not found"))?;
        survey.set_img_format(format)?;
        // Request for the allsky first
        // The allsky is not mandatory present in a HiPS service but it is better to first try to search for it
        let cfg = survey.get_config();

        //Request the allsky for the small tile size
        let tile_size = survey.get_config().get_tile_size();
        //al_core::log(&format!("tile size {}", tile_size));
        //Request the allsky for the small tile size
        if tile_size <= 128 {
            // Request the allsky
            self.downloader.fetch(query::Allsky::new(survey.get_config()));
            // tell the survey to not download tiles which order is <= 3 because the allsky
            // give them already
            survey.set_min_depth(survey.get_config().delta_depth());
        } else {
            let cfg = survey.get_config();
            for texture_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                for cell in texture_cell.get_tile_cells(survey.get_config()) {
                    let query = query::Tile::new(&cell, survey.get_config());
                    self.tile_fetcher
                        .append_base_tile(query, &mut self.downloader);
                }
            }
        }
        self.downloader.fetch(query::PixelMetadata::new(survey.get_config()));
        

        // Once its added, request the tiles in the view (unless the viewer is at depth 0)
        self.look_for_new_tiles();

        self.request_redraw = true;

        Ok(())
    }

    // Width and height given are in pixels
    fn set_projection<Q: Projection>(mut self, width: f32, height: f32) -> App<Q> {
        // Recompute the ndc_to_clip
        self.camera.set_screen_size::<Q>(width, height);
        // Recompute clip zoom factor
        self.camera.set_aperture::<Q>(self.camera.get_aperture());

        self.surveys.set_projection::<Q>();

        self.look_for_new_tiles();
        self.request_redraw = true;

        App {
            gl: self.gl,
            tile_fetcher: self.tile_fetcher,

            colormaps: self.colormaps,
            fbo_ui: self.fbo_ui,
            fbo_view: self.fbo_view,
            final_rendering_pass: self.final_rendering_pass,
            manager: self.manager,
            exec: self.exec,
            resources: self.resources,

            inertial_move_animation: self.inertial_move_animation,
            prev_cam_position: self.prev_cam_position,
            prev_center: self.prev_center,
            out_of_fov: self.out_of_fov,
            tasks_finished: self.tasks_finished,
            catalog_loaded: self.catalog_loaded,
            shaders: self.shaders,
            camera: self.camera,
            downloader: self.downloader,
            surveys: self.surveys,
            time_start_blending: self.time_start_blending,
            request_redraw: self.request_redraw,
            rendering: self.rendering,
            grid: self.grid,
            p: std::marker::PhantomData,
        }
    }

    fn get_coo_system(&self) -> &CooSystem {
        self.camera.get_system()
    }

    fn get_max_fov(&self) -> f64 {
        P::aperture_start().0
    }

    fn add_catalog(&mut self, name: String, table: JsValue, colormap: String) {
        let mut exec_ref = self.exec.borrow_mut();
        let table = table;
        let c = self.colormaps.get(&colormap);

        exec_ref
            .spawner()
            .spawn(TaskType::ParseTableTask, async move {
                let mut stream = ParseTableTask::<[f32; 2]>::new(table);
                let mut results: Vec<Source> = vec![];

                while let Some(item) = stream.next().await {
                    let item: &[f32] = item.as_ref();
                    results.push(item.into());
                }

                let mut stream_sort = BuildCatalogIndex::new(results);
                while stream_sort.next().await.is_some() {}

                // The stream is finished, we get the sorted sources
                let results = stream_sort.sources;

                TaskResult::TableParsed {
                    name,
                    sources: results.into_boxed_slice(),
                    colormap: c,
                }
            });
    }

    fn resize(&mut self, width: f32, height: f32) {
        self.camera.set_screen_size::<P>(width, height);
        // resize the view fbo
        //self.fbo_view.resize(w as usize, h as usize);
        // resize the ui fbo
        //self.fbo_ui.resize(w as usize, h as usize);

        // launch the new tile requests
        self.look_for_new_tiles();
        self.manager.set_kernel_size(&self.camera);

        self.request_redraw = true;
    }

    fn set_survey_url(&mut self, past_url: &str, new_url: &str) -> Result<(), JsValue> {
        self.surveys.set_survey_url(past_url, new_url)
    }

    fn set_catalog_colormap(&mut self, name: String, colormap: String) -> Result<(), JsValue> {
        let colormap = self.colormaps.get(&colormap);

        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_colormap(colormap);

        self.request_redraw = true;

        Ok(())
    }

    fn set_catalog_opacity(&mut self, name: String, opacity: f32) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_alpha(opacity);

        self.request_redraw = true;

        Ok(())
    }

    fn set_kernel_strength(&mut self, name: String, strength: f32) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_strength(strength);

        self.request_redraw = true;

        Ok(())
    }

    fn set_grid_cfg(&mut self, cfg: GridCfg) -> Result<(), JsValue> {
        self.grid.set_cfg::<P>(cfg, &self.camera)?;
        self.request_redraw = true;

        Ok(())
    }

    fn set_coo_system(&mut self, coo_system: CooSystem) {
        self.camera.set_coo_system::<P>(coo_system);
        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    fn world_to_screen(&self, lonlat: &LonLatT<f64>) -> Result<Option<Vector2<f64>>, String> {
        //let lonlat = crate::coo_conversion::to_galactic(*lonlat);
        let model_pos_xyz = lonlat.vector();

        let screen_pos = P::view_to_screen_space(&model_pos_xyz, &self.camera);
        Ok(screen_pos)
    }

    /// World to screen projection
    ///
    /// sources coordinates are given in ICRS j2000
    fn world_to_screen_vec(&self, sources: &Vec<JsValue>) -> Result<Box<[f64]>, JsValue> {
        // Select the HiPS layer rendered lastly
        let mut r = Vec::with_capacity(sources.len() * 2);
        for s in sources {
            let source: S = s
                .into_serde()
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            let lonlat = LonLatT::new(ArcDeg(source.ra).into(), ArcDeg(source.dec).into());

            let xyz = lonlat.vector();
            if let Some(s_xy) = P::view_to_screen_space(&xyz, &self.camera) {
                r.push(s_xy.x);
                r.push(s_xy.y);
            } else {
                r.push(std::f64::NAN);
                r.push(std::f64::NAN);
            }
        }

        Ok(r.into_boxed_slice())
    }

    fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        // Select the HiPS layer rendered lastly
        P::screen_to_model_space(pos, &self.camera).map(|model_pos| model_pos.lonlat())
    }

    fn view_to_icrsj2000_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64> {
        let icrsj2000_pos: Vector4<_> = lonlat.vector();
        let view_system = self.camera.get_system();
        let (ra, dec) = math::lonlat::xyzw_to_radec(&coosys::apply_coo_system(
            &view_system,
            &CooSystem::ICRSJ2000,
            &icrsj2000_pos,
        ));

        LonLatT::new(ra, dec)
    }

    fn icrsj2000_to_view_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64> {
        let icrsj2000_pos: Vector4<_> = lonlat.vector();
        let view_system = self.camera.get_system();
        let (ra, dec) = math::lonlat::xyzw_to_radec(&coosys::apply_coo_system(
            &CooSystem::ICRSJ2000,
            &view_system,
            &icrsj2000_pos,
        ));

        LonLatT::new(ra, dec)
    }

    fn set_center(&mut self, lonlat: &LonLatT<f64>) {
        self.prev_cam_position = self.camera.get_center().truncate();
        let icrsj2000_pos: Vector4<_> = lonlat.vector();

        let view_pos = coosys::apply_coo_system(
            &CooSystem::ICRSJ2000,
            self.camera.get_system(),
            &icrsj2000_pos,
        );
        let rot = Rotation::from_sky_position(&view_pos);

        // Apply the rotation to the camera to go
        // to the next lonlat
        self.camera.set_rotation::<P>(&rot);
        self.look_for_new_tiles();

        // And stop the current inertia as well if there is one
        self.inertial_move_animation = None;
    }

    fn press_left_button_mouse(&mut self, _sx: f32, _sy: f32) {
        self.prev_center = self.camera.get_center().truncate();
        self.inertial_move_animation = None;
        self.out_of_fov = false;
    }

    fn release_left_button_mouse(&mut self, _sx: f32, _sy: f32) {
        // Check whether the center has moved
        // between the pressing and releasing
        // of the left button.
        //
        // Do not start inerting if:
        // * the mouse has not moved
        // * the mouse is out of the projection
        // * the mouse has not been moved since a certain
        //   amount of time
        let center = self.camera.get_center().truncate();
        let now = Time::now();
        let time_of_last_move = self.camera.get_time_of_last_move();
        //debug!(now);
        //debug!(time_of_last_move);
        if self.out_of_fov
            || self.prev_center == center
            || (now - time_of_last_move) >= DeltaTime::from_millis(30.0)
        {
            return;
        }

        /*if self.ui.lock().pos_over_ui() {
            return;
        }*/
        // Start inertia here

        // Angular distance between the previous and current
        // center position
        let x = self.prev_cam_position;
        let axis = x.cross(center).normalize();
        let d0 = math::vector::angle3(&x, &center);

        self.inertial_move_animation = Some(InertiaAnimation {
            d0,
            axis,
            time_start_anim: Time::now(),
        });

        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    /*fn start_moving_to(&mut self, lonlat: &LonLatT<f64>) {
        // Get the XYZ cartesian position from the lonlat
        let goal_pos: Vector4<f64> = lonlat.vector();

        // Convert these positions to rotations
        let start_anim_rot = *self.camera.get_rotation();
        let goal_anim_rot = Rotation::from_sky_position(&goal_pos);

        // Set the moving animation object
        self.move_animation = Some(MoveAnimation {
            time_start_anim: Time::now(),
            start_anim_rot,
            goal_anim_rot,
        });
    }*/

    fn rotate_around_center(&mut self, theta: ArcDeg<f64>) {
        self.camera.set_rotation_around_center::<P>(theta.into());
        // New tiles can be needed and some tiles can be removed
        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    fn get_rotation_around_center(&self) -> &Angle<f64> {
        self.camera.get_rotation_around_center()
    }

    fn set_fov(&mut self, fov: Angle<f64>) {
        // For the moment, no animation is triggered.
        // The fov is directly set
        self.camera.set_aperture::<P>(fov);
        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    fn project_line(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>> {
        let v1: Vector3<f64> = LonLatT::new(ArcDeg(lon1).into(), ArcDeg(lat1).into()).vector();
        let v2: Vector3<f64> = LonLatT::new(ArcDeg(lon2).into(), ArcDeg(lat2).into()).vector();

        line::project_along_great_circles::<P>(&v1, &v2, &self.camera)
    }

    fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        // Select the HiPS layer rendered lastly
        if let Some(w1) = P::screen_to_model_space(&Vector2::new(s1x, s1y), &self.camera) {
            if let Some(w2) = P::screen_to_model_space(&Vector2::new(s2x, s2y), &self.camera) {
                let cur_pos = w1.truncate();
                //let cur_pos = w1.truncate();
                let next_pos = w2.truncate();
                //let next_pos = w2.truncate();
                if cur_pos != next_pos {
                    let axis = cur_pos.cross(next_pos).normalize();
                    let d = math::vector::angle3(&cur_pos, &next_pos);
                    self.prev_cam_position = self.camera.get_center().truncate();

                    // Apply the rotation to the camera to
                    // go from the current pos to the next position
                    self.camera.rotate::<P>(&(-axis), d);
                    self.look_for_new_tiles();
                }
                return;
            }
        }

        self.out_of_fov = true;
    }

    // Accessors
    fn get_center(&self) -> LonLatT<f64> {
        self.camera.get_center().lonlat()
    }

    fn get_norder(&self) -> i32 {
        self.surveys.get_depth() as i32
    }

    fn get_clip_zoom_factor(&self) -> f64 {
        self.camera.get_clip_zoom_factor()
    }

    fn get_fov(&self) -> f64 {
        let deg: ArcDeg<f64> = self.camera.get_aperture().into();
        deg.0
    }

    fn get_gl_canvas(&self) -> Option<js_sys::Object> {
        self.gl.canvas()
    }

    fn is_rendering(&self) -> bool {
        self.rendering
    }
}
