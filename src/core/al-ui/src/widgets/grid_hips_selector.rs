use egui::Color32;

use wasm_bindgen_futures;
use std::sync::{Arc, Mutex};

use crate::{Event, painter::WebGl2Painter};

struct SurveyThumbnailDesc {
    id: &'static str,
    regime: &'static str,
    url: &'static str,
}

struct SurveyThumbnail {
    desc: SurveyThumbnailDesc,
    // The image
    index_thumbnail: usize,
}

const SIZE_SURVEY_THUMBNAIL: (usize, usize) = (64, 64);
const SURVEY_THUMBNAILS: &'static [SurveyThumbnail] = &[
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "SDSS9 color",
            regime: "Optical",
            url: "SDSS/DR9/color",
        },
        index_thumbnail: 0
    }
];

pub struct SurveyGrid {
    thumbnail_texture: egui::TextureId,
    thumbnail_texture_size: egui::Vec2,
    open: bool,
}
use wasm_bindgen::prelude::*;
use img_pixel::{RgbImage, RgbaImage, ImageBuffer};
use super::SurveyWidget;
impl SurveyGrid {
    pub fn new(painter: &mut WebGl2Painter) -> Result<Self, JsValue> {
        //let thumbnail_img_path = resources.get_filename("ui_thumbnail").unwrap();

        let (user_texture, size_thumbnail_tex) = {
            let size_thumbnail_img = (64, 64);

            let width_thumbnail_img = size_thumbnail_img.0 as u32;
            let height_thumbnail_img = size_thumbnail_img.1 as u32;
            let image_buf: RgbaImage = ImageBuffer::from_raw(
                width_thumbnail_img,
                height_thumbnail_img,
                include_bytes!("../../img/CDS_P_SDSS9_color.png").to_vec()
            ).ok_or(JsValue::from_str("Decoding UI texture failed"))?;
            let mut data_rgba = Vec::with_capacity((width_thumbnail_img as usize) * (height_thumbnail_img as usize) * 4);

            let srgba_pixels = {
                let data  = &image_buf.as_raw()[..];
                for (r, (g, b)) in data.iter().zip(data.iter().skip(1).zip(data.iter().skip(2)).step_by(3)) {
                    data_rgba.push(*r);
                    data_rgba.push(*g);
                    data_rgba.push(*b);
                    data_rgba.push(255);
                }
                unsafe { std::slice::from_raw_parts(data_rgba.as_ptr() as *const Color32, data_rgba.len() >> 2) }
            };
            // register the texture to the ui backend
            let user_texture = painter.alloc_user_texture(size_thumbnail_img, srgba_pixels);
            (user_texture, size_thumbnail_img)
        };

        let open = false;
        Ok(
            Self {
                thumbnail_texture: user_texture,
                thumbnail_texture_size: egui::Vec2::new(size_thumbnail_tex.0 as f32, size_thumbnail_tex.1 as f32),
                open,
            }
        )
    }

    pub fn show(&mut self, ui: &mut egui::Ui, events: Arc<Mutex<Vec<Event>>>, s_id_selected: &mut String, s_list: Arc<Mutex<Vec<SurveyWidget>>>) {
        if !self.open {
            return;
        }

        egui::Window::new("")
        /*.frame(egui::Frame::none()
            .stroke(Stroke::none())
        )*/
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0))
        .show(ui.ctx(), |ui| {
            egui::Grid::new("Surveys browsing").show(ui, |ui| {
                for (idx, thumbnail) in SURVEY_THUMBNAILS.iter().enumerate() {
                    if ui
                        .add(egui::ImageButton::new(self.thumbnail_texture, self.thumbnail_texture_size).uv(egui::Rect {
                            min: egui::Pos2::new(
                                ((thumbnail.index_thumbnail % 4) as f32) * (SIZE_SURVEY_THUMBNAIL.0 as f32) / self.thumbnail_texture_size.x,
                                ((thumbnail.index_thumbnail / 4) as f32) * (SIZE_SURVEY_THUMBNAIL.1 as f32) / self.thumbnail_texture_size.y
                            ),
                            max: egui::Pos2::new(
                                ((thumbnail.index_thumbnail % 4) as f32 + 1.0) * (SIZE_SURVEY_THUMBNAIL.0 as f32) / self.thumbnail_texture_size.x,
                                ((thumbnail.index_thumbnail / 4) as f32 + 1.0) * (SIZE_SURVEY_THUMBNAIL.1 as f32) / self.thumbnail_texture_size.y
                            )
                        }))
                        .on_hover_text(thumbnail.desc.regime)
                        .clicked()
                    {
                        *s_id_selected = thumbnail.desc.id.to_string();
                    }

                    if idx % 4 == 0 {
                        ui.end_row();
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Add")).clicked() {
                    // TODO. You will not be able to add a new survey if there is a color one
                    let selected_survey_compatible = true;
                    if selected_survey_compatible {
                        let s_id_selected = s_id_selected.clone();
                        let s_list = s_list.clone();
                        let fut = async move {
                            let url = format!("https://alaskybis.u-strasbg.fr/{}", s_id_selected);
                            let new_survey = SurveyWidget::new(url).await;

                            // get the SimpleHiPS from the SurveyWidget
                            let mut image_surveys = vec![new_survey.get_hips_config()];
                            for survey in s_list.lock().unwrap().iter() {
                                image_surveys.push(survey.get_hips_config());
                            }

                            events.lock().unwrap().push(Event::ImageSurveys(image_surveys));
                            s_list.lock().unwrap().push(new_survey);
                        };

                        wasm_bindgen_futures::spawn_local(fut);
                    }
                }

                if ui.add(egui::Button::new("Cancel")).clicked() {
                    self.open = false;
                }
            });
        });
    }
}
