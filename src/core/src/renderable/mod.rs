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

use al_api::hips::ImageMetadata;
use al_api::color::ColorRGB;
use al_api::hips::HiPSCfg;

use al_core::VertexArrayObject;
use al_core::SliceData;
use al_core::shader::Shader;
use al_core::WebGlContext;
use al_core::image::format::ImageFormatType;
use al_core::colormap::Colormaps;

use crate::Abort;
use crate::ProjectionType;
use crate::renderable::image::FitsImage;
use crate::camera::CameraViewPort;
use crate::shader::ShaderId;
use crate::{shader::ShaderManager, survey::config::HiPSConfig};

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use hips::raytracing::RayTracer;

use web_sys::{WebGl2RenderingContext};
use wasm_bindgen::JsValue;
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) type Url = String;
type LayerId = String;
pub struct Layers {
    // Surveys to query
    surveys: HashMap<Url, HiPS>,
    images: HashMap<Url, FitsImage>,
    // The meta data associated with a layer
    meta: HashMap<LayerId, ImageMetadata>,
    // Hashmap between urls and layers
    urls: HashMap<LayerId, Url>,
    // Layers given in a specific order to draw
    layers: Vec<LayerId>,

    raytracer: RayTracer,
    // A vao that takes all the screen
    screen_vao: VertexArrayObject,

    background_color: ColorRGB,

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

pub struct FitsCfg {
    /// Layer name
    pub layer: String,
    pub url: String,
    pub fits: FitsImage,
    /// Its color
    pub meta: ImageMetadata,
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
        let layers = Vec::new();

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let raytracer = RayTracer::new(gl, &projection)?;
        let gl = gl.clone();

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

