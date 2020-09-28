#[macro_use]
extern crate itertools_num;
extern crate serde_derive;
extern crate serde_json;
extern crate num;
extern crate task_async_executor;
extern crate console_error_panic_hook;
extern crate fitsreader;
use std::panic;

#[macro_use]
mod utils;

use wasm_bindgen::{prelude::*, JsCast};

mod shader;
mod shaders;
pub mod renderable;
mod finite_state_machine;
pub mod camera;
mod core;
mod math;
#[path = "../img/myfont.rs"]
mod myfont;
mod transfert_function;
mod projeted_grid;
//mod mouse_inertia;
mod event_manager;
mod color;
mod healpix_cell;
mod buffer;
mod rotation;
mod sphere_geometry;
mod cdshealpix;
mod async_task;
mod time;
mod image_fmt;
pub use image_fmt::FormatImageType;

use crate::{
    shader::{Shader, ShaderManager},
    renderable::{
        HiPSSphere, TextManager, Angle, ArcDeg,
        grid::ProjetedGrid,
        catalog::{Source, Manager},
        projection::{Aitoff, Orthographic, Mollweide, AzimutalEquidistant, Mercator, Projection},
    },
    camera::CameraViewport,
    finite_state_machine:: {UserMoveSphere, UserZoom, FiniteStateMachine, MoveSphere},
    math::{LonLatT, LonLat},
    async_task::{TaskResult, TaskType},
    buffer::HiPSConfig,
};

use std::{
    rc::Rc,
    collections::HashSet
};

use cgmath::Vector4;

use async_task::AladinTaskExecutor;
use web_sys::WebGl2RenderingContext;
struct App {
    gl: WebGl2Context,

    shaders: ShaderManager,
    camera: CameraViewport,

    buffer: TileBuffer,
    surveys: ImageSurveys,
    view: HEALPixCellsInView,
    
    rasterizer: Rasterizer,
    raytracer: RayTracer,
    time_start_blending: Time,

    // The grid renderable
    grid: ProjetedGrid,
    // Catalog manager
    manager: Manager,
    // Text example
    text_manager: TextManager,

    // Finite State Machine declarations
    user_move_fsm: UserMoveSphere,
    user_zoom_fsm: UserZoom,
    move_fsm: MoveSphere,

    // Task executor
    exec: AladinTaskExecutor,
    resources: Resources,

    move_animation: Option<MoveAnimation>,
    tasks_finished: bool,
}

#[derive(Debug, Deserialize)]
pub struct Resources(HashMap<String, String>);

impl Resources {
    pub fn get_filename<'a>(&'a self, name: &str) -> Option<&String> {
        self.0.get(name)
    }
}

use cgmath::Vector2;
use futures::stream::StreamExt; // for `next`

use crate::shaders::Colormap;
use crate::rotation::SphericalRotation;
use crate::finite_state_machine::move_renderables;
struct MoveAnimation {
    start_anim_rot: SphericalRotation<f32>,
    goal_anim_rot: SphericalRotation<f32>,
    time_start_anim: Time,
}

const BLEND_TILE_ANIM_DURATION: f32 = 500.0; // in ms

impl App {
    fn new(gl: &WebGl2Context, _events: &EventManager, mut shaders: ShaderManager, resources: Resources) -> Result<Self, JsValue> {
        //gl.enable(WebGl2RenderingContext::BLEND);
        //gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);
        //gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
        gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);

        let hips_definition = HiPSDefinition {
            id: String::from("SDSS/DR9/color"),
            url: String::from("http://alasky.u-strasbg.fr/SDSS/DR9/color"),
            name: String::from("SDSS/DR9/color"),
            maxOrder: 10,
            frame: Frame { label: String::from("J2000"), system: String::from("J2000") },
            tileSize: 512,
            minCutout: 0.0,
            maxCutout: 1.0,
            format: String::from("jpeg"),
            bitpix: 0,
        };

        let config = HiPSConfig::new(gl, hips_definition)?;

        // camera definition
        // HiPS definition
        /*let config = HiPSConfig::new(
            gl,
            String::from("http://alasky.u-strasbg.fr/SDSS/DR9/color"), // Name of the HiPS
            10, // max depth of the HiPS
            512, // Size of a texture tile
            0.0,
            1.0,
            TransferFunction::Asinh,
            FormatImageType::JPG, // Format of the tile texture images,
            None,
        );*/

        log("shaders compiled");
        //panic!(format!("{:?}", aa));
        let camera = CameraViewport::new::<Orthographic>(&gl);

        // The tile buffer responsible for the tile requests
        let buffer = TileBuffer::new(gl, &camera);
        // The surveys storing the textures of the resolved tiles
        let surveys = ImageSurveys::new();
        // A view on the survey
        // This will be updated whenever the user does an action
        // to get the current cells being in the FoV
        let view = HEALPixCellsInView::new();
        // Two mode of render, each storing a specific VBO
        // - The rasterizer draws the HEALPix cells being in the current view
        // This mode of rendering is used for small FoVs
        let rasterizer = Rasterizer::new(&gl, &shaders);
        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new::<Orthographic>(&gl, &camera, &shaders);

