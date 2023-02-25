use crate::{
    async_task::{BuildCatalogIndex, ParseTableTask, TaskExecutor, TaskResult, TaskType},
    camera::CameraViewPort,
    downloader::Downloader,
    math::{
        self,
        angle::{Angle, ArcDeg},
        lonlat::{LonLat, LonLatT},
    },
    renderable::{
        catalog::{Manager, Source},
        grid::ProjetedGrid,
        moc::MOC,
        image::FitsImage, FitsCfg,
    },
    healpix::coverage::HEALPixCoverage,
    shader::ShaderManager,
    renderable::Layers,
    tile_fetcher::TileFetcherQueue,
    time::DeltaTime,
};

use al_core::WebGlContext;
use al_core::colormap::{Colormap, Colormaps};

use al_api::{
    coo_system::CooSystem,
    grid::GridCfg,
    hips::{ImageMetadata, HiPSCfg, FITSCfg}, fov::FoV,
};
use crate::Abort;
use super::coosys;
use cgmath::Vector4;

use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashSet;

use crate::renderable::final_pass::RenderPass;
use al_core::FrameBufferObject;

pub struct App {
    pub gl: WebGlContext,

    //ui: GuiRef,
    shaders: ShaderManager,
    camera: CameraViewPort,

    downloader: Downloader,
    tile_fetcher: TileFetcherQueue,
    layers: Layers,

    time_start_blending: Time,
    request_redraw: bool,
    rendering: bool,

    // The grid renderable
    grid: ProjetedGrid,
    // The moc renderable
    moc: MOC,
    // Catalog manager
    manager: Manager,

    // Task executor
    exec: Rc<RefCell<TaskExecutor>>,

    //move_animation: Option<MoveAnimation>,
    //zoom_animation: Option<ZoomAnimation>,
    inertial_move_animation: Option<InertiaAnimation>,
    prev_cam_position: Vector3<f64>,
    prev_center: Vector3<f64>,
    out_of_fov: bool,
    //tasks_finished: bool,
    catalog_loaded: bool,
    start_time_frame: Time,
    last_time_request_for_new_tiles: Time,
    request_for_new_tiles: bool,

    _final_rendering_pass: RenderPass,
    _fbo_view: FrameBufferObject,
    _fbo_ui: FrameBufferObject,

    colormaps: Colormaps,

    projection: ProjectionType,
}

use cgmath::{Vector2, Vector3};
use futures::stream::StreamExt; // for `next`

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

use al_api::resources::Resources;
use crate::downloader::query;
use crate::downloader::request;

impl App {
    pub fn new(
        gl: &WebGlContext,
        mut shaders: ShaderManager,
        resources: Resources,
    ) -> Result<Self, JsValue> {
        let gl = gl.clone();
        let exec = Rc::new(RefCell::new(TaskExecutor::new()));

        let projection = ProjectionType::Sin(mapproj::zenithal::sin::Sin);
        gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );

        // TODO: https://caniuse.com/?search=scissor is not supported for safari <= 14.1
        // When it will be supported nearly everywhere, we will need to uncomment this line to
        // enable it
        //gl.enable(WebGl2RenderingContext::SCISSOR_TEST);

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        // The tile buffer responsible for the tile requests
        let downloader = Downloader::new();

        let camera = CameraViewPort::new(&gl, CooSystem::ICRSJ2000, &projection);
        let screen_size = &camera.get_screen_size();

        let _fbo_view = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;
        let _fbo_ui = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;

