extern crate console_error_panic_hook;
extern crate fitsreader;
extern crate itertools_num;
extern crate num;
extern crate rand;
extern crate serde_derive;
extern crate serde_json;
extern crate task_async_executor;
extern crate num_traits;
use std::panic;

#[macro_use]
mod utils;

use wasm_bindgen::{prelude::*, JsCast};

mod async_task;
mod buffer;
pub mod camera;
mod cdshealpix;
mod color;
mod core;
mod healpix_cell;
mod image_fmt;
mod math;
pub mod renderable;
mod rotation;
mod shader;
mod shaders;
mod sphere_geometry;
mod time;
mod transfert_function;
mod line;
pub use image_fmt::FormatImageType;

use crate::{
    async_task::{TaskResult, TaskType},
    camera::CameraViewPort,
    math::{LonLat, LonLatT},
    renderable::{
        catalog::{Manager, Source},
        grid::ProjetedGrid,
        projection::{
            Aitoff, AzimuthalEquidistant, Gnomonic, Mercator, Mollweide, Orthographic, Projection,
        },
        Angle, ArcDeg,
    },
    shader::{Shader, ShaderManager},
};

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use cgmath::Vector4;

use crate::{buffer::TileDownloader, renderable::image_survey::ImageSurveys};
use async_task::TaskExecutor;
use web_sys::WebGl2RenderingContext;
struct App {
    gl: WebGl2Context,

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
    // Text example
    //text_manager: TextManager,

    // Finite State Machine declarations
    /*user_move_fsm: UserMoveSphere,
    user_zoom_fsm: UserZoom,
    move_fsm: MoveSphere,*/
    // Task executor
    exec: Rc<RefCell<TaskExecutor>>,
    resources: Resources,

    move_animation: Option<MoveAnimation>,
    zoom_animation: Option<ZoomAnimation>,
    inertial_move_animation: Option<InertiaAnimation>,
    prev_cam_position: Vector3<f64>,
    prev_center: Vector3<f64>,
    out_of_fov: bool,
    tasks_finished: bool,
}

#[derive(Debug, Deserialize)]
pub struct Resources(HashMap<String, String>);

impl Resources {
    pub fn get_filename<'a>(&'a self, name: &str) -> Option<&String> {
        self.0.get(name)
    }
}

use cgmath::{Vector2, Vector3};
use futures::stream::StreamExt; // for `next`

use crate::rotation::Rotation;
use crate::shaders::Colormap;
struct MoveAnimation {
    start_anim_rot: Rotation<f64>,
    goal_anim_rot: Rotation<f64>,
    time_start_anim: Time,
    goal_pos: Vector3<f64>,
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
    zooming: bool,
}

const BLEND_TILE_ANIM_DURATION: f32 = 500.0; // in ms
use crate::renderable::angle::ArcSec;
use crate::time::Time;

use crate::buffer::Tile;
use crate::renderable::image_survey::HiPS;
use cgmath::InnerSpace;
impl App {
    fn new(
        gl: &WebGl2Context,
        mut shaders: ShaderManager,
        resources: Resources,
    ) -> Result<Self, JsValue> {
        let gl = gl.clone();
        let exec = Rc::new(RefCell::new(TaskExecutor::new()));
        //gl.enable(WebGl2RenderingContext::BLEND);
        //gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);

        gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        //gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        let sdss = SimpleHiPS {
            properties: HiPSProperties {
                url: String::from("http://alasky.u-strasbg.fr/SDSS/DR9/color"),

                max_order: 10,
                frame: Frame {
                    label: String::from("J2000"),
                    system: String::from("J2000"),
                },
                tile_size: 512,
                format: HiPSFormat::Image {
                    format: String::from("jpeg"),
                },
                min_cutout: None,
                max_cutout: None,
            },
            color: HiPSColor::Color,
        };

        let camera = CameraViewPort::new::<Orthographic>(&gl);

        // The tile buffer responsible for the tile requests
        let downloader = TileDownloader::new();
        // The surveys storing the textures of the resolved tiles
        let mut surveys = ImageSurveys::new::<Orthographic>(&gl, &camera, &mut shaders, &resources);

