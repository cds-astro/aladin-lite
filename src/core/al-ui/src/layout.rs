use crate::painter::WebGl2Painter;
use crate::widgets::SurveyWidget;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LayerLayout {
    selection_open: bool,
    survey_name_selected: String,

    surveys: Arc<Mutex<Vec<SurveyWidget>>>,
    s_select_w: SurveyGrid,
}

use crate::widgets::SurveyGrid;
use crate::Event;
use al_core::log::log;
use egui::Stroke;

use crate::hips::{Frame, HiPSProperties, SimpleHiPS, HiPSColor::Grayscale2Color, HiPSFormat};
/*const HiPS: &'static [(&'static str, SimpleHiPS)] = &[
    ("GALEX AIS-FD", SimpleHiPS {
        properties: HiPSProperties {
            url: Cow::Borrowed("https://alaskybis.u-strasbg.fr/GALEX/GR6-03-2014/AIS-FD"),
            max_order: 8,
            frame: Frame {
                label: Cow::Borrowed("J2000"),
                system: Cow::Borrowed("J2000")
            },
            tile_size: 512,
            format: HiPSFormat::FITSImage {
                bitpix: -32
            },
            min_cutout: Some(0.0),
            max_cutout: Some(0.003)
        },
        color: Grayscale2Color {
            color: [
                1.0,
                0.0,
                0.0
            ],
            k: 1.0,
            transfer: Cow::Borrowed("asinh"),
        },
        layer: Cow::Borrowed("base")
    })
];*/

const SURVEYS_NAME: &'static [&'static str] = &[
    "GALEX/GR6-03-2014/AIS-FD"
];
use wasm_bindgen::prelude::JsValue;
use std::sync::{Arc, Mutex};
impl LayerLayout {
    pub fn new(ui_backend: &mut WebGl2Painter) -> Result<Self, JsValue> {
        let survey_grid_widget = SurveyGrid::new(ui_backend)?;
        Ok(Self {
            selection_open: false,
            survey_name_selected: String::new(),
            surveys: Arc::new(Mutex::new(vec![])),
            s_select_w: survey_grid_widget,
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui, events: Arc<Mutex<Vec<Event>>>) {
        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::none())
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                //use super::View as _;
                self.ui(ui, events);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, events: Arc<Mutex<Vec<Event>>>) {
        ui.label("Layers");
        ui.separator();

        for survey in &mut *self.surveys.lock().unwrap() {
            survey.show(ui, events.clone());
        }

        // TODO: check if you can add a new survey
        // it is not possible if:
        // - a color survey is already selected 
        // - a grayscale survey mapped to a colormap object is selected
        if ui.add(egui::Button::new("Add survey")).clicked() {
            self.selection_open = true;
        }

        if self.selection_open {
            self.s_select_w.show(ui, events, &mut self.survey_name_selected, self.surveys.clone())
        }
        /*{
            //let survey = self.survey.clone();
            //let s = survey.lock().unwrap();
            if let Some(s) = &*self.survey.clone().lock().unwrap()  {
                ui.group(|ui| {
                    ui.label("Description:");

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Title:");
                            ui.label(&s.obs_title);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Category:");
                            ui.label(&s.client_category);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Regime:");
                            ui.label(&s.obs_regime);
                        });

                        let scroll_area = ScrollArea::vertical()
                            .max_height(200.0)
                            .auto_shrink([false; 2]);

                        ui.separator();
                        scroll_area.show(ui, |ui| {
                            ui.label("Description:");
                            ui.label(&s.obs_description);
                        });
                    });
                });
            }
        }*/
    }

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