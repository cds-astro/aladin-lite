use crate::painter::WebGl2Painter;
use crate::widgets::SurveyWidget;

/// Shows off one example of each major type of widget.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LayerLayout {
    survey_name_selected: String,

    surveys: Arc<Mutex<Vec<SurveyWidget>>>,
    s_select_w: SurveyGrid,
}

use crate::widgets::SurveyGrid;
use crate::Event;

use wasm_bindgen::prelude::JsValue;
use std::sync::{Arc, Mutex};
use crate::widgets::survey::Color;
impl LayerLayout {
    pub fn new(ui_backend: &mut WebGl2Painter) -> Result<Self, JsValue> {
        let survey_grid_widget = SurveyGrid::new(ui_backend)?;
        Ok(Self {
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
                self.ui(ui, events.clone());
            });

        let mut new_survey_added = false;
        self.s_select_w.show(ui, &mut self.survey_name_selected, &mut new_survey_added);
        
        if new_survey_added {
            let s_id_selected = self.survey_name_selected.clone();
            let s_list = self.surveys.clone();
            let events = events.clone();
            let fut = async move {
                let url = format!("https://alaskybis.u-strasbg.fr/{}", s_id_selected);
                let new_survey = SurveyWidget::new(url).await;
                let mut can_surveys_be_added = true;
                // check if the new survey is compatible with the ones already pushed
                for s in s_list.lock().unwrap().iter() {
                    match s.color() {
                        Color::Color(_) => (),
                        _ => {
                            can_surveys_be_added = false;
                            break;
                        }
                    }
                }

                if can_surveys_be_added {
                    // get the SimpleHiPS from the SurveyWidget
                    let mut image_surveys = vec![new_survey.get_hips_config()];
                    for survey in s_list.lock().unwrap().iter() {
                        image_surveys.push(survey.get_hips_config());
                    }

                    events.lock().unwrap().push(Event::ImageSurveys(image_surveys));
                    s_list.lock().unwrap().push(new_survey);
                }
            };

            wasm_bindgen_futures::spawn_local(fut);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, events: Arc<Mutex<Vec<Event>>>) {
        ui.label("Layers");
        ui.separator();
        let surveys = &mut *self.surveys.lock().unwrap();
        let mut update_viewed_surveys = false;
        for idx in (0..surveys.len()).rev() {
            surveys[idx].show(ui);

            if surveys[idx].update_survey {
                update_viewed_surveys = true;
            }

            if surveys[idx].removed() {
                surveys.remove(idx);
                update_viewed_surveys = true;
            }
        }

        // TODO: check if you can add a new survey
        // it is not possible if:
        // - a color survey is already selected 
        // - a grayscale survey mapped to a colormap object is selected
        if ui.add(egui::Button::new("Add survey")).clicked() {
            self.s_select_w.open();
        }

        if update_viewed_surveys {
            let mut image_surveys = vec![];
            for survey in surveys.iter() {
                if survey.visible {
                    image_surveys.push(survey.get_hips_config());
                }
            }

            events.lock().unwrap()
                .push(Event::ImageSurveys(image_surveys));
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