        let time_start_blending = Time::now();

        // Catalog definition
        let manager = Manager::new(&gl, &mut shaders, &camera, &resources);

        // Text 
        let font = myfont::FONT_CONFIG;
        let text_manager = TextManager::new(&gl, font, &mut shaders);
        /*let _text = TextUponSphere::new(
            String::from("Aladin-Lite"),
            //&Vector2::new(300_f32, 100_f32),
            &Vector4::new(0.0, 1.0, 0.0, 1.0),
            &gl,
            &_text_manager,
            &shaders,
        );*/

        // Grid definition
        let grid = ProjetedGrid::new::<Orthographic>(&gl, &camera, &mut shaders, &text_manager);

        // Finite State Machines definitions
        let user_move_fsm = UserMoveSphere::init();
        let user_zoom_fsm = UserZoom::init();
        let move_fsm = MoveSphere::init();

        let gl = gl.clone();
        let exec = AladinTaskExecutor::new();

        // Variable storing the location to move to
        let move_animation = None;
        let tasks_finished = false;
        let start_render_time = Time::now();
        let app = App {
            gl,

            shaders,

            camera,

            buffer,
            surveys,
            view,
            rasterizer,
            raytracer,
            time_start_blending,
            // The grid renderable
            grid,
            // The catalog renderable
            manager,
            text_manager,
            
            // Finite state machines,
            user_move_fsm,
            user_zoom_fsm,
            move_fsm,

            exec,
            resources,

            move_animation,

            tasks_finished,
        };
        Ok(app)
    } 

    fn look_for_new_tiles(&mut self, camera: &CameraViewPort) {
        // Move the views of the different active surveys
        self.surveys.update_views(camera);

        // Loop over the surveys
        for (root_url, survey) in self.surveys.iter() {
            let view = self.surveys.get_view_survey(root_url);
            let cells_in_fov = view.get_cells();

            for cell in cells_in_fov.iter() {
                let already_available = survey.contains_tile(cell);
                let is_cell_new = view.is_new(cell);

                if already_available {
                    // Remove and append the texture with an updated
                    // time_request
                    if is_cell_new {
                        // New cells are 
                        self.time_start_blending = Time::now();
                    }
                    survey.update_priority(cell, is_cell_new);
                } else {
                    // Submit the request to the buffer
                    let root_url = survey.get_root_url();
                    let format = survey.get_image_format();
                    let tile = Tile {
                        root_url,
                        format,
                        cell
                    };
                    self.buffer.request_tile(&tile);
                }
            }
        }
    }

    // Run async tasks:
    // - parsing catalogs
    // - copying textures to GPU
    // Return true when a task is complete. This always lead
    // to a redraw of aladin lite
    fn run_tasks<P: Projection>(&mut self, dt: DeltaTime) -> Result<HashSet<Tile>, JsValue> {
        //crate::log(&format!("last frame duration (ms): {:?}", dt));
        let results = self.exec.run(dt.0 * 0.5_f32);
        self.tasks_finished = !results.is_empty();

        // Retrieve back all the tiles that have been
        // copied to the GPU
        // This is important for the tile buffer to know which
        // requests can be reused to query more tiles
        let mut tiles_available = HashSet::new();
        for result in results {
            match result {
                TaskResult::TableParsed { name, sources} => {
                    log("CATALOG FINISHED PARSED");
                    self.manager.add_catalog::<P>(name, sources, Colormap::BluePastelRed, &mut self.shaders, &self.camera, self.sphere.config());
                },
                TaskResult::TileSentToGPU { tile } => {
                    tiles_available.insert(tile);
                }
            }
        }

        Ok(tiles)
    }

    fn update<P: Projection>(&mut self,
        dt: DeltaTime,
        events: &EventManager,
    ) -> Result<bool, JsValue> {
        let available_tiles = self.run_tasks::<P>(dt)?;
        let is_there_new_available_tiles = !available_tiles.is_empty();

        // Run the FSMs
        //self.user_move_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);
        //self.user_zoom_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);
        //self.move_fsm.run::<P>(dt, &mut self.sphere, &mut self.manager, &mut self.grid, &mut self.camera, &events);

        // Check if there is an move animation to do
        if let Some(MoveAnimation {start_anim_rot, goal_anim_rot, time_start_anim} ) = self.move_animation {
            let t = (utils::get_current_time() - time_start_anim.as_millis())/1000_f32;
        
            // Undamped angular frequency of the oscillator
            // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
            //
            // In a damped harmonic oscillator system: w0 = sqrt(k / m)
            // where: 
            // * k is the stiffness of the ressort
            // * m is its mass
            let alpha = 1_f32 + (0_f32 - 1_f32) * (5_f32 * t + 1_f32) * ((-5_f32 * t).exp());
            let p = start_anim_rot.slerp(&goal_anim_rot, alpha);
    
            camera.set_rotation::<P>(&p);
            self.look_for_new_tiles();

            // Animation stop criteria
            let cursor_pos = self.get_center::<P>();
            let goal_pos = location;
            let err = math::ang_between_vect(&goal_pos, &cursor_pos);
            let thresh: Angle<f32> = ArcSec(2_f32).into();
            if err < thresh {
                self.move_animation = None;
            }
        }

        // Update the grid in consequence
        //self.grid.update_label_positions::<P>(&self.gl, &mut self.text_manager, &self.camera, &self.shaders);
        //self.text.update_from_camera::<P>(&self.camera);
        {
            // Newly available tiles must lead to
            if is_there_new_available_tiles {
                self.time_latest_available_tile = Time::now();
            }

            // 1. Surveys must be aware of the new available tiles
            self.surveys.set_available_tiles(available_tiles);
            // 2. Get the resolved tiles and push them to the image surveys
            let resolved_tiles = self.buffer.get_resolved_tiles(available_tiles);
            self.surveys.add_resolved_tiles(resolved_tiles, &mut self.exec);
            // 3. Try sending new tile requests after 
            self.buffer.try_sending_tile_requests();
        }

        // The rendering is done following these different situations:
        // - the camera has moved
        let has_camera_moved = camera.has_camera_moved();
        // - there is at least one tile in its blending phase
        let blending_anim_occuring = (Time::now() - self.time_latest_available_tile) < BLEND_TILE_ANIM_DURATION;
        let render = blending_anim_occuring | has_camera_moved;

        // Finally update the camera that reset the flag camera changed
        if has_camera_moved {
            self.manager.update(&self.camera);
        }

        // Then we compute different boolean for the update of the VBO
        // for the rasterizer
        // The rasterizer has a buffer containing:
        // - The vertices of the HEALPix cells for the most refined survey
        // - The starting and ending uv for the blending animation
        // - The time for each HEALPix cell at which the animation begins
        //
        // Each of these data can be changed at different circumstances:
        // - The vertices are changed if:
        //     * new cells are added/removed (because new cells are added)
        //   to the previous frame.
        // - The UVs are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU 
        // - The starting blending animation times are changed if:
        //     * new cells are added/removed (because new cells are added)
        //     * there are new available tiles for the GPU
        let view_most_refined = self.surveys.get_most_refined_view();
        let new_cells_added = view_most_refined.is_there_new_cells_added();

        let update_positions = new_cells_added;
        let update_uv = update_vertices | is_there_new_available_tiles;
        let update_starting_blending_times = update_uv;

        if update_positions {
            let positions = vec![];
            for cell in view_most_refined.get_cells() {

            }

            self.rasterizer.set_positions();
        } else {

        }

        Ok(render)
    }

    fn render<P: Projection>(&mut self, _enable_grid: bool) {
        // Render the scene
        self.gl.clear_color(0.08, 0.08, 0.08, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // Draw renderables here
        let camera = &self.camera;
        let shaders = &mut self.shaders;
        self.gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE);

        // Draw the HiPS sphere
        self.sphere.draw::<P>(
            &self.gl,
            shaders,
            camera,
        );
        self.gl.enable(WebGl2RenderingContext::BLEND);

        //self.gl.blend_func_separate(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE);
        // Draw the catalog
        self.manager.draw::<P>(
            &self.gl,
            shaders,
            camera
        );
        
        // Draw the grid
        /*self.grid.draw::<P>(
            &self.gl,
            shaders,
            camera,
            &self.text_manager
        );*/

        /*self.text_manager.draw(
            &self.gl,
            shaders,
            camera
        );*/
        self.gl.disable(WebGl2RenderingContext::BLEND);
    }

    fn set_image_survey<P: Projection>(&mut self, hips_definition: HiPSDefinition) -> Result<(), JsValue> {
        self.sphere.set_image_survey::<P>(hips_definition, &mut self.camera, &mut self.exec)
    }

    fn set_projection<P: Projection>(&mut self) {
        self.camera.reset::<P>(self.sphere.config());
        // Update the camera
        {
            let cam = self.camera;
            let screen_size = cam.get_screen_size();
            let aperture = cam.get_aperture();
            
            cam.set_aperture::<P>(aperture);
            cam.resize::<P>(screen_size.x, screen_size.y);
        }
        self.sphere.set_projection::<P>(&self.camera, &mut self.shaders);
    }
    fn add_catalog(&mut self, name: String, table: JsValue) {
        let spawner = self.exec.spawner();
        let table = table;
        spawner.spawn(TaskType::ParseTable, async {
            let mut stream = async_task::ParseTable::<[f32; 4]>::new(table);
            crate::log("BEGIN PARSING");
            let mut results: Vec<Source> = vec![];
        
            while let Some(item) = stream.next().await {
                let item: &[f32] = item.as_ref();
                results.push(item.into());
            }
        
            crate::log("END PARSING");
            TaskResult::TableParsed { name, sources: results }
        });
    }

    fn resize_window<P: Projection>(&mut self, width: f32, height: f32, _enable_grid: bool) {
        self.camera.resize::<P>(width, height);

        // Launch the new tile requests
        self.look_for_new_tiles();
        manager.set_kernel_size(&self.camera);
    }

    fn set_colormap(&mut self, name: String, colormap: Colormap) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_colormap(colormap);
    }

    fn set_heatmap_opacity(&mut self, name: String, opacity: f32) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_alpha(opacity);
    }

    fn set_kernel_strength<P: Projection>(&mut self, name: String, strength: f32) {
        self.manager.get_mut_catalog(&name)
            .unwrap()
            .set_strength(strength);
    }

    pub fn set_grid_color(&mut self, _red: f32, _green: f32, _blue: f32) {
        //self.grid.set_color(red, green, blue);
    }

    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) {
        //self.sphere.set_cutouts(min_cutout, max_cutout);
    }

    pub fn set_transfer_func(&mut self, id: String) {
        //self.sphere.set_transfer_func(TransferFunction::new(&id));
    }

    pub fn set_fits_colormap(&mut self, colormap: Colormap) {
        //self.sphere.set_fits_colormap(colormap);
    }

    pub fn set_grid_opacity(&mut self, _alpha: f32) {
        //self.grid.set_alpha(alpha);
    }

    pub fn screen_to_world<P: Projection>(&self, pos: &Vector2<f32>) -> Result<LonLatT<f32>, String> {
        let model_pos = P::screen_to_model_space(pos, &self.camera).ok_or(format!("{:?} is out of projection", pos))?;
        Ok(model_pos.lonlat())
    }

    pub fn set_center<P: Projection>(&mut self, lonlat: &LonLatT<f32>, events: &mut EventManager) {
        let xyz: Vector4<f32> = lonlat.vector();
        let rot = SphericalRotation::from_sky_position(&xyz);
        self.camera.set_rotation::<P>(&rot);
        self.look_for_new_tiles();
    }

    pub fn start_moving_to<P: Projection>(&mut self, lonlat: &LonLatT<f32>) {
        // Get the XYZ cartesian position from the lonlat
        let cursor_pos: Vector4<f32> = self.get_center().vector();
        let goal_pos: Vector4<f32> = lonlat.vector();

        // Convert these positions to rotations
        let start_anim_rot = SphericalRotation::from_sky_position(&cursor_pos);
        let goal_anim_rot = SphericalRotation::from_sky_position(&goal_pos);

        // Set the moving animation object
        self.move_animation = Some(MoveAnimation {
            time_start_moving: Time::now(),
            start_anim_rot,
            goal_anim_rot,
        });
    }

    /*pub fn move_camera<P: Projection>(&mut self, pos1: &LonLatT<f32>, pos2: &LonLatT<f32>) {
        let model2world = self.camera.get_inverted_model_mat();

        let m1: Vector4<f32> = pos1.vector();
        let w1 = model2world * m1;
        let m2: Vector4<f32> = pos2.vector();
        let w2 = model2world * m2;

        move_renderables::<P>(
            &w1,
            &w2,
            //&mut self.sphere,
            &mut self.manager,
            &mut self.grid,
            &mut self.camera
        );
    }*/
    
    pub fn set_fov<P: Projection>(&mut self, fov: &Angle<f32>) {
        // Change the camera rotation
        self.camera.set_aperture::<P>(*fov);
        self.look_for_new_tiles();
    }

    // Accessors
    fn get_center<P: Projection>(&self) -> LonLatT<f32> {
        let center_pos = self.camera.compute_center_model_pos::<P>();
        center_pos.lonlat()
    }

    fn get_fov(&self) -> f32 {
        let deg: ArcDeg<f32> = self.camera.get_aperture().into();
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

        let canvas = document.get_elements_by_class_name("aladin-imageCanvas").get_with_index(0).unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context_options = js_sys::JSON::parse(&"{\"antialias\":false}").unwrap();
        let inner = Rc::new(
            canvas.get_context_with_context_options("webgl2", context_options.as_ref())
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap()
        );

        WebGl2Context {
            inner,
        }
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
}

