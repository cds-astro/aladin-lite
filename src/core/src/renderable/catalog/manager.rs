use crate::ShaderManager;

use al_api::coo_system::CooSystem;
use al_api::resources::Resources;

use al_core::colormap::Colormap;
use al_core::Colormaps;
use al_core::FrameBufferObject;
use al_core::{VecData, VertexArrayObject, WebGlContext};

use crate::ProjectionType;
use std::collections::HashMap;

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

// Num of shapes
const _NUM_SHAPES: usize = 5;
pub struct Manager {
    gl: WebGlContext,
    //kernels: HashMap<&'static str, Texture2D>,
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
        let _kernel_filename = resources.get_filename("kernel").unwrap_abort();
        let _params = &[
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
        ];
        /*let kernels = [
            (
                "gaussian",
                Texture2D::create_from_path::<_, RGBA8U>(gl, "kernel", &kernel_filename, params)?,
            ),
            (
                "plus",
                Texture2D::create_from_raw_pixels::<R8UI>(
                    gl,
                    3,
                    3,
                    params,
                    Some(&[0, 0xff, 0, 0xff, 0xff, 0xff, 0, 0xff, 0]),
                )?,
            ),
            (
                "square",
                Texture2D::create_from_raw_pixels::<R8UI>(
                    gl,
                    3,
                    3,
                    params,
                    Some(&[0xff, 0xff, 0xff, 0xff, 0, 0xff, 0xff, 0xff, 0xff]),
                )?,
            ),
        ]
        .into();*/
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

        let fbo = FrameBufferObject::new(gl, 768, 768).unwrap_abort();

        let gl = gl.clone();
        let mut manager = Manager {
            gl,
            //kernels,
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
        sources: Box<[LonLatT<f32>]>,
        colormap: Colormap,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) {
        // Create the HashMap storing the source indices with respect to the
        // HEALPix cell at depth 7 in which they are contained
        let catalog = Catalog::new::<P>(&self.gl, colormap, sources);

        // Update the number of sources loaded
        //self.num_sources += num_instances_in_catalog as usize;
        self.catalogs.insert(name, catalog);
        camera.register_view_frame(CooSystem::ICRS, proj);

        // At this point, all the sources memory will be deallocated here
        // These sources have been copied to the GPU so we do not need them
        // in the CPU side

        // We also keep a hash map of all the sources indices located in HEALPix cell
        // at depth 7
    }

    pub fn remove_catalog<P: Projection>(
        &mut self,
        name: String,
        camera: &mut CameraViewPort,
        proj: &ProjectionType,
    ) {
        // Update the number of sources loaded
        //self.num_sources += num_instances_in_catalog as usize;
        self.catalogs.remove(&name);
        camera.unregister_view_frame(CooSystem::ICRS, proj);
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

    pub fn update(&mut self, camera: &mut CameraViewPort) {
        // Render only the sources in the current field of view
        // Cells that are of depth > 7 are not handled by the hashmap (limited to depth 7)
        // For these cells, we draw all the sources lying in the ancestor cell of depth 7 containing
        // this cell
        //if camera.get_aperture() > P::RASTER_THRESHOLD_ANGLE {
        if camera.get_field_of_view().is_allsky() {
            let cells = crate::healpix::cell::ALLSKY_HPX_CELLS_D0;

            for catalog in self.catalogs.values_mut() {
                catalog.update(cells);
            }
        } else {
            let depth = camera.get_texture_depth().min(7);
            let cells = camera.get_hpx_cells(depth, CooSystem::ICRS);

            for catalog in self.catalogs.values_mut() {
                catalog.update(&cells);
            }
        }
    }

    pub fn draw(
        &self,
        gl: &WebGlContext,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        colormaps: &Colormaps,
        fbo: Option<&FrameBufferObject>,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        for catalog in self.catalogs.values() {
            catalog.draw(gl, shaders, self, camera, colormaps, fbo, projection)?;
        }

        Ok(())
    }
}

use crate::healpix::index_vector::IdxVec;
use crate::LonLatT;

pub struct Catalog {
    colormap: Colormap,
    num_instances: i32,
    index_vec: IdxVec,
    alpha: f32,
    strength: f32,
    lonlat: Box<[LonLatT<f32>]>,
    vertex_array_object_catalog: VertexArrayObject,
}
use crate::healpix::cell::HEALPixCell;
use crate::{camera::CameraViewPort, math::projection::Projection, utils};
use al_core::SliceData;
use cgmath::Vector2;

const MAX_SOURCES_PER_CATALOG: f32 = 50000.0;

use crate::Abort;
impl Catalog {
    fn new<P: Projection>(
        gl: &WebGlContext,
        colormap: Colormap,
        mut lonlat: Box<[LonLatT<f32>]>,
    ) -> Catalog {
        let alpha = 1_f32;
        let strength = 1_f32;
        let index_vec = IdxVec::from_coo(&mut lonlat);
        let num_instances = lonlat.len() as i32;

        //let sources = unsafe { utils::transmute_boxed_slice(sources) };

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
                    SliceData(&[]),
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
                    SliceData(&[]),
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
        Self {
            alpha,
            strength,
            colormap,
            num_instances,
            index_vec,
            lonlat,

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
            let sources_idx = self.index_vec.get_item_indices_inside_hpx_cell(cell);
            total_sources += (sources_idx.end - sources_idx.start) as usize;
        }

        total_sources
    }