        // The surveys storing the textures of the resolved tiles
        let layers = Layers::new(&gl, &projection)?;

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources)?;

        // Grid definition
        let grid = ProjetedGrid::new(&gl, &camera, &resources, &projection)?;

        // Variable storing the location to move to
        let inertial_move_animation = None;
        //let tasks_finished = false;
        let request_redraw = false;
        let rendering = true;
        let prev_cam_position = camera.get_center().truncate();
        let prev_center = Vector3::new(0.0, 1.0, 0.0);
        let out_of_fov = false;
        let catalog_loaded = false;

        let colormaps = Colormaps::new(&gl)?;

        let _final_rendering_pass = RenderPass::new(&gl)?;
        let tile_fetcher = TileFetcherQueue::new();

        //let ui = Gui::new(aladin_div_name, &gl)?;
        let start_time_frame = Time::now();
        let last_time_request_for_new_tiles = Time::now();

        let request_for_new_tiles = true;

        let moc = MOC::new(&gl);

        gl.clear_color(0.15, 0.15, 0.15, 1.0);

        Ok(App {
            gl,
            start_time_frame,
            //ui,
            shaders,

            camera,

            last_time_request_for_new_tiles,
            request_for_new_tiles,
            downloader,
            layers,

            time_start_blending,
            rendering,
            request_redraw,
            // The grid renderable
            grid,
            // MOCs renderable
            moc,
            // The catalog renderable
            manager,
            exec,
            prev_center,

            _fbo_view,
            _fbo_ui,
            _final_rendering_pass,

            inertial_move_animation,
            prev_cam_position,
            out_of_fov,

            //tasks_finished,
            catalog_loaded,

            tile_fetcher,

            colormaps,
            projection,
        })
    }

    fn look_for_new_tiles(&mut self) -> Result<(), JsValue> {
        // Move the views of the different active surveys
        self.tile_fetcher.clear();
        // Loop over the surveys
        for survey in self.layers.values_mut_hips() {
            // do not add tiles if the view is already at depth 0
            let view = survey.get_view();
            let depth_tile = view.get_depth();

            let min_depth_tile = survey.get_min_depth_tile();
            if depth_tile >= min_depth_tile {
                let mut tile_cells = survey
                    .get_view()
                    .get_cells()
                    //.flat_map(|texture_cell| texture_cell.get_tile_cells(survey.get_config()))
                    .flat_map(|cell| {
                        let texture_cell = cell.get_texture_cell(&survey.get_config());
                        texture_cell.get_tile_cells(&survey.get_config())
                    })
                    .collect::<HashSet<_>>();

                if depth_tile >= min_depth_tile + 3 {
                    // Retrieve the grand-grand parent cells but not if it is root ones as it may interfere with already done requests
                    let tile_cells_ancestor = tile_cells
                        .iter()
                        .map(|tile_cell| tile_cell.ancestor(3))
                        .collect::<HashSet<_>>();

                    tile_cells.extend(tile_cells_ancestor);
                }

                // Do not request the cells where no data is present (we know it from its moc)
                /*let (tile_cells_in_moc, tile_cells_out_moc) = if let Some(moc) = survey.get_moc() {
                    tile_cells.into_iter()
                        .partition(|tile_cell| {
                            moc.contains(tile_cell)
                        })
                } else {
                    (tile_cells, HashSet::new())
                };*/

                if let Some(moc) = survey.get_moc() {
                    tile_cells = tile_cells.into_iter()
                        .filter(|tile_cell| {
                            moc.contains(tile_cell)
                        })
                        .collect();
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

        Ok(())
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
                    self.manager.add_catalog(
                        name,
                        sources,
                        colormap,
                        &mut self.shaders,
                        &self.camera,
                        self.surveys.get_view().unwrap_abort(),
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
                    self.manager.add_catalog(
                        name,
                        sources,
                        colormap,
                        &mut self.shaders,
                        &self.camera,
                        self.surveys.get_view().unwrap_abort(),
                    );
                    self.catalog_loaded = true;
                    self.request_redraw = true;
                } //TaskResult::TileSentToGPU { tile } => todo!()
            }
        }

        Ok(())
    }*/
}

use al_api::cell::HEALPixCellProjeted;
use crate::downloader::request::Resource;

use crate::healpix::cell::HEALPixCell;
use al_api::color::ColorRGB;
use crate::downloader::request::tile::Tile;

impl App {
    pub(crate) fn set_background_color(&mut self, color: ColorRGB) {
        self.layers.set_background_color(color);
        self.request_redraw = true;
    }