        let (survey, color) = sdss.create(&gl, &camera, &surveys, exec.clone())?;
        surveys.add_simple_survey(survey, color, 0);

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources);

        // Grid definition
        let grid = ProjetedGrid::new::<Orthographic>(&gl, &camera, &mut shaders)?;

        // Variable storing the location to move to
        let move_animation = None;
        let zoom_animation = None;
        let inertial_move_animation = None;
        let tasks_finished = false;
        let request_redraw = false;
        let _start_render_time = Time::now();
        let rendering = true;
        let prev_cam_position = camera.get_center().truncate();
        let prev_center = Vector3::new(0.0, 1.0, 0.0);
        let out_of_fov = false;
        let app = App {
            gl,

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

            move_animation,
            zoom_animation,
            inertial_move_animation,
            prev_cam_position,
            out_of_fov,

            tasks_finished,
        };

        Ok(app)
    }

    fn look_for_new_tiles(&mut self) {
        // Move the views of the different active surveys
        self.surveys.refresh_views(&self.camera);
        // Loop over the surveys
        let mut tiles = Vec::new();
        for (survey_id, survey) in self.surveys.iter_mut() {
            let already_available_cells = {
                let mut already_available_cells = HashSet::new();

                let textures = survey.get_textures();
                let view = survey.get_view();

                let texture_cells_in_fov = view.get_cells();

                for texture_cell in texture_cells_in_fov.iter() {
                    for cell in texture_cell.get_tile_cells(&textures.config()) {
                        let already_available = textures.contains_tile(&cell);
                        let is_cell_new = view.is_new(&cell);

                        if already_available {
                            // Remove and append the texture with an updated
                            // time_request
                            if is_cell_new {
                                // New cells are
                                self.time_start_blending = Time::now();
                            }
                            already_available_cells.insert((cell, is_cell_new));
                        } else {
                            // Submit the request to the buffer
                            let format = textures.config().format();
                            let root_url = survey_id.clone();
                            let tile = Tile {
                                root_url,
                                format,
                                cell,
                            };

                            tiles.push(tile);
                        }
                    }
                }

                already_available_cells
            };
            let textures = survey.get_textures_mut();

            for (cell, is_new_cell) in already_available_cells {
                textures.update_priority(&cell, is_new_cell);
            }
        }
        // Launch the new tile requests
        self.downloader.request_tiles(tiles);
    }

    // Run async tasks:
    // - parsing catalogs
    // - copying textures to GPU
    // Return true when a task is complete. This always lead
    // to a redraw of aladin lite
    fn run_tasks<P: Projection>(&mut self, dt: DeltaTime) -> Result<HashSet<Tile>, JsValue> {
        //crate::log(&format!("last frame duration (ms): {:?}", dt));
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
                    removeLoadingInfo();
                    self.request_redraw = true;
                }
                TaskResult::TileSentToGPU { tile } => {
                    tiles_available.insert(tile);
                }
            }
        }

        Ok(tiles_available)
    }

    fn update<P: Projection>(&mut self, dt: DeltaTime) -> Result<(), JsValue> {
        let available_tiles = self.run_tasks::<P>(dt)?;
        let is_there_new_available_tiles = !available_tiles.is_empty();

        // Check if there is an move animation to do
        if let Some(MoveAnimation {
            start_anim_rot,
            goal_anim_rot,
            time_start_anim,
            goal_pos,
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
            let cursor_pos = self.camera.get_center().truncate();
            //let err = math::ang_between_vect(&goal_pos, &cursor_pos);

            //let thresh: Angle<f64> = ArcSec(2.0).into();
            if 1.0 - alpha < 1e-5 {
                self.move_animation = None;
            }
        }

        // Check if there is an zoom animation to do
        if let Some(ZoomAnimation {
            time_start_anim,
            start_fov,
            goal_fov,
            w0,
            zooming
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
            let thresh = (start_fov - goal_fov).abs()*1e-2;
            if err < thresh {
                self.zoom_animation = None;
            }
        }

        if let Some(InertiaAnimation {
            time_start_anim,
            d0,
            axis,
        }) = self.inertial_move_animation {
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
            let thresh: Angle<f64> = d0*1e-3;
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
        self.rendering = blending_anim_occuring | has_camera_moved | self.request_redraw;
        self.request_redraw = false;

        // Finally update the camera that reset the flag camera changed
        if has_camera_moved {
            self.manager
                .update::<P>(&self.camera, self.surveys.get_view().unwrap());
        }
        self.grid.update::<P>(&self.camera);

        Ok(())
    }

    fn render<P: Projection>(&mut self) -> Result<(), JsValue> {
        if self.rendering {
            // Render the scene
            self.gl.clear_color(0.08, 0.08, 0.08, 1.0);
            self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            //self.gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);
            self.surveys.draw::<P>(&self.camera, &mut self.shaders);
            self.gl.enable(WebGl2RenderingContext::BLEND);

            // Draw the catalog
            self.manager
                .draw::<P>(&self.gl, &mut self.shaders, &self.camera);

            self.grid.draw::<P>(&self.camera, &mut self.shaders)?;
            self.gl.disable(WebGl2RenderingContext::BLEND);

            // Reset the flags about the user action
            self.camera.reset();
        }

        Ok(())
    }

    fn set_simple_hips<P: Projection>(&mut self, hips: SimpleHiPS) -> Result<(), JsValue> {
        let (survey, color) =
            hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;
        let id = survey.get_textures().config().root_url.clone();

        let new_survey = self.surveys.add_simple_survey(survey, color, 0);

        if new_survey {
            self.downloader.clear_requests();
            let config = self.surveys.get(&id).unwrap().get_textures().config();
            self.downloader.request_base_tiles(&config);
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }
    fn set_overlay_simple_hips<P: Projection>(&mut self, hips: SimpleHiPS) -> Result<(), JsValue> {
        let (survey, color) =
            hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;
        let id = survey.get_textures().config().root_url.clone();

        let new_survey = self.surveys.add_simple_survey(survey, color, 1);

        if new_survey {
            self.downloader.clear_requests();
            let config = self.surveys.get(&id).unwrap().get_textures().config();
            self.downloader.request_base_tiles(&config);
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn remove_overlay(&mut self) {
        self.surveys.remove_overlay();
        self.request_redraw = true;
    }

    fn set_composite_hips<P: Projection>(&mut self, hipses: CompositeHiPS) -> Result<(), JsValue> {
        let mut surveys = Vec::new();
        let mut colors = Vec::new();
        let mut survey_ids = Vec::new();
        let mut survey_formats = Vec::new();

        for hips in hipses.into_iter() {
            let (survey, color) =
                hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;

            survey_ids.push(survey.get_id().to_string());
            survey_formats.push(survey.get_textures().config.format());

            surveys.push(survey);
            colors.push(color);
        }

        let new_survey_ids = self.surveys.add_composite_surveys(surveys, colors, 0);
        self.downloader.clear_requests();

        if !new_survey_ids.is_empty() {
            for id in new_survey_ids.iter() {
                let config = &self.surveys.get(id).unwrap().get_textures().config;
                self.downloader.request_base_tiles(config);
            }
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn set_overlay_composite_hips<P: Projection>(
        &mut self,
        hipses: CompositeHiPS,
    ) -> Result<(), JsValue> {
        let mut surveys = Vec::new();
        let mut colors = Vec::new();
        let mut survey_ids = Vec::new();
        let mut survey_formats = Vec::new();

        for hips in hipses.into_iter() {
            let (survey, color) =
                hips.create(&self.gl, &self.camera, &self.surveys, self.exec.clone())?;

            survey_ids.push(survey.get_id().to_string());
            survey_formats.push(survey.get_textures().config.format());

            surveys.push(survey);
            colors.push(color);
        }

        let new_survey_ids = self.surveys.add_composite_surveys(surveys, colors, 1);

        if !new_survey_ids.is_empty() {
            self.downloader.clear_requests();
            for id in new_survey_ids.iter() {
                let config = &self.surveys.get(id).unwrap().get_textures().config;
                self.downloader.request_base_tiles(config);
            }
            // Once its added, request its tiles
            self.look_for_new_tiles();
        }
        self.request_redraw = true;

        Ok(())
    }

    fn set_overlay_opacity(&mut self, opacity: f32) -> Result<(), JsValue> {
        self.surveys.set_overlay_opacity(opacity);
        self.request_redraw = true;
        Ok(())
    }

    fn set_projection<P: Projection>(&mut self) {
        self.camera.set_projection::<P>();
        self.surveys
            .set_projection::<P>(&self.camera, &mut self.shaders, &self.resources);

        self.look_for_new_tiles();
        self.request_redraw = true;
    }
    fn get_max_fov<P: Projection>(&self) -> f64 {
        P::aperture_start().0
    }
    fn set_longitude_reversed<P: Projection>(&mut self, reversed: bool) {
        /*if reversed {
            self.gl.cull_face(WebGl2RenderingContext::BACK);
        } else {
            self.gl.cull_face(WebGl2RenderingContext::FRONT);
        }*/

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
        exec_ref.spawner().spawn(TaskType::ParseTable, async {
            let mut stream = async_task::ParseTable::<[f32; 2]>::new(table);
            let mut results: Vec<Source> = vec![];

            while let Some(item) = stream.next().await {
                let item: &[f32] = item.as_ref();
                results.push(item.into());
            }

            let mut stream_sort = async_task::BuildCatalogIndex::new(results);
            while stream_sort.next().await.is_some() {}

            // The stream is finished, we get the sorted sources
            let results = stream_sort.sources;
            let colormap: Colormap = colormap.into();

            TaskResult::TableParsed {
                name,
                sources: results,
                colormap,
            }
        });
    }

    fn resize_window<P: Projection>(&mut self, width: f32, height: f32) {
        self.camera.set_screen_size::<P>(width, height);

        // Launch the new tile requests
        self.look_for_new_tiles();
        self.manager.set_kernel_size(&self.camera);
    }

    fn set_catalog_colormap(&mut self, name: String, colormap: Colormap) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_colormap(colormap);

        self.request_redraw = true;

        Ok(())
    }

    fn set_heatmap_opacity(&mut self, name: String, opacity: f32) -> Result<(), JsValue> {
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

    pub fn set_grid_color(&mut self, color: Color) {
        self.grid.set_color(color);
        self.request_redraw = true;
    }

    /*pub fn set_cutouts(&mut self, survey_url: &str, min_cutout: f32, max_cutout: f32) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut()
            .set_cutouts(min_cutout, max_cutout);

        Ok(())
    }

    pub fn set_transfer_func(&mut self, survey_url: &str, h: TransferFunction) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut()
            .set_transfer_function(h);

        Ok(())
    }

    pub fn set_fits_colormap(&mut self, survey_url: &str, colormap: Colormap) -> Result<(), String> {
        let survey = self.surveys.get(survey_url)
            .ok_or(format!("{} survey not found!", survey_url))?;
        survey.config_mut().set_fits_colormap(colormap);

        Ok(())
    }*/

    pub fn set_grid_opacity(&mut self, _alpha: f32) {
        //self.grid.set_alpha(alpha);
    }

    pub fn enable_grid<P: Projection>(&mut self) {
        self.grid.enable::<P>(&self.camera);
        self.request_redraw = true;
    }

    pub fn hide_grid_labels(&mut self) {
        self.grid.hide_labels(&self.camera);
        self.request_redraw = true;
    }

    pub fn show_grid_labels(&mut self) {
        self.grid.show_labels();
        self.request_redraw = true;
    }

    pub fn disable_grid(&mut self) {
        self.grid.disable(&self.camera);
        self.request_redraw = true;
    }

    pub fn world_to_screen<P: Projection>(
        &self,
        lonlat: &LonLatT<f64>,
    ) -> Result<Option<Vector2<f64>>, String> {
        let model_pos_xyz = lonlat.vector();
        let screen_pos = P::model_to_screen_space(&model_pos_xyz, &self.camera);
        Ok(screen_pos)
    }

    pub fn screen_to_world<P: Projection>(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        if let Some(model_pos) = P::screen_to_model_space(pos, &self.camera) {
            Some(model_pos.lonlat())
        } else {
            None
        }
    }

    pub fn set_center<P: Projection>(&mut self, lonlat: &LonLatT<f64>) {
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

    pub fn press_left_button_mouse(&mut self) {
        self.prev_center = self.camera.get_center().truncate();
        self.inertial_move_animation = None;
        self.move_animation = None;
        self.out_of_fov = false;
    }

    pub fn release_left_button_mouse(&mut self) {
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
        if self.out_of_fov || self.prev_center == center || (now - time_of_last_move) >= DeltaTime::from_millis(30.0) {
            return;
        }
        // Start inertia here

        // Angular distance between the previous and current
        // center position
        let x = self.prev_cam_position;
        let axis = x.cross(center)
            .normalize();
        let d0 = math::ang_between_vect(&x, &center);

        self.inertial_move_animation = Some(
            InertiaAnimation {
                d0,
                axis,
                time_start_anim: Time::now()
            }
        );
    }

    pub fn start_moving_to<P: Projection>(&mut self, lonlat: &LonLatT<f64>) {
        // Get the XYZ cartesian position from the lonlat
        let _cursor_pos = self.camera.get_center();
        let goal_pos: Vector4<f64> = lonlat.vector();

        // Convert these positions to rotations
        let start_anim_rot = *self.camera.get_rotation();
        let goal_anim_rot = Rotation::from_sky_position(&goal_pos);

        // Set the moving animation object
        self.move_animation = Some(MoveAnimation {
            time_start_anim: Time::now(),
            start_anim_rot,
            goal_anim_rot,
            goal_pos: goal_pos.truncate(),
        });
    }

    pub fn start_zooming_to<P: Projection>(&mut self, fov: Angle<f64>) {
        // Code handling the update of the animation
        // when a wheel event is detected AND current
        // zooming is not achieved!
        /*if let Some(zoom) = &mut self.zoom_animation {
            let zooming = fov < self.camera.get_aperture();
            if zooming != zoom.zooming {
                let start_fov = self.camera.get_aperture();
                let goal_fov = fov;
                let zooming = goal_fov < start_fov;
                // Set the moving animation object
                self.zoom_animation = Some(ZoomAnimation {
                    time_start_anim: Time::now(),
                    start_fov,
                    goal_fov,
                    w0: 35.0,
                    zooming
                });
            } else {
                //crate::log("sdfsdf");
                zoom.goal_fov = fov;
                // Change de time start anim so that the current time
                // matches the current fov after changing the goal fov
                let d = (self.camera.get_aperture() - zoom.goal_fov)/(zoom.start_fov - zoom.goal_fov); // in (0, 1)
                if d.0 > 0.0 && d.0 <= 1.0 {
                    crate::log("sdfsd");
                    let u = -math::lambert_wm1(-(d.0 as f32)/std::f32::consts::E) - 1.0;
                    let t = u / (zoom.w0 as f32); // sec
                    let st = Time::now() - Time(t*1000.0);
                    zoom.time_start_anim = Time(st.0);
                }
            }
        }*/

        // Convert these positions to rotations
        /*let start_fov = self.camera.get_aperture();
        let goal_fov = fov;
        let zooming = goal_fov < start_fov;
        // Set the moving animation object
        self.zoom_animation = Some(ZoomAnimation {
            time_start_anim: Time::now(),
            start_fov,
            goal_fov,
            w0: 35.0,
            zooming
        });*/

        // For the moment, no animation is triggered.
        // The fov is directly set
        self.camera.set_aperture::<P>(fov);
        self.look_for_new_tiles();
    }

    pub fn project_line<P: Projection>(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>> {
        let v1: Vector3<f64> = LonLatT::new(ArcDeg(lon1).into(), ArcDeg(lat1).into()).vector();
        let v2: Vector3<f64> = LonLatT::new(ArcDeg(lon2).into(), ArcDeg(lat2).into()).vector();

        line::project::<P>(&v1, &v2, &self.camera)
    }

    pub fn go_from_to<P: Projection>(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        if let Some(pos1) = self.screen_to_world::<P>(&Vector2::new(s1x, s1y)) {
            if let Some(pos2) = self.screen_to_world::<P>(&Vector2::new(s2x, s2y)) {
                let model2world = self.camera.get_m2w();
                let m1: Vector4<f64> = pos1.vector();
                let w1 = model2world * m1;
                let m2: Vector4<f64> = pos2.vector();
                let w2 = model2world * m2;
        
                let r = self.camera.get_rotation();
        
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

        /*// Stop the current animation if there is one
        self.move_animation = None;
        // And stop the current inertia as well if there is one
        self.inertial_move_animation = None;*/
        

        
    }

    pub fn set_fov<P: Projection>(&mut self, fov: &Angle<f64>) {
        // Change the camera rotation
        self.camera.set_aperture::<P>(*fov);
        self.look_for_new_tiles();
    }

    // Accessors
    fn get_center<P: Projection>(&self) -> LonLatT<f64> {
        //let center_pos = self.camera.compute_center_model_pos::<P>();
        self.camera.get_center().lonlat()
    }
    
    fn get_clip_zoom_factor(&self) -> f64 {
        self.camera.get_clip_zoom_factor()
    }

    fn get_fov(&self) -> f64 {
        let deg: ArcDeg<f64> = self.camera.get_aperture().into();
        deg.0
    }
}

#[derive(Clone)]
pub struct WebGl2Context {
    inner: Rc<WebGl2RenderingContext>,
}

impl WebGl2Context {
    fn new() -> WebGl2Context {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let canvas = document
            .get_elements_by_class_name("aladin-imageCanvas")
            .get_with_index(0)
            .unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context_options = js_sys::JSON::parse(&"{\"antialias\":false}").unwrap();
        let inner = Rc::new(
            canvas
                .get_context_with_context_options("webgl2", context_options.as_ref())
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap(),
        );

        WebGl2Context { inner }
    }
}

use std::ops::Deref;
impl Deref for WebGl2Context {
    type Target = WebGl2RenderingContext;

    fn deref(&self) -> &WebGl2RenderingContext {
        &self.inner
    }
}

enum ProjectionType {
    Aitoff,
    MollWeide,
    Arc,
    Mercator,
    Ortho,
    Gnomonic,
}

impl ProjectionType {
    fn set_projection(&mut self, app: &mut App, name: String) -> Result<(), JsValue> {
        match name.as_str() {
            "aitoff" => {
                app.set_projection::<Aitoff>();
                *self = ProjectionType::Aitoff;
                Ok(())
            },
            "sinus" => {
                app.set_projection::<Orthographic>();
                *self = ProjectionType::Ortho;
                Ok(())
            },
            "mollweide" => {
                app.set_projection::<Mollweide>();
                *self = ProjectionType::MollWeide;
                Ok(())
            },
            "arc" => {
                app.set_projection::<AzimuthalEquidistant>();
                *self = ProjectionType::Arc;
                Ok(())
            },
            "tan" => {
                app.set_projection::<Gnomonic>();
                *self = ProjectionType::Gnomonic;
                Ok(())
            },
            "mercator" => {
                app.set_projection::<Mercator>();
                *self = ProjectionType::Mercator;
                Ok(())
            },
            _ => Err(format!("{} is not a valid projection name. aitoff, arc, sinus, tan, mollweide and mercator are accepted", name).into())
        }
    }

    fn set_longitude_reversed(&mut self, app: &mut App, reversed: bool) {
        match self {
            ProjectionType::Aitoff => app.set_longitude_reversed::<Aitoff>(reversed),
            ProjectionType::MollWeide => app.set_longitude_reversed::<Mollweide>(reversed),
            ProjectionType::Ortho => app.set_longitude_reversed::<Orthographic>(reversed),
            ProjectionType::Arc => app.set_longitude_reversed::<AzimuthalEquidistant>(reversed),
            ProjectionType::Gnomonic => app.set_longitude_reversed::<Gnomonic>(reversed),
            ProjectionType::Mercator => app.set_longitude_reversed::<Mercator>(reversed),
        };
    }

    fn set_catalog_colormap(
        &self,
        app: &mut App,
        name: String,
        colormap: Colormap,
    ) -> Result<(), JsValue> {
        app.set_catalog_colormap(name, colormap)
    }

    fn world_to_screen(
        &self,
        app: &App,
        lonlat: &LonLatT<f64>,
    ) -> Result<Option<Vector2<f64>>, String> {
        match self {
            ProjectionType::Aitoff => app.world_to_screen::<Aitoff>(lonlat),
            ProjectionType::MollWeide => app.world_to_screen::<Mollweide>(lonlat),
            ProjectionType::Ortho => app.world_to_screen::<Orthographic>(lonlat),
            ProjectionType::Arc => app.world_to_screen::<AzimuthalEquidistant>(lonlat),
            ProjectionType::Gnomonic => app.world_to_screen::<Gnomonic>(lonlat),
            ProjectionType::Mercator => app.world_to_screen::<Mercator>(lonlat),
        }
    }

    fn get_max_fov(&self, app: &App) -> f64 {
        match self {
            ProjectionType::Aitoff => app.get_max_fov::<Aitoff>(),
            ProjectionType::MollWeide => app.get_max_fov::<Mollweide>(),
            ProjectionType::Ortho => app.get_max_fov::<Orthographic>(),
            ProjectionType::Arc => app.get_max_fov::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.get_max_fov::<Gnomonic>(),
            ProjectionType::Mercator => app.get_max_fov::<Mercator>(),
        }
    }

    fn screen_to_world(&self, app: &App, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        match self {
            ProjectionType::Aitoff => app.screen_to_world::<Aitoff>(pos),
            ProjectionType::MollWeide => app.screen_to_world::<Mollweide>(pos),
            ProjectionType::Ortho => app.screen_to_world::<Orthographic>(pos),
            ProjectionType::Arc => app.screen_to_world::<AzimuthalEquidistant>(pos),
            ProjectionType::Gnomonic => app.screen_to_world::<Gnomonic>(pos),
            ProjectionType::Mercator => app.screen_to_world::<Mercator>(pos),
        }
    }

    fn go_from_to(&self, app: &mut App, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        match self {
            ProjectionType::Aitoff => app.go_from_to::<Aitoff>(s1x, s1y, s2x, s2y),
            ProjectionType::MollWeide => app.go_from_to::<Mollweide>(s1x, s1y, s2x, s2y),
            ProjectionType::Ortho => app.go_from_to::<Orthographic>(s1x, s1y, s2x, s2y),
            ProjectionType::Arc => app.go_from_to::<AzimuthalEquidistant>(s1x, s1y, s2x, s2y),
            ProjectionType::Gnomonic => app.go_from_to::<Gnomonic>(s1x, s1y, s2x, s2y),
            ProjectionType::Mercator => app.go_from_to::<Mercator>(s1x, s1y, s2x, s2y),
        }
    }

    fn update(&mut self, app: &mut App, dt: DeltaTime) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.update::<Aitoff>(dt),
            ProjectionType::MollWeide => app.update::<Mollweide>(dt),
            ProjectionType::Ortho => app.update::<Orthographic>(dt),
            ProjectionType::Arc => app.update::<AzimuthalEquidistant>(dt),
            ProjectionType::Gnomonic => app.update::<Gnomonic>(dt),
            ProjectionType::Mercator => app.update::<Mercator>(dt),
        }
    }

    fn render(&mut self, app: &mut App) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.render::<Aitoff>()?,
            ProjectionType::MollWeide => app.render::<Mollweide>()?,
            ProjectionType::Ortho => app.render::<Orthographic>()?,
            ProjectionType::Arc => app.render::<AzimuthalEquidistant>()?,
            ProjectionType::Gnomonic => app.render::<Gnomonic>()?,
            ProjectionType::Mercator => app.render::<Mercator>()?,
        };

        Ok(())
    }

    pub fn add_catalog(&mut self, app: &mut App, name: String, table: JsValue, colormap: String) {
        app.add_catalog(name, table, colormap);
    }

    pub fn set_simple_hips(&mut self, app: &mut App, hips: SimpleHiPS) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_simple_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_simple_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_simple_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_simple_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_simple_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_simple_hips::<Mercator>(hips),
        }
    }

    pub fn set_composite_hips(
        &mut self,
        app: &mut App,
        hips: CompositeHiPS,
    ) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_composite_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_composite_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_composite_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_composite_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_composite_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_composite_hips::<Mercator>(hips),
        }
    }

    pub fn set_overlay_simple_hips(
        &mut self,
        app: &mut App,
        hips: SimpleHiPS,
    ) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_overlay_simple_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_overlay_simple_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_overlay_simple_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_overlay_simple_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_overlay_simple_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_overlay_simple_hips::<Mercator>(hips),
        }
    }

    pub fn remove_overlay_hips(&mut self, app: &mut App) -> Result<(), JsValue> {
        app.remove_overlay();

        Ok(())
    }

    pub fn set_overlay_composite_hips(
        &mut self,
        app: &mut App,
        hips: CompositeHiPS,
    ) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_overlay_composite_hips::<Aitoff>(hips),
            ProjectionType::MollWeide => app.set_overlay_composite_hips::<Mollweide>(hips),
            ProjectionType::Ortho => app.set_overlay_composite_hips::<Orthographic>(hips),
            ProjectionType::Arc => app.set_overlay_composite_hips::<AzimuthalEquidistant>(hips),
            ProjectionType::Gnomonic => app.set_overlay_composite_hips::<Gnomonic>(hips),
            ProjectionType::Mercator => app.set_overlay_composite_hips::<Mercator>(hips),
        }
    }
    pub fn set_overlay_opacity(&mut self, app: &mut App, opacity: f32) -> Result<(), JsValue> {
        app.set_overlay_opacity(opacity)
    }

    pub fn resize(&mut self, app: &mut App, width: f32, height: f32) {
        match self {
            ProjectionType::Aitoff => app.resize_window::<Aitoff>(width, height),
            ProjectionType::MollWeide => app.resize_window::<Mollweide>(width, height),
            ProjectionType::Ortho => app.resize_window::<Orthographic>(width, height),
            ProjectionType::Arc => app.resize_window::<AzimuthalEquidistant>(width, height),
            ProjectionType::Gnomonic => app.resize_window::<Gnomonic>(width, height),
            ProjectionType::Mercator => app.resize_window::<Mercator>(width, height),
        };
    }

    pub fn set_kernel_strength(
        &mut self,
        app: &mut App,
        name: String,
        strength: f32,
    ) -> Result<(), JsValue> {
        app.set_kernel_strength(name, strength)
    }

    pub fn set_heatmap_opacity(
        &mut self,
        app: &mut App,
        name: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        app.set_heatmap_opacity(name, opacity)
    }

    pub fn set_center(&mut self, app: &mut App, lonlat: LonLatT<f64>) {
        match self {
            ProjectionType::Aitoff => app.set_center::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.set_center::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.set_center::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.set_center::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.set_center::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.set_center::<Mercator>(&lonlat),
        };
    }

    pub fn start_moving_to(&mut self, app: &mut App, lonlat: LonLatT<f64>) {
        match self {
            ProjectionType::Aitoff => app.start_moving_to::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.start_moving_to::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.start_moving_to::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.start_moving_to::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.start_moving_to::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.start_moving_to::<Mercator>(&lonlat),
        };
    }

    pub fn start_zooming_to(&mut self, app: &mut App, fov: Angle<f64>) {
        match self {
            ProjectionType::Aitoff => app.start_zooming_to::<Aitoff>(fov),
            ProjectionType::MollWeide => app.start_zooming_to::<Mollweide>(fov),
            ProjectionType::Ortho => app.start_zooming_to::<Orthographic>(fov),
            ProjectionType::Arc => app.start_zooming_to::<AzimuthalEquidistant>(fov),
            ProjectionType::Gnomonic => app.start_zooming_to::<Gnomonic>(fov),
            ProjectionType::Mercator => app.start_zooming_to::<Mercator>(fov),
        };
    }

    pub fn project_line(&self, app: &App, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>> {
        match self {
            ProjectionType::Aitoff => app.project_line::<Aitoff>(lon1, lat1, lon2, lat2),
            ProjectionType::MollWeide => app.project_line::<Mollweide>(lon1, lat1, lon2, lat2),
            ProjectionType::Ortho => app.project_line::<Orthographic>(lon1, lat1, lon2, lat2),
            ProjectionType::Arc => app.project_line::<AzimuthalEquidistant>(lon1, lat1, lon2, lat2),
            ProjectionType::Gnomonic => app.project_line::<Gnomonic>(lon1, lat1, lon2, lat2),
            ProjectionType::Mercator => app.project_line::<Mercator>(lon1, lat1, lon2, lat2),
        }
    }

    pub fn set_fov(&mut self, app: &mut App, fov: Angle<f64>) {
        match self {
            ProjectionType::Aitoff => app.set_fov::<Aitoff>(&fov),
            ProjectionType::MollWeide => app.set_fov::<Mollweide>(&fov),
            ProjectionType::Ortho => app.set_fov::<Orthographic>(&fov),
            ProjectionType::Arc => app.set_fov::<AzimuthalEquidistant>(&fov),
            ProjectionType::Gnomonic => app.set_fov::<Gnomonic>(&fov),
            ProjectionType::Mercator => app.set_fov::<Mercator>(&fov),
        };
    }

    pub fn get_center(&self, app: &App) -> LonLatT<f64> {
        match self {
            ProjectionType::Aitoff => app.get_center::<Aitoff>(),
            ProjectionType::MollWeide => app.get_center::<Mollweide>(),
            ProjectionType::Ortho => app.get_center::<Orthographic>(),
            ProjectionType::Arc => app.get_center::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.get_center::<Gnomonic>(),
            ProjectionType::Mercator => app.get_center::<Mercator>(),
        }
    }

    pub fn enable_grid(&mut self, app: &mut App) {
        match self {
            ProjectionType::Aitoff => app.enable_grid::<Aitoff>(),
            ProjectionType::MollWeide => app.enable_grid::<Mollweide>(),
            ProjectionType::Ortho => app.enable_grid::<Orthographic>(),
            ProjectionType::Arc => app.enable_grid::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.enable_grid::<Gnomonic>(),
            ProjectionType::Mercator => app.enable_grid::<Mercator>(),
        };
    }
    pub fn hide_grid_labels(&mut self, app: &mut App) {
        app.hide_grid_labels();
    }
    pub fn show_grid_labels(&mut self, app: &mut App) {
        app.show_grid_labels();
    }
    pub fn disable_grid(&mut self, app: &mut App) {
        app.disable_grid();
    }

    pub fn set_grid_color(&mut self, app: &mut App, color: Color) {
        app.set_grid_color(color);
    }
    /*pub fn set_cutouts(&mut self, app: &mut App, min_cutout: f32, max_cutout: f32) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::MollWeide => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Ortho => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Arc => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Mercator => app.set_cutouts(min_cutout, max_cutout),
        };
    }

    pub fn set_transfer_func(&mut self, app: &mut App, h: TransferFunction) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_transfer_func(h),
            ProjectionType::MollWeide => app.set_transfer_func(h),
            ProjectionType::Ortho => app.set_transfer_func(h),
            ProjectionType::Arc => app.set_transfer_func(h),
            ProjectionType::Mercator => app.set_transfer_func(h),
        };
    }

    pub fn set_fits_colormap(&mut self, app: &mut App, colormap: Colormap) -> Result<(), String> {
        match self {
            ProjectionType::Aitoff => app.set_fits_colormap(colormap),
            ProjectionType::MollWeide => app.set_fits_colormap(colormap),
            ProjectionType::Ortho => app.set_fits_colormap(colormap),
            ProjectionType::Arc => app.set_fits_colormap(colormap),
            ProjectionType::Mercator => app.set_fits_colormap(colormap),
        };
    }*/

    pub fn set_grid_opacity(&mut self, app: &mut App, alpha: f32) {
        app.set_grid_opacity(alpha);
    }
}

