/// This module handles the lines rendering code
pub mod great_circle_arc;
pub mod parallel_arc;

use crate::math::projection::ProjectionType;
use crate::shader::ShaderManager;
use al_api::coo_system::CooSystem;
use al_core::VertexArrayObject;
use al_core::WebGlContext;
use crate::math::angle::ToAngle;
use crate::math::lonlat::LonLat;

use al_api::color::ColorRGBA;

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
    V: AsRef<[[f64; 2]]>,
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

    fn create_instanced_vao(&mut self, label: &'static str, coo_space: CooSpace) {
        if self.instanced_line_vaos.contains_key(label) {
            return;
        }

        let mut vao = VertexArrayObject::new(&self.gl);        

        if coo_space == CooSpace::LonLat {
            vao.bind_for_update()
            // Store the cartesian position of the center of the source in the a instanced VBO
            .add_instanced_array_buffer(
                "ndc_pos",
                8 * std::mem::size_of::<f32>(),
                &[2, 2, 2, 2],
                &[0, 2 * std::mem::size_of::<f32>(), 4 * std::mem::size_of::<f32>(), 6 * std::mem::size_of::<f32>()],
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
        } else if coo_space == CooSpace::Model {
            vao.bind_for_update()
                // Store the cartesian position of the center of the source in the a instanced VBO
                .add_instanced_array_buffer(
                    "ndc_pos",
                    6 * std::mem::size_of::<f32>(),
                    &[3, 3],
                    &[0, 3 * std::mem::size_of::<f32>()],
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
        } else if coo_space == CooSpace::NDC {
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
        } else {
            todo!();
        }

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
        V: AsRef<[[f64; 2]]>,
    {
        self.create_instanced_vao(label, coo_space.clone());

        let vao = self.instanced_line_vaos.get_mut(label).unwrap();

        let mut buf: Vec<f32> = vec![];
    
        let num_instances = if coo_space == CooSpace::LonLat {
            for PathVertices { vertices } in paths {
                let vertices = vertices.as_ref();
                let path_vertices_buf_iter = vertices
                    .iter()
                    .zip(vertices.iter().skip(1))
                    .flat_map(|(a, b)| {
                        let hia0 = a[0] as f32;
                        let lwa0 = (a[0] - (hia0 as f64)) as f32;

                        let hib0 = b[0] as f32;
                        let lwb0 = (b[0] - (hib0 as f64)) as f32;

                        let hia1 = a[1] as f32;
                        let lwa1 = (a[1] - (hia1 as f64)) as f32;

                        let hib1 = b[1] as f32;
                        let lwb1 = (b[1] - (hib1 as f64)) as f32;
                        [hia0, hia1, lwa0, lwa1, hib0, hib1, lwb0, lwb1]
                    });

                buf.extend(path_vertices_buf_iter);
            }

            buf.len() / 8
        } else if coo_space == CooSpace::NDC {
            for PathVertices { vertices } in paths {
                let vertices = vertices.as_ref();
                let path_vertices_buf_iter = vertices
                    .iter()
                    .zip(vertices.iter().skip(1))
                    .flat_map(|(a, b)| {
                        [a[0] as f32, a[1] as f32, b[0] as f32, b[1] as f32]
                    });

                buf.extend(path_vertices_buf_iter);
            }

            buf.len() / 4
        } else if coo_space == CooSpace::Model {
            for PathVertices { vertices } in paths {
                let vertices = vertices.as_ref();
                let path_vertices_buf_iter = vertices
                    .iter()
                    .zip(vertices.iter().skip(1))
                    .flat_map(|(a, b)| {
                        let xyz1 = crate::math::lonlat::radec_to_xyz(a[0].to_angle(), a[1].to_angle());
                        let xyz2 = crate::math::lonlat::radec_to_xyz(b[0].to_angle(), b[1].to_angle());

                        [
                            xyz1[0] as f32, xyz1[1] as f32, xyz1[2] as f32,
                            xyz2[0] as f32, xyz2[1] as f32, xyz2[2] as f32,
                        ]
                    });

                buf.extend(path_vertices_buf_iter);
            }

            buf.len() / 6
        } else {
            todo!();
        };

        vao.bind_for_update().update_instanced_array(
            "ndc_pos",
            WebGl2RenderingContext::DYNAMIC_DRAW,
            VecData(&buf),
        );

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

                    let center_lonlat = camera.get_center().lonlat();
                    let cclat = center_lonlat.lat().to_radians().cos();

                    crate::shader::get_shader(
                        &self.gl,
                        shaders,
                        "line_inst_lonlat_f64.vert",
                        "line_base.frag",
                    )?
                    .bind(&self.gl)
                    .attach_uniforms_from(camera)
                    .attach_uniform("u_2world", &icrs2world)
                    .attach_uniform("u_color", &meta.color)
                    .attach_uniform("pos_angle", &(camera.get_position_angle().to_radians() as f32))
                    .attach_uniform("center_lonlat", &center_lonlat)
                    .attach_uniform("cclat", &(cclat as f32))
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
                },
                CooSpace::Model => {
                    let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
                    let view2world = camera.get_m2w();
                    let icrs2world = view2world * icrs2view;

                    crate::shader::get_shader(
                        &self.gl,
                        shaders,
                        "line_inst_xyz.vert",
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
