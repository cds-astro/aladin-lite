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

const SIZE_SURVEY_THUMBNAIL: egui::Vec2 = egui::Vec2 { x: 64.0, y: 64.0 };
const SURVEY_THUMBNAILS: &'static [SurveyThumbnail] = &[
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "SDSS9 color",
            regime: "Optical",
            url: "SDSS/DR9/color",
        },
        index_thumbnail: 0
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "DSS2 NIR",
            regime: "Optical",
            url: "DSS2/NIR",
        },
        index_thumbnail: 1
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "HLA SDSSz",
            regime: "Optical",
            url: "HLA/SDSSz",
        },
        index_thumbnail: 2
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "PanSTARRS DR1 g",
            regime: "Optical",
            url: "PanSTARRS/DR1/g",
        },
        index_thumbnail: 3
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "PanSTARRS DR1 z",
            regime: "Optical",
            url: "PanSTARRS/DR1/z",
        },
        index_thumbnail: 4
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "I 345 gaia2",
            regime: "Optical",
            url: "DM/I/345/gaia2",
        },
        index_thumbnail: 5
    },
    SurveyThumbnail {
        desc: SurveyThumbnailDesc {
            id: "GALEXGR6 AIS FUV",
            regime: "UV",
            url: "GALEXGR6/AIS/FUV",
        },
        index_thumbnail: 6
    }
];

pub struct SurveyGrid {
    thumbnail_texture: egui::TextureId,
    thumbnail_texture_size: egui::Vec2,
    open: bool,
}
use wasm_bindgen::prelude::*;
use super::SurveyWidget;
impl SurveyGrid {
    pub fn new(painter: &mut WebGl2Painter) -> Result<Self, JsValue> {
        //let thumbnail_img_path = resources.get_filename("ui_thumbnail").unwrap();

        let (user_texture, size_thumbnail_tex) = {
            let size_thumbnail_img = (320, 192);

            let image_buf = img_pixel::load_from_memory_with_format(
                include_bytes!("../../img/tileset.png"),
                img_pixel::ImageFormat::Png
            ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            //let mut data_rgba = Vec::with_capacity((width_thumbnail_img as usize) * (height_thumbnail_img as usize) * 4);
            let data = image_buf.into_bytes();

            let srgba_pixels = {
                unsafe { std::slice::from_raw_parts(data.as_ptr() as *const Color32, data.len() >> 2) }
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

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, s_id_selected: &mut String, add: &mut bool) {
        if !self.open {
            return;
        }

        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::none())
            .show(ui, |ui| {
                egui::Grid::new("Surveys browsing").show(ui, |ui| {
                    for (idx, thumbnail) in SURVEY_THUMBNAILS.iter().enumerate() {
                        if ui
                            .add(egui::ImageButton::new(self.thumbnail_texture, SIZE_SURVEY_THUMBNAIL)
                                .uv(egui::Rect {
                                    min: egui::Pos2::new(
                                        ((thumbnail.index_thumbnail % 5) as f32) * SIZE_SURVEY_THUMBNAIL.x / self.thumbnail_texture_size.x,
                                        ((thumbnail.index_thumbnail / 5) as f32) * SIZE_SURVEY_THUMBNAIL.y / self.thumbnail_texture_size.y
                                    ),
                                    max: egui::Pos2::new(
                                        ((thumbnail.index_thumbnail % 5) as f32 + 1.0) * SIZE_SURVEY_THUMBNAIL.x / self.thumbnail_texture_size.x,
                                        ((thumbnail.index_thumbnail / 5) as f32 + 1.0) * SIZE_SURVEY_THUMBNAIL.y / self.thumbnail_texture_size.y
                                    )
                                })
                            )
                            .on_hover_text(thumbnail.desc.regime)
                            .clicked()
                        {
                            *s_id_selected = thumbnail.desc.url.to_string();
                        }

                        if idx % 5 == 0 {
                            ui.end_row();
                        }
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Add")).clicked() {
                    // TODO. You will not be able to add a new survey if there is a color one
                    let selected_survey_compatible = true;
                    *add = true;
                }

                if ui.add(egui::Button::new("Cancel")).clicked() {
                    self.open = false;
                }
            });
        });
    }
}

mod tests {
    use img_pixel::{RgbaImage, ImageBuffer};

    #[test]
    fn test_open_png_image() {
        let size_thumbnail_img = (64, 64);

        let width_thumbnail_img = size_thumbnail_img.0 as u32;
        let height_thumbnail_img = size_thumbnail_img.1 as u32;
        let image_buf: RgbaImage = ImageBuffer::from_raw(
            width_thumbnail_img,
            height_thumbnail_img,
            include_bytes!("../../img/CDS_P_SDSS9_color.png").to_vec()
        ).unwrap();

        assert_eq!(image_buf.pixels().len(), size_thumbnail_img.0 * size_thumbnail_img.1);
    }
}