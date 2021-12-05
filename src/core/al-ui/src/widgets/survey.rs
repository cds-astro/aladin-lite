use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Default)]
#[derive(Deserialize, Serialize)]
struct Properties {
    #[serde(default="default_empty_string")]
    obs_description: String,

    #[serde(default="default_float")]
    moc_sky_fraction: String,

    #[serde(default="default_empty_string")]
    bib_reference: String,

    #[serde(default="default_empty_string")]
    bib_reference_url: String,

    #[serde(default="default_empty_string")]
    obs_regime: String,

    #[serde(default="default_empty_string")]
    prov_progenitor: String,

    #[serde(default="default_empty_string")]
    client_category: String,

    #[serde(default="default_empty_string")]
    obs_collection: String,

    #[serde(default="default_empty_string")]
    obs_title: String,

    #[serde(default="default_float")]
    em_min: String,

    #[serde(default="default_float")]
    em_max: String,

    #[serde(default="default_int")]
    hips_order: String,

    #[serde(default="default_empty_string")]
    hips_pixel_bitpix: String,

    #[serde(default="default_format")]
    hips_tile_format: String,

    #[serde(default="default_int")]
    hips_tile_width: String,

    #[serde(default="default_empty_string")]
    hips_pixel_cut: String,

    #[serde(default="default_frame")]
    hips_frame: String,
}

fn default_empty_string() -> String {
    String::new()
}

fn default_float() -> String {
    String::from("0.0")
}

fn default_int() -> String {
    String::from("0")
}

fn default_format() -> String {
    String::from("jpg")
}

fn default_frame() -> String {
    String::from("equatorial")
}

#[derive(Default, Debug)]
struct PropertiesParsed {
    obs_description: String,
    moc_sky_fraction: f32,
    bib_reference: String,
    bib_reference_url: String,
    obs_regime: String,
    em_min: f32,
    em_max: f32,
    hips_order: u8,
    hips_pixel_bitpix: Option<i32>,
    hips_pixel_cut: Option<[f32; 2]>,
    hips_tile_format: String,
    hips_tile_width: i32,
    hips_frame: String,
    prov_progenitor: String,
    client_category: String,
    obs_collection: String,
    obs_title: String,
}

impl PropertiesParsed {
    fn is_png_or_jpg_image(&self) -> bool {
        self.hips_tile_format.contains("png") || self.hips_tile_format.contains("jpg")
    }

    fn is_fits_image(&self) -> bool {
        self.hips_tile_format.contains("fits")
    }
}
use egui::Color32;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/hpxImageSurvey.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = fetchSurveyMetadata)]
    async fn fetch_survey_metadata(url: String) -> Result<JsValue, JsValue>;
}


use wasm_bindgen_futures;
use std::sync::{Arc, Mutex};
async fn request_survey_properties(url: String) -> PropertiesParsed {
    let res: Properties = fetch_survey_metadata(url)
        .await
        .unwrap()
        .into_serde()
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .unwrap();

    al_core::log::log(&format!("{:?}", res));
    let cuts: Vec<_> = res.hips_pixel_cut.split(" ").collect();

    let cuts = if cuts.len() != 2 {
        None
    } else {
        let (c0, c1) = (cuts[0].parse::<f32>(), cuts[1].parse::<f32>());
        if c0.is_err() || c1.is_err() {
            None
        } else {
            Some([c0.unwrap(), c1.unwrap()])
        }
    };

    let properties = PropertiesParsed {
        obs_collection: res.obs_collection,
        obs_description: res.obs_description,
        obs_regime: res.obs_regime,
        moc_sky_fraction: res.moc_sky_fraction.parse::<f32>().unwrap(),
        bib_reference: res.bib_reference,
        bib_reference_url: res.bib_reference_url,
        em_min: res.em_min.parse::<f32>().unwrap(),
        em_max: res.em_max.parse::<f32>().unwrap(),
        hips_order: res.hips_order.parse::<u8>().unwrap(),
        hips_pixel_bitpix: res.hips_pixel_bitpix.parse::<i32>().ok(),
        hips_tile_format: res.hips_tile_format,
        hips_tile_width: res.hips_tile_width.parse::<i32>().unwrap(),
        hips_pixel_cut: cuts,
        hips_frame: res.hips_frame,
        prov_progenitor: res.prov_progenitor,
        client_category: res.client_category,
        obs_title: res.obs_title,
    };

    properties
}

#[derive(PartialEq)]
enum TransferFunction {
    ASINH,
    LINEAR,
    POW,
    SIN
}

pub enum Color {
    Image,
    Color(egui::Color32),
    Colormap {
        reversed: bool,
        name: String,
    }
}

pub struct SurveyWidget {
    url: String,
    properties: PropertiesParsed,

    visible: bool,
    edition_mode: bool,

    // Edition mode
    color: Color,
    // In case of a fits HiPS
    transfer_func: Option<TransferFunction>,
    // In case of a fits HiPS
    cutouts: Option<[f32; 2]>
}

use cgmath::num_traits::Pow;
use crate::Event;

