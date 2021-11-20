
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Enum {
    First,
    Second,
    Third,
}

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AlUserInterface {
    enabled: bool,
    visible: bool,
    boolean: bool,
    radio: Enum,
    scalar: f32,
    string: String,
    color: egui::Color32,
    animate_progress_bar: bool,
}

impl Default for AlUserInterface {
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
use al_core::FrameBufferObject;
impl al_ui::App for crate::App {
    fn get_framebuffer_ui(&self) -> &FrameBufferObject {
        &self.fbo_ui
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Settings")
            .min_width(150.0)
            .default_width(190.0)
            .default_pos(egui::Pos2 { x: 0.0, y: 100.0 })
            .collapsible(true)
            .show(ui.ctx(), |ui| {
                ui.scope(|ui| {
                    ui.set_visible(self.ui_layout.visible);
                    ui.set_enabled(self.ui_layout.enabled);
        
                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        //.striped(true)
                        .show(ui, |ui| {                    
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
                
                            ui.add(doc_link_label("ComboBox", "ComboBox"));
                    
                            egui::ComboBox::from_label("Select the projection")
                                .selected_text(format!("{:?}", self.ui_layout.radio))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.ui_layout.radio, Enum::First, "Aitoff");
                                    ui.selectable_value(&mut self.ui_layout.radio, Enum::Second, "Orthographic");
                                    ui.selectable_value(&mut self.ui_layout.radio, Enum::Third, "Mercator");
                                
                                    match self.ui_layout.radio {
                                        Enum::First => self.set_projection::<crate::projection::Aitoff>(),
                                        Enum::Second => {
                                            self.set_projection::<crate::projection::Orthographic>();
                                            self.set_grid_color(crate::Color::new(1.0, 0.0, 0.0, 0.2));
                                        },
                                        Enum::Third => self.set_projection::<crate::projection::Mercator>(),
                                        _ => (),
                                    }
                                });

                            ui.end_row();
                    
                            
                    
                            ui.add(doc_link_label("Plot", "plot"));
                            ui.add(example_plot());
                            ui.end_row();
                        });
                });
        
                ui.separator();
        
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.ui_layout.visible, "Visible")
                        .on_hover_text("Uncheck to hide all the widgets.");
                    if self.ui_layout.visible {
                        ui.checkbox(&mut self.ui_layout.enabled, "Interactive")
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
            });
    }
}

impl AlUserInterface {
    fn name(&self) -> &'static str {
        "Aladin Lite User Interface"
    }

    fn gallery_grid_contents(&mut self, ui: &mut egui::Ui) {
        
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