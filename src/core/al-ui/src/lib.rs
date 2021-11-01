mod painter;
use painter::WebGl2Painter;

mod input;
use input::*;
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

pub struct Gui {
    //backend: egui_web::WebBackend,
    //web_input: egui_web::backend::WebInput,
    pub input: egui_web::WebInput,
    pub painter: WebGl2Painter,
    ctx: egui::CtxRef,

    gallery: WidgetGallery,

    events: Vec<egui::Event>,

    pub needs_repaint: std::sync::Arc<NeedRepaint>,
    pub last_text_cursor_pos: Option<egui::Pos2>,

    pub aladin_lite_div: String,
}

use al_core::log::*;
use al_core::WebGl2Context;
impl Gui {
    pub fn new(aladin_lite_div: &str, gl: &WebGl2Context) -> Result<GuiRef, JsValue> {
        /*let mut backend = egui_web::WebBackend::new("mycanvas")
            .expect("Failed to make a web backend for egui");
        */
        let ctx = egui::CtxRef::default();
        let painter = WebGl2Painter::new(aladin_lite_div, gl.clone())?;
        let mut input: egui_web::backend::WebInput = Default::default();

        let gallery = WidgetGallery::default();

        let events = vec![];

        let gui = Self {
            ctx,
            painter,

            input,

            gallery,

            events,

            needs_repaint: Default::default(),
            last_text_cursor_pos: None,

            aladin_lite_div: aladin_lite_div.to_string(),
        };

        let gui_ref = GuiRef(std::sync::Arc::new(egui::mutex::Mutex::new(gui)));

        install_canvas_events(&gui_ref)?;
        install_document_events(&gui_ref)?;
        install_text_agent(&gui_ref)?;

        Ok(gui_ref)
    }

    pub fn add_event(&mut self, event: egui::Event) {
        self.events.push(event);
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

    pub fn render(&mut self) -> Result<(), JsValue> {
        //let canvas_size = egui_web::canvas_size_in_points(self.painter.canvas_id());
        let canvas_size = egui::vec2(
            self.painter.canvas.width() as f32,
            self.painter.canvas.height() as f32,
        );
        let raw_input = self.input.new_frame(canvas_size);

        //self.web_backend.begin_frame(raw_input);

        //raw_input.events = self.events.clone();
        //self.events.clear();

        self.ctx.begin_frame(raw_input);

        let mut open = true;
        self.gallery.show(&self.ctx, &mut open);

        self.painter.upload_egui_texture(&self.ctx.texture());

        let (output, shapes) = self.ctx.end_frame();
        let clipped_meshes = self.ctx.tessellate(shapes); // create triangles to paint
        handle_output(&output, self);
        self.painter.paint_meshes(clipped_meshes, 1.0)?;

        Ok(())
    }

    pub fn redraw_needed(&mut self) -> bool {
        let redraw = self.needs_repaint.fetch_and_clear();
        redraw
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Enum {
    First,
    Second,
    Third,
}

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WidgetGallery {
    enabled: bool,
    visible: bool,
    boolean: bool,
    radio: Enum,
    scalar: f32,
    string: String,
    color: egui::Color32,
    animate_progress_bar: bool,
}

impl Default for WidgetGallery {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            boolean: false,
            radio: Enum::First,
            scalar: 42.0,
            string: Default::default(),
            color: egui::Color32::LIGHT_BLUE.linear_multiply(0.5),
            animate_progress_bar: false,
        }
    }
}

impl WidgetGallery {
    fn name(&self) -> &'static str {
        "🗄 Widget Gallery"
    }

    fn show(&mut self, ctx: &egui::CtxRef, open: &mut bool) {
        let my_frame = egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(my_frame)
            .show(ctx, |ui| {
                egui::Window::new("egui_demo_panel")
                    .min_width(150.0)
                    .default_width(190.0)
                    .default_pos(egui::Pos2 { x: 0.0, y: 100.0 })
                    .collapsible(true)
                    .show(ctx, |ui| {
                        //use super::View as _;
                        self.ui(ui);
                    });
            });
    }
}

impl WidgetGallery {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.set_visible(self.visible);
            ui.set_enabled(self.enabled);

            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                //.striped(true)
                .show(ui, |ui| {
                    self.gallery_grid_contents(ui);
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.visible, "Visible")
                .on_hover_text("Uncheck to hide all the widgets.");
            if self.visible {
                ui.checkbox(&mut self.enabled, "Interactive")
                    .on_hover_text("Uncheck to inspect how the widgets look when disabled.");
            }
        });

        ui.separator();

        ui.vertical_centered(|ui| {
            let tooltip_text = "The full egui documentation.\nYou can also click the different widgets names in the left column.";
            ui.hyperlink("https://docs.rs/egui/").on_hover_text(tooltip_text);
            /*ui.add(crate::__egui_github_link_file!(
                "Source code of the widget gallery"
            ));*/
        });
    }
}