impl ProjectionType {
    fn set_projection(&mut self, app: &mut App, name: String) {
        *self = match name.as_str() {
            "aitoff" => {
                app.set_projection::<Aitoff>();
                ProjectionType::Aitoff
            },
            "orthographic" => {
                app.set_projection::<Orthographic>();
                ProjectionType::Ortho
            },
            "mollweide" => {
                app.set_projection::<Mollweide>();
                ProjectionType::MollWeide
            },
            "arc" => {
                app.set_projection::<AzimutalEquidistant>();
                ProjectionType::Arc
            },
            "mercator" => {
                app.set_projection::<Mercator>();
                ProjectionType::Mercator
            },
            _ => unimplemented!()
        }
    }

    fn set_colormap(&self, app: &mut App, name: String, colormap: Colormap) {
        match self {
            ProjectionType::Aitoff => app.set_colormap(name, colormap),
            ProjectionType::MollWeide => app.set_colormap(name, colormap),
            ProjectionType::Ortho => app.set_colormap(name, colormap),
            ProjectionType::Arc => app.set_colormap(name, colormap),
            ProjectionType::Mercator => app.set_colormap(name, colormap),
        };
    }

    fn screen_to_world(&self, app: &App, pos: &Vector2<f32>) -> Result<LonLatT<f32>, String> {
        match self {
            ProjectionType::Aitoff => app.screen_to_world::<Aitoff>(pos),
            ProjectionType::MollWeide => app.screen_to_world::<Mollweide>(pos),
            ProjectionType::Ortho => app.screen_to_world::<Orthographic>(pos),
            ProjectionType::Arc => app.screen_to_world::<Mollweide>(pos),
            ProjectionType::Mercator => app.screen_to_world::<Mercator>(pos),
        }
    }

