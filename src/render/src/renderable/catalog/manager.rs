use web_sys::{
    WebGlFramebuffer,
    WebGl2RenderingContext
};
use crate::{
    WebGl2Context,
    ShaderManager,
    Colormap,
    core::{
        Texture2D,
        VertexArrayObject,
        VecData,
    },
    shader::Shader,
};
use super::source::Source;
use std::collections::HashMap;
use crate::renderable::projection::*;

#[derive(Debug)]
pub enum Error {
    CatalogNotPresent { message: String }
}

pub struct Manager {
    gl: WebGl2Context,
    kernel_texture: Texture2D,

    fbo: Option<WebGlFramebuffer>,
    fbo_texture: Texture2D,

    // VAOs
    vertex_array_object_screen: VertexArrayObject,

    catalogs: HashMap<String, Catalog>,
    num_sources: usize,
    kernel_size: Vector2<f32>,
}

use crate::FormatImageType;
use crate::image_fmt::PNG;
use crate::Resources;
impl Manager {
    pub fn new(gl: &WebGl2Context, shaders: &mut ShaderManager, camera: &CameraViewPort, resources: &Resources) -> Self {
        // Load the texture of the gaussian kernel
        let kernel_filename = resources.get_filename("kernel").unwrap();
        let kernel_texture = Texture2D::create(gl, "kernel", &kernel_filename, &[
                (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR),
                (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR),
                
                // Prevents s-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE),
                // Prevents t-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE),
            ],
            FormatImageType::PNG
        );
        //let _ext = gl.get_extension("EXT_color_buffer_float");
        // Initialize texture for framebuffer
        let fbo_texture = Texture2D::create_empty(
            gl,
            768,
            768,
            &[
                (WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR),
                (WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR),
                
                // Prevents s-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE),
                // Prevents t-coordinate wrapping (repeating)
                (WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE),
            ],
            FormatImageType::PNG,
        );
        // Create and bind the framebuffer
        let fbo = gl.create_framebuffer();
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, fbo.as_ref());
        // attach the texture as the first color attachment
        fbo_texture.attach_to_framebuffer();
        // Unbind the framebuffer
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        // Create the VAO for the screen
        let vertex_array_object_screen = {
            let vertices = vec![
                -1.0_f32, -1.0_f32, 0.0_f32, 0.0_f32,
                1.0_f32, -1.0_f32, 1.0_f32, 0.0_f32,
                1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32,
                -1.0_f32, 1.0_f32, 0.0_f32, 1.0_f32,
            ];

            let indices: Vec<u16> = vec![
                0, 1, 2,
                0, 2, 3,
            ];

            let mut vao = VertexArrayObject::new(gl);
            let colormap = Colormap::BluePastelRed;
            let shader = colormap.get_shader(gl, shaders);
            shader.bind(gl)
                .bind_vertex_array_object(&mut vao)
                    // Store the screen and uv of the billboard in a VBO
                    .add_array_buffer(
                        4 * std::mem::size_of::<f32>(),
                        &[2, 2],
                        &[0, 2 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(&vertices),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(indices.as_ref()),
                    )
                    // Unbind the buffer
                    .unbind();
            vao
        };

        let catalogs = HashMap::new();
        let num_sources = 0;
        let kernel_size = Vector2::new(0.0, 0.0);

        let gl = gl.clone();
        let mut manager = Manager {
            gl,
            kernel_texture,

            fbo,
            fbo_texture,

            vertex_array_object_screen,

            catalogs,
            num_sources,
            kernel_size
        };

        manager.set_kernel_size(camera);

        manager
    }

