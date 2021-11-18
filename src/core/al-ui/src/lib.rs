mod painter;
use painter::WebGl2Painter;

mod input;
pub use input::GuiRef;

use egui;
use egui_web::Painter;
use wasm_bindgen::{prelude::*, JsCast};

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
pub trait Ui {
    fn show(&mut self, ui: &mut egui::Ui);
}

pub struct Gui
{
    //backend: egui_web::WebBackend,
    //web_input: egui_web::backend::WebInput,
    pub input: egui_web::WebInput,
    pub painter: WebGl2Painter,
    ctx: egui::CtxRef,

    pub needs_repaint: std::sync::Arc<NeedRepaint>,
    pub last_text_cursor_pos: Option<egui::Pos2>,

    pub aladin_lite_div: String,
}
use al_core::FrameBufferObject;
use al_core::WebGl2Context;
impl Gui {
    pub fn new(aladin_lite_div: &str, gl: &WebGl2Context) -> Result<GuiRef, JsValue> {
        /*let mut backend = egui_web::WebBackend::new("mycanvas")
            .expect("Failed to make a web backend for egui");
        */
        let ctx = egui::CtxRef::default();
        let painter = WebGl2Painter::new(aladin_lite_div, gl.clone())?;
        let input: egui_web::backend::WebInput = Default::default();

        let gui = Self {
            ctx,
            painter,

            input,

            needs_repaint: Default::default(),
            last_text_cursor_pos: None,

            aladin_lite_div: aladin_lite_div.to_string(),
        };

        let gui_ref = GuiRef(std::sync::Arc::new(egui::mutex::Mutex::new(gui)));

        input::install_canvas_events(gui_ref.clone())?;
        input::install_document_events(gui_ref.clone())?;
        input::install_text_agent(gui_ref.clone())?;

        Ok(gui_ref)
    }

    /*pub fn canvas_id(&self) -> &str  {
        self.painter.canvas_id()
    }*/

    pub fn egui_ctx(&self) -> &egui::CtxRef {
        &self.ctx
    }

    pub fn is_pointer_over_ui(&self) -> bool {
        self.ctx.wants_pointer_input()
    }

    pub fn pos_over_ui(&self, sx: f32, sy: f32) -> bool {
        let egui::layers::LayerId { order, .. } =
            self.ctx.layer_id_at(egui::Pos2::new(sx, sy)).unwrap();
        order != egui::layers::Order::Background
    }

    pub fn draw<U: Ui>(&mut self, ui: &mut U, fbo: &FrameBufferObject) -> Result<(), JsValue> {
        //let canvas_size = egui_web::canvas_size_in_points(self.painter.canvas_id());
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
            egui::CentralPanel::default()
                .frame(f)
                .show(&self.ctx, |egui| {
                    ui.show(egui)
                });
        }

        self.painter.upload_egui_texture(&self.ctx.texture());

        let (output, shapes) = self.ctx.end_frame();
        input::handle_output(&output, self);

        if self.redraw_needed() {
            let clipped_meshes = self.ctx.tessellate(shapes); // create triangles to paint
            let s = self;
            fbo.draw_onto(move || {
                s.painter.paint_meshes(clipped_meshes, 1.0)?;

                Ok(())
            })?;
        }

        Ok(())
    }

    pub fn redraw_needed(&mut self) -> bool {
        let redraw = self.needs_repaint.fetch_and_clear();
        redraw
    }
}