impl WidgetGallery {
    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        let Self {
            enabled: _,
            visible: _,
            boolean,
            radio,
            scalar,
            string,
            color,
            animate_progress_bar,
        } = self;

        ui.add(doc_link_label("Label", "label,heading"));
        ui.label("Welcome to the widget gallery!");
        ui.end_row();

        ui.add(doc_link_label("Hyperlink", "Hyperlink"));
        use egui::special_emojis::GITHUB;
        ui.hyperlink_to(
            format!("{} egui home page", GITHUB),
            "https://github.com/emilk/egui",
        );
        ui.end_row();

        ui.add(doc_link_label("TextEdit", "TextEdit,text_edit"));
        ui.add(egui::TextEdit::singleline(string).hint_text("Write something here"));
        ui.end_row();

        ui.add(doc_link_label("Button", "button"));
        if ui.button("Click me!").clicked() {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Checkbox", "checkbox"));
        ui.checkbox(boolean, "Checkbox");
        ui.end_row();

        ui.add(doc_link_label("RadioButton", "radio"));
        ui.horizontal(|ui| {
            ui.radio_value(radio, Enum::First, "First");
            ui.radio_value(radio, Enum::Second, "Second");
            ui.radio_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label(
            "SelectableLabel",
            "selectable_value,SelectableLabel",
        ));
        ui.horizontal(|ui| {
            ui.selectable_value(radio, Enum::First, "First");
            ui.selectable_value(radio, Enum::Second, "Second");
            ui.selectable_value(radio, Enum::Third, "Third");
        });
        ui.end_row();

        ui.add(doc_link_label("ComboBox", "ComboBox"));

        egui::ComboBox::from_label("Take your pick")
            .selected_text(format!("{:?}", radio))
            .show_ui(ui, |ui| {
                ui.selectable_value(radio, Enum::First, "First");
                ui.selectable_value(radio, Enum::Second, "Second");
                ui.selectable_value(radio, Enum::Third, "Third");
            });
        ui.end_row();

        ui.add(doc_link_label("Slider", "Slider"));
        ui.add(egui::Slider::new(scalar, 0.0..=360.0).suffix("°"));
        ui.end_row();

        ui.add(doc_link_label("DragValue", "DragValue"));
        ui.add(egui::DragValue::new(scalar).speed(1.0));
        ui.end_row();

        ui.add(doc_link_label("ProgressBar", "ProgressBar"));
        let progress = *scalar / 360.0;
        let progress_bar = egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(*animate_progress_bar);
        *animate_progress_bar = ui
            .add(progress_bar)
            .on_hover_text("The progress bar can be animated!")
            .hovered();
        ui.end_row();

        ui.add(doc_link_label("Color picker", "color_edit"));
        ui.color_edit_button_srgba(color);
        ui.end_row();

        ui.add(doc_link_label("Image", "Image"));
        ui.image(egui::TextureId::Egui, [24.0, 16.0])
            .on_hover_text("The egui font texture was the convenient choice to show here.");
        ui.end_row();

        ui.add(doc_link_label("ImageButton", "ImageButton"));
        if ui
            .add(egui::ImageButton::new(egui::TextureId::Egui, [24.0, 16.0]))
            .on_hover_text("The egui font texture was the convenient choice to show here.")
            .clicked()
        {
            *boolean = !*boolean;
        }
        ui.end_row();

        ui.add(doc_link_label("Separator", "separator"));
        ui.separator();
        ui.end_row();

        ui.add(doc_link_label("CollapsingHeader", "collapsing"));
        ui.collapsing("Click to see what is hidden!", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    "Not much, as it turns out - but here is a gold star for you for checking:",
                );
                ui.colored_label(egui::Color32::GOLD, "☆");
            });
        });
        ui.end_row();

        ui.add(doc_link_label("Plot", "plot"));
        ui.add(example_plot());
        ui.end_row();

        /*ui.hyperlink_to(
            "Custom widget:",
            super::toggle_switch::url_to_file_source_code(),
        );
        ui.add(super::toggle_switch::toggle(boolean)).on_hover_text(
            "It's easy to create your own widgets!\n\
            This toggle switch is just 15 lines of code.",
        );*/
        ui.end_row();
    }
}

fn example_plot() -> egui::plot::Plot {
    use egui::plot::{Line, Plot, Value, Values};
    let n = 128;
    let line = Line::new(Values::from_values_iter((0..=n).map(|i| {
        use std::f64::consts::TAU;
        let x = egui::remap(i as f64, 0.0..=(n as f64), -TAU..=TAU);
        Value::new(x, x.sin())
    })));
    Plot::new("example_plot")
        .line(line)
        .height(32.0)
        .data_aspect(1.0)
}

fn doc_link_label<'a>(title: &'a str, search_term: &'a str) -> impl egui::Widget + 'a {
    let label = format!("{}:", title);
    let url = format!("https://docs.rs/egui?search={}", search_term);
    move |ui: &mut egui::Ui| {
        ui.hyperlink_to(label, url).on_hover_ui(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Search egui docs for");
                ui.code(search_term);
            });
        })
    }
}
