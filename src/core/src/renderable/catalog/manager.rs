use super::source::Source;
use crate::ShaderManager;
use al_api::colormap::Colormap;
use al_api::resources::Resources;

use al_core::FrameBufferObject;
use al_core::{
    shader::Shader, Texture2D, VecData, VertexArrayObject, WebGlContext,
};

use std::collections::HashMap;
use std::iter::FromIterator;
use web_sys::WebGl2RenderingContext;

#[derive(Debug)]
pub enum Error {
    CatalogNotPresent { message: String },
}
use wasm_bindgen::JsValue;
impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        match err {
            Error::CatalogNotPresent { message } => message.into(),
        }
    }
}

pub struct Manager {
    gl: WebGlContext,
    kernel_texture: Texture2D,

    fbo: FrameBufferObject,

    // VAOs
    vertex_array_object_screen: VertexArrayObject,

    catalogs: HashMap<String, Catalog>,
    kernel_size: Vector2<f32>,
}

impl Manager {
    pub fn new(
        gl: &WebGlContext,
        _shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        resources: &Resources,
    ) -> Result<Self, JsValue> {
        // Load the texture of the gaussian kernel
        let kernel_filename = resources.get_filename("kernel").unwrap();
        let kernel_texture = Texture2D::create_from_path::<_, al_core::image::format::RGBA8U>(
            gl,
            "kernel",
            &kernel_filename,
            &[
                (
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                (
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR,
                ),
                // Prevents s-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
                // Prevents t-coordinate wrapping (repeating)
                (
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE,
                ),
            ],
        )?;

        // Create the VAO for the screen
        let vertex_array_object_screen = {
            let vertices = [
                -1.0_f32, -1.0_f32, 0.0_f32, 0.0_f32, 1.0_f32, -1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32,
                1.0_f32, 1.0_f32, 1.0_f32, -1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32,
            ];
            let _position = [
                -1.0_f32, -1.0_f32, 1.0_f32, -1.0_f32, 1.0_f32, 1.0_f32, -1.0_f32, 1.0_f32,
            ];
            let _uv = [
                0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32,
            ];

            let indices = [0_u16, 1, 2, 0, 2, 3];

            let mut vao = VertexArrayObject::new(gl);
            //let shader = Colormap::get_catalog_shader(gl, shaders)?;
            #[cfg(feature = "webgl2")]
            vao.bind_for_update()
                // Store the screen and uv of the billboard in a VBO
                .add_array_buffer(
                    "vertices",
                    4 * std::mem::size_of::<f32>(),
                    &[2, 2],
                    &[0, 2 * std::mem::size_of::<f32>()],
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(&vertices),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(indices.as_ref()),
                )
                // Unbind the buffer
                .unbind();
            #[cfg(feature = "webgl1")]
            vao.bind_for_update()
                // Store the screen and uv of the billboard in a VBO
                .add_array_buffer(
                    2,
                    "position",
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(position.as_ref()),
                )
                .add_array_buffer(
                    2,
                    "uv",
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(uv.as_ref()),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(indices.as_ref()),
                )
                // Unbind the buffer
                .unbind();
            vao
        };

        let catalogs = HashMap::new();
        let kernel_size = Vector2::new(0.0, 0.0);

        let fbo = FrameBufferObject::new(gl, 768, 768).unwrap();

        let gl = gl.clone();
        let mut manager = Manager {
            gl,
            kernel_texture,

            fbo,

            vertex_array_object_screen,

            catalogs,
            kernel_size,
        };

        manager.set_kernel_size(camera);

        Ok(manager)
    }

    // Private method adding a catalog into the manager
    pub fn add_catalog<P: Projection>(
        &mut self,
        name: String,
        sources: Box<[Source]>,
        colormap: Colormap,
        _shaders: &mut ShaderManager,
        _camera: &CameraViewPort,
        _view: &HEALPixCellsInView,
    ) {
        // Create the HashMap storing the source indices with respect to the
        // HEALPix cell at depth 7 in which they are contained
        let catalog = Catalog::new::<P>(&self.gl, colormap, sources);

        // Update the number of sources loaded
        //self.num_sources += num_instances_in_catalog as usize;
        self.catalogs.insert(name, catalog);

        // At this point, all the sources memory will be deallocated here
        // These sources have been copied to the GPU so we do not need them
        // in the CPU side

        // We also keep a hash map of all the sources indices located in HEALPix cell
        // at depth 7
    }

    pub fn set_kernel_size(&mut self, camera: &CameraViewPort) {
        let size = camera.get_screen_size();
        self.kernel_size = Vector2::new(32.0 / size.x, 32.0 / size.y);
    }

    pub fn get_mut_catalog(&mut self, name: &str) -> Result<&mut Catalog, Error> {
        self.catalogs.get_mut(name).ok_or(Error::CatalogNotPresent {
            message: format!("{} catalog is not present!", name),
        })
    }

    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort, view: &HEALPixCellsInView) {
        // Render only the sources in the current field of view
        // Cells that are of depth > 7 are not handled by the hashmap (limited to depth 7)
        // For these cells, we draw all the sources lying in the ancestor cell of depth 7 containing
        // this cell
        //if camera.get_aperture() > P::RASTER_THRESHOLD_ANGLE {
        if camera.get_field_of_view().is_allsky() {
            let cells = crate::healpix::cell::ALLSKY_HPX_CELLS_D0;

            for catalog in self.catalogs.values_mut() {
                catalog.update::<P>(cells, camera);
            }
        } else {
            let cells = Vec::from_iter(
                view.get_cells()
                    .map(|&cell| {
                        let d = cell.depth();
                        if d > 7 {
                            cell.ancestor(d - 7)
                        } else {
                            cell
                        }
                    })
                    // This will delete the doublons if there is
                    .collect::<HashSet<_>>(),
            );

            for catalog in self.catalogs.values_mut() {
                catalog.update::<P>(&cells, camera);
            }
        }
    }

