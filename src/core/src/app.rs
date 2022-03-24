use crate::{angle::{Angle, ArcDeg}, async_task::TaskExecutor, async_task::{BuildCatalogIndex, ParseTableTask, TaskResult, TaskType}, buffer::TileDownloader, camera::CameraViewPort, coo_conversion::CooSystem, line, math, math::{LonLat, LonLatT}, projection::{Orthographic, Projection}, renderable::{
        catalog::{Manager, Source},
        grid::ProjetedGrid,
        survey::image_survey::ImageSurveys,
        labels::{RenderManager, TextRenderManager},
    }, shader::ShaderManager, shaders::Colormaps, time::DeltaTime, utils};

use al_core::{
    resources::Resources,
    pixel::PixelType, WebGlContext
};

use al_api::hips::SimpleHiPS;
use al_api::color::Color;
use al_api::hips::ImageSurveyMeta;
use al_api::grid::GridCfg;

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
use al_ui::{Gui, GuiRef};
pub struct App<P>
where
    P: Projection,
{
    pub gl: WebGlContext,

    ui: GuiRef,

    shaders: ShaderManager,
    camera: CameraViewPort,

    downloader: TileDownloader,
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
    pub resources: Resources,

    move_animation: Option<MoveAnimation>,
    zoom_animation: Option<ZoomAnimation>,
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

use crate::rotation::Rotation;
use al_api::colormap::Colormap;
use crate::renderable::survey::image_survey::Url;
struct MoveAnimation {
    start_anim_rot: Rotation<f64>,
    goal_anim_rot: Rotation<f64>,
    time_start_anim: Time,
}

/// State for inertia
struct InertiaAnimation {
    // Initial angular distance
    d0: Angle<f64>,
    // Vector of rotation
    axis: Vector3<f64>,
    // The time when the inertia begins
    time_start_anim: Time,
}

struct ZoomAnimation {
    time_start_anim: Time,
    start_fov: Angle<f64>,
    goal_fov: Angle<f64>,
    w0: f64,
}
use al_core::log::log;
use cgmath::Rad;
const BLEND_TILE_ANIM_DURATION: f32 = 500.0; // in ms
use crate::buffer::Tile;
use crate::time::Time;
use cgmath::InnerSpace;
use wasm_bindgen::JsCast;

use crate::projection::*;
type OrthoApp = App<Orthographic>;
type AitoffApp = App<Aitoff>;
type MollweideApp = App<Mollweide>;
type ArcApp = App<AzimuthalEquidistant>;
type TanApp = App<Gnomonic>;
type MercatorApp = App<Mercator>;

#[enum_dispatch]
pub enum AppType {
    OrthoApp,
    AitoffApp,
    MollweideApp,
    ArcApp,
    TanApp,
    MercatorApp
}

impl<P> App<P>
where
    P: Projection
{
    pub fn new(gl: &WebGlContext, aladin_div_name: &str, mut shaders: ShaderManager, resources: Resources) -> Result<Self, JsValue> {
        let gl = gl.clone();
        let exec = Rc::new(RefCell::new(TaskExecutor::new()));

        gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        // The tile buffer responsible for the tile requests
        let downloader = TileDownloader::new();

        let camera = CameraViewPort::new::<Orthographic>(&gl, CooSystem::ICRSJ2000);
        let screen_size = &camera.get_screen_size();

        let fbo_view = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;
        let fbo_ui = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;

        // The surveys storing the textures of the resolved tiles
        let surveys = ImageSurveys::new::<Orthographic>(&gl, &camera, &mut shaders);

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources)?;

        // Grid definition
        let grid = ProjetedGrid::new::<Orthographic>(&gl, &camera, &mut shaders, GridCfg {
            color: Color::new(0.0, 1.0, 0.0, 1.0),
            enabled: false,
            labels: true,
        })?;

        // Variable storing the location to move to
        let move_animation = None;
        let zoom_animation = None;
        let inertial_move_animation = None;
        let tasks_finished = false;
        let request_redraw = false;
        let rendering = true;
        let prev_cam_position = camera.get_center().truncate();
        let prev_center = Vector3::new(0.0, 1.0, 0.0);
        let out_of_fov = false;
        let catalog_loaded = false;

        let colormaps = Colormaps::new(&gl, &resources)?;

        let final_rendering_pass = RenderPass::new(&gl, screen_size.x as i32, screen_size.y as i32)?;
        let ui = Gui::new(aladin_div_name, &gl)?;

        Ok(App {
            gl,
            ui,
            
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

            move_animation,
            zoom_animation,
            inertial_move_animation,
            prev_cam_position,
            out_of_fov,

            tasks_finished,
            catalog_loaded,

            colormaps,
            p: std::marker::PhantomData
        })
    }

    fn look_for_new_tiles(&mut self) {
        // Move the views of the different active surveys
        self.surveys.refresh_views(&self.camera);
        // Loop over the surveys
        let mut not_available_tiles = Vec::new();
        for (survey_id, survey) in self.surveys.iter_mut() {
            //let num_cells = survey.get_view().num_of_cells();
            //let delta_depth = survey.get_textures().config().delta_depth();
            //let num_tiles = num_cells * (1 << (2 * delta_depth));
            //let mut already_available_tiles = Vec::with_capacity(num_tiles);

            let mut tile_cells = survey.get_view().get_cells()
                .map(|texture_cell| {
                    texture_cell.get_tile_cells(survey.get_textures().config())
                })
                .flatten()
                .collect::<Vec<_>>();

            if survey.get_view().get_depth() >= 3 {
                let tile_cells_ancestor = tile_cells.iter()
                    .map(|tile_cell| {
                        tile_cell.ancestor(3)
                    })
                    .collect::<HashSet<_>>();
            
                tile_cells.extend(tile_cells_ancestor);
            }

            for tile_cell in tile_cells {
                let already_available = survey.get_textures().contains_tile(&tile_cell);
                let is_tile_new = survey.get_view().is_new(&tile_cell);

                if already_available {
                    // Remove and append the texture with an updated
                    // time_request
                    //if is_tile_new {
                        // The viewport has new cells. So we can potentially do
                        // some GPU blending between tiles.
                        // Thus, we update the uniform
                    //    self.time_start_blending = Time::now();
                    //}

                    survey.get_textures_mut()
                        .update_priority(&tile_cell);

                    //already_available_tiles.push((tile_cell, is_tile_new));
                } else {
                    // Submit the request to the buffer
                    let format = survey.get_textures().config().format();
                    let root_url = survey_id.clone();
                    let tile = Tile {
                        root_url,
                        format,
                        cell: tile_cell,
                    };

                    not_available_tiles.push(tile);
                }
            }
        }
        // Launch the new tile requests
        self.downloader.request_tiles(not_available_tiles);
    }

    // Run async tasks:
    // - parsing catalogs
    // - copying textures to GPU
    // Return true when a task is complete. This always lead
    // to a redraw of aladin lite
    fn run_tasks(&mut self, dt: DeltaTime) -> Result<HashSet<Tile>, JsValue> {
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
                        &self.surveys.get_view().unwrap(),
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
    }
}

