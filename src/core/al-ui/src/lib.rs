use al_core;

mod painter;
use al_api::hips::SimpleHiPS;
use painter::WebGl2Painter;

mod input;
pub use input::GuiRef;

mod layout;
pub mod widgets;

use egui;
use egui_web::Painter;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

// ----------------------------------------------------------------------------

use std::sync::atomic::Ordering::SeqCst;

pub struct NeedRepaint(std::sync::atomic::AtomicBool);

impl Default for NeedRepaint {
    fn default() -> Self {
        Self(true.into())
    }
}

impl NeedRepaint {
    pub fn fetch_and_clear(&self) -> bool {
        self.0.swap(false, SeqCst)
    }

    pub fn set_true(&self) {
        self.0.store(true, SeqCst);
    }
}

impl epi::RepaintSignal for NeedRepaint {
    fn request_repaint(&self) {
        self.0.store(true, SeqCst);
    }
}

#[derive(Debug)]
pub enum Event {
    Grid {
        color: [f32; 4],
        line_thickness: f32,
        enable: bool,
    },
    Location {
        name: String,
    },
    ImageSurveys(Vec<SimpleHiPS>),
}

pub struct Gui {
    pub input: egui_web::WebInput,
    pub painter: WebGl2Painter,
    ctx: egui::CtxRef,

    pub needs_repaint: std::sync::Arc<NeedRepaint>,
    pub last_text_cursor_pos: Option<egui::Pos2>,

    pub aladin_lite_div: String,

    // The layout contains all the ui definition
    layout: layout::LayerLayout,
    clipped_meshes: Option<Vec<egui::ClippedMesh>>,

    pub mouse_on_ui: bool,
    pub mouse_pressed: bool,
    pub cur_mouse_pos: egui::Pos2,

    events: Arc<Mutex<Vec<Event>>>,
}
use al_core::WebGlContext;
use cgmath::Vector2;
use egui::Pos2;
use std::sync::{Arc, Mutex};
impl Gui {
    pub fn new(aladin_lite_div: &str, gl: &WebGlContext) -> Result<GuiRef, JsValue> {
        let ctx = egui::CtxRef::default();
        let mut painter = WebGl2Painter::new(aladin_lite_div, gl.clone())?;
        let input: egui_web::backend::WebInput = Default::default();

        let layout = layout::LayerLayout::new(&mut painter)?;
        let mouse_on_ui = false;
        let cur_mouse_pos = egui::Pos2::ZERO;
        let events = Arc::new(Mutex::new(vec![]));

        let gui = Self {
            ctx,
            painter,

            input,
            layout,
            clipped_meshes: None,

            mouse_on_ui,
            mouse_pressed: false,
            cur_mouse_pos,

            needs_repaint: Default::default(),
            last_text_cursor_pos: None,

            aladin_lite_div: aladin_lite_div.to_string(),

            events,
        };

        let gui_ref = GuiRef(std::sync::Arc::new(egui::mutex::Mutex::new(gui)));

        input::install_canvas_events(gui_ref.clone())?;
        input::install_document_events(gui_ref.clone())?;
        input::install_text_agent(gui_ref.clone())?;

        Ok(gui_ref)
    }

    pub fn egui_ctx(&self) -> &egui::CtxRef {
        &self.ctx
    }

    pub fn pos_over_ui(&self) -> bool {
        self.mouse_on_ui
    }

    pub fn update(&mut self) -> Arc<Mutex<Vec<Event>>> {
        let canvas_size = egui::vec2(
            self.painter.canvas.width() as f32,
            self.painter.canvas.height() as f32,
        );
        let raw_input = self.input.new_frame(canvas_size);
        self.ctx.begin_frame(raw_input);

        // Define the central panel containing the ui
        {
            let f = egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                ..Default::default()
            };
            let layout = &mut self.layout;
            let events = self.events.clone();
            let response = egui::CentralPanel::default()
                .frame(f)
                .show(&self.ctx, |ui| {
                    /*egui::containers::panel::SidePanel::left(
                        "Menu"
                    )
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if ui.add(egui::Button::new("Search")).clicked() {
                                //do_stuff();
                            }
                            if ui.add(egui::Button::new("Layers")).clicked() {
                                //do_stuff();
                            }
                            if ui.add(egui::Button::new("Projections")).clicked() {
                                //do_stuff();
                            }
                            if ui.add(egui::Button::new("Grid")).clicked() {
                                //do_stuff();
                            }
                            if ui.add(egui::Button::new("Pipette")).clicked() {
                                //do_stuff();
                            }
                        });
                    });*/

                    layout.show(ui, events)
                });

            self.mouse_on_ui = response.inner.contains(self.cur_mouse_pos);
        }
        self.painter.upload_egui_texture(&self.ctx.texture());

        let (output, shapes) = self.ctx.end_frame();
        self.clipped_meshes = Some(self.ctx.tessellate(shapes)); // create triangles to paint

        input::handle_output(&output, self);

        self.events.clone()
    }

    pub fn draw(&mut self, gl: &WebGlContext, pixels_per_point: f32) -> Result<(), JsValue> {
        if let Some(meshes) = self.clipped_meshes.take() {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func(
                WebGl2RenderingContext::ONE,
                WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            ); // premultiplied alpha

            self.painter.paint_meshes(meshes, pixels_per_point)?;

            gl.disable(WebGl2RenderingContext::BLEND);
        }

        Ok(())
    }

    pub fn redraw_needed(&mut self) -> bool {
        let redraw = self.needs_repaint.fetch_and_clear();
        redraw
    }
}