    /*fn move_camera(&self, app: &mut App, pos1: &LonLatT<f32>, pos2: &LonLatT<f32>) {
        match self {
            ProjectionType::Aitoff => app.move_camera::<Aitoff>(pos1, pos2),
            ProjectionType::MollWeide => app.move_camera::<Mollweide>(pos1, pos2),
            ProjectionType::Ortho => app.move_camera::<Orthographic>(pos1, pos2),
            ProjectionType::Arc => app.move_camera::<Mollweide>(pos1, pos2),
            ProjectionType::Mercator => app.move_camera::<Mercator>(pos1, pos2),
        }
    }*/

    fn update(&mut self, app: &mut App, dt: DeltaTime, events: &EventManager) -> Result<bool, JsValue> {
        match self {
            ProjectionType::Aitoff => app.update::<Aitoff>(dt, events),
            ProjectionType::MollWeide => app.update::<Mollweide>(dt, events),
            ProjectionType::Ortho => app.update::<Orthographic>(dt, events),
            ProjectionType::Arc => app.update::<Mollweide>(dt, events),
            ProjectionType::Mercator => app.update::<Mercator>(dt, events),
        }
    }

    fn render(&mut self, app: &mut App, enable_grid: bool) {
        match self {
            ProjectionType::Aitoff => app.render::<Aitoff>(enable_grid),
            ProjectionType::MollWeide => app.render::<Mollweide>(enable_grid),
            ProjectionType::Ortho => app.render::<Orthographic>(enable_grid),
            ProjectionType::Arc => app.render::<Mollweide>(enable_grid),
            ProjectionType::Mercator => app.render::<Mercator>(enable_grid),
        };
    }

