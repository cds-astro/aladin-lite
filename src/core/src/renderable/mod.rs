pub mod catalog;
pub mod final_pass;
pub mod grid;
pub mod labels;
pub mod moc;
pub mod image;
pub mod hips;

pub use hips::HiPS;

pub use labels::TextRenderManager;
pub use catalog::Manager;
pub use grid::ProjetedGrid;

use al_api::hips::ImageSurveyMeta;
use al_api::color::ColorRGB;

use al_core::VertexArrayObject;
use al_core::SliceData;
use al_core::shader::Shader;
use al_core::WebGlContext;
use al_core::image::format::ImageFormatType;
use al_core::webgl_ctx::GlWrapper;

use crate::Abort;
use crate::ProjectionType;
use crate::renderable::image::FitsImage;
use crate::camera::CameraViewPort;
use crate::colormap::Colormaps;
use crate::shader::ShaderId;
use crate::{shader::ShaderManager, survey::config::HiPSConfig};
use crate::SimpleHiPS;

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use hips::raytracing::RayTracer;

use web_sys::{WebGl2RenderingContext};
use wasm_bindgen::JsValue;
use std::borrow::Cow;
use std::collections::{HashSet, HashMap};

pub(crate) type Url = String;
type LayerId = String;
pub struct Layers {
    // Surveys to query
    surveys: HashMap<Url, HiPS>,
    images: HashMap<Url, FitsImage>,
    // The meta data associated with a layer
    meta: HashMap<LayerId, ImageSurveyMeta>,
    // Hashmap between urls and layers
    urls: HashMap<LayerId, Url>,
    // Layers given in a specific order to draw
    ids: Vec<LayerId>,

    most_precise_survey: Url,

    raytracer: RayTracer,
    // A vao that takes all the screen
    screen_vao: VertexArrayObject,

    background_color: ColorRGB,

    depth: u8,

    gl: WebGlContext,
}

const DEFAULT_BACKGROUND_COLOR: ColorRGB = ColorRGB { r: 0.05, g: 0.05, b: 0.05 };

fn get_backgroundcolor_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
    shaders.get(
        gl,
        &ShaderId(
            Cow::Borrowed("RayTracerFontVS"),
            Cow::Borrowed("RayTracerFontFS"),
        ),
    )
    .unwrap_abort()
}

impl Layers {
    pub fn new(
        gl: &WebGlContext,
        projection: &ProjectionType
    ) -> Result<Self, JsValue> {
        let surveys = HashMap::new();
        let images = HashMap::new();
        let meta = HashMap::new();
        let urls = HashMap::new();
        let ids = Vec::new();

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new(gl, &projection)?;
        let gl = gl.clone();
        let most_precise_survey = String::new();

        let mut screen_vao = VertexArrayObject::new(&gl);
        #[cfg(feature = "webgl2")]
        screen_vao.bind_for_update()
            .add_array_buffer_single(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                SliceData::<f32>(&[
                    -1.0, -1.0,
                    1.0, -1.0,
                    1.0, 1.0,
                    -1.0, 1.0,
                ]),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, SliceData::<u16>(&[0, 1, 2, 0, 2, 3]))
            // Unbind the buffer
            .unbind();

        #[cfg(feature = "webgl1")]
        screen_vao.bind_for_update()
            .add_array_buffer(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                SliceData::<f32>(&[
                    -1.0, -1.0,
                    1.0, -1.0,
                    1.0, 1.0,
                    -1.0, 1.0,
                ]),
            )
            // Set the element buffer
            .add_element_buffer(WebGl2RenderingContext::STATIC_DRAW, SliceData::<u16>(&[0, 1, 2, 0, 2, 3]))
            // Unbind the buffer
            .unbind();

        let depth = 0;
        let background_color = DEFAULT_BACKGROUND_COLOR;
        Ok(Layers {
            surveys,
            images,

            meta,
            urls,
            ids,

            most_precise_survey,

            raytracer,
            depth,

            background_color,
            screen_vao,

            gl,
        })
    }

    pub fn set_survey_url(&mut self, past_url: String, new_url: String) -> Result<(), JsValue> {
        if let Some(mut survey) = self.surveys.remove(&past_url) {
            // update the root_url
            survey.get_config_mut()
                .set_root_url(new_url.clone());
            
            self.surveys.insert(new_url.clone(), survey);

            // update all the layer urls
            for url in self.urls.values_mut() {
                if *url == past_url {
                    *url = new_url.clone(); 
                }
            }

            if self.most_precise_survey == past_url {
                self.most_precise_survey = new_url.clone();
            }

            Ok(())
        } else {
            Err(JsValue::from_str("Survey not found"))
        }
    }

