use crate::math::projection::ProjectionType;
use crate::shader::ShaderManager;
use al_api::coo_system::CooSystem;
use al_core::VertexArrayObject;
use al_core::WebGlContext;

use al_api::color::ColorRGBA;

pub struct PolylineRenderer {
    gl: WebGlContext,
    vao: VertexArrayObject,

    color: ColorRGBA,
    thickness: f32,
    num_instances: usize,
}
use wasm_bindgen::JsValue;

use al_core::VecData;
use web_sys::WebGl2RenderingContext;

use crate::camera::CameraViewPort;

use super::Shape;

use super::Catalog;

impl PolylineRenderer {
    /// Init the buffers, VAO and shader
    pub fn new<'a>(gl: &WebGlContext, catalog: &Catalog) -> Result<Self, JsValue> {
        let lines = catalog
            .shapes
            .iter()
            .flat_map(|s| {
                let mut v = vec![];
                match s {
                    Shape::PolyLine(vertices) => {
                        for (v1, v2) in vertices.iter().zip(vertices.iter().skip(1)) {
                            v.extend_from_slice(&[
                                v1.lon().to_radians(),
                                v1.lat().to_radians(),
                                v2.lon().to_radians(),
                                v2.lat().to_radians(),
                            ])
                        }
                    }
                    _ => (),
                }

                v
            })
            .collect::<Vec<_>>();

        let num_instances = lines.len() / 4;

        // Create the VAO for the screen
        let mut vao = VertexArrayObject::new(&gl);
        vao.bind_for_update()
            // Store the cartesian position of the center of the source in the a instanced VBO
            .add_instanced_array_buffer(
                "line",
                9 * std::mem::size_of::<f32>(),
                &[2, 2],
                &[0, 2 * std::mem::size_of::<f32>()],
                WebGl2RenderingContext::STATIC_DRAW,
                VecData::<f32>(&lines),
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

        let gl = gl.clone();

        Ok(Self {
            gl,
            vao,
            color: catalog.color,
            thickness: catalog.thickness,
            num_instances,
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

    pub fn draw(
        &mut self,
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

        // draw the instanced lines
        let icrs2view = CooSystem::ICRS.to(camera.get_coo_system());
        let view2world = camera.get_m2w();
        let icrs2world = view2world * icrs2view;

        crate::shader::get_shader(&self.gl, shaders, "line_inst_lonlat.vert", "line_base.frag")?
            .bind(&self.gl)
            .attach_uniforms_from(camera)
            .attach_uniform("u_2world", &icrs2world)
            .attach_uniform("u_color", &self.color)
            .attach_uniform("u_width", &self.thickness)
            .attach_uniform("u_proj", proj)
            .bind_vertex_array_object_ref(&self.vao)
            .draw_elements_instanced_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                0,
                self.num_instances as i32,
            );

        //self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}