    pub fn add_catalog(&mut self, app: &mut App, name: String, table: JsValue) {
        match self {
            ProjectionType::Aitoff => app.add_catalog(name, table),
            ProjectionType::MollWeide => app.add_catalog(name, table),
            ProjectionType::Ortho => app.add_catalog(name, table),
            ProjectionType::Arc => app.add_catalog(name, table),
            ProjectionType::Mercator => app.add_catalog(name, table),
        };
    }

    pub fn set_image_survey(&mut self, app: &mut App, hips_definition: HiPSDefinition) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.set_image_survey::<Aitoff>(hips_definition),
            ProjectionType::MollWeide => app.set_image_survey::<Mollweide>(hips_definition),
            ProjectionType::Ortho => app.set_image_survey::<Orthographic>(hips_definition),
            ProjectionType::Arc => app.set_image_survey::<Mollweide>(hips_definition),
            ProjectionType::Mercator => app.set_image_survey::<Mercator>(hips_definition),
        }
    }

    pub fn resize(&mut self, app: &mut App, width: f32, height: f32, enable_grid: bool) {       
        match self {
            ProjectionType::Aitoff => app.resize_window::<Aitoff>(width, height, enable_grid),
            ProjectionType::MollWeide => app.resize_window::<Mollweide>(width, height, enable_grid),
            ProjectionType::Ortho => app.resize_window::<Orthographic>(width, height, enable_grid),
            ProjectionType::Arc => app.resize_window::<Mollweide>(width, height, enable_grid),
            ProjectionType::Mercator => app.resize_window::<Mercator>(width, height, enable_grid),
        }; 
    }

    pub fn set_kernel_strength(&mut self, app: &mut App, name: String, strength: f32) {        
        match self {
            ProjectionType::Aitoff => app.set_kernel_strength::<Aitoff>(name, strength),
            ProjectionType::MollWeide => app.set_kernel_strength::<Mollweide>(name, strength),
            ProjectionType::Ortho => app.set_kernel_strength::<Orthographic>(name, strength),
            ProjectionType::Arc => app.set_kernel_strength::<Mollweide>(name, strength),
            ProjectionType::Mercator => app.set_kernel_strength::<Mercator>(name, strength),
        };
    }

    pub fn set_heatmap_opacity(&mut self, app: &mut App, name: String, opacity: f32) {       
        match self {
            ProjectionType::Aitoff => app.set_heatmap_opacity(name, opacity),
            ProjectionType::MollWeide => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Ortho => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Arc => app.set_heatmap_opacity(name, opacity),
            ProjectionType::Mercator => app.set_heatmap_opacity(name, opacity),
        }; 
    }

    pub fn set_center(&mut self, app: &mut App, lonlat: LonLatT<f32>, events: &mut EventManager) {
        match self {
            ProjectionType::Aitoff => app.set_center::<Aitoff>(&lonlat, events),
            ProjectionType::MollWeide => app.set_center::<Mollweide>(&lonlat, events),
            ProjectionType::Ortho => app.set_center::<Orthographic>(&lonlat, events),
            ProjectionType::Arc => app.set_center::<Mollweide>(&lonlat, events),
            ProjectionType::Mercator => app.set_center::<Mercator>(&lonlat, events),
        };
    }

    pub fn start_moving_to(&mut self, app: &mut App, lonlat: LonLatT<f32>) {
        match self {
            ProjectionType::Aitoff => app.start_moving_to::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.start_moving_to::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.start_moving_to::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.start_moving_to::<Mollweide>(&lonlat),
            ProjectionType::Mercator => app.start_moving_to::<Mercator>(&lonlat),
        };
    }

    pub fn set_fov(&mut self, app: &mut App, fov: Angle<f32>) {
        match self {
            ProjectionType::Aitoff => app.set_fov::<Aitoff>(&fov),
            ProjectionType::MollWeide => app.set_fov::<Mollweide>(&fov),
            ProjectionType::Ortho => app.set_fov::<Orthographic>(&fov),
            ProjectionType::Arc => app.set_fov::<Aitoff>(&fov),
            ProjectionType::Mercator => app.set_fov::<Mercator>(&fov),
        };
    }

    pub fn get_center(&self, app: &App) -> LonLatT<f32> {
        match self {
            ProjectionType::Aitoff => app.get_center::<Aitoff>(),
            ProjectionType::MollWeide => app.get_center::<Mollweide>(),
            ProjectionType::Ortho => app.get_center::<Orthographic>(),
            ProjectionType::Arc => app.get_center::<Aitoff>(),
            ProjectionType::Mercator => app.get_center::<Mercator>(),
        }
    }

    pub fn set_grid_color(&mut self, app: &mut App, red: f32, green: f32, blue: f32) {
        match self {
            ProjectionType::Aitoff => app.set_grid_color(red, green, blue),
            ProjectionType::MollWeide => app.set_grid_color(red, green, blue),
            ProjectionType::Ortho => app.set_grid_color(red, green, blue),
            ProjectionType::Arc => app.set_grid_color(red, green, blue),
            ProjectionType::Mercator => app.set_grid_color(red, green, blue),
        };
    }

    pub fn set_cutouts(&mut self, app: &mut App, min_cutout: f32, max_cutout: f32) {
        match self {
            ProjectionType::Aitoff => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::MollWeide => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Ortho => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Arc => app.set_cutouts(min_cutout, max_cutout),
            ProjectionType::Mercator => app.set_cutouts(min_cutout, max_cutout),
        };
    }

    pub fn set_transfer_func(&mut self, app: &mut App, id: String) {
        match self {
            ProjectionType::Aitoff => app.set_transfer_func(id),
            ProjectionType::MollWeide => app.set_transfer_func(id),
            ProjectionType::Ortho => app.set_transfer_func(id),
            ProjectionType::Arc => app.set_transfer_func(id),
            ProjectionType::Mercator => app.set_transfer_func(id),
        };
    }

    pub fn set_fits_colormap(&mut self, app: &mut App, colormap: Colormap) {
        match self {
            ProjectionType::Aitoff => app.set_fits_colormap(colormap),
            ProjectionType::MollWeide => app.set_fits_colormap(colormap),
            ProjectionType::Ortho => app.set_fits_colormap(colormap),
            ProjectionType::Arc => app.set_fits_colormap(colormap),
            ProjectionType::Mercator => app.set_fits_colormap(colormap),
        };
    }

    pub fn set_grid_opacity(&mut self, app: &mut App, alpha: f32) {
        match self {
            ProjectionType::Aitoff => app.set_grid_opacity(alpha),
            ProjectionType::MollWeide => app.set_grid_opacity(alpha),
            ProjectionType::Ortho => app.set_grid_opacity(alpha),
            ProjectionType::Arc => app.set_grid_opacity(alpha),
            ProjectionType::Mercator => app.set_grid_opacity(alpha),
        };
    }
}