        let background_color = DEFAULT_BACKGROUND_COLOR;
        Ok(Layers {
            surveys,
            images,

            meta,
            urls,
            layers,

            raytracer,

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

        // Check whether a survey to plot is allsky
        // if neither are, we draw a font
        // if there are, we do not draw nothing
        let render_background_color = !self.layers.iter()
            .any(|layer| {
                let meta = self.meta.get(layer).unwrap_abort();
                let url = self.urls.get(layer).unwrap_abort();
                if let Some(survey) = self.surveys.get(url) {
                    let hips_cfg = survey.get_config();
                    (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0
                } else {
                    // image fits case
                    false
                }
            });

        // Need to render transparency font
        if render_background_color {
            let background_color = &self.background_color;

            let vao = if raytracing {
                raytracer.get_vao()
            } else {
                // define a vao that consists of 2 triangles for the screen
                &self.screen_vao
            };

            get_backgroundcolor_shader(&self.gl, shaders).bind(&self.gl).attach_uniforms_from(camera)
                .attach_uniform("color", &background_color)
                .bind_vertex_array_object_ref(vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        None,
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        0,
                    );
        }


        // The first layer must be paint independently of its alpha channel
        self.gl.enable(WebGl2RenderingContext::BLEND);
        // Pre loop over the layers to see if a HiPS is entirely covering those behind
        // so that we do not have to render those
        let mut idx_start_layer = 0;
        for (idx_layer, layer) in self.layers.iter().enumerate().skip(1) {
            let meta = self.meta.get(layer).expect("Meta should be found");

            let url = self.urls.get(layer).expect("Url should be found");
            if let Some(survey) = self.surveys.get_mut(url) {
                let hips_cfg = survey.get_config();

                let fully_covering_survey = (survey.is_allsky() || hips_cfg.get_format() == ImageFormatType::RGB8U) && meta.opacity == 1.0;
                if fully_covering_survey {
                    idx_start_layer = idx_layer;
                }
            }
        }

        let rendered_layers = &self.layers[idx_start_layer..];
        for layer in rendered_layers {
            let draw_opt = self.meta.get(layer).expect("Meta should be found");
            if draw_opt.visible() {
                // 1. Update the survey if necessary
                let url = self.urls.get(layer).expect("Url should be found");
                if let Some(survey) = self.surveys.get_mut(url) {
                    survey.update(camera, projection);

                    // 2. Draw it if its opacity is not null
                    survey.draw(
                        shaders,
                        colormaps,
                        camera,
                        raytracer,
                        draw_opt
                    )?;
                } else if let Some(image) = self.images.get_mut(url) {
                    image.update(camera, projection)?;

                    // 2. Draw it if its opacity is not null
                    image.draw(
                        shaders,
                        colormaps,
                        draw_opt,
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

    pub fn remove_layer(&mut self, layer: &str, camera: &mut CameraViewPort, projection: &ProjectionType) -> Result<usize, JsValue> {
        let err_layer_not_found = JsValue::from_str(&format!("Layer {:?} not found, so cannot be removed.", layer));
        // Color configs, and urls are indexed by layer
        self.meta.remove(layer)
            .ok_or(err_layer_not_found.clone())?;
        let url = self.urls.remove(layer).ok_or(err_layer_not_found.clone())?;
        // layer from layers does also need to be removed
        let id_layer = self.layers.iter()
            .position(|l| layer == l)
            .ok_or(err_layer_not_found)?;
        self.layers.remove(id_layer);

        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values()
            .any(|meta| {
                meta.longitude_reversed
            });

        camera.set_longitude_reversed(longitude_reversed, projection);

        // Check if the url is still used
        let url_still_used = self.urls.values().any(|rem_url| rem_url == &url);
        if url_still_used {
            // Keep the resource whether it is a HiPS or a FITS
            Ok(id_layer)
        } else {
            // Resource not needed anymore
            if let Some(_) = self.surveys.remove(&url) {
                // A HiPS has been found and removed
                Ok(id_layer)
            } else if let Some(_) = self.images.remove(&url) {
                // A FITS image has been found and removed
                Ok(id_layer)
            } else {
                Err(JsValue::from_str(&format!("Url found {:?} is associated to no surveys.", url)))
            }
        }
    }

    pub fn rename_layer(
        &mut self,
        layer: &str,
        new_layer: &str,
    ) -> Result<(), JsValue> {
        let err_layer_not_found = JsValue::from_str(&format!("Layer {:?} not found, so cannot be removed.", layer));

        // layer from layers does also need to be removed
        let id_layer = self.layers.iter()
            .position(|l| layer == l)
            .ok_or(err_layer_not_found.clone())?;
    
        self.layers[id_layer] = new_layer.to_string();

        let meta = self.meta.remove(layer)
            .ok_or(err_layer_not_found.clone())?;
        let url = self.urls.remove(layer).ok_or(err_layer_not_found)?;

        // Add the new
        self.meta.insert(new_layer.to_string(), meta);
        self.urls.insert(new_layer.to_string(), url);

        Ok(())
    }

    pub fn swap_layers(
        &mut self,
        first_layer: &str,
        second_layer: &str,
    ) -> Result<(), JsValue> {
        let id_first_layer = self.layers.iter()
            .position(|l| l == first_layer)
            .ok_or(JsValue::from_str(&format!("Layer {:?} not found, so cannot be removed.", first_layer)))?;
        let id_second_layer = self.layers.iter()
            .position(|l| l == second_layer)
            .ok_or(JsValue::from_str(&format!("Layer {:?} not found, so cannot be removed.", second_layer)))?;

        self.layers.swap(id_first_layer, id_second_layer);

        Ok(())
    }

    pub fn add_image_survey(
        &mut self,
        gl: &WebGlContext,
        hips: HiPSCfg,
        camera: &mut CameraViewPort,
        projection: &ProjectionType
    ) -> Result<&HiPS, JsValue> {
        let HiPSCfg {
            layer,
            properties,
            meta,
        } = hips;

        // 1. Add the layer name
        let layer_already_found = self.layers.iter()
            .any(|l| {
                l == &layer
            });

        let idx = if layer_already_found {
            let idx = self.remove_layer(&layer, camera, projection)?;
            idx
        } else {
            self.layers.len()
        };

        self.layers.insert(idx, layer.to_string());

        // 2. Add the image survey
        let url = String::from(properties.get_url());
        // The layer does not already exist
        // Let's check if no other hipses points to the
        // same url than `hips`
        let url_already_found = self.surveys.keys()
            .any(|hips_url| {
                hips_url == &url
            });

        if !url_already_found {
            // The url is not processed yet
            let cfg = HiPSConfig::new(&properties, meta.img_format)?;

            /*if let Some(initial_ra) = properties.get_initial_ra() {
                if let Some(initial_dec) = properties.get_initial_dec() {
                    camera.set_center::<P>(&LonLatT(Angle((initial_ra).to_radians()), Angle((initial_dec).to_radians())), &properties.get_frame());
                }
            }

            if let Some(initial_fov) = properties.get_initial_fov() {
                camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
            }*/

            let hips = HiPS::new(cfg, gl, camera)?;
            self.surveys.insert(url.clone(), hips);
        }

        self.urls.insert(layer.clone(), url.clone());

        // 3. Add the meta information of the layer
        self.meta.insert(layer.clone(), meta);
        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values()
            .any(|meta| {
                meta.longitude_reversed
            });

        camera.set_longitude_reversed(longitude_reversed, projection);

        // Refresh the views of all the surveys
        // this is necessary to compute the max depth between the surveys
        self.refresh_views(camera);

        let hips = self.surveys.get(&url).ok_or(JsValue::from_str("HiPS not found"))?;
        Ok(hips)
    }

    pub fn add_image_fits(
        &mut self,
        fits: FitsCfg,
        camera: &mut CameraViewPort,
        projection: &ProjectionType
    ) -> Result<&FitsImage, JsValue> {
        let FitsCfg {
            layer,
            url,
            fits,
            meta,
        } = fits;

        // 1. Add the layer name
        let layer_already_found = self.layers.iter()
            .any(|s| {
                s == &layer
            });

        let idx = if layer_already_found {
            let idx = self.remove_layer(&layer, camera, projection)?;
            idx
        } else {
            self.layers.len()
        };

        self.layers.insert(idx, layer.to_string());

        // 2. Add the meta information of the layer
        self.meta.insert(layer.clone(), meta);
        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values()
            .any(|meta| {
                meta.longitude_reversed
            });

        camera.set_longitude_reversed(longitude_reversed, projection);

        // 3. Add the fits image
        // The layer does not already exist
        // Let's check if no other hipses points to the
        // same url than `hips`
        let fits_already_found = self.images.keys()
            .any(|image_url| {
                image_url == &url
            });

        if !fits_already_found {
            // The fits has not been loaded yet
            /*if let Some(initial_ra) = properties.get_initial_ra() {
                if let Some(initial_dec) = properties.get_initial_dec() {
                    camera.set_center::<P>(&LonLatT(Angle((initial_ra).to_radians()), Angle((initial_dec).to_radians())), &properties.get_frame());
                }
            }

            if let Some(initial_fov) = properties.get_initial_fov() {
                camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
            }*/

            self.images.insert(url.clone(), fits);
        }

        self.urls.insert(layer.clone(), url.clone());

        let fits = self.images.get(&url).ok_or(JsValue::from_str("Fits image not found"))?;
        Ok(fits)
    }

    pub fn get_layer_cfg(&self, layer: &str) -> Result<ImageMetadata, JsValue> {
        self.meta
            .get(layer)
            .cloned()
            .ok_or_else(|| JsValue::from(js_sys::Error::new("Survey not found")))
    }

    pub fn set_layer_cfg(
        &mut self,
        layer: String,
        meta: ImageMetadata,
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
        for survey in self.surveys.values_mut() {
            survey.refresh_view(camera);
        }
    }

    // Accessors
    // HiPSes getters
    pub fn get_hips_from_layer(&self, layer: &str) -> Option<&HiPS> {
        self.urls.get(layer)
            .map(|url| self.surveys.get(url))
            .flatten()
    }

    pub fn get_mut_hips_from_layer(&mut self, layer: &str) -> Option<&mut HiPS> {
        if let Some(url) = self.urls.get_mut(layer) {
            self.surveys.get_mut(url)
        } else {
            None
        }
    }

    pub fn get_mut_hips_from_url(&mut self, root_url: &str) -> Option<&mut HiPS> {
        self.surveys.get_mut(root_url)
    }

    pub fn get_hips_from_url(&mut self, root_url: &str) -> Option<&HiPS> {
        self.surveys.get(root_url)
    }

    pub fn values_hips(&self) -> impl Iterator<Item = &HiPS> {
        self.surveys.values()
    }

    pub fn values_mut_hips(&mut self) -> impl Iterator<Item = &mut HiPS> {
        self.surveys.values_mut()
    }

    // Fits images getters
    pub fn get_mut_image_from_layer(&mut self, layer: &str) -> Option<&mut FitsImage> {
        if let Some(url) = self.urls.get(layer) {
            self.images.get_mut(url)
        } else {
            None
        }
    }

    pub fn get_image_from_layer(&self, layer: &str) -> Option<&FitsImage> {
        self.urls.get(layer)
            .map(|url| {
                self.images.get(url)
            }).flatten()
    }
}