#[enum_dispatch(AppType)]
pub trait AppTrait {
    /// View
    fn is_ready(&self) -> Result<bool, JsValue>;
    fn resize(&mut self, width: f32, height: f32);
    // Low level method for updating the view
    fn update(&mut self, dt: DeltaTime, force: bool) -> Result<(), JsValue>;
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
    fn set_image_survey_color_cfg(&mut self, layer: String, meta: ImageSurveyMeta) -> Result<(), JsValue>;

    fn read_pixel(&self, x: f64, y: f64, base_url: &str) -> Result<PixelType, JsValue>;
    fn set_projection<Q: Projection>(self) -> App<Q>;
    fn set_longitude_reversed(&mut self, reversed: bool);

    // Catalog
    fn add_catalog(&mut self, name: String, table: JsValue, colormap: String);
    fn is_catalog_loaded(&mut self) -> bool;
    fn set_catalog_colormap(&mut self, name: String, colormap: String) -> Result<(), JsValue>;
    fn set_catalog_opacity(&mut self, name: String, opacity: f32) -> Result<(), JsValue>;
    fn set_kernel_strength(&mut self, name: String, strength: f32) -> Result<(), JsValue>;
    
    // Grid
    fn set_grid_cfg(&mut self, cfg: GridCfg);

    // Coo System
    fn set_coo_system(&mut self, coo_system: CooSystem);