    // Cells are of depth <= 7
    fn update(&mut self, cells: &[HEALPixCell]) {
        let num_sources_in_fov = self.get_total_num_sources_in_fov(cells) as f32;
        // reset the sources in the frame
        let mut sources: Vec<_> = vec![];
        // depth < 7
        for cell in cells {
            let delta_depth = (7_i8 - cell.depth() as i8).max(0);

            for c in cell.get_children_cells(delta_depth as u8) {
                // Define the total number of sources being in this kernel depth tile
                let sources_in_cell = self.index_vec.get_item_indices_inside_hpx_cell(&c);
                let num_sources_in_kernel_cell =
                    (sources_in_cell.end - sources_in_cell.start) as usize;
                if num_sources_in_kernel_cell > 0 {
                    let num_sources = (((num_sources_in_kernel_cell as f32) / num_sources_in_fov)
                        * MAX_SOURCES_PER_CATALOG) as usize;

                    let mut idx = self.index_vec.get_item_indices_inside_hpx_cell(&c);
                    if num_sources < idx.end - idx.start {
                        // use a selection of num_sources items
                        idx = idx.start..(idx.start + num_sources);
                    }

                    sources.extend(&self.lonlat[idx]);
                }
            }
        }
        self.num_instances = sources.len() as i32;
        let sources = unsafe { utils::transmute_vec::<LonLatT<f32>, f32>(sources).unwrap() };

        // Update the vertex buffer
        #[cfg(feature = "webgl1")]
        self.vertex_array_object_catalog
            .bind_for_update()
            .update_instanced_array("center", VecData(&sources));

        #[cfg(feature = "webgl2")]
        self.vertex_array_object_catalog
            .bind_for_update()
            .update_instanced_array(
                "center",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&sources),
            );
    }

    fn draw(
        &self,
        gl: &WebGlContext,
        shaders: &mut ShaderManager,
        manager: &Manager, // catalog manager
        camera: &CameraViewPort,
        colormaps: &Colormaps,
        fbo: Option<&FrameBufferObject>,
        projection: &ProjectionType,
    ) -> Result<(), JsValue> {
        // If the catalog is transparent, simply discard the draw
        if self.alpha > 0_f32 {
            // Render to the FRAMEBUFFER
            // Render the scene
            manager.fbo.draw_onto(
                || {
                    gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

                    let shader = match projection {
                        ProjectionType::Sin(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogOrtVS", "CatalogOrtFS")
                        }
                        ProjectionType::Ait(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogAitVS", "CatalogFS")
                        }
                        ProjectionType::Mer(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogMerVS", "CatalogFS")
                        }
                        ProjectionType::Mol(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogMolVS", "CatalogFS")
                        }
                        /*ProjectionType::Arc(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogArcVS", "CatalogFS")
                        }*/
                        ProjectionType::Tan(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogTanVS", "CatalogFS")
                        }
                        /*ProjectionType::Hpx(_) => {
                            crate::shader::get_shader(gl, shaders, "CatalogHpxVS", "CatalogFS")
                        }*/
                        _ => todo!(),
                    }?;
                    let shader_bound = shader.bind(gl);

                    shader_bound
                        .attach_uniforms_from(camera)
                        // Attach catalog specialized uniforms
                        //.attach_uniform("kernel_texture", &manager.kernels["gaussian"]) // Gaussian kernel texture
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

                let shader = crate::shader::get_shader(
                    gl,
                    shaders,
                    "ColormapCatalogVS",
                    "ColormapCatalogFS",
                )?;
                //self.colormap.get_shader(gl, shaders);
                let shaderbound = shader.bind(gl);
                shaderbound
                    .attach_uniform("texture_fbo", &manager.fbo.texture) // FBO density texture computed just above
                    .attach_uniform("alpha", &self.alpha) // Alpha channel
                    .attach_uniforms_with_params_from(&self.colormap, colormaps)
                    .attach_uniforms_from(colormaps)
                    .attach_uniform("reversed", &0.0_f32)
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