use crate::event_manager::{
 EventManager,
 MoveToLocation,
 SetCenterLocation,
 StartInertia,
 SetFieldOfView,
 ZoomToLocation
};
use crate::time::DeltaTime;
#[wasm_bindgen]
pub struct WebClient {
    gl: WebGl2Context,
    // The app
    app: App,
    projection: ProjectionType,
    // Stores all the possible events
    // with some associated data    
    events: EventManager,

    // The time between the previous and the current
    // frame
    dt: DeltaTime,

    // Some booleans for enabling/desabling
    // specific computations
    enable_inertia: bool,
    enable_grid: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use serde::Deserialize; 
#[derive(Debug, Deserialize)]
pub struct FileSrc {
    pub id: String,
    pub content: String,
}
use crate::transfert_function::TransferFunction;
use std::collections::HashMap;
use crate::healpix_cell::HEALPixCell;


#[derive(Deserialize)]
#[derive(Debug)]
pub struct Frame {
    pub label: String,
    pub system: String,
}
use js_sys::JsString;
#[derive(Deserialize)]
#[derive(Debug)]
pub struct HiPSDefinition {
    pub id: String,
    pub url: String,
    pub name: String,
    pub maxOrder: u8,
    pub frame: Frame,
    pub tileSize: i32,
    pub format: String,
    pub minCutout: f32,
    pub maxCutout: f32,
    pub bitpix: i32,
}

#[wasm_bindgen]
impl WebClient {
    /// Create a new web client
    #[wasm_bindgen(constructor)]
    pub fn new(shaders: &JsValue, resources: &JsValue) -> Result<WebClient, JsValue> {
        crate::log(&format!("shaders manager2"));

        let shaders = shaders.into_serde::<Vec<FileSrc>>().unwrap();
        let resources = resources.into_serde::<Resources>().unwrap();
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let gl = WebGl2Context::new();
        let events = EventManager::new();
        crate::log(&format!("shaders manager"));

        let shaders = ShaderManager::new(&gl, shaders).unwrap();
        let app = App::new(&gl, &events, shaders, resources)?;

        //let appconfig = AppConfig::Ortho(app);
        let dt = DeltaTime::zero();
        let enable_inertia = false;
        let enable_grid = true;
        let projection = ProjectionType::Ortho;

        let webclient = WebClient {
            gl,
            app,
            projection,

            events,

            dt,
            enable_inertia,
            enable_grid,
        };

        Ok(webclient)
    }