    pub fn draw<P: Projection>(
        &self,
        gl: &WebGlContext,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        colormaps: &Colormaps,
        fbo: Option<&FrameBufferObject>,
    ) -> Result<(), JsValue> {
        gl.enable(WebGl2RenderingContext::BLEND);
        for catalog in self.catalogs.values() {
            catalog.draw::<P>(gl, shaders, self, camera, colormaps, fbo)?;
        }
        gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}

use super::catalog::SourceIndices;

pub struct Catalog {
    colormap: Colormap,
    num_instances: i32,
    indices: SourceIndices,
    alpha: f32,
    strength: f32,
    current_sources: Vec<f32>,
    sources: Box<[f32]>,
    vertex_array_object_catalog: VertexArrayObject,
}
use crate::healpix::cell::HEALPixCell;
use crate::{camera::CameraViewPort, math::projection::Projection, utils};
use al_core::SliceData;
use cgmath::Vector2;
use std::collections::HashSet;
const MAX_SOURCES_PER_CATALOG: f32 = 50000.0;

use crate::colormap::Colormaps;
use crate::survey::view::HEALPixCellsInView;
impl Catalog {
    fn new<P: Projection>(
        gl: &WebGlContext,
        colormap: Colormap,
        sources: Box<[Source]>,
    ) -> Catalog {
        let alpha = 1_f32;
        let strength = 1_f32;
        let indices = SourceIndices::new(&sources);
        let num_instances = sources.len() as i32;

        let sources = unsafe { utils::transmute_boxed_slice(sources) };

        let vertex_array_object_catalog = {
            #[cfg(feature = "webgl2")]
            let vertices = [
                -0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32, 0.5_f32, -0.5_f32, 1.0_f32, 0.0_f32, 0.5_f32,
                0.5_f32, 1.0_f32, 1.0_f32, -0.5_f32, 0.5_f32, 0.0_f32, 1.0_f32,
            ];
            #[cfg(feature = "webgl1")]
            let offset = [
                -0.5_f32, -0.5_f32, 0.5_f32, -0.5_f32, 0.5_f32, 0.5_f32, -0.5_f32, 0.5_f32,
            ];
            #[cfg(feature = "webgl1")]
            let uv = [
                0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32,
            ];

            let indices = [0_u16, 1, 2, 0, 2, 3];

            let mut vao = VertexArrayObject::new(gl);

            //let shader = Orthographic::get_catalog_shader(gl, shaders);
            #[cfg(feature = "webgl2")]
            vao.bind_for_update()
                // Store the UV and the offsets of the billboard in a VBO
                .add_array_buffer(
                    "vertices",
                    4 * std::mem::size_of::<f32>(),
                    &[2, 2],
                    &[0, 2 * std::mem::size_of::<f32>()],
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(vertices.as_ref()),
                )
                // Store the cartesian position of the center of the source in the a instanced VBO
                .add_instanced_array_buffer(
                    "center",
                    3 * std::mem::size_of::<f32>(),
                    &[3],
                    &[0],
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    SliceData(sources.as_ref()),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(indices.as_ref()),
                )
                // Unbind the buffer
                .unbind();
            #[cfg(feature = "webgl1")]
            vao.bind_for_update()
                .add_instanced_array_buffer(
                    3,
                    "center",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    SliceData(sources.as_ref()),
                )
                // Store the UV and the offsets of the billboard in a VBO
                .add_array_buffer(
                    2,
                    "offset",
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(offset.as_ref()),
                )
                .add_array_buffer(
                    2,
                    "uv",
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(uv.as_ref()),
                )
                // Store the cartesian position of the center of the source in the a instanced VBO
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::STATIC_DRAW,
                    SliceData(indices.as_ref()),
                )
                // Unbind the buffer
                .unbind();

            vao
        };
        let current_sources = vec![];
        Self {
            alpha,
            strength,
            colormap,
            num_instances,
            indices,
            current_sources,
            sources,

            vertex_array_object_catalog,
        }
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn set_colormap(&mut self, colormap: Colormap) {
        self.colormap = colormap;
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    fn get_total_num_sources_in_fov(&self, cells: &[HEALPixCell]) -> usize {
        let mut total_sources = 0;

        for cell in cells {
            let sources_idx = self.indices.get_source_indices(cell);
            total_sources += (sources_idx.end - sources_idx.start) as usize;
        }

        total_sources
    }

    // Cells are of depth <= 7
    fn update<P: Projection>(&mut self, cells: &[HEALPixCell], _camera: &CameraViewPort) {
        let num_sources_in_fov = self.get_total_num_sources_in_fov(cells) as f32;
        // reset the sources in the frame
        self.current_sources.clear();
        // depth < 7
        for cell in cells {
            let delta_depth = (7_i8 - cell.depth() as i8).max(0);

            for c in cell.get_children_cells(delta_depth as u8) {
                // Define the total number of sources being in this kernel depth tile
                let sources_in_cell = self.indices.get_source_indices(&c);
                let num_sources_in_kernel_cell =
                    (sources_in_cell.end - sources_in_cell.start) as usize;
                if num_sources_in_kernel_cell > 0 {
                    let num_sources = ((num_sources_in_kernel_cell as f32) / num_sources_in_fov)
                        * MAX_SOURCES_PER_CATALOG;

                    let sources =
                        self.indices
                            .get_k_sources(&self.sources, &c, num_sources as usize, 0);
                    self.current_sources.extend(sources);
                }
            }
        }
        //self.current_sources.shrink_to_fit();

        // Update the vertex buffer
        self.num_instances = (self.current_sources.len() / Source::num_f32()) as i32;
        #[cfg(feature = "webgl1")]
        self.vertex_array_object_catalog
            .bind_for_update()
            .update_instanced_array("center", VecData(&self.current_sources));

        #[cfg(feature = "webgl2")]
        self.vertex_array_object_catalog
            .bind_for_update()
            .update_instanced_array("center", VecData(&self.current_sources));
    }

    fn draw<P: Projection>(
        &self,
        gl: &WebGlContext,
        shaders: &mut ShaderManager,
        manager: &Manager, // catalog manager
        camera: &CameraViewPort,
        colormaps: &Colormaps,
        fbo: Option<&FrameBufferObject>,
    ) -> Result<(), JsValue> {
        // If the catalog is transparent, simply discard the draw
        if self.alpha > 0_f32 {
            // Render to the FRAMEBUFFER
            // Render the scene
            manager.fbo.draw_onto(
                || {
                    gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                    let shader = P::get_catalog_shader(gl, shaders);
                    let shader_bound = shader.bind(gl);

                    shader_bound
                        .attach_uniforms_from(camera)
                        // Attach catalog specialized uniforms
                        .attach_uniform("kernel_texture", &manager.kernel_texture) // Gaussian kernel texture
                        .attach_uniform("strength", &self.strength) // Strengh of the kernel
                        .attach_uniform("current_time", &utils::get_current_time())
                        .attach_uniform("kernel_size", &manager.kernel_size)
                        .bind_vertex_array_object_ref(&self.vertex_array_object_catalog)
                        .draw_elements_instanced_with_i32(
                            WebGl2RenderingContext::TRIANGLES,
                            0,
                            self.num_instances as i32,
                        );
                    Ok(())
                },
                fbo,
            )?;

            // Render to the heatmap to the screen
            {
                // Set the camera
                let size = camera.get_screen_size();
                gl.viewport(0, 0, size.x as i32, size.y as i32);

                let shader = shaders.get(
                    gl,
                    &ShaderId(
                        Cow::Borrowed("ColormapCatalogVS"),
                        Cow::Borrowed("ColormapCatalogFS"),
                    ),
                )?;
                //self.colormap.get_shader(gl, shaders);
                let shaderbound = shader.bind(gl);
                shaderbound
                    .attach_uniform("texture_fbo", &manager.fbo.texture) // FBO density texture computed just above
                    .attach_uniform("alpha", &self.alpha) // Alpha channel
                    .attach_uniforms_from(&self.colormap)
                    .attach_uniforms_from(colormaps)
                    .attach_uniform("reversed", &0.0)
                    .bind_vertex_array_object_ref(&manager.vertex_array_object_screen)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        None,
                        WebGl2RenderingContext::UNSIGNED_SHORT,
                        0,
                    );
            }
        }

        Ok(())
    }
}
/*pub fn get_catalog_shader<'a>(
    gl: &WebGlContext,
    shaders: &'a mut ShaderManager,
) -> Result<&'a Shader, JsValue> {
    shaders
        .get(
            gl,
            &ShaderId(
                Cow::Borrowed("ColormapCatalogVS"),
                Cow::Borrowed("ColormapCatalogFS"),
            ),
        )
        .map_err(|e| e.into())
}*/

pub trait CatalogShaderProjection {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader;
}

use crate::math::projection::Aitoff;
use crate::shader::ShaderId;
use std::borrow::Cow;

impl CatalogShaderProjection for Aitoff {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("CatalogAitoffVS"), Cow::Borrowed("CatalogFS")),
            )
            .unwrap()
    }
}
use crate::math::projection::Mollweide;

