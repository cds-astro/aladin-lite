use crate::browser_support::BrowserFeaturesSupport;
use crate::downloader::request::moc::MOCRequest;
use crate::math::angle::ToAngle;
use crate::math::spectra::Freq;
use crate::renderable::hips::HiPS;
use crate::renderable::image::Image;
use crate::renderable::ImageLayer;
use crate::tile_fetcher::HiPSLocalFiles;
use crate::Abort;
use crate::{
    camera::CameraViewPort,
    downloader::Downloader,
    healpix::moc::SpaceMoc,
    inertia::Inertia,
    math::{
        self,
        angle::{Angle, ArcDeg},
        lonlat::{LonLat, LonLatT},
    },
    renderable::grid::ProjetedGrid,
    renderable::Layers,
    renderable::{catalog::Manager, moc::MOCRenderer},
    shader::ShaderManager,
    tile_fetcher::TileFetcherQueue,
    time::DeltaTime,
};
use al_api::moc::MOCOptions;
use al_core::image::bitmap::Bitmap;
use al_core::image::fits::FitsImage;
use al_core::image::ImageType;
use fitsrs::WCS;
use moclib::qty::{Frequency, MocQty};
use std::io::Cursor;

use wasm_bindgen::prelude::*;

use al_core::colormap::{Colormap, Colormaps};
use al_core::WebGlContext;

use super::coosys;
use al_api::{
    coo_system::CooSystem,
    grid::GridCfg,
    hips::{HiPSCfg, ImageMetadata},
};

use crate::healpix::moc::Moc;

use web_sys::{HtmlElement, WebGl2RenderingContext};

use std::cell::RefCell;
use std::rc::Rc;

use crate::renderable::final_pass::RenderPass;
use al_core::FrameBufferObject;

use al_api::image::ImageParams;

pub struct App {
    pub gl: WebGlContext,

    //ui: GuiRef,
    shaders: ShaderManager,
    pub camera: CameraViewPort,
    worker: Worker,

    downloader: Rc<RefCell<Downloader>>,
    tile_fetcher: TileFetcherQueue,
    layers: Layers,

    time_start_blending: Time,
    request_redraw: bool,
    rendering: bool,

    // The grid renderable
    grid: ProjetedGrid,
    // The moc renderable
    moc: MOCRenderer,
    // Catalog manager
    manager: Manager,

    // Task executor
    //exec: Rc<RefCell<TaskExecutor>>,
    inertia: Option<Inertia>,
    north_up: bool,
    disable_inertia: Rc<RefCell<bool>>,
    dist_dragging: f32,
    time_start_dragging: Time,
    time_mouse_high_vel: Time,
    dragging: bool,
    vel_history: Vec<f32>,

    prev_cam_position: Vector3<f64>,
    //prev_center: Vector3<f64>,
    out_of_fov: bool,
    //tasks_finished: bool,
    catalog_loaded: bool,
    last_time_request_for_new_tiles: Time,
    request_for_new_tiles: bool,

    _final_rendering_pass: RenderPass,
    _fbo_view: FrameBufferObject,
    _fbo_ui: FrameBufferObject,
    //line_renderer: RasterizedLineRenderer,
    colormaps: Colormaps,

    pub projection: ProjectionType,

    cubic_tile_recv: async_channel::Receiver<WorkerResponse>,

    // Async data receivers
    //img_send: async_channel::Sender<ImageLayer>,
    img_recv: async_channel::Receiver<ImageLayer>,
    ack_img_send: async_channel::Sender<ImageParams>,

    browser_features_support: BrowserFeaturesSupport,
    //ack_img_recv: async_channel::Receiver<ImageParams>,
    // callbacks
    //callback_position_changed: js_sys::Function,
}

use cgmath::{Vector2, Vector3, Zero};

use crate::math::projection::*;
pub const BLENDING_ANIM_DURATION: DeltaTime = DeltaTime::from_millis(200.0); // in ms
                                                                             //use crate::buffer::Tile;
use crate::time::Time;
use cgmath::InnerSpace;

use crate::downloader::query::{self, CellDesc};
use crate::downloader::request::RequestType;
use al_api::resources::Resources;

impl App {
    pub fn new(
        gl: &WebGlContext,
        aladin_div: &HtmlElement,
        mut shaders: ShaderManager,
        resources: Resources,
        // Callbacks
        //callback_position_changed: js_sys::Function,
    ) -> Result<Self, JsValue> {
        let gl = gl.clone();
        //let exec = Rc::new(RefCell::new(TaskExecutor::new()));

        let projection = ProjectionType::Sin(mapproj::zenithal::sin::Sin);

        // TODO: https://caniuse.com/?search=scissor is not supported for safari <= 14.1
        // When it will be supported nearly everywhere, we will need to uncomment this line to
        // enable it
        //gl.enable(WebGl2RenderingContext::SCISSOR_TEST);

        //gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);
        //gl.enable(WebGl2RenderingContext::CULL_FACE);

        // The tile buffer responsible for the tile requests
        let downloader = Rc::new(RefCell::new(Downloader::new()));

        let camera = CameraViewPort::new(&gl, CooSystem::ICRS, &projection);
        let screen_size = &camera.get_screen_size();

        let _fbo_view =
            FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;
        let _fbo_ui = FrameBufferObject::new(&gl, screen_size.x as usize, screen_size.y as usize)?;

        // The hipss storing the textures of the resolved tiles
        let layers = Layers::new(&gl, &projection)?;

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources)?;

        // Grid definition
        let grid = ProjetedGrid::new(gl.clone(), aladin_div)?;

        // Variable storing the location to move to
        let inertia = None;
        let disable_inertia = Rc::new(RefCell::new(false));

        //let tasks_finished = false;
        let request_redraw = false;
        let rendering = true;
        let prev_cam_position = *camera.get_center();
        //let prev_center = Vector3::new(0.0, 1.0, 0.0);
        let out_of_fov = false;
        let catalog_loaded = false;

        let colormaps = Colormaps::new(&gl)?;

        let _final_rendering_pass = RenderPass::new(&gl)?;
        let tile_fetcher = TileFetcherQueue::new();

        //let ui = Gui::new(aladin_div_name, &gl)?;
        let last_time_request_for_new_tiles = Time::now();

        let request_for_new_tiles = true;

        let moc = MOCRenderer::new(&gl)?;
        gl.clear_color(0.1, 0.1, 0.1, 1.0);

        let (_, img_recv) = async_channel::unbounded::<ImageLayer>();
        let (ack_img_send, _) = async_channel::unbounded::<ImageParams>();
        let (cubic_tile_send, cubic_tile_recv) = async_channel::unbounded::<WorkerResponse>();

        let dist_dragging = 0.0;
        let time_start_dragging = Time::now();
        let dragging = false;
        let time_mouse_high_vel = Time::now();