use crate::time::DeltaTime;
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

#[wasm_bindgen(module = "/catalog.js")]
extern "C" {
    fn removeLoadingInfo();
}

use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct FileSrc {
    pub id: String,
    pub content: String,
}
use crate::transfert_function::TransferFunction;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Frame {
    pub label: String,
    pub system: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HiPSProperties {
    pub url: String,

    pub max_order: u8,
    pub frame: Frame,
    pub tile_size: i32,
    pub min_cutout: Option<f32>,
    pub max_cutout: Option<f32>,
    pub format: HiPSFormat,
}

#[derive(Deserialize, Debug)]
pub enum HiPSFormat {
    FITSImage { bitpix: i32 },
    Image { format: String },
}

#[derive(Deserialize, Debug)]
pub enum HiPSColor {
    Grayscale2Colormap {
        colormap: String,
        transfer: String,
    },
    Grayscale2Color {
        color: [f32; 3],
        transfer: String,
        k: f32, // contribution of the component
    },
    Color,
}

#[derive(Deserialize, Debug)]
pub struct SimpleHiPS {
    properties: HiPSProperties,
    color: HiPSColor,
}

#[derive(Deserialize, Debug)]
pub struct CompositeHiPS {
    hipses: Vec<SimpleHiPS>,
}
use std::iter::IntoIterator;
impl IntoIterator for CompositeHiPS {
    type Item = SimpleHiPS;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.hipses.into_iter()
    }
}

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
    pub fn update(&mut self, dt: f32) -> Result<(), JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get back the
        // world coordinates of the center of projection in (ra, dec)
        self.projection.update(
            &mut self.app,
            // Time of the previous frame rendering
            self.dt,
        )?;

        Ok(())
    }

    /// Update our WebGL Water application.
    pub fn render(&mut self) -> Result<(), JsValue> {
        self.projection.render(&mut self.app)?;

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

    /// Change grid opacity
    pub fn set_grid_opacity(&mut self, alpha: f32) -> Result<(), JsValue> {
        self.projection.set_grid_opacity(&mut self.app, alpha);

        Ok(())
    }

    /*#[wasm_bindgen(js_name = setCutouts)]
    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) -> Result<(), JsValue> {
        self.projection.set_cutouts(&mut self.app, min_cutout, max_cutout).map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setFitsColormap)]
    pub fn set_fits_colormap(&mut self, colormap: String) -> Result<(), JsValue> {
        let colormap = Colormap::new(&colormap);
        self.projection.set_fits_colormap(&mut self.app, colormap).map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setTransferFunction)]
    pub fn set_transfer_func(&mut self, id: String) -> Result<(), JsValue> {
        let h = TransferFunction::new(&id);
        self.projection.set_transfer_func(&mut self.app, h).map_err(|e| e.into())?;

        Ok(())
    }*/

    // Set primary image survey
    #[wasm_bindgen(js_name = setSimpleHiPS)]
    pub fn set_simple_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {
        let hips: SimpleHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        //crate::log(&format!("simple HiPS: {:?}", hips));

        self.projection.set_simple_hips(&mut self.app, hips)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setCompositeHiPS)]
    pub fn set_composite_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {
        let hips: CompositeHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        //crate::log(&format!("Composite HiPS: {:?}", hips));

        self.projection.set_composite_hips(&mut self.app, hips)?;

        Ok(())
    }

    // Set an overlay HiPS
    #[wasm_bindgen(js_name = setOverlaySimpleHiPS)]
    pub fn set_overlay_simple_hips(&mut self, hips: JsValue) -> Result<(), JsValue> {
        let hips: SimpleHiPS = hips.into_serde().map_err(|e| e.to_string())?;
        //crate::log(&format!("simple HiPS: {:?}", hips));

        self.projection
            .set_overlay_simple_hips(&mut self.app, hips)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = removeOverlayHiPS)]
    pub fn remove_overlay_hips(&mut self) -> Result<(), JsValue> {
        self.projection.remove_overlay_hips(&mut self.app)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setOverlayCompositeHiPS)]
    pub fn set_overlay_composite_hips(&mut self, hipses: JsValue) -> Result<(), JsValue> {
        let hipses: CompositeHiPS = hipses.into_serde().map_err(|e| e.to_string())?;
        //crate::log(&format!("Composite HiPS: {:?}", hipses));

        self.projection
            .set_overlay_composite_hips(&mut self.app, hipses)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = setOverlayOpacity)]
    pub fn set_overlay_opacity(&mut self, opacity: f32) -> Result<(), JsValue> {
        self.projection
            .set_overlay_opacity(&mut self.app, opacity)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = worldToScreen)]
    pub fn world_to_screen(&self, lon: f64, lat: f64) -> Result<Option<Box<[f64]>>, JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        if let Some(screen_pos) = self.projection.world_to_screen(&self.app, &lonlat)? {
            Ok(Some(Box::new([screen_pos.x, screen_pos.y])))
        } else {
            Ok(None)
        }
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

        //log(&format!("{:?}", target_fov));
        let target_fov = ArcDeg(target_fov).into();
        self.projection.start_zooming_to(&mut self.app, target_fov);
        //self.projection.set_fov(&mut self.app, ArcDeg(fov).into());

        Ok(())
    }

    #[wasm_bindgen(js_name = projectLine)]
    pub fn project_line(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Result<Box<[f64]>, JsValue> {
        let vertices = self.projection.project_line(&self.app, lon1, lat1, lon2, lat2);

        let vertices = vertices.into_iter()
            .map(|v| vec![v.x, v.y])
            .flatten()
            .collect::<Vec<_>>();

        Ok(vertices.into_boxed_slice())
    }

    #[wasm_bindgen(js_name = enableGrid)]
    pub fn enable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.enable_grid(&mut self.app);

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

    #[wasm_bindgen(js_name = disableGrid)]
    pub fn disable_grid(&mut self) -> Result<(), JsValue> {
        self.projection.disable_grid(&mut self.app);

        Ok(())
    }

    #[wasm_bindgen(js_name = getMaxFieldOfView)]
    pub fn get_max_fov(&mut self) -> f64 {
        self.projection.get_max_fov(&mut self.app)
    }
    /// Set directly the center position
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f64]>, JsValue> {
        let center = self.projection.get_center(&self.app);

        let lon_deg: ArcDeg<f64> = center.lon().into();
        let lat_deg: ArcDeg<f64> = center.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    #[wasm_bindgen(js_name = getClipZoomFactor)]
    pub fn get_clip_zoom_factor(&self) -> Result<f64, JsValue> {
        Ok(self.app.get_clip_zoom_factor())
    }

    /*#[wasm_bindgen(js_name = startInertia)]
    pub fn start_inertia(&mut self) -> Result<(), JsValue> {
        //self.projection.set_center(&mut self.app, ArcDeg(lon).into(), ArcDeg(lat).into());
        // Tell the finite state machines the center has manually been changed
        self.events.enable::<StartInertia>(());

        Ok(())
    }*/
    /*
    /// Initiate a finite state machine that will zoom until a fov is reached
    /// while moving to a specific location
    #[wasm_bindgen(js_name = zoomToLocation)]
    pub fn set_zoom(&mut self, lon: f32, lat: f32, fov: f32) -> Result<(), JsValue> {
        self.events.enable::<ZoomToLocation>(
            (
                LonLatT::new(
                    ArcDeg(lon).into(),
                    ArcDeg(lat).into()
                ),
                ArcDeg(fov).into()
            )
        );

        Ok(())
    }*/
    /// Initiate a finite state machine that will move to a specific location
    #[wasm_bindgen(js_name = moveToLocation)]
    pub fn start_moving_to(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        // Enable the MouseLeftButtonReleased event
        let location = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        self.projection.start_moving_to(&mut self.app, location);

        /*self.events.enable::<MoveToLocation>(
            LonLatT::new(
                ArcDeg(lon).into(),
                ArcDeg(lat).into()
            )
        );*/

        Ok(())
    }

    /*#[wasm_bindgen(js_name = zoomToLocation)]
    pub fn start_zooming_to(&mut self, fov: f32) -> Result<(), JsValue> {
        let fov: Angle<f32> = ArcDeg(fov).into();

        self.projection.start_zooming_to(&mut self.app, fov.0);

        Ok(())
    }*/

    /// Set directly the center position
    #[wasm_bindgen(js_name = setCenter)]
    pub fn set_center(&mut self, lon: f64, lat: f64) -> Result<(), JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        self.projection.set_center(&mut self.app, lonlat);

        Ok(())
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

    #[wasm_bindgen(js_name = goFromTo)]
    pub fn go_from_to(
        &mut self,
        s1x: f64,
        s1y: f64,
        s2x: f64,
        s2y: f64,
    ) -> Result<(), JsValue> {
        self.projection.go_from_to(&mut self.app, s1x, s1y, s2x, s2y);

        Ok(())
    }

    /// CATALOG INTERFACE METHODS
    /// Add new catalog
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

    /// Resize the window
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.projection.resize(&mut self.app, width, height);

        Ok(())
    }
}


mod tests {
    /*#[test]
    fn lambert_w() {
        let d = -10.0;
        let x = -d/std::f32::consts::E;
        //let x = 1.0_f32;
        let mut wn = 1.0_f32;
        let mut k = 0;
        loop {
            //let tmp = wn1;
            let wn1 = wn - (wn*wn.exp() - x)/((1.0 + wn)*wn.exp());
            if (wn1 - wn).abs() <= 1e-3 {
                break;
            }
            wn = wn1;
            k += 1;
        }

        println!("{} {}", k, wn);
    }*/
    #[test]
    fn lambert_wm1() {
        let d = 0.1;
        let x = -d/std::f32::consts::E;

        let w = super::math::lambert_wm1(x);

        println!("{}", w);
    }
}