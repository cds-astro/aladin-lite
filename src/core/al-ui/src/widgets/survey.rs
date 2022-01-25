use std::borrow::BorrowMut;

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
use egui::{Color32, InnerResponse, Response};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/hpxImageSurvey.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = fetchSurveyMetadata)]
    async fn fetch_survey_metadata(url: String) -> Result<JsValue, JsValue>;
}

use wasm_bindgen_futures;
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
    LOG,
    SQRT
}

#[derive(PartialEq)]
enum ColorOption {
    RGB,
    Color,
    Colormap,
}

pub struct SurveyWidget {
    url: String,
    properties: PropertiesParsed,

    pub visible: bool,
    edition_mode: bool,
    quit: bool,
    color_option: ColorOption,
    pub update_survey: bool,

    /* Edition mode */
    // Color panel
    color_cfg: HiPSColor,
    color: Color32,
    k: f32,
    colormap: String,
    reversed: bool,
    // Blending panel
    blend_cfg: BlendCfg,
    opacity: f32,
    // FITS specific panel
    transfer_func: Option<TransferFunction>,
    cutouts: Option<[f32; 2]>,
    cut_range: std::ops::RangeInclusive<f32>
}

use cgmath::num_traits::Pow;

use crate::painter::WebGlRenderingCtx;
use al_api::blend::{
    BlendCfg, BlendFactor, BlendFunc
};
use al_api::hips::{Frame, HiPSColor, HiPSFormat, HiPSProperties, SimpleHiPS};
impl SurveyWidget {
    pub async fn new(url: String) -> Self {
        let properties = request_survey_properties(url.clone()).await;
        let cutouts = properties.hips_pixel_cut.clone();
        let cut_range = if let Some(c) = properties.hips_pixel_cut.clone() {
            let lc = c[1] - c[0];
            let half_lc = 0.5 * lc;
    
            let c_min = c[0] - half_lc;
            let c_max = c[1] + half_lc;

            c_min..=c_max
        } else {
            0.0..=0.0
        };

       
        let color_cfg = if properties.is_fits_image() {
            HiPSColor::Grayscale2Color {
                color: [1.0, 0.0, 0.0],
                transfer: "asinh".to_string(),
                k: 1.0,
            }
        } else {
            HiPSColor::Color
        };

        let k = 1.0;
        let color = Color32::RED;
        let colormap = String::from("blackwhite");
        let reversed = false;

        let transfer_func = Some(TransferFunction::ASINH);

        let blend_cfg = BlendCfg {
            src_color_factor: BlendFactor::SrcAlpha,
            dst_color_factor: BlendFactor::OneMinusSrcAlpha,
            func: BlendFunc::FuncAdd,
        };
        let opacity = 1.0;
        let is_grayscale_image = properties.is_fits_image();
        Self {
            url,
            properties,

            quit: false,
            visible: true,
            edition_mode: false,

            update_survey: false,
            color_option: if is_grayscale_image { ColorOption::Color } else { ColorOption::RGB },
            // edition mode
            // Color
            color_cfg, // color config
            color, // color
            k, // strength color
            colormap,
            reversed,
            

            blend_cfg,
            opacity,
            transfer_func,
            cutouts,
            cut_range
        }
    }

    /*pub fn color(&self) -> &HiPSColor {
        &self.color
    }*/

    pub fn get_hips_config(&self) -> SimpleHiPS {
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

        let opacity = if !self.visible {
            0.0
        } else {
            self.opacity
        };

        let hips = SimpleHiPS {
            color: self.color_cfg.clone(),
            blend_cfg: self.blend_cfg.clone(),
            opacity: opacity,
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

    pub fn removed(&self) -> bool {
        self.quit
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.update_survey = false;

        egui::Frame::popup(ui.style())
            .stroke(egui::Stroke::none())
            .show(ui, |ui| {
                ui.set_max_width(270.0);
                let edition_mode = self.edition_mode;
                let mut info = false;

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.quit, true, "❌");
                    ui.selectable_value(&mut info,true, "ℹ");
                    ui.selectable_value(&mut self.edition_mode, !edition_mode, "🖊");
                    let v = self.visible;
                    if ui.selectable_value(&mut self.visible, !v, "👁").clicked() {
                        self.update_survey = true;
                    }
                });

                ui.label(&self.properties.obs_title);
            });