    pub fn reset_frame(&mut self) {
        for survey in self.surveys.values_mut() {
            survey.reset_frame();
        }
    }

    pub fn set_projection(&mut self, projection: &ProjectionType) -> Result<(), JsValue> {
        // Recompute the raytracer
        self.raytracer = RayTracer::new(&self.gl, &projection)?;
        Ok(())
    }

    pub fn set_background_color(&mut self, color: ColorRGB) {
        self.background_color = color;
    }

    pub fn draw(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        projection: &ProjectionType
    ) -> Result<(), JsValue> {
        let raytracer = &self.raytracer;
        let raytracing = raytracer.is_rendering(camera/* , depth_texture*/);

        // The first layer must be paint independently of its alpha channel
        self.gl.enable(WebGl2RenderingContext::BLEND);
        // Check whether a survey to plot is allsky
        // if neither are, we draw a font
        // if there are, we do not draw nothing
        if !self.surveys.is_empty() {
            let not_render_transparency_font = self.ids.iter()
                .any(|layer| {
                    let meta = self.meta.get(layer).unwrap_abort();
                    let url = self.urls.get(layer).unwrap_abort();
                    let survey = self.surveys.get(url).unwrap_abort();
                    let hips_cfg = survey.get_config();

                    (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0
                });

            // Need to render transparency font
            if !not_render_transparency_font {
                let opacity = self.surveys.values()
                    .fold(std::f32::MAX, |mut a, s| {
                        a = a.min(s.get_fading_factor()); a
                    });
                let background_color = &self.background_color * opacity;

                let vao = if raytracing {
                    raytracer.get_vao()
                } else {
                    // define a vao that consists of 2 triangles for the screen
                    &self.screen_vao
                };

                get_backgroundcolor_shader(&self.gl, shaders).bind(&self.gl).attach_uniforms_from(camera)
                    .attach_uniform("color", &background_color)
                    .attach_uniform("opacity", &opacity)
                    .bind_vertex_array_object_ref(vao)
                        .draw_elements_with_i32(
                            WebGl2RenderingContext::TRIANGLES,
                            None,
                            WebGl2RenderingContext::UNSIGNED_SHORT,
                            0,
                        );
            }
        }

        // Pre loop over the layers to see if a HiPS is entirely covering those behind
        // so that we do not have to render those
        let mut idx_start_layer = 0;
        for (idx_layer, layer) in self.ids.iter().enumerate().skip(1) {
            let meta = self.meta.get(layer).expect("Meta should be found");

            let url = self.urls.get(layer).expect("Url should be found");
            let survey = self.surveys.get_mut(url).unwrap_abort();
            let hips_cfg = survey.get_config();

            let fully_covering_survey = (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0;
            if fully_covering_survey {
                idx_start_layer = idx_layer;
            }
        }

        let rendered_layers = &self.ids[idx_start_layer..];
        for layer in rendered_layers {
            let draw_opt = self.meta.get(layer).expect("Meta should be found");
            if draw_opt.visible() {
                // 1. Update the survey if necessary
                let url = self.urls.get(layer).expect("Url should be found");
                if let Some(survey) = self.surveys.get_mut(url) {
                    let ImageSurveyMeta {
                        color,
                        opacity,
                        blend_cfg,
                        ..
                    } = draw_opt;

                    survey.update(camera, projection);

                    // 2. Draw it if its opacity is not null
                    blend_cfg.enable(&self.gl, || {
                        survey.draw(
                            raytracer,
                            shaders,
                            camera,
                            color,
                            *opacity,
                            colormaps,
                        )?;
    
                        Ok(())
                    })?;
                } else if let Some(image) = self.images.get_mut(url) {
                    image.update(camera, projection)?;

                    // 2. Draw it if its opacity is not null
                    image.draw(
                        shaders,
                        colormaps,
                        &draw_opt,
                    )?;
                }
            }
        }

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    pub fn set_image_surveys(
        &mut self,
        hipses: Vec<SimpleHiPS>,
        gl: &WebGlContext,
        camera: &mut CameraViewPort,
        projection: &ProjectionType
    ) -> Result<(), JsValue> {
        // 1. Check if layer duplicated have been given
        for i in 0..hipses.len() {
            for j in 0..i {
                if hipses[i].get_layer() == hipses[j].get_layer() {
                    let layer = &hipses[i].get_layer();
                    return Err(JsValue::from_str(&format!(
                        "{:?} layer name are duplicates",
                        layer
                    )));
                }
            }
        }

        let mut current_needed_surveys = HashSet::new();
        for hips in hipses.iter() {
            let url = hips.get_properties().get_url();
            current_needed_surveys.insert(url);
        }

        // Remove surveys that are not needed anymore
        self.surveys = self
            .surveys
            .drain()
            .filter(|(_, m)| current_needed_surveys.contains(&m.get_config().root_url))
            .collect();

        // Create the new surveys
        let mut max_depth_among_surveys = 0;

        self.meta.clear();
        self.ids.clear();
        self.urls.clear();

        let _num_surveys = hipses.len();
        let mut longitude_reversed = false;
        for SimpleHiPS {
            layer,
            properties,
            meta,
            img_format,
            ..
        } in hipses.into_iter()
        {
            let config = HiPSConfig::new(&properties, img_format)?;
            //camera.set_longitude_reversed(meta.longitude_reversed);

            // Get the most precise survey from all the ones given
            let url = properties.get_url();
            let max_order = properties.get_max_order();
            if max_order > max_depth_among_surveys {
                max_depth_among_surveys = max_order;
                self.most_precise_survey = url.clone();
            }

            // Add the new surveys
            if !self.surveys.contains_key(&url) {
                let survey = HiPS::new(config, gl, camera)?;
                self.surveys.insert(url.clone(), survey);

                // A new survey has been added and it is lonely
                /*if num_surveys == 1 {
                    if let Some(initial_ra) = properties.get_initial_ra() {
                        if let Some(initial_dec) = properties.get_initial_dec() {
                            camera.set_center::<P>(&LonLatT(Angle((initial_ra).to_radians()), Angle((initial_dec).to_radians())), &properties.get_frame());
                        }
                    }

                    if let Some(initial_fov) = properties.get_initial_fov() {
                        camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
                    }
                }*/
            }

            longitude_reversed |= meta.longitude_reversed;

            self.meta.insert(layer.clone(), meta);
            self.urls.insert(layer.clone(), url);

            self.ids.push(layer);
        }

        camera.set_longitude_reversed(longitude_reversed, &projection);

        Ok(())
    }

    pub fn add_fits_image(&mut self) {

    }

    pub fn get_layer_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue> {
        self.meta
            .get(layer)
            .cloned()
            .ok_or_else(|| JsValue::from(js_sys::Error::new("Survey not found")))
    }

    pub fn set_layer_cfg(
        &mut self,
        layer: String,
        meta: ImageSurveyMeta,
        camera: &CameraViewPort,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        if let Some(meta_old) = self.meta.get(&layer) {
            if !meta_old.visible() && meta.visible() {
                if let Some(survey) = self.get_mut_hips_from_layer(&layer) {
                    survey.recompute_vertices(camera, projection);
                }

                if let Some(image) = self.get_mut_image_from_layer(&layer) {
                    image.update(camera, projection)?;
                }
            }
        }

        // Expect the image survey to be found in the hash map
        self.meta.insert(layer.clone(), meta).ok_or_else(|| {
            JsValue::from(js_sys::Error::new(&format!("{:?} layer not found", layer)))
        })?;

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        let ready = self
            .surveys
            .iter()
            .map(|(_, survey)| survey.is_ready())
            .fold(true, |acc, x| acc & x);

        ready
    }

    pub fn refresh_views(&mut self, camera: &mut CameraViewPort) {
        self.depth = 0;

        for survey in self.surveys.values_mut() {
            survey.refresh_view(camera);
            
            self.depth = self.depth.max(survey.get_depth());
        }
    }

    // Accessors
    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    // HiPSes getters
    pub fn get_hips_from_layer(&self, id: &str) -> Option<&HiPS> {
        self.urls.get(id).map(|url| self.surveys.get(url).unwrap_abort())
    }

    pub fn get_mut_hips_from_layer(&mut self, id: &str) -> Option<&mut HiPS> {
        let url = self.urls.get_mut(id);
        if let Some(url) = url {
            self.surveys.get_mut(url)
        } else {
            None
        }
    }

    pub fn get_mut_hips(&mut self, root_url: &str) -> Option<&mut HiPS> {
        self.surveys.get_mut(root_url)
    }

    pub fn values_hips(&self) -> impl Iterator<Item = &HiPS> {
        self.surveys.values()
    }

    pub fn values_mut_hips(&mut self) -> impl Iterator<Item = &mut HiPS> {
        self.surveys.values_mut()
    }

    // Fits images getters
    pub fn get_mut_image_from_layer(&mut self, id: &str) -> Option<&mut FitsImage> {
        let url = self.urls.get_mut(id);
        if let Some(url) = url {
            self.images.get_mut(url)
        } else {
            None
        }
    }
}