    pub(crate) fn get_visible_cells(&self, depth: u8) -> Box<[HEALPixCellProjeted]> {
        let coverage = crate::survey::view::compute_view_coverage(&self.camera, depth, &CooSystem::ICRSJ2000);

        let cells: Vec<_> = coverage.flatten_to_fixed_depth_cells()
            .filter_map(|ipix| {
                let cell = HEALPixCell(depth, ipix);

                if let Ok(v) = crate::survey::view::vertices(&cell, &self.camera, &self.projection) {
                    let vx = [v[0].x, v[1].x, v[2].x, v[3].x];
                    let vy = [v[0].y, v[1].y, v[2].y, v[3].y];
    
                    let projeted_cell = HEALPixCellProjeted {
                        ipix,
                        vx,
                        vy
                    };
                    crate::survey::view::project(projeted_cell, &self.camera, &self.projection)
                } else {
                    None
                }
            })
            .collect();

        cells.into_boxed_slice()
    }

    pub(crate) fn is_catalog_loaded(&self) -> bool {
        self.catalog_loaded
    }

    pub(crate) fn is_ready(&self) -> Result<bool, JsValue> {
        let res = self.layers.is_ready();

        Ok(res)
    }

    pub(crate) fn get_moc(&self, params: &al_api::moc::MOC) -> Option<&HEALPixCoverage> {
        self.moc.get(params)
    }

    pub(crate) fn add_moc(&mut self, params: al_api::moc::MOC, moc: HEALPixCoverage) -> Result<(), JsValue> {
        self.moc.insert(moc, params, &self.camera, &self.projection);

        Ok(())
    }

    pub(crate) fn remove_moc(&mut self, params: &al_api::moc::MOC) -> Result<(), JsValue> {
        self.moc.remove(params, &self.camera)
            .ok_or_else(|| JsValue::from_str("MOC not found"))?;

        Ok(())
    }