    // Private method adding a catalog into the manager
    pub fn add_catalog<P: Projection>(&mut self, name: String, sources: Vec<Source>, colormap: Colormap, shaders: &mut ShaderManager, camera: &CameraViewPort, view: &HEALPixCellsInView) {
        // Create the HashMap storing the source indices with respect to the
        // HEALPix cell at depth 7 in which they are contained
        let catalog = Catalog::new::<P>(
            &self.gl,
            shaders, 
            colormap,
            sources,
            view,
            camera,
        );

        // Update the number of sources loaded
        //self.num_sources += num_instances_in_catalog as usize;

        // Append the new sources to the existing instanced vbo
        /*let sources: Vec<f32> = unsafe { flatten_vec(sources) };
        self.vertex_array_object_catalog.bind_for_update()
            .append_to_instanced_array(0, VecData(&sources));
        */
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
        self.catalogs.get_mut(name)
            .ok_or(Error::CatalogNotPresent {
                message: format!("{} catalog is not present!", name)
            })
    }
    pub fn get_catalog(&self, name: &str) -> Result<&Catalog, Error> {
        self.catalogs.get(name)
            .ok_or(Error::CatalogNotPresent {
                message: format!("{} catalog is not present!", name)
            })
    }

    pub fn update<P: Projection>(&mut self, camera: &CameraViewPort, view: &HEALPixCellsInView) {
        // Render only the sources in the current field of view
        // Cells that are of depth > 7 are not handled by the hashmap (limited to depth 7)
        // For these cells, we draw all the sources lying in the ancestor cell of depth 7 containing
        // this cell
        let HEALPixCells { mut depth, cells } = view.get_cells();
        let cells = cells.into_iter()
            .map(|&cell| {
                let d = cell.depth();
                if d > 7 {
                    depth = 7;
                    cell.ancestor(d - 7)
                } else {
                    depth = d;
                    cell
                }
            })
            // This will delete the doublons if there is
            .collect::<HashSet<_>>();

        let cells = HEALPixCells {
            cells,
            depth
        };

        for catalog in self.catalogs.values_mut() {
            catalog.update::<P>(&cells, camera);
        }
    }

    pub fn draw<P: Projection>(&self, gl: &WebGl2Context, shaders: &mut ShaderManager, camera: &CameraViewPort) {
        for catalog in self.catalogs.values() {
            catalog.draw::<P>(&gl, shaders, self, camera);
        }
    }
}

use super::catalog::SourceIndices;
pub struct Catalog {
    colormap: Colormap,
    num_instances: i32,
    indices: SourceIndices,
    alpha: f32,
    strength: f32,
    sources: Vec<f32>,
    vertex_array_object_catalog: VertexArrayObject,
    max_density: f32,
}

