/// This module handles the lines rendering code
pub mod great_circle_arc;
pub mod parallel_arc;

use crate::math::projection::ProjectionType;
use crate::shader::ShaderManager;
use al_api::coo_system::CooSystem;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use super::Renderer;
use al_api::color::ColorRGBA;
use al_core::SliceData;

struct Meta {
    color: ColorRGBA,
    thickness: f32,
    off_indices: usize,
    num_indices: usize,
    coo_space: CooSpace,
}

#[derive(Clone)]
pub enum Style {
    None,
    Dashed,
    Dotted,
}

pub struct RasterizedLineRenderer {
    gl: WebGlContext,
    //vao: VertexArrayObject,

    //vao_idx: usize,

    instanced_line_vaos: HashMap<&'static str, VertexArrayObject>,
    meta_instanced: HashMap<&'static str, Meta>,
}
use wasm_bindgen::JsValue;

use al_core::VecData;
use web_sys::WebGl2RenderingContext;

use crate::camera::CameraViewPort;

use crate::coo_space::CooSpace;
use std::collections::HashMap;

#[repr(C)]
pub struct PathVertices<V>
where
    V: AsRef<[[f32; 2]]>,
{
    pub vertices: V,
}

impl RasterizedLineRenderer {
    /// Init the buffers, VAO and shader
    pub fn new(gl: &WebGlContext) -> Result<Self, JsValue> {
        // Create the VAO for the screen
        /*let mut vao = VertexArrayObject::new(gl);

        vao.bind_for_update()
            .add_array_buffer(
                "ndc_pos",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                SliceData::<f32>(&[]),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                SliceData::<u32>(&[]),
            )
            .unbind();
        */
        let meta_instanced = HashMap::new();
        let gl = gl.clone();

        let instanced_line_vaos = HashMap::new();
        Ok(Self {
            gl,
            //vao_idx: 0,
            instanced_line_vaos,
            meta_instanced,
            //vao,
        })
    }