    pub(crate) fn set_moc_params(&mut self, params: al_api::moc::MOC) -> Result<(), JsValue> {
        self.moc.set_params(params, &self.camera, &self.projection)
            .ok_or_else(|| JsValue::from_str("MOC not found"))?;
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn update(&mut self, _dt: DeltaTime) -> Result<(), JsValue> {
        //let available_tiles = self.run_tasks(dt)?;
        if let Some(InertiaAnimation {
            time_start_anim,
            d0,
            axis,
        }) = self.inertial_move_animation
        {
            let t = ((Time::now() - time_start_anim).as_millis() / 1000.0) as f64;

            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where:
            // * k is the stiffness of the ressort
            // * m is its mass
            let w0 = 5.0;
            // The angular distance goes from d0 to 0.0
            let d = d0 * ((-w0 * t).exp());
            /*let alpha = 1_f32 + (0_f32 - 1_f32) * (10_f32 * t + 1_f32) * (-10_f32 * t).exp();
            let alpha = alpha * alpha;
            let fov = start_fov * (1_f32 - alpha) + goal_fov * alpha;*/
            self.camera.rotate(&axis, d, &self.projection);
            // The threshold stopping criteria must be dependant
            // of the zoom level, in this case the initial angular distance
            // speed
            let thresh: Angle<f64> = d0 * 1e-3;
            if d < thresh {
                self.inertial_move_animation = None;
                // When the inertia is stopped we can look for new tiles
                self.request_for_new_tiles = true;
            }
        }

        // The rendering is done following these different situations:
        // - the camera has moved
        let has_camera_moved = self.camera.has_moved();
        {
            // Newly available tiles must lead to
            // 1. Surveys must be aware of the new available tiles
            //self.surveys.set_available_tiles(&available_tiles);
            // 2. Get the resolved tiles and push them to the image surveys
            /*let is_there_new_available_tiles = self
            .downloader
            .get_resolved_tiles(/*&available_tiles, */&mut self.surveys);*/
            let rscs = self.downloader.get_received_resources();

            let mut num_tile_received = 0;
            let mut tile_copied = false;
            for rsc in rscs.into_iter() {
                if !has_camera_moved || (Time::now() - self.start_time_frame < DeltaTime::from(24.0)) || !tile_copied {
                    match rsc {
                        Resource::Tile(tile) => {
                            let is_tile_root = tile.cell().is_root();

                            if let Some(survey) = self.layers.get_mut_hips_from_url(&tile.get_hips_url()) {
                                let cfg = survey.get_config();
                                if cfg.get_format() == tile.format {
                                    // If the format of the survey has changed then we discard tiles of the previous format

                                    if is_tile_root {
                                        let is_missing = tile.missing();
                                        let Tile {
                                            cell,
                                            image,
                                            time_req,
                                            ..
                                        } = tile;
            
                                        let image = if is_missing {
                                            // Otherwise we push nothing, it is probably the case where:
                                            // - an request error occured on a valid tile
                                            // - the tile is not present, e.g. chandra HiPS have not the 0, 1 and 2 order tiles
                                            None
                                        } else {
                                            Some(image)
                                        };
                                        survey.add_tile(&cell, image, time_req)?;
                                        tile_copied = true;

                                        self.request_redraw = true;
                                    } else {
                                        let fov_coverage = survey.get_view().get_coverage();
                                        let texture_cell = tile.cell().get_texture_cell(cfg);
                                        let included_or_near_coverage = texture_cell.get_tile_cells(cfg)
                                            .any(|neighbor_tile_cell| {
                                                fov_coverage.contains(&neighbor_tile_cell)
                                            });

                                        // do not perform tex_sub costly GPU calls while the camera is moving
                                        if included_or_near_coverage && !has_camera_moved {
                                            let is_missing = tile.missing();
                                            let Tile {
                                                cell,
                                                image,
                                                time_req,
                                                ..
                                            } = tile;

                                            let image = if is_missing {
                                                // Otherwise we push nothing, it is probably the case where:
                                                // - an request error occured on a valid tile
                                                // - the tile is not present, e.g. chandra HiPS have not the 0, 1 and 2 order tiles
                                                None
                                            } else {
                                                Some(image)
                                            };

                                            survey.add_tile(&cell, image, time_req)?;
                                            tile_copied = true;

                                            self.request_redraw = true;
                                        } else {
                                            self.downloader.cache_rsc(Resource::Tile(tile));
                                        }
                                    }
                                }
                            }

                            num_tile_received += 1;
                        }
                        Resource::Allsky(allsky) => {
                            let hips_url = allsky.get_hips_url();

                            if let Some(survey) = self.layers.get_mut_hips_from_url(hips_url) {
                                let is_missing = allsky.missing();
                                if is_missing {
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
                                    // tell the survey to not download tiles which order is <= 3 because the allsky
                                    // give them already
                                    survey.add_allsky(allsky)?;
                                    // Once received ask for redraw
                                    self.request_redraw = true;
                                }
                            }
                        },
                        Resource::PixelMetadata(metadata) => {
                            if let Some(hips) = self.layers.get_mut_hips_from_url(&metadata.hips_url) {
                                let mut cfg = hips.get_config_mut();

                                if let Some(metadata) = *metadata.value.lock().unwrap_abort() {
                                    cfg.blank = metadata.blank;
                                    cfg.offset = metadata.offset;
                                    cfg.scale = metadata.scale;
                                }
                            }
                        },
                        Resource::Moc(moc) => {
                            let moc_url = moc.get_url();
                            let url = &moc_url[..moc_url.find("/Moc.fits").unwrap_abort()];
                            if let Some(hips) = self.layers.get_mut_hips_from_url(url) {
                                let request::moc::Moc {
                                    moc,
                                    ..
                                } = moc;
    
                                if let Some(moc) = &*moc.lock().unwrap_abort() {
                                    hips.set_moc(moc.clone());

                                    self.request_for_new_tiles = true;
                                    self.request_redraw = true;
                                };
                            }
                        },
                    }
                } else {
                    self.downloader.delay_rsc(rsc);
                }
            }

            if num_tile_received > 0 {
                self.tile_fetcher
                    .notify(num_tile_received, &mut self.downloader);
                self.time_start_blending = Time::now();
            }
            //self.layers.add_resolved_tiles(resolved_tiles);
            // 3. Try sending new tile requests after
            //self.downloader.try_sending_tile_requests()?;
        }

        // Then, check for new tiles
        if has_camera_moved {
            self.layers.refresh_views(&mut self.camera);
        }

        if self.request_for_new_tiles && Time::now() - self.last_time_request_for_new_tiles > DeltaTime::from(500_f32) {
            self.look_for_new_tiles()?;

            self.request_for_new_tiles = false;
            self.last_time_request_for_new_tiles = Time::now();
        }

        // - there is at least one tile in its blending phase
        let blending_anim_occuring =
            (Time::now().0 - self.time_start_blending.0) < BLENDING_ANIM_DURATION;

        let mut start_fading = false;
        for hips in self.layers.values_hips() {
            if let Some(start_time) = hips.get_ready_time() {
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
            // Catalogues update
            /*if let Some(view) = self.layers.get_view() {
                self.manager.update(&self.camera, view);
            }*/
            self.grid.update(&self.camera, &self.projection);
            // MOCs update
            self.moc.update(&self.camera, &self.projection);
        }

        /*{
            let events = self.ui.lock().update();
            let mut events = events.lock().unwrap_abort();

            for event in events.drain(..) {
                match event {
                    al_ui::Event::ImageSurveys(surveys) => self.set_image_surveys(surveys)?,
                    _ => { todo!() }
                    //al_ui::Event::ReverseLongitude(longitude_reversed) => { self.set_longitude_reversed(longitude_reversed)? }
                }
            }
        }*/

        self.draw(false)?;

        Ok(())
    }

    pub(crate) fn reset_north_orientation(&mut self) {
        // Reset the rotation around the center if there is one
        self.camera.set_rotation_around_center(Angle(0.0), &self.projection);
        // Reset the camera position to its current position
        // this will keep the current position but reset the orientation
        // so that the north pole is at the top of the center.
        self.set_center(&self.get_center());
    }

    pub(crate) fn read_pixel(&self, pos: &Vector2<f64>, layer: &str) -> Result<JsValue, JsValue> {
        if let Some(lonlat) = self.screen_to_world(pos) {
            if let Some(survey) = self.layers.get_hips_from_layer(layer) {
                survey.read_pixel(&lonlat, &self.camera)
            } else if let Some(_image) = self.layers.get_image_from_layer(layer) {
                Err(JsValue::from_str("TODO: read pixel value"))
            } else {
                Err(JsValue::from_str("Survey not found"))
            }
        } else {
            Err(JsValue::from_str(&"position is out of projection"))
        }
    }

    pub(crate) fn draw(&mut self, force_render: bool) -> Result<(), JsValue> {
        self.start_time_frame = Time::now();

        /*let scene_redraw = self.rendering | force_render;
        let mut ui = self.ui.lock();

        if scene_redraw {
            let shaders = &mut self.shaders;
            let gl = self.gl.clone();
            let camera = &self.camera;

            let grid = &mut self.grid;
            let layers = &mut self.layers;
            let catalogs = &self.manager;
            let colormaps = &self.colormaps;
            let fbo_view = &self.fbo_view;

            fbo_view.draw_onto(
                move || {
                    // Render the scene
                    gl.clear_color(0.00, 0.00, 0.00, 1.0);
                    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                    layers.draw(camera, shaders, colormaps);

                    // Draw the catalog
                    catalogs.draw(&gl, shaders, camera, colormaps, fbo_view)?;

                    grid.draw(camera, shaders)?;

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

        self.layers.reset_frame();*/

        let scene_redraw = self.rendering | force_render;
        //let mut ui = self.ui.lock();
        //let ui_redraw = ui.redraw_needed();
        //if scene_redraw || ui_redraw {
        if scene_redraw {
            let shaders = &mut self.shaders;

            let grid = &mut self.grid;
            let layers = &mut self.layers;
            //let catalogs = &self.manager;
            let colormaps = &self.colormaps;
            let camera = &self.camera;
            // Render the scene
            // Clear all the screen first (only the region set by the scissor)
            self.gl.clear(web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT);

            layers.draw(camera, shaders, colormaps, &self.projection)?;
            self.moc.draw(shaders, camera);

            // Draw the catalog
            //let fbo_view = &self.fbo_view;
            //catalogs.draw(&gl, shaders, camera, colormaps, fbo_view)?;
            //catalogs.draw(&gl, shaders, camera, colormaps, None, self.projection)?;
            grid.draw(camera, shaders)?;

            //let dpi  = self.camera.get_dpi();
            //ui.draw(&gl, dpi)?;

            // Reset the flags about the user action
            self.camera.reset();

            if self.rendering {
                self.layers.reset_frame();
                self.moc.reset_frame();
            }
        }

        Ok(())
    }

    pub(crate) fn remove_layer(&mut self, layer: &str) -> Result<(), JsValue> {
        self.layers.remove_layer(layer, &mut self.camera, &self.projection)?;

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn rename_layer(&mut self, layer: &str, new_layer: &str) -> Result<(), JsValue> {
        self.layers.rename_layer(&layer, &new_layer)
    }

    pub(crate) fn swap_layers(&mut self, first_layer: &str, second_layer: &str) -> Result<(), JsValue> {
        self.layers.swap_layers(first_layer, second_layer)?;

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn add_image_survey(&mut self, hips_cfg: HiPSCfg) -> Result<(), JsValue> {
        let hips = self.layers.add_image_survey(&self.gl, hips_cfg, &mut self.camera, &self.projection)?;
        self.tile_fetcher.launch_starting_hips_requests(hips, &mut self.downloader);

        // Once its added, request the tiles in the view (unless the viewer is at depth 0)
        self.request_for_new_tiles = true;
        self.request_redraw = true;
        //self.grid.update(&self.camera, &self.projection);

        Ok(())
    }

    pub(crate) fn add_image_fits(&mut self, cfg: FITSCfg, bytes: &[u8]) -> Result<FoV, JsValue> {
        let FITSCfg { layer, url, meta } = cfg;
        
        let fits = FitsImage::new(&self.gl, bytes)?;
        let center = fits.get_center();
        let fov = FoV {
            ra: center.lon().to_degrees(),
            dec: center.lat().to_degrees(),
            fov: 1.0
        };

        let cfg = FitsCfg {
            layer,
            url,
            fits,
            meta,
        };
        self.layers.add_image_fits(cfg, &mut self.camera, &self.projection)?;

        // Once its added, request the tiles in the view (unless the viewer is at depth 0)
        self.request_redraw = true;

        Ok(fov)
    }

    pub(crate) fn get_layer_cfg(&self, layer: &str) -> Result<ImageMetadata, JsValue> {
        self.layers.get_layer_cfg(layer)
    }

    pub(crate) fn set_hips_url(&mut self, past_url: String, new_url: String) -> Result<(), JsValue> {
        self.layers.set_survey_url(past_url, new_url.clone())?;

        let hips = self.layers.get_hips_from_url(&new_url).unwrap_abort();
        // Relaunch the base tiles for the survey to be ready with the new url
        self.tile_fetcher.launch_starting_hips_requests(hips, &mut self.downloader);

        Ok(())
    }

    pub(crate) fn set_image_survey_color_cfg(
        &mut self,
        layer: String,
        meta: ImageMetadata,
    ) -> Result<(), JsValue> {
        self.request_redraw = true;

        let old_meta = self.layers.get_layer_cfg(&layer)?;
        // Set the new meta
        let new_img_fmt = meta.img_format;
        self.layers.set_layer_cfg(layer.clone(), meta, &self.camera, &self.projection)?;

        if old_meta.img_format != new_img_fmt {
            // The image format has been changed
            let hips = self.layers
                .get_mut_hips_from_layer(&layer)
                .ok_or_else(|| JsValue::from_str("Layer not found"))?;
            hips.set_img_format(new_img_fmt)?;

            // Relaunch the base tiles for the survey to be ready with the new url
            self.tile_fetcher.launch_starting_hips_requests(hips, &mut self.downloader);     

            // Once its added, request the tiles in the view (unless the viewer is at depth 0)
            self.request_for_new_tiles = true;
            self.request_redraw = true;
        }

        Ok(())
    }

    // Width and height given are in pixels
    pub(crate) fn set_projection(&mut self, projection: ProjectionType) -> Result<(), JsValue> {
        self.projection = projection;

        // Recompute the ndc_to_clip
        self.camera.set_projection(&self.projection);
        // Recompute clip zoom factor
        self.layers.set_projection(&self.projection)?;

        self.request_for_new_tiles = true;
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn get_max_fov(&self) -> f64 {
        self.projection.aperture_start()
    }

    pub(crate) fn get_longitude_reversed(&self) -> bool {
        self.camera.get_longitude_reversed()
    }

    pub(crate) fn add_catalog(&mut self, name: String, table: JsValue, _colormap: String) {
        let mut exec_ref = self.exec.borrow_mut();
        let table = table;

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
                }
            });
    }

    pub(crate) fn resize(&mut self, width: f32, height: f32) {
        self.camera.set_screen_size(width, height, &self.projection);
        self.camera.set_aperture(self.camera.get_aperture(), &self.projection);
        // resize the view fbo
        //self.fbo_view.resize(w as usize, h as usize);
        // resize the ui fbo
        //self.fbo_ui.resize(w as usize, h as usize);

        // launch the new tile requests
        self.request_for_new_tiles = true;
        self.manager.set_kernel_size(&self.camera);

        self.request_redraw = true;
    }

    pub(crate) fn set_survey_url(&mut self, past_url: String, new_url: String) -> Result<(), JsValue> {
        self.layers.set_survey_url(past_url, new_url)
    }

    pub(crate) fn set_catalog_opacity(&mut self, name: String, opacity: f32) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_alpha(opacity);

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_kernel_strength(&mut self, name: String, strength: f32) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_strength(strength);

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_grid_cfg(&mut self, cfg: GridCfg) -> Result<(), JsValue> {
        self.grid.set_cfg(cfg, &self.camera, &self.projection)?;
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_coo_system(&mut self, coo_system: CooSystem) {
        self.camera.set_coo_system(coo_system, &self.projection);
        self.request_for_new_tiles = true;

        self.request_redraw = true;
    }

    pub(crate) fn world_to_screen(&self, ra: f64, dec: f64) -> Option<Vector2<f64>> {
        let lonlat = LonLatT::new(ArcDeg(ra).into(), ArcDeg(dec).into());
        let model_pos_xyz = lonlat.vector();

        self.projection.view_to_screen_space(&model_pos_xyz, &self.camera)
    }

    pub(crate) fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        // Select the HiPS layer rendered lastly
        self.projection.screen_to_model_space(pos, &self.camera).map(|model_pos| model_pos.lonlat())
    }

    pub(crate) fn view_to_icrsj2000_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64> {
        let icrsj2000_pos: Vector4<_> = lonlat.vector();
        let view_system = self.camera.get_system();
        let (ra, dec) = math::lonlat::xyzw_to_radec(&coosys::apply_coo_system(
            view_system,
            &CooSystem::ICRSJ2000,
            &icrsj2000_pos,
        ));

        LonLatT::new(ra, dec)
    }

    pub(crate) fn icrsj2000_to_view_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64> {
        let icrsj2000_pos: Vector4<_> = lonlat.vector();
        let view_system = self.camera.get_system();
        let (ra, dec) = math::lonlat::xyzw_to_radec(&coosys::apply_coo_system(
            &CooSystem::ICRSJ2000,
            view_system,
            &icrsj2000_pos,
        ));

        LonLatT::new(ra, dec)
    }

    pub(crate) fn set_center(&mut self, lonlat: &LonLatT<f64>) {
        self.prev_cam_position = self.camera.get_center().truncate();
        self.camera.set_center(lonlat, &CooSystem::ICRSJ2000, &self.projection);
        self.request_for_new_tiles = true;

        // And stop the current inertia as well if there is one
        self.inertial_move_animation = None;
    }

    pub(crate) fn press_left_button_mouse(&mut self, _sx: f32, _sy: f32) {
        self.prev_center = self.camera.get_center().truncate();
        self.inertial_move_animation = None;
        self.request_for_new_tiles = true;
        self.out_of_fov = false;
    }

    pub(crate) fn release_left_button_mouse(&mut self, _sx: f32, _sy: f32) {
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
        let axis = self.prev_cam_position.cross(center).normalize();

        let delta_time = ((now - time_of_last_move).0 as f64).max(1.0);
        let delta_angle = math::vector::angle3(&self.prev_cam_position, &center);

        self.inertial_move_animation = Some(InertiaAnimation {
            d0: delta_angle * 3.0 / delta_time,
            axis,
            time_start_anim: Time::now(),
        });
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

    pub(crate) fn rotate_around_center(&mut self, theta: ArcDeg<f64>) {
        self.camera.set_rotation_around_center(theta.into(), &self.projection);
        // New tiles can be needed and some tiles can be removed
        self.request_for_new_tiles = true;

        self.request_redraw = true;
    }

    pub(crate) fn get_rotation_around_center(&self) -> &Angle<f64> {
        self.camera.get_rotation_around_center()
    }

    pub(crate) fn set_fov(&mut self, fov: Angle<f64>) {
        // For the moment, no animation is triggered.
        // The fov is directly set
        self.camera.set_aperture(fov, &self.projection);
        self.request_for_new_tiles = true;
        self.request_redraw = true;
    }

    /*pub(crate) fn project_line(&self, lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> Vec<Vector2<f64>> {
        let v1: Vector3<f64> = LonLatT::new(ArcDeg(lon1).into(), ArcDeg(lat1).into()).vector();
        let v2: Vector3<f64> = LonLatT::new(ArcDeg(lon2).into(), ArcDeg(lat2).into()).vector();

        line::project_along_great_circles(&v1, &v2, &self.camera, self.projection)
    }*/

    pub(crate) fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        // Select the HiPS layer rendered lastly
        if let Some(w1) = self.projection.screen_to_model_space(&Vector2::new(s1x, s1y), &self.camera) {
            if let Some(w2) = self.projection.screen_to_model_space(&Vector2::new(s2x, s2y), &self.camera) {
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
                    self.camera.rotate(&(-axis), d, &self.projection);
                    self.request_for_new_tiles = true;
                }
                return;
            }
        }

        self.out_of_fov = true;
    }

    pub(crate) fn add_cmap(&mut self, label: String, cmap: Colormap) -> Result<(), JsValue> {
        self.colormaps.add_cmap(label, cmap)
    }

    // Accessors
    pub(crate) fn get_center(&self) -> LonLatT<f64> {
        self.camera.get_center().lonlat()
    }

    pub(crate) fn get_norder(&self) -> i32 {
        self.camera.get_tile_depth() as i32
    }

    pub(crate) fn get_clip_zoom_factor(&self) -> f64 {
        self.camera.get_clip_zoom_factor()
    }

    pub(crate) fn get_fov(&self) -> f64 {
        let deg: ArcDeg<f64> = self.camera.get_aperture().into();
        deg.0
    }

    pub(crate) fn get_colormaps(&self) -> &Colormaps {
        &self.colormaps
    }

    pub(crate) fn get_gl_canvas(&self) -> Option<js_sys::Object> {
        self.gl.canvas()
    }

    pub(crate) fn is_rendering(&self) -> bool {
        self.rendering
    }
}