use crate::hips::{Frame, HiPSColor, HiPSFormat, HiPSProperties, SimpleHiPS};
impl SurveyWidget {
    pub async fn new(url: String) -> Self {
        let properties = request_survey_properties(url.clone()).await;
        let cutouts = properties.hips_pixel_cut.clone();
        let color = if properties.is_fits_image() {
            Color::Color(Color32::RED)
        } else {
            Color::Image
        };

        let transfer_func = if properties.is_fits_image() {
            Some(TransferFunction::ASINH)
        } else {
            None
        };

        Self {
            url,
            properties,

            visible: false,
            edition_mode: false,

            // edition mode
            color,
            transfer_func,
            cutouts,
        }
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn get_hips_config(&self) -> SimpleHiPS {
        let color = match &self.color {
            Color::Image => {
                HiPSColor::Color
            },
            Color::Color(c) => {
                let transfer = match self.transfer_func.as_ref().unwrap() {
                    TransferFunction::ASINH => String::from("asinh"),
                    TransferFunction::LINEAR => String::from("linear"),
                    TransferFunction::POW => String::from("pow"),
                    TransferFunction::SIN => String::from("sin"),
                };

                HiPSColor::Grayscale2Color {
                    color: [(c.r() as f32)/255.0, (c.g() as f32)/255.0, (c.b() as f32)/255.0],
                    transfer,
                    k: 1.0
                }
            },
            Color::Colormap {
                reversed, name
            } => {
                let transfer = match self.transfer_func.as_ref().unwrap() {
                    TransferFunction::ASINH => String::from("asinh"),
                    TransferFunction::LINEAR => String::from("linear"),
                    TransferFunction::POW => String::from("pow"),
                    TransferFunction::SIN => String::from("sin"),
                };

                HiPSColor::Grayscale2Colormap {
                    reversed: *reversed,
                    colormap: name.to_string(),
                    transfer,
                }
            }
        };

        let props = &self.properties;
        let max_order = props.hips_order;
        let frame = Frame {
            label: String::from("J2000"),
            system: String::from("J2000")
        };
        let tile_size = props.hips_tile_width;
        let min_cutout = if let Some(c) = self.cutouts {
            Some(c[0])
        } else {
            None
        };
        let max_cutout = if let Some(c) = self.cutouts {
            Some(c[1])
        } else {
            None
        };
        let format = if props.is_fits_image() {
            HiPSFormat::FITSImage {
                bitpix: props.hips_pixel_bitpix.unwrap()
            }
        } else {
            if props.hips_tile_format.contains("png") {
                HiPSFormat::Image {
                    format: String::from("png")
                }
            } else {
                HiPSFormat::Image {
                    format: String::from("jpeg")
                }
            }
        };
        let hips = SimpleHiPS {
            layer: String::from("base"),
            color: color,
            properties: HiPSProperties {
                url: self.url.clone(),
                max_order,
                frame,
                tile_size,
                min_cutout,
                max_cutout,
                format,
            }
        };

        al_core::log::log(&format!("{:?}", hips));

        hips
    }

    pub fn show(&mut self, ui: &mut egui::Ui, events: Arc<Mutex<Vec<Event>>>) {
        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::none())
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                let mut quit = false;
                let edition_mode = self.edition_mode;
                let visible = self.visible;
                let mut info = false;

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut quit, true, "❌");
                    ui.selectable_value(&mut info,true, "ℹ");
                    ui.selectable_value(&mut self.edition_mode, !edition_mode, "🖊");
                    ui.selectable_value(&mut self.visible, !visible, "👁");
                });

                ui.label(&self.properties.obs_title);
            });

        if self.edition_mode {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::none())
                .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            match &mut self.color {
                                Color::Color(c) => {
                                    ui.label("Color picker");
                                    if ui.color_edit_button_srgba(c).changed() {
                                        events.lock().unwrap().push(
                                            Event::ImageSurveys(vec![self.get_hips_config()])
                                        );
                                    }
                                },
                                Color::Image => (),
                                _ => todo!()
                                //Color::Colormap => todo!()
                            };
                        });

                        if let Some(t) = &mut self.transfer_func {
                            ui.separator();
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.selectable_value(
                                        t,
                                        TransferFunction::ASINH, 
                                        "asinh"
                                    );
                                    ui.selectable_value(
                                        t,
                                        TransferFunction::SIN,
                                        "sin",
                                    );
                                    ui.selectable_value(
                                        t,
                                        TransferFunction::LINEAR,
                                        "linear",
                                    );
                                    ui.selectable_value(
                                        t,
                                        TransferFunction::POW, 
                                        "pow"
                                    );
                                });
                                ui.separator();
                                ui.label("Transfer function");
                                match t {
                                    TransferFunction::ASINH => plot(ui, |x| x.asinh()),
                                    TransferFunction::LINEAR => plot(ui, |x| x),
                                    TransferFunction::POW => plot(ui, |x| x.pow(2.0)),
                                    TransferFunction::SIN => plot(ui, |x| x.sin())
                                }
                            });
                        }

                        if let Some(c) = &mut self.cutouts {
                            ui.separator();
                            ui.label("Cutouts:");
                            ui.add(
                                egui::widgets::Slider::new(&mut c[0], (-1e5 as f32)..=(1e5 as f32))
                                    .text("left"),
                            );

                            ui.add(
                                egui::widgets::Slider::new(&mut c[1], (-1e5 as f32)..=(1e5 as f32))
                                    .text("right"),
                            );
                        }
                    });
        }
    }
}

fn plot(ui: &mut egui::Ui, f: impl Fn(f32) -> f32) {
    use egui::plot::{Line, Value, Values, Plot};
    let sin = (0..100).map(|i| {
        let x = i as f32 * 0.01;
        Value::new(x, f(x))
    });
    let line = Line::new(Values::from_values_iter(sin));
    ui.add(
        Plot::new(
            "Transfer function"
        )
        .legend(egui::widgets::plot::Legend::default())
        .allow_drag(false)
        .allow_zoom(false)
        .line(line)
        .view_aspect(1.0)
        .show_background(false)
    );
}