use crate::{
    Projection,
    camera::CameraViewPort,
    utils,
};
use cgmath::Vector2;
use std::collections::HashSet;
use crate::healpix_cell::{HEALPixCell, HEALPixTilesIter};
const MAX_SOURCES_PER_CATALOG: f32 = 50000.0;
use crate::HiPSConfig;
use crate::renderable::view_on_surveys::depth_from_pixels_on_screen;
use crate::renderable::{HEALPixCells, HEALPixCellsInView};
impl Catalog {
    fn new<P: Projection>(
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        colormap: Colormap,
        mut sources: Vec<Source>,
        view: &HEALPixCellsInView,
        camera: &CameraViewPort,
    ) -> Catalog {
        let alpha = 1_f32;
        let strength = 1_f32;
        let indices = SourceIndices::new(&mut sources);
        let num_instances = sources.len() as i32;
        let sources = unsafe { utils::flatten_vec(sources) };

        let vertex_array_object_catalog = {
            let vertices = vec![
                -0.5_f32, -0.5_f32, 0.0_f32, 0.0_f32,
                0.5_f32, -0.5_f32, 1.0_f32, 0.0_f32,
                0.5_f32, 0.5_f32, 1.0_f32, 1.0_f32,
                -0.5_f32, 0.5_f32, 0.0_f32, 1.0_f32, 
            ];

            let indices: Vec<u16> = vec![
                0, 1, 2,
                0, 2, 3,
            ];

            let mut vao = VertexArrayObject::new(gl);

            let shader = Orthographic::get_catalog_shader(gl, shaders);
            shader.bind(gl)
                .bind_vertex_array_object(&mut vao)
                    // Store the UV and the offsets of the billboard in a VBO
                    .add_array_buffer(
                        4 * std::mem::size_of::<f32>(),
                        &[2, 2],
                        &[0, 2 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(vertices.as_ref()),
                    )
                    // Store the cartesian position of the center of the source in the a instanced VBO
                    .add_instanced_array_buffer(
                        std::mem::size_of::<Source>(),
                        &[3, 2],
                        &[0, 3 * std::mem::size_of::<f32>()],
                        WebGl2RenderingContext::DYNAMIC_DRAW,
                        VecData(&sources),
                    )
                    // Set the element buffer
                    .add_element_buffer(
                        WebGl2RenderingContext::STATIC_DRAW,
                        VecData(indices.as_ref()),
                    )
                    // Unbind the buffer
                    .unbind();
            vao
        };
        let max_density = 1.0;
        let mut catalog = Self {
            alpha,
            strength,
            colormap,
            num_instances,
            indices,
            sources,
            vertex_array_object_catalog,
            max_density
        };
        catalog.set_max_density::<P>(view, camera);
        catalog
    }

    fn set_max_density<P: Projection>(&mut self, view: &HEALPixCellsInView, camera: &CameraViewPort) {
        let HEALPixCells { depth, cells } = view.get_cells();

        let cells = cells.into_iter()
            .map(|&cell| {
                let d = cell.depth();
                if d > 7 {
                    cell.ancestor(d - 7)
                } else {
                    cell
                }
            })
            // This will delete the doublons if there is
            .collect::<HashSet<_>>();

        let num_sources_in_fov = self.get_total_num_sources_in_fov(&cells) as f32;

        //self.max_density = self.compute_max_density::<P>(camera.depth_precise(config) + 5.0);
        let d_kernel = depth_from_pixels_on_screen(camera, 32);
        self.max_density = self.compute_max_density::<P>(d_kernel);
        if num_sources_in_fov > MAX_SOURCES_PER_CATALOG {
            let d = (MAX_SOURCES_PER_CATALOG / num_sources_in_fov);
            self.max_density *= d*d;
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

    pub fn get_alpha(&self) -> f32 {
        self.alpha
    }

    fn compute_max_density<P: Projection>(&self, d: f32) -> f32 {
        let d0 = d.floor() as usize;
        let d1 = d0 + 1;
        let lambda = d - (d0 as f32);
        let max_density_d0 =  self.indices.max_density(d0) as f32;
        let max_density_d1 =  self.indices.max_density(d1) as f32;
        
        let max_density = (1_f32 - lambda) * max_density_d0 + lambda * max_density_d1;
        max_density
    }

    fn get_total_num_sources_in_fov(&self, cells: &HashSet<HEALPixCell>) -> usize {
        let mut total_sources = 0;

        for cell in cells {
            let sources_idx = self.indices.get_source_indices(&cell);
            total_sources += (sources_idx.end - sources_idx.start) as usize;
        }

        total_sources
    }

    // Cells are of depth <= 7
    fn update<P: Projection>(&mut self, cells: &HEALPixCells, camera: &CameraViewPort) {
        let HEALPixCells {ref depth, ref cells} = cells;
        let mut current_sources = vec![];

        let depth_kernel = (depth + 6).min(7);

        let num_sources_in_fov = self.get_total_num_sources_in_fov(&cells) as f32;
        
        let d_kernel = depth_from_pixels_on_screen(camera, 32);
        self.max_density = self.compute_max_density::<P>(d_kernel);
        //self.max_density = self.compute_max_density::<P>(camera.depth_precise(config) + 5.0);
        if num_sources_in_fov > MAX_SOURCES_PER_CATALOG {
            let d = (MAX_SOURCES_PER_CATALOG / num_sources_in_fov);
            self.max_density *= d*d;
        }
        // depth < 7
        for cell in cells {
            let delta_depth = (depth_kernel as i8 - cell.depth() as i8).max(0);

            for c in cell.get_children_cells(delta_depth as u8) {
                // Define the total number of sources being in this kernel depth tile
                let sources_in_cell = self.indices.get_source_indices(&c);
                let num_sources_in_kernel_cell = (sources_in_cell.end - sources_in_cell.start) as usize;
                if num_sources_in_kernel_cell > 0 {
                    let num_sources = ((num_sources_in_kernel_cell as f32)/num_sources_in_fov)*MAX_SOURCES_PER_CATALOG;

                    let sources = self.indices.get_k_sources(&self.sources, &c, num_sources as usize);
                    current_sources.extend(sources);
                }
            }
        }

        // Update the vertex buffer
        self.num_instances = (current_sources.len() / Source::num_f32()) as i32;
        crate::log(&format!("NUM SOURCES CURRENT: {:?}", self.num_instances));

        self.vertex_array_object_catalog.bind_for_update()
            .update_instanced_array(0, VecData(&current_sources));
    }

    fn draw<P: Projection>(
        &self,
        gl: &WebGl2Context,
        shaders: &mut ShaderManager,
        manager: &Manager, // catalog manager
        camera: &CameraViewPort
    ) {
        // If the catalog is transparent, simply discard the draw
        if self.alpha == 0_f32 {
            return;
        }
        // Render to the FRAMEBUFFER
        {
            // bind the FBO
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, manager.fbo.as_ref());
            let (fbo_width, fbo_height) = manager.fbo_texture.get_size();
            // Set the camera
            gl.viewport(0, 0, fbo_width as i32, fbo_height as i32);
            gl.scissor(0, 0, fbo_width as i32, fbo_height as i32);

            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            //crate::log(&format!("offset: {}, num instances: {}", self.base_instance, self.num_instances));
            let shader = P::get_catalog_shader(gl, shaders);
            let shader_bound = shader.bind(gl);
            let kernel_tex = manager.kernel_texture.bind();
            // Uniforms associated to the camera
            //crate::log(&format!("max density: {:?}", self.max_density));
            shader_bound.attach_uniforms_from(camera)
                // Attach catalog specialized uniforms
                .attach_uniform("kernel_texture", &kernel_tex) // Gaussian kernel texture
                .attach_uniform("strength", &self.strength) // Strengh of the kernel
                .attach_uniform("model", camera.get_m2w())
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("kernel_size", &manager.kernel_size)
                .attach_uniform("max_density", &self.max_density)
                .bind_vertex_array_object_ref(&self.vertex_array_object_catalog)
                    .draw_elements_instanced_with_i32(WebGl2RenderingContext::TRIANGLES,
                        0,
                        self.num_instances
                    );

            // Unbind the FBO
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        }

        // Render to the heatmap to the screen
        {
            // Set the camera
            let size = camera.get_screen_size();
            gl.viewport(0, 0, size.x as i32, size.y as i32);

            let fbo_tex = manager.fbo_texture.bind();

            let shader = self.colormap.get_shader(gl, shaders);
            shader.bind(gl)
                .attach_uniform("texture_fbo", &fbo_tex) // FBO density texture computed just above
                .attach_uniform("alpha", &self.alpha) // Alpha channel
                .bind_vertex_array_object_ref(&manager.vertex_array_object_screen)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        None,
                        WebGl2RenderingContext::UNSIGNED_SHORT
                    );
        }
    }
}
pub trait CatalogShaderProjection {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader;
}

use std::borrow::Cow;
use crate::shader::ShaderId;
impl CatalogShaderProjection for Aitoff {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogAitoffVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}
impl CatalogShaderProjection for Mollweide {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogMollVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}
impl CatalogShaderProjection for AzimuthalEquidistant {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogOrthoVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}
impl CatalogShaderProjection for Mercator {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogMercatorVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}
impl CatalogShaderProjection for Orthographic {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogOrthoVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}
impl CatalogShaderProjection for Gnomonic {
    fn get_catalog_shader<'a>(gl: &WebGl2Context, shaders: &'a mut ShaderManager) -> &'a Shader {
        shaders.get(
            gl,
            &ShaderId(
                Cow::Borrowed("CatalogOrthoVS"),
                Cow::Borrowed("CatalogFS")
            )
        ).unwrap()
    }
}