    /// Main update method
    pub fn update(&mut self, dt: f32) -> Result<bool, JsValue> {
        // dt refers to the time taking (in ms) rendering the previous frame
        self.dt = DeltaTime::from_millis(dt);

        // Update the application and get back the
        // world coordinates of the center of projection in (ra, dec)
        let render = self.projection.update(
            &mut self.app,
            // Time of the previous frame rendering 
            self.dt,
            // Event manager
            &self.events,
        )?;

        // Reset all the user events at the end of the frame
        self.events.reset();

        Ok(render)
    }
    
    /// Update our WebGL Water application. `index.html` will call this function in order
    /// to begin rendering.
    pub fn render(&mut self, min_value: f32, max_value: f32) -> Result<(), JsValue> {
        self.projection.render(&mut self.app, self.enable_grid);

        Ok(())
    }

    /// Change the current projection of the HiPS
    pub fn set_projection(&mut self, name: String) -> Result<(), JsValue> {
        self.projection.set_projection(&mut self.app, name);

        Ok(())
    }

    /// Enable mouse inertia
    pub fn enable_inertia(&mut self) -> Result<(), JsValue> {
        self.enable_inertia = true;

        Ok(())
    }
    /// Disable mouse inertia
    pub fn disable_inertia(&mut self) -> Result<(), JsValue> {
        self.enable_inertia = false;

        Ok(())
    }

    /// Enable equatorial grid
    pub fn enable_equatorial_grid(&mut self) -> Result<(), JsValue> {
        self.enable_grid = true;

        //self.projection.enable_grid(&mut self.app);

        Ok(())
    }

    /// Disable equatorial grid
    pub fn disable_equatorial_grid(&mut self) -> Result<(), JsValue> {
        self.enable_grid = false;

        Ok(())
    }
    
    /// Change grid color
    pub fn set_grid_color(&mut self, red: f32, green: f32, blue: f32) -> Result<(), JsValue> {
        self.projection.set_grid_color(&mut self.app, red, green, blue);

        Ok(())
    }

    /// Change grid opacity
    pub fn set_grid_opacity(&mut self, alpha: f32) -> Result<(), JsValue> {
        self.projection.set_grid_opacity(&mut self.app, alpha);

        Ok(())
    }
    
    #[wasm_bindgen(js_name = setCutouts)]
    pub fn set_cutouts(&mut self, min_cutout: f32, max_cutout: f32) -> Result<(), JsValue> {
        self.projection.set_cutouts(&mut self.app, min_cutout, max_cutout);

        Ok(())
    }

    #[wasm_bindgen(js_name = setFitsColormap)]
    pub fn set_fits_colormap(&mut self, colormap: String) -> Result<(), JsValue> {
        let colormap = Colormap::new(&colormap);
        self.projection.set_fits_colormap(&mut self.app, colormap);

        Ok(())
    }

    #[wasm_bindgen(js_name = setTransferFunction)]
    pub fn set_transfer_func(&mut self, id: String) -> Result<(), JsValue> {
        self.projection.set_transfer_func(&mut self.app, id);

        Ok(())
    }