        if self.edition_mode {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::none())
                .show(ui, |ui| {
                    let mut ui_changed = false;
                    ui.vertical_centered(|ui| {
                        self.color_picker_widget(ui, &mut ui_changed);

                        ui.group(|ui| {
                            if let Some(t) = &mut self.transfer_func {
                                egui::Grid::new("").show(ui, |ui| {
                                    // Plot widget
                                    match t {
                                        TransferFunction::ASINH => plot(ui, |x| x.asinh()),
                                        TransferFunction::LINEAR => plot(ui, |x| x),
                                        TransferFunction::POW => plot(ui, |x| x.pow(2.0)),
                                        TransferFunction::SQRT => plot(ui, |x| x.sqrt()),
                                        TransferFunction::LOG => plot(ui, |x| (1000.0*x + 1.0).ln()/1000_f32.ln()),
                                    }
        
                                    // Selection of the transfer function
                                    ui.vertical(|ui| {
                                        ui_changed |= ui.selectable_value(
                                            t,
                                            TransferFunction::ASINH, 
                                            "asinh"
                                        ).clicked();
        
                                        ui_changed |= ui.selectable_value(
                                            t,
                                            TransferFunction::LOG,
                                            "log",
                                        ).clicked();
        
                                        ui_changed |= ui.selectable_value(
                                            t,
                                            TransferFunction::LINEAR,
                                            "linear",
                                        ).clicked();
        
                                        ui_changed |= ui.selectable_value(
                                            t,
                                            TransferFunction::POW, 
                                            "pow2"
                                        ).clicked();
        
                                        ui_changed |= ui.selectable_value(
                                            t,
                                            TransferFunction::SQRT, 
                                            "sqrt"
                                        ).clicked();
                                    });
                                    ui.end_row();
                                });
                                }
        
                                if let Some(c) = &mut self.cutouts {
                                    ui.separator();
                                    ui.label("Cutouts:");
                                    ui_changed |= ui.add(
                                        egui::widgets::Slider::new(&mut c[0], self.cut_range.clone())
                                            .text("left")
                                    ).changed();
        
                                    ui_changed |= ui.add(
                                        egui::widgets::Slider::new(&mut c[1], self.cut_range.clone())
                                            .text("right"),
                                    ).changed();
                                }
                            });
    
                            ui.separator();
                            blend_widget(ui, &mut self.blend_cfg, &mut self.opacity, &mut ui_changed);
                    });
                    if ui_changed {
                        self.update_survey = true;
                    }
                });
        }
    }

    fn color_picker_widget(&mut self, ui: &mut egui::Ui, ui_changed: &mut bool) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if !self.properties.is_fits_image() {
                    *ui_changed |= ui.selectable_value(
                        &mut self.color_option,
                        ColorOption::RGB,
                        "RGB"
                    ).clicked();
                }

                *ui_changed |= ui.selectable_value(
                    &mut self.color_option,
                    ColorOption::Colormap,
                    "Colormap"
                ).clicked();
        
                *ui_changed |= ui.selectable_value(
                    &mut self.color_option,
                    ColorOption::Color,
                    "Color"
                ).clicked();
            });

            let transfer = match self.transfer_func.as_ref().unwrap() {
                TransferFunction::ASINH => String::from("asinh"),
                TransferFunction::LINEAR => String::from("linear"),
                TransferFunction::POW => String::from("pow2"),
                TransferFunction::LOG => String::from("log"),
                TransferFunction::SQRT => String::from("sqrt"),
            };

            match self.color_option {
                ColorOption::Color => {
                    ui.label("Color picker");
                    *ui_changed |= ui.color_edit_button_srgba(&mut self.color).changed();
                    *ui_changed |= ui.add(
                        egui::widgets::Slider::new(&mut self.k, 0.0..=2.0)
                            .text("Strength"),
                    ).changed();
                    self.color_cfg = HiPSColor::Grayscale2Color {
                        color: [
                            (self.color.r() as f32)/255.0,
                            (self.color.g() as f32)/255.0,
                            (self.color.b() as f32)/255.0
                        ],
                        transfer,
                        k: self.k
                    };
                },
                ColorOption::Colormap => {
                    egui::ComboBox::from_label("Colormap")
                    .selected_text(format!("{:?}", self.colormap))
                    .show_ui(ui, |ui| {
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "blackwhite".to_string(), "blackwhite").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "blues".to_string(), "blues").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "parula".to_string(), "parula").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "rainbow".to_string(), "rainbow").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "RdBu".to_string(), "RdBu").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "RdYiBu".to_string(), "RdYiBu").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "redtemperature".to_string(), "redtemperature").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "spectral".to_string(), "spectral").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "summer".to_string(), "summer").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "YIGnBu".to_string(), "YIGnBu").clicked();
                        *ui_changed |= ui.selectable_value(&mut self.colormap, "YIOrBr".to_string(), "YIOrBr").clicked();
                    });

                    *ui_changed |= ui.add(egui::Checkbox::new(&mut self.reversed, "Reversed")).changed();

                    self.color_cfg = HiPSColor::Grayscale2Colormap {
                        colormap: self.colormap.clone(),
                        transfer,
                        reversed: self.reversed,
                    };
                },
                ColorOption::RGB => {
                    self.color_cfg = HiPSColor::Color;
                }
            }
        });
    }
}