    /*pub fn add_fill_paths<V>(
        &mut self,
        paths: impl Iterator<Item = PathVertices<V>>,
        color: &ColorRGBA,
        coo_space: CooSpace,
    ) where
        V: AsRef<[[f32; 2]]>,
    {
        let mut num_indices = 0;
        let off_indices = self.indices.len();
        let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();

        let mut tessellator = FillTessellator::new();

        //let mut num_vertices = 0;
        for path in paths {
            let mut path_builder = Path::builder();

            let PathVertices {
                vertices, /*, closed */
            } = path;

            let line = vertices.as_ref();

            if !line.is_empty() {
                let v = &line[0];
                path_builder.begin(point(v[0], v[1]));

                for v in line.iter().skip(1) {
                    //let v = clamp_ndc_vertex(v);
                    path_builder.line_to(point(v[0], v[1]));
                }

                path_builder.end(false);
            }

            // Create the destination vertex and index buffers.
            let p = path_builder.build();
            // Let's use our own custom vertex type instead of the default one.
            // Will contain the result of the tessellation.
            let num_vertices = (self.vertices.len() / 2) as u32;

            // Compute the tessellation.
            tessellator
                .tessellate_with_ids(
                    p.id_iter(),
                    &p,
                    Some(&p),
                    &FillOptions::default()
                        .with_intersections(false)
                        .with_fill_rule(FillRule::NonZero)
                        .with_tolerance(5e-3),
                    &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
                        vertex.position().to_array()
                    })
                    .with_vertex_offset(num_vertices),
                )
                .unwrap_abort();
        }
        let VertexBuffers { vertices, indices } = geometry;
        num_indices += indices.len();

        self.vertices.extend(vertices.iter().flatten());
        self.indices.extend(indices.iter());

        //al_core::info!("num vertices fill", nv);

        self.meta.push(Meta {
            off_indices,
            num_indices,
            thickness: 1.0,
            color: color.clone(),
            coo_space,
        });
    }*/

    fn create_instanced_vao(&mut self, label: &'static str) {
        if self.instanced_line_vaos.contains_key(label) {
            return;
        }

        let mut vao = VertexArrayObject::new(&self.gl);        

        vao.bind_for_update()
            // Store the cartesian position of the center of the source in the a instanced VBO
            .add_instanced_array_buffer(
                "ndc_pos",
                4 * std::mem::size_of::<f32>(),
                &[2, 2],
                &[0, 2 * std::mem::size_of::<f32>()],
                WebGl2RenderingContext::DYNAMIC_DRAW,
                &[] as &[f32],
            )
            .add_array_buffer(
                "vertices",
                2 * std::mem::size_of::<f32>(),
                &[2],
                &[0],
                WebGl2RenderingContext::STATIC_DRAW,
                &[
                    0_f32, -0.5_f32, 1_f32, -0.5_f32, 1_f32, 0.5_f32, 0_f32, 0.5_f32,
                ] as &[f32],
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::STATIC_DRAW,
                &[0_u16, 1_u16, 2_u16, 0_u16, 2_u16, 3_u16] as &[u16],
            )
            // Unbind the buffer
            .unbind();

        self.instanced_line_vaos.insert(label, vao);
    }

    pub fn add_stroke_paths<V>(
        &mut self,
        paths: impl Iterator<Item = PathVertices<V>>,
        thickness: f32,
        color: &ColorRGBA,
        _style: &Style,
        coo_space: CooSpace,
        label: &'static str,
    ) where
        V: AsRef<[[f32; 2]]>,
    {
        self.create_instanced_vao(label);

        let vao = self.instanced_line_vaos.get_mut(label).unwrap();

        let mut buf: Vec<f32> = vec![];

        for PathVertices { vertices } in paths {
            let vertices = vertices.as_ref();
            let path_vertices_buf_iter = vertices
                .iter()
                .zip(vertices.iter().skip(1))
                .flat_map(|(a, b)| [a[0], a[1], b[0], b[1]]);

            buf.extend(path_vertices_buf_iter);
        }

        vao.bind_for_update().update_instanced_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&buf),
        );

        let num_instances = buf.len() / 4;

        self.meta_instanced.insert(label, Meta {
            off_indices: 0,
            thickness,
            num_indices: num_instances,
            color: *color,
            coo_space,
        });
    }

    pub fn draw(
        &self,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        proj: &ProjectionType,
    ) -> Result<(), JsValue> {
        //self.gl.enable(WebGl2RenderingContext::BLEND);
        /*self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );*/

        //self.gl.disable(WebGl2RenderingContext::CULL_FACE);
        /*{
            let shader =
                crate::shader::get_shader(&self.gl, shaders, "line_base.vert", "line_base.frag")?
                    .bind(&self.gl);
            for meta in self.meta.iter() {
                shader
                    .attach_uniform("u_color", &meta.color) // Strengh of the kernel
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(meta.num_indices as i32),
                        WebGl2RenderingContext::UNSIGNED_INT,
                        (meta.off_indices * std::mem::size_of::<u32>()) as i32,
                    );
            }
        }*/
        //self.gl.enable(WebGl2RenderingContext::CULL_FACE);

        // draw the instanced lines
        for (label, meta) in self.meta_instanced.iter() {
            let vao = self.instanced_line_vaos.get(label).unwrap();

            match meta.coo_space {
                CooSpace::NDC => {
                    crate::shader::get_shader(
                        &self.gl,
                        shaders,
                        "line_inst_ndc.vert",
                        "line_base.frag",
                    )?
                    .bind(&self.gl)
                    .attach_uniform("u_color", &meta.color)
                    .attach_uniform("u_width", &(camera.get_width()))
                    .attach_uniform("u_height", &(camera.get_height()))
                    .attach_uniform("u_thickness", &meta.thickness)
                    .bind_vertex_array_object_ref(vao)
                    .draw_elements_instanced_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        0,
                        meta.num_indices as i32,
                    );
                }
                CooSpace::LonLat => {
                    let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
                    let view2world = camera.get_m2w();
                    let icrs2world = view2world * icrs2view;

                    crate::shader::get_shader(
                        &self.gl,
                        shaders,
                        "line_inst_lonlat.vert",
                        "line_base.frag",
                    )?
                    .bind(&self.gl)
                    .attach_uniforms_from(camera)
                    .attach_uniform("u_2world", &icrs2world)
                    .attach_uniform("u_color", &meta.color)
                    .attach_uniform("u_width", &(camera.get_width()))
                    .attach_uniform("u_height", &(camera.get_height()))
                    .attach_uniform("u_thickness", &meta.thickness)
                    .attach_uniform("u_proj", proj)
                    .bind_vertex_array_object_ref(vao)
                    .draw_elements_instanced_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        0,
                        meta.num_indices as i32,
                    );
                }
                _ => (),
            }
        }
        //self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