        let browser_features_support = BrowserFeaturesSupport::new();

        let vel_history = vec![];
        let worker = create_worker()?;
        // Send the ack to the js promise so that she finished
        let onmessage = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
            move |event: web_sys::MessageEvent| {
                let data = event.data();

                let bytes_js = js_sys::Reflect::get(&data, &"bytes".into())
                    .unwrap()
                    .dyn_into::<js_sys::Uint8Array>()
                    .expect("is not a uint8 buffer");

                // Zero-copy clone from JS memory
                let mut bytes = vec![0u8; bytes_js.length() as usize];
                bytes_js.copy_to(&mut bytes);

                let meta: WorkerResponseMeta = serde_wasm_bindgen::from_value(data).unwrap();

                let response = WorkerResponse {
                    bytes,
                    tile_size: meta.tile_size,
                    tile_depth: meta.tile_depth,
                    cell: meta.cell,
                    hips_cdid: meta.hips_cdid,
                };
                let c = cubic_tile_send.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    c.send(response).await.unwrap_throw();
                })
            },
        );

        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        // 🚨 VERY IMPORTANT: prevent the closure from being dropped
        onmessage.forget();

        gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        Ok(App {
            gl,
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
            //exec,
            //prev_center,
            _fbo_view,
            _fbo_ui,
            _final_rendering_pass,

            //line_renderer,

            // inertia
            inertia,
            disable_inertia,
            dist_dragging,
            time_start_dragging,
            time_mouse_high_vel,
            dragging,
            vel_history,
            worker,

            prev_cam_position,
            out_of_fov,

            //tasks_finished,
            catalog_loaded,

            tile_fetcher,
            north_up: false,
            colormaps,
            projection,

            //img_send,
            img_recv,
            ack_img_send,
            cubic_tile_recv,

            browser_features_support, //ack_img_recv,
        })
    }

    fn _update_hips_location(&mut self) {
        let camera = &self.camera;
        for hips in self.layers.get_mut_hipses() {
            if let HiPS::D3(hips) = hips {
                hips.set_cursor_location(camera);
            }
        }
    }

    fn look_for_new_tiles(&mut self) -> Result<(), JsValue> {
        // Move the views of the different active hipss
        self.tile_fetcher.clear();
        // Loop over the hipss
        for hips in self.layers.get_mut_hipses() {
            /*if self.camera.get_tile_depth() == 0 {
                match hips {
                    HiPS::D2(h) => {
                        let query = query::Allsky::new(h.get_config(), None);
                        if self.downloader.borrow().is_queried(&query.id) {
                            // do not ask for tiles if we download the allsky
                            continue;
                        }
                    }
                    // no Allsky generated for HiPS3D
                    HiPS::D3(_) => (),
                }
            }*/

            hips.look_for_new_tiles(
                &mut self.tile_fetcher,
                &self.camera,
                &self.browser_features_support,
            );
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
                        self.hipss.get_view().unwrap_abort(),
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
                        self.hipss.get_view().unwrap_abort(),
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

use crate::healpix::cell::{HEALPixCell, HEALPixFreqCell};

use al_api::color::ColorRGB;

impl App {
    pub(crate) fn set_background_color(&mut self, color: ColorRGB) {
        self.layers.set_background_color(color);
        self.request_redraw = true;
    }

    pub(crate) fn get_visible_cells(&self, depth: u8) -> Box<[HEALPixCellProjeted]> {
        // Convert the camera frame vertices to ICRS before doing the moc
        let coverage = crate::camera::build_fov_coverage(
            depth,
            self.camera.get_field_of_view(),
            self.camera.get_center(),
            self.camera.get_coo_system(),
            CooSystem::ICRS,
            &self.projection,
        );

        let cells: Vec<_> = coverage
            .flatten_to_fixed_depth_cells()
            .filter_map(|ipix| {
                // This cell is defined in ICRS
                let cell = HEALPixCell(depth, ipix);

                let v = cell.vertices();
                let proj2screen = |(lon, lat): &(f64, f64)| -> Option<[f64; 2]> {
                    // 1. convert to xyz
                    let xyz = crate::math::lonlat::radec_to_xyz(lon.to_angle(), lat.to_angle());
                    // 2. get it back to the camera frame system
                    let xyz = crate::coosys::apply_coo_system(
                        CooSystem::ICRS,
                        self.camera.get_coo_system(),
                        &xyz,
                    );

                    // 3. project on screen
                    self.projection
                        .model_to_clip_space(&xyz, &self.camera)
                        .map(|p| [p.x, p.y])
                };

                if let (Some(c1), Some(c2), Some(c3), Some(c4)) = (
                    proj2screen(&v[0]),
                    proj2screen(&v[1]),
                    proj2screen(&v[2]),
                    proj2screen(&v[3]),
                ) {
                    let c: [[f64; 2]; 4] = [c1, c2, c3, c4];

                    let mut j = c.len() - 1;
                    for i in 0..c.len() {
                        if crate::math::vector::dist2(&c[j], &c[i]) > 0.05 {
                            return None;
                        }

                        j = i;
                    }

                    let v1 = crate::clip_to_screen_space(&c[0].into(), &self.camera);
                    let v2 = crate::clip_to_screen_space(&c[1].into(), &self.camera);
                    let v3 = crate::clip_to_screen_space(&c[2].into(), &self.camera);
                    let v4 = crate::clip_to_screen_space(&c[3].into(), &self.camera);

                    let vx = [v1[0], v2[0], v3[0], v4[0]];
                    let vy = [v1[1], v2[1], v3[1], v4[1]];

                    Some(HEALPixCellProjeted { ipix, vx, vy })
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

    pub(crate) fn get_moc(&self, moc_uuid: &str) -> Option<&SpaceMoc> {
        self.moc.get_hpx_coverage(moc_uuid)
    }

    pub(crate) fn add_moc(&mut self, moc: SpaceMoc, options: MOCOptions) -> Result<(), JsValue> {
        self.moc
            .push_back(moc, options, &mut self.camera, &self.projection);
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn remove_moc(&mut self, moc_uuid: &str) -> Result<(), JsValue> {
        self.moc
            .remove(moc_uuid, &mut self.camera, &self.projection)
            .ok_or_else(|| JsValue::from_str("MOC not found"))?;

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_moc_options(&mut self, options: MOCOptions) -> Result<(), JsValue> {
        self.moc
            .set_options(options)
            .ok_or_else(|| JsValue::from_str("MOC not found"))?;
        self.request_redraw = true;

        Ok(())
    }

    /*pub(crate) fn set_callback_position_changed(&mut self, callback: js_sys::Function) {
        self.callback_position_changed = callback;
    }*/

    pub(crate) fn is_inerting(&self) -> bool {
        self.inertia.is_some()
    }

    pub(crate) fn update(&mut self, dt: f64) -> Result<bool, JsValue> {
        // a timer stopping the frame if it takes too long
        // useful for garanting a framerate
        let rendering_timer = Time::now();

        if let Some(inertia) = self.inertia.as_mut() {
            inertia.apply(&mut self.camera, &self.projection, dt);
            // Always request for new tiles while moving
            self.request_for_new_tiles = true;

            // The threshold stopping criteria must be dependant
            // of the zoom level, in this case the initial angular distance
            // speed
            let thresh_speed = inertia.get_start_ampl() * 1e-4;
            let cur_speed = inertia.get_cur_speed();

            if cur_speed < thresh_speed {
                self.inertia = None;
            }
        }

        // Check for async retrieval
        if let Ok(img) = self.img_recv.try_recv() {
            let params = img.get_params();
            self.layers.add_image(
                img,
                &mut self.camera,
                &self.projection,
                &mut self.tile_fetcher,
            )?;
            self.request_redraw = true;

            // Send the ack to the js promise so that she finished
            let ack_img_send = self.ack_img_send.clone();
            wasm_bindgen_futures::spawn_local(async move {
                ack_img_send.send(params).await.unwrap_throw();
            })
        }

        if let Ok(WorkerResponse {
            cell,
            hips_cdid,
            tile_size,
            tile_depth,
            bytes,
            ..
        }) = self.cubic_tile_recv.try_recv()
        {
            if let Some(HiPS::D3(hips)) = self.layers.get_mut_hips_from_cdid(&hips_cdid) {
                hips.push_tile_from_png(
                    &cell,
                    bytes.into_boxed_slice(),
                    (tile_size, tile_size, tile_depth),
                    Time::now(),
                )?;
            }

            self.request_redraw = true;
        }

        let has_camera_moved = self.camera.has_moved();

        {
            // Newly available tiles must lead to
            // 1. Surveys must be aware of the new available tiles
            //self.hipss.set_available_tiles(&available_tiles);
            // 2. Get the resolved tiles and push them to the image hipss
            /*let is_there_new_available_tiles = self
            .downloader
            .get_resolved_tiles(/*&available_tiles, */&mut self.hipss);*/
            //self.tile_fetcher.clear();

            if self.request_for_new_tiles
                && Time::now() - self.last_time_request_for_new_tiles > DeltaTime::from(500.0)
            {
                self.look_for_new_tiles()?;

                self.request_for_new_tiles = false;
                self.last_time_request_for_new_tiles = Time::now();
            }

            // Tiles are fetched if:
            //let fetch_tiles =
            // * the user is not panning the view
            // * or the user is but did not move for at least 100ms
            //(Time::now() - self.camera.get_time_of_last_move() >= DeltaTime(100.0) || !self.dragging) &&
            // * no inertia action is in progress
            //self.inertia.is_none() &&
            // * the user is not zooming
            // !self.camera.has_zoomed();

            //if fetch_tiles {
            self.tile_fetcher.notify(self.downloader.clone(), None);
            //}
        }

        let rscs_received = self.downloader.borrow_mut().get_received_resources();

        let mut tile_copied = false;

        const MAX_FRAME_TIME: DeltaTime = DeltaTime::from_millis(1000.0 / 40.0);

        // - there is at least one tile in its blending phase
        let blending_anim_occuring =
            (Time::now() - self.time_start_blending) < BLENDING_ANIM_DURATION;

        for rsc in rscs_received {
            if Time::now() - rendering_timer >= MAX_FRAME_TIME {
                self.downloader.borrow_mut().delay(rsc);
                continue;
            }

            match rsc {
                RequestType::Tile(tile) => {
                    /*if self.camera.has_moved() {
                        self.downloader
                        .borrow_mut()
                        .delay(RequestType::Tile(tile));
                        continue;
                    }*/

                    if let Some(hips) = self.layers.get_mut_hips_from_cdid(&tile.hips_cdid) {
                        let cfg = hips.get_config();

                        if cfg.get_format() == tile.format {
                            let fov_coverage = self.camera.get_cov(cfg.get_frame());
                            let hpx_cell = tile.cell.get_hpx();

                            let included_in_coverage = fov_coverage.intersects_cell(hpx_cell);

                            //let is_tile_root = tile.cell().depth() == delta_depth;
                            //let _depth = tile.cell().depth();
                            // do not perform tex_sub costly GPU calls while the camera is zooming
                            if hpx_cell.is_root() || included_in_coverage {
                                let image = tile.request.get_data().clone();

                                // 1. For FITS tiles, parse the bscale/bzero and optional blank
                                // FIXME. We should consider these constants as per tiles and not for
                                // whole HiPS they belong to.
                                if let Some(ImageType::FitsRawBytes {
                                    raw_bytes: raw_bytes_buf,
                                    ..
                                }) = &*image.borrow()
                                {
                                    // check if the metadata has not been set
                                    if hips.get_fits_params().is_none() {
                                        let raw_bytes = raw_bytes_buf.to_vec();

                                        let FitsImage {
                                            bscale,
                                            bzero,
                                            blank,
                                            ..
                                        } = FitsImage::from_raw_bytes(raw_bytes.as_slice())?[0];
                                        hips.set_fits_params(bscale, bzero, blank);
                                    }
                                };

                                // 2. Add the tile to its HiPS
                                if let Some(img) = &*image.borrow() {
                                    // For PNG/JPEG cubic tiles, all the slices are in the lonely image
                                    match (&tile.cell, hips) {
                                        (CellDesc::HiPS2D { cell, .. }, HiPS::D2(hips)) => {
                                            hips.push_tile(cell, img, tile.request.time_request)?
                                        }
                                        (
                                            CellDesc::HiPSCube { cell, channel, .. },
                                            HiPS::D3(hips),
                                        ) => {
                                            // We build an artificial cube
                                            let f_hash = (*channel / 32) as u64;
                                            let slice_idx = (*channel % 32) as u16;

                                            let cell = HEALPixFreqCell::new(
                                                *cell,
                                                f_hash,
                                                Frequency::<u64>::MAX_DEPTH,
                                            );
                                            hips.push_tile_slice(
                                                &cell,
                                                img,
                                                tile.request.time_request,
                                                slice_idx,
                                            )?
                                        }
                                        (
                                            CellDesc::HiPS3D {
                                                cell,
                                                tile_size,
                                                tile_depth,
                                            },
                                            HiPS::D3(hips),
                                        ) => {
                                            // As the decoding and copying to the GPU of cubic tile is more costly
                                            // (not that much but there is more because they are smaller)
                                            // then we delay their treatment through the frames
                                            if tile_copied {
                                                self.downloader
                                                    .borrow_mut()
                                                    .delay(RequestType::Tile(tile));
                                                continue;
                                            }
                                            tile_copied = true;

                                            // TODO PNG/JPG case to handle here
                                            match img {
                                                ImageType::ImageRgba8u {
                                                    image: Bitmap { image, .. },
                                                } => {
                                                    let msg = js_sys::Object::new();
                                                    js_sys::Reflect::set(
                                                        &msg,
                                                        &"bitmap".into(),
                                                        image,
                                                    )?;
                                                    js_sys::Reflect::set(
                                                        &msg,
                                                        &"tileSize".into(),
                                                        &JsValue::from_f64(*tile_size as f64),
                                                    )?;
                                                    js_sys::Reflect::set(
                                                        &msg,
                                                        &"tileDepth".into(),
                                                        &JsValue::from_f64(*tile_depth as f64),
                                                    )?;
                                                    js_sys::Reflect::set(
                                                        &msg,
                                                        &"cell".into(),
                                                        &serde_wasm_bindgen::to_value(&cell)
                                                            .expect("Failed to serialize"),
                                                    )?;

                                                    js_sys::Reflect::set(
                                                        &msg,
                                                        &"HiPS".into(),
                                                        &JsValue::from_str(&tile.hips_cdid),
                                                    )?;

                                                    // Transfer ownership (zero-copy)
                                                    let transfer = js_sys::Array::of1(image);

                                                    self.worker.post_message_with_transfer(
                                                        &msg, &transfer,
                                                    )?;
                                                }
                                                ImageType::ImageRgb8u {
                                                    image: Bitmap { image, .. },
                                                } => {
                                                    let document = web_sys::window()
                                                        .unwrap_abort()
                                                        .document()
                                                        .unwrap_abort();
                                                    let canvas = document
                                                        .create_element("canvas")?
                                                        .dyn_into::<web_sys::HtmlCanvasElement>()?;
                                                    canvas.set_width(image.width());
                                                    canvas.set_height(image.height());
                                                    let context = canvas
                                                        .get_context("2d")?
                                                        .unwrap_abort()
                                                        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
                                                    // Get the data once for all for the whole image
                                                    // This takes time so better do it once and not repeatly
                                                    context.draw_image_with_image_bitmap(
                                                        image, 0.0, 0.0,
                                                    )?;

                                                    // Cut the png in several tile images. See page 3 of
                                                    // https://aladin.cds.unistra.fr/java/DocTechHiPS3D.pdf
                                                    let tile_depth = *tile_depth;
                                                    let num_cols =
                                                        (tile_depth as f32).sqrt().floor() as u32;
                                                    let num_rows = ((tile_depth as f32)
                                                        / (num_cols as f32))
                                                        .ceil()
                                                        as u32;

                                                    let tile_size = *tile_size;

                                                    let bytes = context
                                                        .get_image_data(
                                                            0_f64,
                                                            0_f64,
                                                            (num_cols * tile_size) as f64,
                                                            (num_rows * tile_size) as f64,
                                                        )?
                                                        .data()
                                                        .0;

                                                    let mut decoded_bytes = vec![
                                                        0_u8;
                                                        (tile_size * tile_size * tile_depth)
                                                            as usize
                                                    ];

                                                    let mut k = 0;
                                                    let mut num_tiles_cropped = 0;
                                                    for y in 0..num_rows {
                                                        let sy = y * tile_size;

                                                        for x in 0..num_cols {
                                                            let sx = x * tile_size;

                                                            for i in sy..(sy + tile_size) {
                                                                for j in sx..(sx + tile_size) {
                                                                    let id_byte = (j + i
                                                                        * num_cols
                                                                        * tile_size)
                                                                        * 4;

                                                                    decoded_bytes[k] =
                                                                        bytes[id_byte as usize];
                                                                    k += 1;
                                                                }
                                                            }

                                                            num_tiles_cropped += 1;

                                                            if num_tiles_cropped == tile_depth {
                                                                break;
                                                            }
                                                        }
                                                        if num_tiles_cropped == tile_depth {
                                                            break;
                                                        }
                                                    }

                                                    hips.push_tile_from_jpeg(
                                                        cell,
                                                        decoded_bytes.into_boxed_slice(),
                                                        (tile_size, tile_size, tile_depth),
                                                        tile.request.time_request,
                                                    )?;
                                                }
                                                ImageType::FitsRawBytes { raw_bytes, size } => hips
                                                    .push_tile_from_fits(
                                                        cell,
                                                        raw_bytes.clone(),
                                                        *size,
                                                        tile.request.time_request,
                                                    )?,
                                                _ => unreachable!(),
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                    self.request_redraw = true;
                                    self.time_start_blending = Time::now();
                                };
                            }
                        }
                    }
                }
                RequestType::Allsky(allsky) => {
                    if let Some(HiPS::D2(hips)) =
                        self.layers.get_mut_hips_from_cdid(&allsky.hips_cdid)
                    {
                        let is_missing = allsky.missing();
                        if is_missing {
                            // The allsky image is missing so we donwload all the tiles contained into
                            // the 0's cell
                            for base_hpx_cell in crate::healpix::cell::ALLSKY_HPX_CELLS_D0 {
                                let query = query::Tile::new(
                                    base_hpx_cell,
                                    hips.get_config(),
                                    &self.browser_features_support,
                                );
                                self.tile_fetcher.append_base_tile(query);
                            }
                        } else {
                            // tell the hips to not download tiles which order is <= 3 because the allsky
                            // give them already
                            hips.add_allsky(allsky)?;
                            // Once received ask for redraw
                            self.request_redraw = true;
                        }
                    }
                }
                RequestType::Moc(moc) => {
                    let moc_hips_cdid = moc.hips_cdid;
                    //let url = &moc_url[..moc_url.find("/Moc.fits").unwrap_abort()];
                    if let Some(hips) = self.layers.get_mut_hips_from_cdid(&moc_hips_cdid) {
                        let MOCRequest { request, .. } = moc;
                        if let Some(moc) = &*request.get_data().borrow() {
                            match (hips, moc) {
                                (HiPS::D2(hips), Moc::Space(moc)) => {
                                    hips.set_moc(moc.clone());
                                }
                                (HiPS::D3(hips), Moc::FreqSpace(moc)) => {
                                    hips.set_moc(moc.clone());
                                }
                                _ => (),
                            }

                            self.request_for_new_tiles = true;
                            self.request_redraw = true;
                        };
                    }
                }
            }
        }

        self.rendering = blending_anim_occuring
            | has_camera_moved
            | self.camera.has_zoomed()
            | self.request_redraw
            | self.inertia.is_some();

        self.draw()?;

        // Reset the flags about the user action
        self.camera.reset();

        Ok(has_camera_moved)
    }

    pub(crate) fn read_pixel(&self, x: f64, y: f64, layer: &str) -> Result<JsValue, JsValue> {
        if let Some(hips) = self.layers.get_hips_from_layer(layer) {
            hips.read_pixel(x, y, &self.camera, &self.projection)
        } else if let Some(_image) = self.layers.get_image_from_layer(layer) {
            // FIXME handle the case of an image
            Ok(JsValue::null())
        } else {
            Err(JsValue::from_str("Survey not found"))
        }
    }

    pub(crate) fn read_line_of_pixels(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        layer: &str,
    ) -> Result<Vec<JsValue>, JsValue> {
        let pixels = crate::math::utils::bresenham(x1, y1, x2, y2)
            .map(|(x, y)| self.read_pixel(x, y, layer))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(pixels)
    }

    pub(crate) fn draw_grid_labels(&mut self) -> Result<(), JsValue> {
        self.grid.draw_labels()
    }

    pub(crate) fn draw(&mut self) -> Result<(), JsValue> {
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

        //let scene_redraw = self.rendering | force_render;

        //let mut ui = self.ui.lock();
        //let ui_redraw = ui.redraw_needed();
        //if scene_redraw || ui_redraw {

        self.request_redraw = false;

        let shaders = &mut self.shaders;

        let gl = self.gl.clone();

        let camera = &mut self.camera;

        let grid = &mut self.grid;
        let moc = &mut self.moc;
        let projection = &self.projection;

        let layers = &mut self.layers;
        //let catalogs = &self.manager;
        let colormaps = &self.colormaps;
        //let fbo_view = &self._fbo_view;
        //let final_rendering_pass = &self._final_rendering_pass;

        //fbo_view.draw_onto(
        //    move || {
        // Render the scene
        // Clear all the screen first (only the region set by the scissor)
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // set the blending options
        layers.draw(camera, shaders, colormaps, projection)?;

        // Draw the catalog
        //let fbo_view = &self.fbo_view;
        //catalogs.draw(&gl, shaders, camera, colormaps, fbo_view)?;
        //catalogs.draw(&gl, shaders, camera, colormaps, None, self.projection)?;
        /*gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );*/
        moc.draw(camera, projection, shaders)?;

        /*gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );*/
        grid.draw(camera, projection, shaders)?;
        //        Ok(())
        //    },
        //    None,
        //)?;

        //final_rendering_pass.draw_on_screen(fbo_view, &mut self.shaders)?;

        Ok(())
    }

    pub(crate) fn remove_layer(&mut self, layer: &str) -> Result<(), JsValue> {
        self.layers.remove_layer(
            layer,
            &mut self.camera,
            &self.projection,
            &mut self.tile_fetcher,
        )?;

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn swap_layers(
        &mut self,
        first_layer: &str,
        second_layer: &str,
    ) -> Result<(), JsValue> {
        self.layers.swap_layers(first_layer, second_layer)?;

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn add_hips(
        &mut self,
        hips_cfg: HiPSCfg,
        local_files: Option<HiPSLocalFiles>,
    ) -> Result<(), JsValue> {
        let cdid = hips_cfg.properties.get_creator_did().to_string();

        let hips = self.layers.add_hips(
            &self.gl,
            hips_cfg,
            &mut self.camera,
            &self.projection,
            &mut self.tile_fetcher,
        )?;

        if let Some(local_files) = local_files {
            self.tile_fetcher.insert_hips_local_files(cdid, local_files);
        }

        self.tile_fetcher
            .launch_starting_hips_requests(hips, self.downloader.clone());

        // Once its added, request the tiles in the view (unless the viewer is at depth 0)
        self.request_for_new_tiles = true;
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn add_rgba_image(
        &mut self,
        layer: String,
        bytes: &[u8],
        wcs: WCS,
        options: ImageMetadata,
    ) -> Result<js_sys::Promise, JsValue> {
        let gl = self.gl.clone();

        let camera_coo_sys = self.camera.get_coo_system();

        match Image::from_rgba_bytes(&gl, bytes, wcs, camera_coo_sys) {
            Ok(image) => {
                let layer = ImageLayer {
                    images: vec![image],
                    id: layer.clone(),
                    layer,
                    options,
                };

                let params = layer.get_params();

                self.layers.add_image(
                    layer,
                    &mut self.camera,
                    &self.projection,
                    &mut self.tile_fetcher,
                )?;

                self.request_redraw = true;

                let promise = js_sys::Promise::resolve(&serde_wasm_bindgen::to_value(&params)?);
                Ok(promise)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn add_fits_image(
        &mut self,
        bytes: &[u8],
        options: ImageMetadata,
        layer: String,
    ) -> Result<js_sys::Promise, JsValue> {
        use fitsrs::hdu::header::ValueMap;

        let gl = self.gl.clone();
        // Stop the current inertia
        // And disable it while the fits has not been loaded
        let camera_coo_sys = self.camera.get_coo_system();

        // FIXME: this is done to prevent the view inerting after being unblocked

        let gz = fitsrs::gz::GzReader::new(Cursor::new(bytes))
            .map_err(|_| JsValue::from_str("Error creating gz wrapper"))?;

        let parse_fits_images_from_bytes =
            |raw_bytes: &[u8]| -> Result<(Vec<Image>, Vec<ValueMap>), JsValue> {
                let (images, headers) = FitsImage::from_raw_bytes(raw_bytes)?
                    .into_iter()
                    .filter_map(
                        |FitsImage {
                             bitpix,
                             bscale,
                             bzero,
                             blank,
                             wcs,
                             raw_bytes,
                             header,
                             ..
                         }| {
                            if let Some(wcs) = wcs {
                                let image = Image::from_fits_hdu(
                                    &gl,
                                    wcs,
                                    bitpix,
                                    raw_bytes.as_ref(),
                                    bscale,
                                    bzero,
                                    blank,
                                    camera_coo_sys,
                                )
                                .ok()?;
                                Some((image, header))
                            } else {
                                None
                            }
                        },
                    )
                    .collect::<(Vec<_>, Vec<_>)>();

                Ok((images, headers))
            };

        let (images, headers) = match gz {
            fitsrs::gz::GzReader::GzReader(bytes) => parse_fits_images_from_bytes(bytes.get_ref())?,
            fitsrs::gz::GzReader::Reader(bytes) => parse_fits_images_from_bytes(bytes.get_ref())?,
        };

        if images.is_empty() {
            Err(JsValue::from_str("no images have been parsed"))
        } else {
            let layer = ImageLayer {
                images,
                id: layer.clone(),

                layer,
                options,
            };

            let params = layer.get_params();
            self.layers.add_image(
                layer,
                &mut self.camera,
                &self.projection,
                &mut self.tile_fetcher,
            )?;
            self.request_redraw = true;

            let obj: js_sys::Object = serde_wasm_bindgen::to_value(&params)?.dyn_into()?;

            use std::iter::FromIterator;
            let arr = js_sys::Array::from_iter(
                headers
                    .iter()
                    .map(|header| serde_wasm_bindgen::to_value(&header).unwrap()),
            );
            js_sys::Reflect::set(&obj, &"headers".into(), &arr).unwrap();

            let promise = js_sys::Promise::resolve(&obj.into());
            Ok(promise)
        }
    }

    pub(crate) fn get_layer_cfg(&self, layer: &str) -> Result<ImageMetadata, JsValue> {
        self.layers.get_layer_cfg(layer)
    }

    pub(crate) fn set_hips_frequency(
        &mut self,
        layer: &str,
        frequency: f32,
    ) -> Result<(), JsValue> {
        let hips = self
            .layers
            .get_mut_hips_from_layer(layer)
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;

        self.request_for_new_tiles = true;

        match hips {
            HiPS::D2(_) => Err(JsValue::from_str("layer do not refers to a cube")),
            HiPS::D3(hips) => {
                hips.set_freq(Freq(frequency as f64));

                Ok(())
            }
        }
    }

    pub(crate) fn get_hips_frequency(&mut self, layer: &str) -> Result<f32, JsValue> {
        let hips = self
            .layers
            .get_mut_hips_from_layer(layer)
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;

        match hips {
            HiPS::D2(_) => Err(JsValue::from_str("layer do not refers to a cube")),
            HiPS::D3(hips) => Ok(hips.get_freq().0 as f32),
        }
    }

    pub(crate) fn get_hips_frequency_window(&mut self, layer: &str) -> Result<[Freq; 2], JsValue> {
        let hips = self
            .layers
            .get_mut_hips_from_layer(layer)
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;

        match hips {
            HiPS::D2(_) => Err(JsValue::from_str("layer do not refers to a cube")),
            HiPS::D3(hips) => Ok(hips.get_freq_window()),
        }
    }

    pub(crate) fn get_freq_from_hash(&mut self, layer: &str, hash: u64) -> Result<f64, JsValue> {
        let hips = self
            .layers
            .get_mut_hips_from_layer(layer)
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;

        match hips {
            HiPS::D2(_) => Err(JsValue::from_str("layer do not refers to a cube")),
            HiPS::D3(hips) => Ok(hips.get_freq_from_hash(hash).0),
        }
    }

    pub(crate) fn get_freq_hash(&mut self, layer: &str, freq: f64) -> Result<u64, JsValue> {
        let hips = self
            .layers
            .get_mut_hips_from_layer(layer)
            .ok_or_else(|| JsValue::from_str("Layer not found"))?;

        match hips {
            HiPS::D2(_) => Err(JsValue::from_str("layer do not refers to a cube")),
            HiPS::D3(hips) => Ok(hips.get_freq_hash(Freq(freq))),
        }
    }

    pub(crate) fn set_image_hips_color_cfg(
        &mut self,
        layer: String,
        meta: ImageMetadata,
    ) -> Result<(), JsValue> {
        let old_meta = self.layers.get_layer_cfg(&layer)?;
        // Set the new meta
        // keep the old meta data
        let new_img_ext = meta.img_format;
        self.layers.set_layer_cfg(layer.clone(), meta)?;

        if old_meta.img_format != new_img_ext {
            // The image format has been changed
            let hips = self
                .layers
                .get_mut_hips_from_layer(&layer)
                .ok_or_else(|| JsValue::from_str("Layer not found"))?;
            hips.set_image_ext(new_img_ext)?;

            // Relaunch the base tiles for the hips to be ready with the new url
            self.tile_fetcher
                .launch_starting_hips_requests(hips, self.downloader.clone());

            // Once its added, request the tiles in the view (unless the viewer is at depth 0)
            self.request_for_new_tiles = true;
        }

        self.request_redraw = true;

        Ok(())
    }

    // Width and height given are in pixels
    pub(crate) fn set_projection(&mut self, projection: ProjectionType) -> Result<(), JsValue> {
        self.projection = projection;

        // Recompute clip zoom factor
        self.layers.set_projection(&self.projection)?;
        // Recompute the ndc_to_clip
        self.camera.set_projection(&self.projection);

        self.request_for_new_tiles = true;
        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn get_max_fov(&self) -> Angle<f64> {
        self.projection.aperture_start()
    }

    pub(crate) fn get_longitude_reversed(&self) -> bool {
        self.camera.get_longitude_reversed()
    }

    pub(crate) fn set_longitude_reversed(&mut self, longitude_reversed: bool) {
        self.camera
            .set_longitude_reversed(longitude_reversed, &self.projection);
    }

    pub(crate) fn add_catalog(&mut self, _name: String, table: JsValue, _colormap: String) {
        //let mut exec_ref = self.exec.borrow_mut();
        let _table = table;

        /*exec_ref
        .spawner()
        .spawn(TaskType::ParseTableTask, async move {
            let mut stream = ParseTableTask::<[f32; 2]>::new(table);
            let mut results: Vec<LonLatT<f32>> = vec![];

            while let Some(item) = stream.next().await {
                results.push(LonLatT::new(item[0].to_angle(), item[1].to_angle()));
            }

            let mut stream_sort = BuildCatalogIndex::new(results);
            while stream_sort.next().await.is_some() {}

            // The stream is finished, we get the sorted sources
            let results = stream_sort.sources;

            TaskResult::TableParsed {
                name,
                sources: results.into_boxed_slice(),
            }
        });*/
    }

    pub(crate) fn resize(&mut self, width: f32, height: f32) {
        self.camera.set_screen_size(width, height, &self.projection);
        //self.camera
        //    .set_zoom_factor(self.camera.get_zoom_factor(), &self.projection);

        // resize the view fbo
        //let screen_size = self.camera.get_screen_size();
        //self._fbo_view
        //    .resize(screen_size.x as usize, screen_size.y as usize);
        // resize the ui fbo
        //self.fbo_ui.resize(w as usize, h as usize);

        // launch the new tile requests
        self.request_for_new_tiles = true;
        //self.manager.set_kernel_size(&self.camera);

        self.request_redraw = true;
    }

    pub(crate) fn set_hips_url(&mut self, cdid: &String, new_url: String) -> Result<(), JsValue> {
        self.layers.set_hips_url(cdid, new_url)
    }

    pub(crate) fn set_catalog_opacity(
        &mut self,
        name: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_alpha(opacity);

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_kernel_strength(
        &mut self,
        name: String,
        strength: f32,
    ) -> Result<(), JsValue> {
        let catalog = self.manager.get_mut_catalog(&name).map_err(|e| {
            let err: JsValue = e.into();
            err
        })?;
        catalog.set_strength(strength);

        self.request_redraw = true;

        Ok(())
    }

    pub(crate) fn set_grid_cfg(&mut self, cfg: GridCfg) -> Result<(), JsValue> {
        self.grid.set_cfg(cfg)?;
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
        let icrs_pos = lonlat.vector();

        self.projection
            .icrs_celestial_to_screen_space(&icrs_pos, &self.camera)
    }

    pub(crate) fn screen_to_world(&self, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        // Select the HiPS layer rendered lastly
        self.projection
            .screen_to_model_space(pos, &self.camera)
            .map(|model_pos| model_pos.lonlat())
    }

    pub(crate) fn screen_to_clip(&self, pos: &Vector2<f64>) -> Vector2<f64> {
        // Select the HiPS layer rendered lastly
        crate::math::projection::screen_to_clip_space(pos, &self.camera)
    }

    pub(crate) fn get_coo_system(&self) -> CooSystem {
        self.camera.get_coo_system()
    }

    pub(crate) fn view_to_icrs_coosys(&self, lonlat: &LonLatT<f64>) -> LonLatT<f64> {
        let celestial_pos = lonlat.vector();
        let view_system = self.camera.get_coo_system();
        let (ra, dec) = math::lonlat::xyz_to_radec(&coosys::apply_coo_system(
            view_system,
            CooSystem::ICRS,
            &celestial_pos,
        ));

        LonLatT::new(ra, dec)
    }

    /// lonlat must be given in icrs frame
    pub(crate) fn set_center(&mut self, lonlat: &LonLatT<f64>) {
        self.prev_cam_position = *self.camera.get_center();

        self.camera.set_center(lonlat, &self.projection);
        self.request_for_new_tiles = true;

        // And stop the current inertia as well if there is one
        self.inertia = None;

        self._update_hips_location();
    }

    pub(crate) fn move_mouse(&mut self, s1x: f32, s1y: f32, s2x: f32, s2y: f32) {
        if self.dragging {
            let from_mouse_pos = [s1x, s1y];
            let to_mouse_pos = [s2x, s2y];
            let dx = crate::math::vector::dist2(&from_mouse_pos, &to_mouse_pos).sqrt();
            self.dist_dragging += dx;

            //let now = Time::now();
            //let dragging_duration = (now - self.time_start_dragging).as_secs();
            //let dragging_vel = self.dist_dragging / dragging_duration;

            // 1. Use smoothed velocity instead of instantaneous velocity
            let dv = dx / (Time::now() - self.camera.get_time_of_last_move()).as_secs();
            self.vel_history.push(dv);
            if self.vel_history.len() > 5 {
                self.vel_history.remove(0);
            }

            if dv > 10000.0 {
                self.time_mouse_high_vel = Time::now();
            }
        }
    }

    pub(crate) fn press_left_button_mouse(&mut self) {
        self.dist_dragging = 0.0;
        self.time_start_dragging = Time::now();
        self.dragging = true;

        self.inertia = None;
        self.request_for_new_tiles = true;
        self.out_of_fov = false;
    }

    pub(crate) fn release_left_button_mouse(&mut self) {
        self.request_for_new_tiles = true;

        self.dragging = false;

        // Check whether the center has moved
        // between the pressing and releasing
        // of the left button.
        //
        // Do not start inerting if:
        // * the mouse has not moved
        // * the mouse is out of the projection
        // * the mouse has not been moved since a certain
        //   amount of time

        //debug!(now);
        //debug!(time_of_last_move);
        if self.out_of_fov {
            return;
        }

        let inertia_disabled: bool = *(self.disable_inertia.borrow_mut());
        if inertia_disabled {
            return;
        }

        if self.dist_dragging == 0.0 {
            return;
        }

        if self.vel_history.len() < 5 {
            return;
        }

        let now = Time::now();
        let avg_vel = self.vel_history.iter().copied().sum::<f32>() / self.vel_history.len() as f32;

        // 2. Clamp minimum + maximum velocities
        let min_vel = 1000.0; // tweak

        // 3. Better condition for “recent acceleration”
        let t_since_drag = (now - self.time_start_dragging).as_secs();
        let t_since_accel = (now - self.time_mouse_high_vel).as_secs();

        let inertia_trigger =
            avg_vel > min_vel || ((t_since_drag < 0.15) || (t_since_accel < 0.15));
        if !inertia_trigger {
            return;
        }

        // Start inertia here
        // Angular distance between the previous and current
        // center position
        let center = self.camera.get_center();
        let axis = self.prev_cam_position.cross(*center).normalize();

        let delta_angle = math::vector::angle3(&self.prev_cam_position, center).to_radians();
        let ampl = (delta_angle * avg_vel as f64) * 5e-3;

        self.inertia = Some(Inertia::new(ampl.to_radians(), axis, self.north_up))
    }

    pub(crate) fn set_position_angle(&mut self, theta: ArcDeg<f64>) {
        self.camera
            .set_position_angle(theta.into(), &self.projection);
        // New tiles can be needed and some tiles can be removed
        self.request_for_new_tiles = true;

        self.request_redraw = true;
    }

    pub(crate) fn get_position_angle(&self) -> Angle<f64> {
        self.camera.get_position_angle()
    }

    pub(crate) fn set_fov(&mut self, fov: f64) {
        // For the moment, no animation is triggered.
        // The fov is directly set
        self.camera.set_aperture(fov, &self.projection);

        // reset the parameters that determine if an inertia is needed
        self.vel_history.clear();
        self.dist_dragging = 0.0;

        self.request_for_new_tiles = true;
        self.request_redraw = true;
    }

    pub(crate) fn set_fov_range(&mut self, min_fov: Option<f64>, max_fov: Option<f64>) {
        self.camera.set_fov_range(
            min_fov.map(|v| v.to_radians()),
            max_fov.map(|v| v.to_radians()),
            &self.projection,
        );
        self.request_for_new_tiles = true;
        self.request_redraw = true;
    }

    pub(crate) fn set_inertia(&mut self, inertia: bool) {
        *self.disable_inertia.borrow_mut() = !inertia;
    }

    pub(crate) fn go_from_to(&mut self, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        // Select the HiPS layer rendered lastly
        if let (Some(w1), Some(w2)) = (
            self.projection
                .screen_to_model_space(&Vector2::new(s1x, s1y), &self.camera),
            self.projection
                .screen_to_model_space(&Vector2::new(s2x, s2y), &self.camera),
        ) {
            let prev_pos = w1;
            let cur_pos = w2;
            if prev_pos != cur_pos {
                let prev_cam_position = *self.camera.get_center();

                if self.north_up {
                    let lonlat1 = prev_pos.lonlat();
                    let lonlat2 = cur_pos.lonlat();

                    let dlon = lonlat2.lon() - lonlat1.lon();
                    let dlat = lonlat2.lat() - lonlat1.lat();

                    self.camera
                        .apply_lonlat_rotation(dlon, dlat, &self.projection);

                    // Detect if a pole has been crossed

                    let north_pole = Vector3::new(0.0, 1.0, 0.0);
                    let south_pole = Vector3::new(0.0, -1.0, 0.0);
                    let cross_north_pole = crate::math::lonlat::is_in(
                        &prev_cam_position,
                        self.camera.get_center(),
                        &north_pole,
                    );
                    let cross_south_pole = crate::math::lonlat::is_in(
                        &prev_cam_position,
                        self.camera.get_center(),
                        &south_pole,
                    );

                    let cross_pole = cross_north_pole | cross_south_pole;

                    // Detect if a pole has been crossed
                    let center = if cross_pole {
                        &prev_cam_position
                    } else {
                        self.camera.get_center()
                    };

                    let fov = self.camera.get_aperture();

                    let pole = if center.y >= 0.0 {
                        north_pole
                    } else {
                        south_pole
                    };

                    let c2p = crate::math::vector::angle3(center, &pole).to_radians();
                    let near_pole = c2p.abs() < 5e-3 * fov;
                    if near_pole || cross_pole {
                        // too near to the pole
                        let axis = center.cross(pole).normalize();
                        use crate::math::rotation::Rotation;
                        let new_center = Rotation::from_axis_angle(&axis, (-5e-3 * fov).to_angle())
                            .rotate(&pole);

                        self.camera.set_center_xyz(&new_center, &self.projection);
                        self.camera
                            .set_position_angle(0.0.to_angle(), &self.projection);
                    }
                } else {
                    /* 1. Rotate by computing the angle between the last and current position */
                    let d = math::vector::angle3(&prev_pos, &cur_pos);
                    let axis = prev_pos.cross(cur_pos).normalize();

                    self.camera
                        .apply_axis_rotation(&(-axis), d, &self.projection);
                }

                self.prev_cam_position = prev_cam_position;
                self.request_for_new_tiles = true;

                self._update_hips_location();
            }
        } else {
            // approx move
            let origin2next = Vector2::new(s2x - s1x, s2y - s1y);

            if origin2next != Vector2::zero() {
                let prev_pos = self.camera.get_center();

                let prev_cam_position = self.get_center().vector();

                let center_screen = self
                    .projection
                    .model_to_screen_space(&prev_cam_position, &self.camera)
                    .unwrap();

                let next_s = origin2next + center_screen;

                if let Some(cur_pos) = self.projection.screen_to_model_space(&next_s, &self.camera)
                {
                    let d = math::vector::angle3(prev_pos, &cur_pos);
                    let axis = prev_pos.cross(cur_pos).normalize();

                    self.camera
                        .apply_axis_rotation(&(-axis), d, &self.projection);

                    self.prev_cam_position = prev_cam_position;
                    self.request_for_new_tiles = true;

                    self._update_hips_location();
                }
            }
        }
    }

    pub(crate) fn lock_north_up(&mut self) {
        self.north_up = true;
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

    pub(crate) fn get_zoom_factor(&self) -> f64 {
        self.camera.get_zoom_factor()
    }

    pub(crate) fn set_zoom_factor(&mut self, zoom_factor: f64) {
        self.camera.set_zoom_factor(zoom_factor, &self.projection);

        // reset the parameters that determine if an inertia is needed
        self.vel_history.clear();
        self.dist_dragging = 0.0;

        self._update_hips_location();

        self.request_for_new_tiles = true;
        self.request_redraw = true;
    }

    pub(crate) fn get_fov(&self) -> [f64; 2] {
        [
            self.camera.get_aperture().to_degrees(),
            self.camera.get_aperture_y().to_degrees(),
        ]
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

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WorkerResponse {
    #[serde(rename = "tileSize")]
    pub tile_size: u32,

    #[serde(rename = "tileDepth")]
    pub tile_depth: u32,

    pub bytes: Vec<u8>,

    #[serde(rename = "HiPSCDid")]
    pub hips_cdid: String,

    pub cell: HEALPixFreqCell,
}

#[derive(Deserialize, Debug)]
pub struct WorkerResponseMeta {
    #[serde(rename = "tileSize")]
    pub tile_size: u32,

    #[serde(rename = "tileDepth")]
    pub tile_depth: u32,

    #[serde(rename = "HiPSCDid")]
    pub hips_cdid: String,

    pub cell: HEALPixFreqCell,
}

use web_sys::{Worker, WorkerOptions};
pub fn create_worker() -> Result<Worker, JsValue> {
    // JS source code of the worker
    let worker_source = r#"
        self.onmessage = (e) => {
            const { bitmap, tileDepth, tileSize, HiPS, cell } = e.data;

            // Compute tiling layout
            const numCols = Math.floor(bitmap.width / tileSize);

            // See HiPS3D doc
            const numRows = Math.ceil(tileDepth / numCols);

            // Create OffscreenCanvas
            const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
            const context = canvas.getContext("2d");

            // Draw full image once
            context.drawImage(bitmap, 0, 0);

            // Extract full region needed
            const imageData = context.getImageData(
                0,
                0,
                numCols * tileSize,
                numRows * tileSize
            );

            const bytes = imageData.data; // Uint8ClampedArray (RGBA)

            // Allocate output buffer (2 bytes per pixel)
            const decodedBytes = new Uint8Array(
                tileSize * tileSize * tileDepth * 2
            );

            let k = 0;
            let numTilesCropped = 0;

            for (let y = 0; y < numRows; y++) {
                const sy = y * tileSize;

                for (let x = 0; x < numCols; x++) {
                    const sx = x * tileSize;

                    for (let i = sy; i < sy + tileSize; i++) {
                        for (let j = sx; j < sx + tileSize; j++) {

                            const idByte = (j + i * numCols * tileSize) * 4;

                            // Copy R channel
                            decodedBytes[k] = bytes[idByte];

                            // Copy A channel
                            decodedBytes[k + 1] = bytes[idByte + 3];

                            k += 2;
                        }
                    }

                    numTilesCropped++;

                    if (numTilesCropped === tileDepth) {
                        break;
                    }
                }

                if (numTilesCropped === tileDepth) {
                    break;
                }
            }

            self.postMessage(
                {
                    tileSize,
                    tileDepth,
                    HiPSCDid: HiPS,
                    cell,
                    bytes: decodedBytes,
                },
                [decodedBytes.buffer] // transfer of ownership
            );
        };
    "#;

    // Create Blob
    let parts = js_sys::Array::of1(&JsValue::from_str(worker_source));
    let blob = web_sys::Blob::new_with_str_sequence(&parts)?;

    // Create object URL
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let opts = WorkerOptions::new();

    let worker = Worker::new_with_options(&url, &opts)?;
    Ok(worker)
}
