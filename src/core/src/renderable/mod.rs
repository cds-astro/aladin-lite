pub mod catalog;
pub mod final_pass;
pub mod grid;
pub mod hips;
pub mod image;
pub mod line;
pub mod moc;
pub mod shape;
pub mod text;
pub mod utils;

use crate::renderable::image::Image;
use crate::tile_fetcher::TileFetcherQueue;

use al_core::image::format::ChannelType;

pub use hips::HiPS;

pub use catalog::Manager;

use al_api::color::ColorRGB;
use al_api::hips::HiPSCfg;
use al_api::hips::ImageMetadata;
use al_api::image::ImageParams;

use al_core::colormap::Colormaps;

use al_core::shader::Shader;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use crate::camera::CameraViewPort;
use crate::shader::ShaderId;
use crate::Abort;
use crate::ProjectionType;
use crate::{shader::ShaderManager, survey::config::HiPSConfig};

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed

use hips::raytracing::RayTracer;

use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

pub trait Renderer {
    fn begin(&mut self);
    fn end(&mut self);
}

pub(crate) type Id = String; // ID of an image, can be an url or a uuidv4
pub(crate) type CreatorDid = String;

type LayerId = String;
pub struct Layers {
    // Surveys to query
    surveys: HashMap<CreatorDid, HiPS>,
    images: HashMap<Id, Vec<Image>>, // an url can contain multiple images i.e. a fits file can contain
    // multiple image extensions
    // The meta data associated with a layer
    meta: HashMap<LayerId, ImageMetadata>,
    // Hashmap between FITS image urls/HiPS creatorDid and layers
    ids: HashMap<LayerId, String>,
    // Layers given in a specific order to draw
    layers: Vec<LayerId>,

    raytracer: RayTracer,
    // A vao that takes all the screen
    screen_vao: VertexArrayObject,

    background_color: ColorRGB,

    gl: WebGlContext,
}

const DEFAULT_BACKGROUND_COLOR: ColorRGB = ColorRGB {
    r: 0.05,
    g: 0.05,
    b: 0.05,
};

fn get_backgroundcolor_shader<'a>(
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
) -> Result<&'a Shader, JsValue> {
    shaders
        .get(
            gl,
            ShaderId(
                "hips_raytracer_backcolor.vert",
                "hips_raytracer_backcolor.frag",
            ),
        )
        .map_err(|e| e.into())
}

pub struct ImageLayer {
    /// Layer name
    pub layer: String,
    pub id: String,
    pub images: Vec<Image>,
    /// Its color
    pub meta: ImageMetadata,
}

impl ImageLayer {
    pub fn get_params(&self) -> ImageParams {
        let cuts = self.images[0].get_cuts();

        let centered_fov = self.images[0].get_centered_fov().clone();
        ImageParams {
            centered_fov,
            min_cut: cuts.start,
            max_cut: cuts.end,
        }
    }
}

impl Layers {
    pub fn new(gl: &WebGlContext, projection: &ProjectionType) -> Result<Self, JsValue> {
        let surveys = HashMap::new();
        let images = HashMap::new();
        let meta = HashMap::new();
        let ids = HashMap::new();
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
        screen_vao
            .bind_for_update()
            .add_array_buffer_single(
                2,
                "pos_clip_space",
                WebGl2RenderingContext::STATIC_DRAW,
                &[-1.0_f32, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0] as &[f32],
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::STATIC_DRAW,
                &[0_u16, 1, 2, 0, 2, 3] as &[u16],
            )
            // Unbind the buffer
            .unbind();

        let background_color = DEFAULT_BACKGROUND_COLOR;
        Ok(Layers {
            surveys,
            images,

            meta,
            ids,
            layers,

            raytracer,

            background_color,
            screen_vao,

            gl,
        })
    }

    pub fn set_survey_url(&mut self, cdid: &CreatorDid, new_url: String) -> Result<(), JsValue> {
        if let Some(survey) = self.surveys.get_mut(cdid) {
            // update the root_url
            survey.get_config_mut().set_root_url(new_url.clone());

            //self.surveys.insert(new_url.clone(), survey);

            // update all the layer urls
            /*for id in self.ids.values_mut() {
                if *id == past_url {
                    *id = new_url.clone();
                }
            }*/

            Ok(())
        } else {
            Err(JsValue::from_str("Survey not found"))
        }
    }