fn blend_widget(ui: &mut egui::Ui, blend: &mut BlendCfg, opacity: &mut f32, update_parent: &mut bool) {
    ui.group(|ui| {
        ui.label("Blending:");
        let mut ui_changed = false;

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Src Color")
                .selected_text(format!("{:?}", blend.src_color_factor))
                .show_ui(ui, |ui| {
                    ui_changed |= ui.selectable_value(&mut blend.src_color_factor, BlendFactor::SrcAlpha, "SrcAlpha").clicked();
                    ui_changed |= ui.selectable_value(&mut blend.src_color_factor, BlendFactor::OneMinusSrcAlpha, "OneMinusSrcAlpha").clicked();
                    ui_changed |= ui.selectable_value(&mut blend.src_color_factor, BlendFactor::One, "One").clicked();
                });
            egui::ComboBox::from_label("Dst Color")
                .selected_text(format!("{:?}", blend.dst_color_factor))
                .show_ui(ui, |ui| {
                    ui_changed |= ui.selectable_value(&mut blend.dst_color_factor, BlendFactor::SrcAlpha, "SrcAlpha").clicked();
                    ui_changed |= ui.selectable_value(&mut blend.dst_color_factor, BlendFactor::OneMinusSrcAlpha, "OneMinusSrcAlpha").clicked();
                    ui_changed |= ui.selectable_value(&mut blend.dst_color_factor, BlendFactor::One, "One").clicked();
                });
        });
    
        #[cfg(feature = "webgl2")]
        egui::ComboBox::from_label("Blend Func")
        .selected_text(format!("{:?}", blend.func))
        .show_ui(ui, |ui| {
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncAdd, "Add").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncSubstract, "Subtract").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncReverseSubstract, "Reverse Subtract").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::Min, "Min").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::Max, "Max").clicked();
        });
        #[cfg(feature = "webgl1")]
        egui::ComboBox::from_label("Blend Func")
        .selected_text(format!("{:?}", blend.func))
        .show_ui(ui, |ui| {
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncAdd, "Add").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncSubstract, "Subtract").clicked();
            ui_changed |= ui.selectable_value(&mut blend.func, BlendFunc::FuncReverseSubstract, "Reverse Subtract").clicked();
        });
    
        ui_changed |= ui.add(
            egui::widgets::Slider::new(opacity, 0.0..=1.0)
                .text("Alpha")
        ).changed();

        if ui_changed {
            *update_parent = true;
        }
    });
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
        .width(100.0)
        .legend(egui::widgets::plot::Legend::default())
        .allow_drag(false)
        .allow_zoom(false)
        .line(line)
        .view_aspect(1.0)
        .show_background(false)
    );
}

