
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Enum {
    First,
    Second,
    Third,
}

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Layout {
    enabled: bool,
    visible: bool,
    boolean: bool,
    radio: Enum,
    scalar: f32,
    string: String,
    color: egui::Color32,
    animate_progress_bar: bool,
}

impl Default for Layout {
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

use crate::Event;

impl Layout {
    /*fn name(&self) -> &'static str {
        "Aladin Lite User Interface"
    }*/

    pub fn show(&mut self, ui: &mut egui::Ui, events: &mut Vec<Event>) {
        egui::Window::new("egui_demo_panel")
            .min_width(150.0)
            .default_width(190.0)
            .default_pos(egui::Pos2 { x: 0.0, y: 100.0 })
            .collapsible(true)
            .show(ui.ctx(), |ui| {
                //use super::View as _;
                self.ui(ui, events);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, events: &mut Vec<Event>) {
        ui.scope(|ui| {
            ui.set_visible(self.visible);
            ui.set_enabled(self.enabled);

            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                //.striped(true)
                .show(ui, |ui| {
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