    /*pub fn reset_frame(&mut self) {
        for survey in self.surveys.values_mut() {
            survey.reset_frame();
        }
    }*/

    pub fn set_projection(&mut self, projection: &ProjectionType) -> Result<(), JsValue> {
        // Recompute the raytracer
        self.raytracer = RayTracer::new(&self.gl, &projection)?;
        Ok(())
    }

    pub fn set_background_color(&mut self, color: ColorRGB) {
        self.background_color = color;
    }

    pub fn get_raytracer(&self) -> &RayTracer {
        &self.raytracer
    }

    pub fn draw(
        &mut self,
        camera: &mut CameraViewPort,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        let raytracer = &self.raytracer;
        let raytracing = camera.is_raytracing(projection);

        // Check whether a survey to plot is allsky
        // if neither are, we draw a font
        // if there are, we do not draw nothing
        let render_background_color = !self.layers.iter().any(|layer| {
            let meta = self.meta.get(layer).unwrap_abort();
            let cdid = self.ids.get(layer).unwrap_abort();
            if let Some(survey) = self.surveys.get(cdid) {
                let hips_cfg = survey.get_config();
                (survey.is_allsky() || hips_cfg.get_format().get_channel() == ChannelType::RGB8U)
                    && meta.opacity == 1.0
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

            get_backgroundcolor_shader(&self.gl, shaders)?
                .bind(&self.gl)
                .attach_uniforms_from(camera)
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

            let id = self.ids.get(layer).expect("Url should be found");
            if let Some(survey) = self.surveys.get_mut(id) {
                let hips_cfg = survey.get_config();

                let fully_covering_survey = (survey.is_allsky()
                    || hips_cfg.get_format().get_channel() == ChannelType::RGB8U)
                    && meta.opacity == 1.0;
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
                let id = self.ids.get(layer).expect("Url should be found");
                if let Some(survey) = self.surveys.get_mut(id) {
                    survey.update(camera, projection);

                    // 2. Draw it if its opacity is not null
                    survey.draw(shaders, colormaps, camera, raytracer, draw_opt, projection)?;
                } else if let Some(images) = self.images.get_mut(id) {
                    // 2. Draw it if its opacity is not null
                    for image in images {
                        image.draw(shaders, colormaps, draw_opt, camera, projection)?;
                    }
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

    pub fn remove_layer(
        &mut self,
        layer: &str,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        tile_fetcher: &mut TileFetcherQueue,
    ) -> Result<usize, JsValue> {
        let err_layer_not_found = JsValue::from_str(&format!(
            "Layer {:?} not found, so cannot be removed.",
            layer
        ));
        // Color configs, and urls are indexed by layer
        self.meta.remove(layer).ok_or(err_layer_not_found.clone())?;
        let id = self.ids.remove(layer).ok_or(err_layer_not_found.clone())?;
        // layer from layers does also need to be removed
        let id_layer = self
            .layers
            .iter()
            .position(|l| layer == l)
            .ok_or(err_layer_not_found)?;
        self.layers.remove(id_layer);

        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values().any(|meta| meta.longitude_reversed);

        camera.set_longitude_reversed(longitude_reversed, proj);

        // Check if the url is still used
        let id_still_used = self.ids.values().any(|rem_id| rem_id == &id);
        if id_still_used {
            // Keep the resource whether it is a HiPS or a FITS
            Ok(id_layer)
        } else {
            // Resource not needed anymore
            if let Some(s) = self.surveys.remove(&id) {
                // A HiPS has been found and removed
                let hips_frame = s.get_config().get_frame();
                // remove the frame
                camera.unregister_view_frame(hips_frame, proj);

                // remove the local files access from the tile fetcher
                tile_fetcher.delete_hips_local_files(s.get_config().get_creator_did());

                Ok(id_layer)
            } else if let Some(_) = self.images.remove(&id) {
                // A FITS image has been found and removed
                Ok(id_layer)
            } else {
                Err(JsValue::from_str(&format!(
                    "Url found {:?} is associated to no surveys.",
                    id
                )))
            }
        }
    }

    pub fn rename_layer(&mut self, layer: &str, new_layer: &str) -> Result<(), JsValue> {
        let err_layer_not_found = JsValue::from_str(&format!(
            "Layer {:?} not found, so cannot be removed.",
            layer
        ));

        // layer from layers does also need to be removed
        let id_layer = self
            .layers
            .iter()
            .position(|l| layer == l)
            .ok_or(err_layer_not_found.clone())?;

        self.layers[id_layer] = new_layer.to_string();

        let meta = self.meta.remove(layer).ok_or(err_layer_not_found.clone())?;
        let id = self.ids.remove(layer).ok_or(err_layer_not_found)?;

        // Add the new
        self.meta.insert(new_layer.to_string(), meta);
        self.ids.insert(new_layer.to_string(), id);

        Ok(())
    }

    pub fn swap_layers(&mut self, first_layer: &str, second_layer: &str) -> Result<(), JsValue> {
        let id_first_layer =
            self.layers
                .iter()
                .position(|l| l == first_layer)
                .ok_or(JsValue::from_str(&format!(
                    "Layer {:?} not found, so cannot be removed.",
                    first_layer
                )))?;
        let id_second_layer =
            self.layers
                .iter()
                .position(|l| l == second_layer)
                .ok_or(JsValue::from_str(&format!(
                    "Layer {:?} not found, so cannot be removed.",
                    second_layer
                )))?;

        self.layers.swap(id_first_layer, id_second_layer);

        Ok(())
    }

    pub fn add_image_hips(
        &mut self,
        gl: &WebGlContext,
        hips: HiPSCfg,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        tile_fetcher: &mut TileFetcherQueue,
    ) -> Result<&HiPS, JsValue> {
        let HiPSCfg {
            layer,
            properties,
            meta,
        } = hips;

        let img_ext = meta.img_format;

        // 1. Add the layer name
        let layer_already_found = self.layers.iter().any(|l| l == &layer);

        let idx = if layer_already_found {
            let idx = self.remove_layer(&layer, camera, proj, tile_fetcher)?;
            idx
        } else {
            self.layers.len()
        };

        self.layers.insert(idx, layer.to_string());

        // 2. Add the meta information of the layer
        self.meta.insert(layer.clone(), meta);
        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values().any(|meta| meta.longitude_reversed);

        camera.set_longitude_reversed(longitude_reversed, proj);

        // 3. Add the image survey
        let creator_did = String::from(properties.get_creator_did());
        // The layer does not already exist
        // Let's check if no other hipses points to the
        // same url than `hips`
        let cdid_already_found = self
            .surveys
            .keys()
            .any(|hips_cdid| hips_cdid == &creator_did);

        if !cdid_already_found {
            // The url is not processed yet
            let cfg = HiPSConfig::new(&properties, img_ext)?;

            /*if let Some(initial_ra) = properties.get_initial_ra() {
                if let Some(initial_dec) = properties.get_initial_dec() {
                    camera.set_center::<P>(&LonLatT::new(initial_ra.to_radians().to_angle()), initial_dec.to_radians().to_angle())), &properties.get_frame());
                }
            }

            if let Some(initial_fov) = properties.get_initial_fov() {
                camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
            }*/
            camera.register_view_frame(cfg.get_frame(), proj);

            let hips = HiPS::new(cfg, gl)?;
            // add the frame to the camera

            self.surveys.insert(creator_did.clone(), hips);
        }

        self.ids.insert(layer.clone(), creator_did.clone());

        let hips = self
            .surveys
            .get(&creator_did)
            .ok_or(JsValue::from_str("HiPS not found"))?;
        Ok(hips)
    }

    pub fn add_image(
        &mut self,
        image: ImageLayer,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
        tile_fetcher: &mut TileFetcherQueue,
    ) -> Result<&[Image], JsValue> {
        let ImageLayer {
            layer,
            id,
            images,
            meta,
        } = image;

        // 1. Add the layer name
        let layer_already_found = self.layers.iter().any(|s| s == &layer);

        let idx = if layer_already_found {
            let idx = self.remove_layer(&layer, camera, proj, tile_fetcher)?;
            idx
        } else {
            self.layers.len()
        };

        self.layers.insert(idx, layer.to_string());

        // 2. Add the meta information of the layer
        self.meta.insert(layer.clone(), meta);
        // Loop over all the meta for its longitude reversed property
        // and set the camera to it if there is at least one
        let longitude_reversed = self.meta.values().any(|meta| meta.longitude_reversed);

        camera.set_longitude_reversed(longitude_reversed, proj);

        // 3. Add the fits image
        // The layer does not already exist
        // Let's check if no other hipses points to the
        // same url than `hips`
        let fits_already_found = self.images.keys().any(|image_id| image_id == &id);

        if !fits_already_found {
            // The fits has not been loaded yet
            /*if let Some(initial_ra) = properties.get_initial_ra() {
                if let Some(initial_dec) = properties.get_initial_dec() {
                    camera.set_center::<P>(&LonLatT::new(Angle((initial_ra).to_radians()), Angle((initial_dec).to_radians())), &properties.get_frame());
                }
            }

            if let Some(initial_fov) = properties.get_initial_fov() {
                camera.set_aperture::<P>(Angle((initial_fov).to_radians()));
            }*/

            self.images.insert(id.clone(), images);
        }

        self.ids.insert(layer.clone(), id.clone());

        let img = self
            .images
            .get(&id)
            .ok_or(JsValue::from_str("Fits image not found"))?;
        Ok(img.as_slice())
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
        camera: &mut CameraViewPort,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        let layer_ref = layer.as_str();

        if let Some(meta_old) = self.meta.get(layer_ref) {
            if !meta_old.visible() && meta.visible() {
                if let Some(survey) = self.get_mut_hips_from_layer(layer_ref) {
                    survey.recompute_vertices(camera, projection);
                }

                if let Some(images) = self.get_mut_image_from_layer(layer_ref) {
                    for image in images {
                        image.recompute_vertices(camera, projection)?;
                    }
                }
            } else if meta_old.visible() && !meta.visible() {
                // There is an important point here, if we hide a specific layer
                // then we must recompute the vertices of the layers underneath
                let layer_idx = self
                    .layers
                    .iter()
                    .position(|l| l == layer_ref)
                    .ok_or(JsValue::from_str("Expect the layer to be found!"))?;

                for idx in 0..layer_idx {
                    let cur_layer = self.layers[idx].clone();

                    if let Some(survey) = self.get_mut_hips_from_layer(&cur_layer) {
                        survey.recompute_vertices(camera, projection);
                    } else if let Some(images) = self.get_mut_image_from_layer(&cur_layer) {
                        for image in images {
                            image.recompute_vertices(camera, projection)?;
                        }
                    }
                }
            }
        }

        // Expect the image survey to be found in the hash map
        self.meta.insert(layer.clone(), meta).ok_or_else(|| {
            JsValue::from(js_sys::Error::new(&format!("{:?} layer not found", layer)))
        })?;

        Ok(())
    }

    // Accessors
    // HiPSes getters
    pub fn get_hips_from_layer(&self, layer: &str) -> Option<&HiPS> {
        self.ids
            .get(layer)
            .map(|cdid| self.surveys.get(cdid))
            .flatten()
    }

    pub fn get_mut_hips_from_layer(&mut self, layer: &str) -> Option<&mut HiPS> {
        if let Some(cdid) = self.ids.get_mut(layer) {
            self.surveys.get_mut(cdid)
        } else {
            None
        }
    }

    pub fn get_mut_hips_from_cdid(&mut self, cdid: &str) -> Option<&mut HiPS> {
        self.surveys.get_mut(cdid)
    }

    pub fn get_hips_from_cdid(&mut self, cdid: &str) -> Option<&HiPS> {
        self.surveys.get(cdid)
    }

    pub fn values_hips(&self) -> impl Iterator<Item = &HiPS> {
        self.surveys.values()
    }

    pub fn values_mut_hips(&mut self) -> impl Iterator<Item = &mut HiPS> {
        self.surveys.values_mut()
    }

    // Fits images getters
    pub fn get_mut_image_from_layer(&mut self, layer: &str) -> Option<&mut [Image]> {
        if let Some(url) = self.ids.get(layer) {
            self.images.get_mut(url).map(|images| images.as_mut_slice())
        } else {
            None
        }
    }

    pub fn get_image_from_layer(&self, layer: &str) -> Option<&[Image]> {
        let images = self
            .ids
            .get(layer)
            .map(|url| self.images.get(url))
            .flatten();

        images.map(|images| images.as_slice())
    }
}