    /// Change HiPS
    #[wasm_bindgen(js_name = setImageSurvey)]
    pub fn set_image_survey(&mut self,
        hips_definition: JsValue,
    ) -> Result<(), JsValue> {
        let hips_definition: HiPSDefinition = hips_definition.into_serde().unwrap();
        crate::log(&format!("hips_def222: {:?}", hips_definition));

        self.projection.set_image_survey(&mut self.app, hips_definition)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = screenToWorld)]
    pub fn screen_to_world(&self, pos_x: f32, pos_y: f32) -> Result<Box<[f32]>, JsValue> {
        let lonlat = self.projection.screen_to_world(&self.app, &Vector2::new(pos_x, pos_y))?;

        let lon_deg: ArcDeg<f32> = lonlat.lon().into();
        let lat_deg: ArcDeg<f32> = lonlat.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
    }

    #[wasm_bindgen(js_name = getFieldOfView)]
    pub fn get_fov(&self) -> Result<f32, JsValue> {
        let fov = self.app.get_fov();
        Ok(fov)
    }

    /// Set directly the field of view (for pinch zooming)
    #[wasm_bindgen(js_name = setFieldOfView)]
    pub fn set_fov(&mut self, fov: f32) -> Result<(), JsValue> {
        self.projection.set_fov(&mut self.app, ArcDeg(fov).into());
        // Tell the finite state machines the fov has manually been changed
        //self.events.enable::<SetFieldOfView>(());

        Ok(())
    }

    /// Set directly the center position
    #[wasm_bindgen(js_name = getCenter)]
    pub fn get_center(&self) -> Result<Box<[f32]>, JsValue> {
        let center = self.projection.get_center(&self.app);

        let lon_deg: ArcDeg<f32> = center.lon().into();
        let lat_deg: ArcDeg<f32> = center.lat().into();

        Ok(Box::new([lon_deg.0, lat_deg.0]))
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
    pub fn start_moving_to(&mut self, lon: f32, lat: f32) -> Result<(), JsValue> {
        // Enable the MouseLeftButtonReleased event
        let location = LonLatT::new(
            ArcDeg(lon).into(),
            ArcDeg(lat).into()
        );
        self.projection.start_moving_to(&mut self.app, lonlat);

        /*self.events.enable::<MoveToLocation>(
            LonLatT::new(
                ArcDeg(lon).into(),
                ArcDeg(lat).into()
            )
        );*/

        Ok(())
    }
    /// Set directly the center position
    #[wasm_bindgen(js_name = setCenter)]
    pub fn set_center(&mut self, lon: f32, lat: f32) -> Result<(), JsValue> {
        let lonlat = LonLatT::new(ArcDeg(lon).into(), ArcDeg(lat).into());
        self.projection.set_center(&mut self.app, lonlat, &mut self.events);

        Ok(())
    }
    /*#[wasm_bindgen(js_name = moveView)]
    pub fn displace(&mut self, lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> Result<(), JsValue> {
        let pos1 = LonLatT::new(
            ArcDeg(lon1).into(),
            ArcDeg(lat1).into()
        );
        let pos2 = LonLatT::new(
            ArcDeg(lon2).into(),
            ArcDeg(lat2).into()
        );
        self.projection.move_camera(&mut self.app, &pos1, &pos2);

        Ok(())
    }*/

    /// CATALOG INTERFACE METHODS
    /// Add new catalog
    pub fn add_catalog(&mut self, name_catalog: String, data: JsValue) -> Result<(), JsValue> {
        self.projection.add_catalog(&mut self.app, name_catalog, data);

        Ok(())
    }

    /// Set the kernel strength
    pub fn set_kernel_strength(&mut self, name_catalog: String, strength: f32) -> Result<(), JsValue> {
        self.projection.set_kernel_strength(&mut self.app, name_catalog, strength);

        Ok(())
    }

    /// Set the heatmap global opacity
    pub fn set_heatmap_opacity(&mut self, name_catalog: String, opacity: f32) -> Result<(), JsValue> {
        self.projection.set_heatmap_opacity(&mut self.app, name_catalog, opacity);

        Ok(())
    }

    pub fn set_colormap(&mut self, name_catalog: String, name_colormap: String) -> Result<(), JsValue> {
        let colormap = match name_colormap.as_str() {
            "BluePastelRed" => Colormap::BluePastelRed,
            "IDL_CB_BrBG" => Colormap::IDLCBBrBG,
            "IDL_CB_YIGnBu" => Colormap::IDLCBYIGnBu,
            "IDL_CB_GnBu" => Colormap::IDLCBGnBu,
            "Red_Temperature" => Colormap::RedTemperature,
            "Black_White_Linear" => Colormap::BlackWhiteLinear,
            _ => panic!("{:?} colormap not recognized!", name_colormap)
        };
        self.projection.set_colormap(&mut self.app, name_catalog, colormap);

        Ok(())
    }

    /// Resize the window
    pub fn resize(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.projection.resize(&mut self.app, width, height, self.enable_grid);

        Ok(())
    }
}

#[wasm_bindgen]
pub struct HEALPixCellJS {
    pub depth: u8,
    pub idx: u64
}