    // Localization
    fn press_left_button_mouse(&mut self, sx: f32, sy: f32);
    fn release_left_button_mouse(&mut self, sx: f32, sy: f32);

    fn set_center(&mut self, lonlat: &LonLatT<f64>);
    fn set_fov(&mut self, fov: Angle<f64>);

    fn start_moving_to(&mut self, lonlat: &LonLatT<f64>);
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

    // Utils
    fn project_line(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>>;
    fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>>;
    fn world_to_screen(&self, lonlat: &LonLatT<f64>) -> Result<Option<Vector2<f64>>, String>;
    fn world_to_screen_vec(&self, sources: &Vec<JsValue>) -> Result<Vec<f64>, JsValue>;

    // UI
    fn over_ui(&self) -> bool;
}

impl<P> AppTrait for App<P>
where
    P: Projection
{
    fn over_ui(&self) -> bool {
        self.ui.lock().pos_over_ui()
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

    fn update(&mut self, dt: DeltaTime, force: bool) -> Result<(), JsValue> {
        let available_tiles = self.run_tasks(dt)?;
        let is_there_new_available_tiles = !available_tiles.is_empty();

        // Check if there is an move animation to do
        if let Some(MoveAnimation {
            start_anim_rot,
            goal_anim_rot,
            time_start_anim,
            ..
        }) = self.move_animation
        {
            let t = (utils::get_current_time() - time_start_anim.as_millis()) / 1000.0;

            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where:
            // * k is the stiffness of the ressort
            // * m is its mass
            let alpha = 1.0 + (0.0 - 1.0) * (5.0 * t + 1.0) * (-5.0 * t).exp();
            let p = start_anim_rot.slerp(&goal_anim_rot, alpha as f64);

            self.camera.set_rotation::<P>(&p);
            self.look_for_new_tiles();

            // Animation stop criteria
            if 1.0 - alpha < 1e-5 {
                self.move_animation = None;
            }
        }

        // Check if there is an zoom animation to do
        /*if let Some(ZoomAnimation {
            time_start_anim,
            start_fov,
            goal_fov,
            w0,
            ..
        }) = self.zoom_animation
        {
            let t = ((utils::get_current_time() - time_start_anim.as_millis()) / 1000.0) as f64;

            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where:
            // * k is the stiffness of the ressort
            // * m is its mass
            let fov = goal_fov + (start_fov - goal_fov) * (w0 * t + 1.0) * ((-w0 * t).exp());
            /*let alpha = 1_f32 + (0_f32 - 1_f32) * (10_f32 * t + 1_f32) * (-10_f32 * t).exp();
            let alpha = alpha * alpha;
            let fov = start_fov * (1_f32 - alpha) + goal_fov * alpha;*/

            self.camera.set_aperture::<P>(fov);
            self.look_for_new_tiles();

            // The threshold stopping criteria must be dependant
            // of the zoom level, in this case we stop when we get
            // to 1% before the goal fov
            let err = (fov - goal_fov).abs();
            let thresh = (start_fov - goal_fov).abs() * 1e-2;
            if err < thresh {
                self.zoom_animation = None;
            }
        }*/

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

        {
            // Newly available tiles must lead to
            if is_there_new_available_tiles {
                self.time_start_blending = Time::now();
            }

            // 1. Surveys must be aware of the new available tiles
            self.surveys.set_available_tiles(&available_tiles);
            // 2. Get the resolved tiles and push them to the image surveys
            let resolved_tiles = self
                .downloader
                .get_resolved_tiles(&available_tiles, &self.surveys);
            self.surveys.add_resolved_tiles(resolved_tiles);
            // 3. Try sending new tile requests after
            self.downloader.try_sending_tile_requests()?;
        }

        // The rendering is done following these different situations:
        // - the camera has moved
        let has_camera_moved = self.camera.has_moved();

        // - there is at least one tile in its blending phase
        let blending_anim_occuring =
            (Time::now().0 - self.time_start_blending.0) < BLEND_TILE_ANIM_DURATION;
        self.rendering = blending_anim_occuring
            | has_camera_moved
            | self.request_redraw;
        self.request_redraw = false;

        // Finally update the camera that reset the flag camera changed
        if has_camera_moved {
            if let Some(view) = self.surveys.get_view() {
                self.manager.update::<P>(&self.camera, view);
            }
        }

        self.grid.update::<P>(&self.camera, force);        
        {
            let events = self.ui.lock().update();
            let mut events = events.lock().unwrap();

            for event in events.drain(..) {
                match event {
                    al_ui::Event::ImageSurveys(surveys) => self.set_image_surveys(surveys)?,
                    _ => ()
                }
            }
        }

        Ok(())
    }


    fn reset_north_orientation(&mut self) {
        // Reset the rotation around the center if there is one
        self.camera.set_rotation_around_center::<P>(Angle(0.0));
        // Reset the camera position to its current position
        // this will keep the current position but reset the orientation
        // so that the north pole is at the top of the center.
        let center = self.get_center();
        self.set_center(&center);
    }

    fn read_pixel(
        &self,
        x: f64,
        y: f64,
        base_url: &str,
    ) -> Result<PixelType, JsValue> {
        let pos = Vector2::new(x, y);
        if let Some(lonlat) = self.screen_to_world(&pos) {
            self.surveys.read_pixel(&lonlat, &base_url.to_string())
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

            fbo_view.draw_onto(move || {
                // Render the scene
                gl.clear_color(0.00, 0.00, 0.00, 1.0);
                gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                surveys.draw::<P>(camera, shaders, colormaps);

                // Draw the catalog
                catalogs.draw::<P>(&gl, shaders, camera, colormaps, fbo_view)?;

                grid.draw::<P>(camera, shaders)?;

                Ok(())
            }, None)?;

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
        let mut ui = self.ui.lock();
        //al_core::log(&format!("dpi {:?}", dpi));
        let ui_redraw = ui.redraw_needed();

        if scene_redraw || ui_redraw {
            let shaders = &mut self.shaders;
            let gl = self.gl.clone();
            let camera = &self.camera;

            let grid = &mut self.grid;
            let surveys = &mut self.surveys;
            let catalogs = &self.manager;
            let colormaps = &self.colormaps;
            let fbo_view = &self.fbo_view;
            // Render the scene
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            surveys.draw::<P>(camera, shaders, colormaps);

            // Draw the catalog
            catalogs.draw::<P>(&gl, shaders, camera, colormaps, fbo_view)?;
            grid.draw::<P>(camera, shaders)?;

            let dpi  = self.camera.get_dpi();
            ui.draw(&gl, dpi)?;

            // Reset the flags about the user action
            self.camera.reset();
        }

        self.surveys.reset_frame();

        Ok(())
    }

    fn set_image_surveys(&mut self, hipses: Vec<SimpleHiPS>) -> Result<(), JsValue> {
        let new_survey_ids = self.surveys.set_image_surveys(
            hipses,
            &self.gl,
            &self.camera,
            self.exec.clone(),
            &self.colormaps,
        )?;
        self.downloader.clear_requests();

        if !new_survey_ids.is_empty() {
            for id in new_survey_ids.iter() {
                let config = &self.surveys.get(id).unwrap().get_textures().config;
                al_core::log::log(&format!("config: {:?}", config));
                self.downloader.request_base_tiles(config);
            }
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn get_image_survey_color_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue> {
        self.surveys.get_image_survey_color_cfg(layer)
    }

    fn set_image_survey_color_cfg(&mut self, layer: String, meta: ImageSurveyMeta) -> Result<(), JsValue> {
        self.request_redraw = true;

        self.surveys.set_image_survey_color_cfg(layer, meta)
    }

    fn set_projection<Q: Projection>(mut self) -> App<Q> {
        self.camera.set_projection::<Q>();
        self.surveys.set_projection::<Q>(
            &self.camera,
            &mut self.shaders,
        );

        self.look_for_new_tiles();
        self.request_redraw = true;

        App {
            p: std::marker::PhantomData,
            gl: self.gl,
            ui: self.ui,
            colormaps: self.colormaps,
            fbo_ui: self.fbo_ui,
            fbo_view: self.fbo_view,
            final_rendering_pass: self.final_rendering_pass,
            manager: self.manager,
            exec: self.exec,
            resources: self.resources,
            move_animation: self.move_animation,
            zoom_animation: self.zoom_animation,
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
            grid: self.grid
        }
    }

    fn get_coo_system(&self) -> &CooSystem {
        &self.camera.get_system()
    }

    fn get_max_fov(&self) -> f64 {
        P::aperture_start().0
    }

    fn set_longitude_reversed(&mut self, reversed: bool) {
        self.camera.set_longitude_reversed(reversed);
        self.surveys.set_longitude_reversed::<P>(
            reversed,
            &self.camera,
            &mut self.shaders,
            &self.resources,
        );

        self.look_for_new_tiles();

        self.request_redraw = true;
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
        let dpi = self.camera.get_dpi();

        let canvas = self.gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas.style().set_property("width", &format!("{}px", width.to_string())).unwrap();
        canvas.style().set_property("height", &format!("{}px", height.to_string())).unwrap();

        let w = (width as f32) * dpi;
        let h = (height as f32 ) * dpi;
        self.camera.set_screen_size::<P>(w, h);
        // resize the view fbo
        self.fbo_view.resize(w as usize, h as usize);
        // resize the ui fbo
        self.fbo_ui.resize(w as usize, h as usize);

        // launch the new tile requests
        self.look_for_new_tiles();
        self.manager.set_kernel_size(&self.camera);
    }

    fn set_catalog_colormap(
        &mut self,
        name: String,
        colormap: String,
    ) -> Result<(), JsValue> {
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

    fn set_grid_cfg(&mut self, cfg: GridCfg) {
        self.grid.set_cfg(cfg);
    }

    fn set_coo_system(&mut self, coo_system: CooSystem) {
        //let icrs2gal = coo_system == CooSystem::GAL && self.system == CooSystem::ICRSJ2000;
        //let gal2icrs = coo_system == CooSystem::ICRSJ2000 && self.system == CooSystem::GAL;

        self.camera.set_coo_system::<P>(coo_system);

        /*if icrs2gal {
            // rotate the camera around the center axis
            // to move the galactic plane straight to the center
            self.camera
                .set_rotation_around_center::<P>(ArcDeg(58.6).into());
        } else if gal2icrs {
            self.camera
                .set_rotation_around_center::<P>(ArcDeg(0.0).into());
        }*/

        self.request_redraw = true;
    }

    fn world_to_screen(
        &self,
        lonlat: &LonLatT<f64>,
    ) -> Result<Option<Vector2<f64>>, String> {
        //let lonlat = crate::coo_conversion::to_galactic(*lonlat);
        let model_pos_xyz = lonlat.vector();

        let screen_pos = P::model_to_screen_space(&model_pos_xyz, &self.camera);
        Ok(screen_pos)
    }

    /// World to screen projection
    ///
    /// sources coordinates are given in ICRS j2000
    fn world_to_screen_vec(
        &self,
        sources: &Vec<JsValue>,
    ) -> Result<Vec<f64>, JsValue> {
        let res: Vec<f64> = sources
            .into_iter()
            .filter_map(|s| {
                let source: S = s
                    .into_serde()
                    .map_err(|e| JsValue::from_str(&e.to_string()))
                    .unwrap();
                let lonlat = LonLatT::new(ArcDeg(source.ra).into(), ArcDeg(source.dec).into());
                //let lonlat = self.app.system.icrs_to_system(lonlat);

                let xyz = lonlat.vector();

                if let Some(s_xy) = P::model_to_screen_space(&xyz, &self.camera) {
                    Some(vec![s_xy.x, s_xy.y])
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        Ok(res)
    }

    fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        if let Some(model_pos) = P::screen_to_model_space(&pos, &self.camera) {
            //let model_pos = self.system.system_to_icrs_coo(model_pos);
            Some(model_pos.lonlat())
        } else {
            None
        }
    }

    fn set_center(&mut self, lonlat: &LonLatT<f64>) {
        self.prev_cam_position = self.camera.get_center().truncate();

        let xyz: Vector4<_> = lonlat.vector();
        let rot = Rotation::from_sky_position(&xyz);

        // Apply the rotation to the camera to go
        // to the next lonlat
        self.camera.set_rotation::<P>(&rot);
        self.look_for_new_tiles();

        // Stop the current animation if there is one
        self.move_animation = None;
        // And stop the current inertia as well if there is one
        self.inertial_move_animation = None;
    }

    fn press_left_button_mouse(&mut self, sx: f32, sy: f32) {
        self.prev_center = self.camera.get_center().truncate();
        self.inertial_move_animation = None;
        self.move_animation = None;
        self.out_of_fov = false;
    }

    fn release_left_button_mouse(&mut self, sx: f32, sy: f32) {
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

        if self.ui.lock().pos_over_ui() {
            return;
        }
        // Start inertia here

        // Angular distance between the previous and current
        // center position
        let x = self.prev_cam_position;
        let axis = x.cross(center).normalize();
        let d0 = math::ang_between_vect(&x, &center);

        self.inertial_move_animation = Some(InertiaAnimation {
            d0,
            axis,
            time_start_anim: Time::now(),
        });

        self.look_for_new_tiles();

        self.request_redraw = true;
    }

    fn start_moving_to(&mut self, lonlat: &LonLatT<f64>) {
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
    }

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

    fn project_line(
        &self,
        lon1: f64,
        lat1: f64,
        lon2: f64,
        lat2: f64,
    ) -> Vec<Vector2<f64>> {
        let v1: Vector3<f64> = LonLatT::new(ArcDeg(lon1).into(), ArcDeg(lat1).into()).vector();
        let v2: Vector3<f64> = LonLatT::new(ArcDeg(lon2).into(), ArcDeg(lat2).into()).vector();

        line::project::<P>(&v1, &v2, &self.camera)
    }

    fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        if let Some(w1) = P::screen_to_world_space(&Vector2::new(s1x, s1y), &self.camera) {
            if let Some(w2) = P::screen_to_world_space(&Vector2::new(s2x, s2y), &self.camera) {
                let r = self.camera.get_final_rotation();

                let cur_pos = r.rotate(&w1).truncate();
                //let cur_pos = w1.truncate();
                let next_pos = r.rotate(&w2).truncate();
                //let next_pos = w2.truncate();
                if cur_pos != next_pos {
                    let axis = cur_pos.cross(next_pos).normalize();
                    let d = math::ang_between_vect(&cur_pos, &next_pos);
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
}