impl CatalogShaderProjection for Mollweide {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("CatalogMollVS"), Cow::Borrowed("CatalogFS")),
            )
            .unwrap()
    }
}
use crate::math::projection::AzimuthalEquidistant;

impl CatalogShaderProjection for AzimuthalEquidistant {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("CatalogArcVS"), Cow::Borrowed("CatalogFS")),
            )
            .unwrap()
    }
}

use crate::math::projection::HEALPix;
impl CatalogShaderProjection for HEALPix {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("CatalogHEALPixVS"),
                    Cow::Borrowed("CatalogFS"),
                ),
            )
            .unwrap()
    }
}

use crate::math::projection::Mercator;

impl CatalogShaderProjection for Mercator {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("CatalogMercatVS"), Cow::Borrowed("CatalogFS")),
            )
            .unwrap()
    }
}
use crate::math::projection::Orthographic;
impl CatalogShaderProjection for Orthographic {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(
                    Cow::Borrowed("CatalogOrthoVS"),
                    Cow::Borrowed("CatalogOrthoFS"),
                ),
            )
            .unwrap()
    }
}
use crate::math::projection::Gnomonic;
impl CatalogShaderProjection for Gnomonic {
    fn get_catalog_shader<'a>(gl: &WebGlContext, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders
            .get(
                gl,
                &ShaderId(Cow::Borrowed("CatalogTanVS"), Cow::Borrowed("CatalogFS")),
            )
            .unwrap()
    }
}
