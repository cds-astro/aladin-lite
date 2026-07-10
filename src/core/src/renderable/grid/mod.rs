pub mod label;
pub mod meridian;
pub mod parallel;
pub mod arc;

use crate::shader::ShaderManager;
use crate::Abort;
use al_core::VecData;
use parallel::Parallel;
use arc::Arc;

use crate::camera::CameraViewPort;
use crate::math::HALF_PI;
use crate::ProjectionType;
use al_api::color::ColorRGBA;
use al_api::grid::GridCfg;
use al_core::VertexArrayObject;
use al_core::WebGlContext;
use web_sys::WebGl2RenderingContext;

use label::Label;
pub struct ProjetedGrid {
    // Properties
    pub color: ColorRGBA,
    pub show_labels: bool,
    pub enabled: bool,
    pub label_scale: f32,
    thickness: f32,

    // Render Text Manager
    text_renderer: TextRenderManager,
    fmt: Formatter,

    //line_style: line::Style,
    meridians: Vec<Meridian>,
    parallels: Vec<Parallel>,

    vao: VertexArrayObject,
    gl: WebGlContext,
}
use self::meridian::Meridian;
use crate::renderable::text::TextRenderManager;
use crate::renderable::Renderer;
use al_api::angle::Formatter;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
impl ProjetedGrid {
    pub fn new(gl: WebGlContext, aladin_div: &HtmlElement) -> Result<ProjetedGrid, JsValue> {
        let text_renderer = TextRenderManager::new(aladin_div)?;

        let color = ColorRGBA {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        };
        let show_labels = true;
        let enabled = false;
        let label_scale = 1.0;
        //let line_style = line::Style::None;
        let fmt = Formatter::Decimal;
        let thickness = 2.0;
        let meridians = Vec::new();
        let parallels = Vec::new();

        let mut vao = VertexArrayObject::new(&gl);
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

        let grid = ProjetedGrid {
            color,
            //line_style,
            show_labels,
            enabled,
            label_scale,
            thickness,

            text_renderer,
            meridians,
            parallels,
            fmt,

            vao,
            gl,
        };
        // Initialize the vertices & labels
        //grid.force_update(camera, projection, line_renderer);

        Ok(grid)
    }

    pub fn set_cfg(&mut self, new_cfg: GridCfg) -> Result<(), JsValue> {
        let GridCfg {
            color,
            opacity,
            thickness,
            show_labels,
            label_size,
            enabled,
            fmt,
        } = new_cfg;

        if let Some(color) = color {
            self.color = ColorRGBA {
                r: color.r,
                g: color.g,
                b: color.b,
                a: self.color.a,
            };
            self.text_renderer.set_color(&self.color);
        }

        if let Some(opacity) = opacity {
            self.color.a = opacity;
            self.text_renderer.set_color(&self.color);
        }

        if let Some(thickness) = thickness {
            // convert thickness in pixels to ndc
            self.thickness = thickness;
        }

        if let Some(show_labels) = show_labels {
            self.show_labels = show_labels;
        }

        if let Some(fmt) = fmt {
            self.fmt = fmt;
        }

        if let Some(label_size) = label_size {
            self.label_scale = label_size;
            self.text_renderer.set_font_size(label_size as u32);
        }

        if let Some(enabled) = enabled {
            self.enabled = enabled;
        }

        Ok(())
    }

    pub fn draw_labels(&mut self) -> Result<(), JsValue> {
        if self.enabled && self.show_labels {
            let labels = self
                .meridians
                .iter()
                .filter_map(|m| m.get_label())
                .chain(self.parallels.iter().filter_map(|p| p.get_label()));

            //let dpi = camera.get_dpi();
            self.text_renderer.begin();
            for Label {
                content,
                position,
                rot,
            } in labels
            {
                let position = position.cast::<f32>().unwrap_abort();
                self.text_renderer
                    .add_label(content, &position, cgmath::Rad(*rot as f32))?;
            }
            self.text_renderer.end();
        }

        Ok(())
    }

    pub fn draw(
        &mut self,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        shaders: &mut ShaderManager,
    ) -> Result<(), JsValue> {
        if self.enabled {
            let fov = camera.get_field_of_view();
            let bbox = fov.get_bounding_box();
            //let max_dim_px = camera.get_width().max(camera.get_height()) as f64;
            //let step_line_px = max_dim_px * 0.15;

            let aspect = camera.get_aspect() as f64;

            // update meridians
            self.meridians = {
                // Select the good step with a binary search
                let step_lon_precised = bbox.get_lon_size() * 0.25;
                let step_lon = select_fixed_step(step_lon_precised.to_degrees()).to_hms();

                // Add meridians
                //let start_lon = bbox.lon_min() - (bbox.lon_min() % step_lon);
                let mut stop_lon = Arc::from_degrees(bbox.lon_max().to_degrees());
                
                let start_idx = (bbox.lon_min() / step_lon.to_degrees().to_radians()).floor() as i32;
                let mut start_lon: Arc = step_lon * start_idx;

                let compute_meridian = |start_lon: Arc, stop_lon: Arc| -> Vec<Meridian> {
                    let mut meridians = vec![];
                    let mut lon = start_lon;
                    while lon.to_degrees() < stop_lon.to_degrees() {
                        if let Some(p) = meridian::get_intersecting_meridian(
                            lon,
                            camera,
                            projection,
                            self.fmt,
                        ) {
                            meridians.push(p);
                        }
                        lon = lon + step_lon;
                    }
                    meridians
                };

                if bbox.all_lon() {
                    start_lon = Arc::from_hms(0, 0, 0.0);
                    stop_lon = Arc::from_hms(24, 0, 0.0);

                    compute_meridian(start_lon, stop_lon)
                } else if bbox.contains_longitude(0.0) {
                    let mut meridians = Vec::new();

                    let max_bound = if start_lon.to_degrees() > stop_lon.to_degrees() { start_lon } else { stop_lon };
                    
                    let lonlat_east: Arc = step_lon * ((max_bound.to_degrees().to_radians() / step_lon.to_degrees().to_radians()).floor() as i32);
                    let lonlat_west: Arc = Arc::from_degrees(360.0 - lonlat_east.to_degrees()).to_hms();

                    meridians.extend(compute_meridian(Arc::from_hms(0, 0, 0.0), lonlat_east));
                    meridians.extend(compute_meridian(lonlat_west, Arc::from_hms(24, 0, 0.0)));

                    meridians
                } else {
                    compute_meridian(start_lon, stop_lon)
                }
            };

            self.parallels = {
                let step_lat_precised = bbox.get_lat_size() * 0.25;
                let step_lat = select_fixed_step(step_lat_precised.to_degrees());

                let decimal_lat_prec = 15;

                let start_idx = (bbox.lat_min() / step_lat.to_degrees().to_radians()).floor() as i32;
                let mut start_lat: Arc = step_lat * start_idx;

                /*if start_lat == -HALF_PI {
                    start_lat += step_lat;
                }*/
                let stop_lat = bbox.lat_max();
                let mut lat = start_lat;

                let mut parallels = vec![];
                while lat.to_degrees() < stop_lat.to_degrees() {
                    if let Some(p) = parallel::get_intersecting_parallel(
                        lat,
                        camera,
                        projection,
                        self.fmt,
                    ) {
                        parallels.push(p);
                    }
                    lat = lat + step_lat;
                }

                parallels
            };

            // update the line buffers
            let paths = self
                .meridians
                .iter()
                .map(|meridian| meridian.get_lines_vertices())
                .chain(
                    self.parallels
                        .iter()
                        .map(|parallel| parallel.get_lines_vertices()),
                )
                .flatten();

            let mut buf: Vec<f32> = vec![];

            for vertices in paths {
                let path_vertices_buf_iter = vertices
                    .iter()
                    .zip(vertices.iter().skip(1))
                    .flat_map(|(a, b)| [a[0], a[1], b[0], b[1]]);

                buf.extend(path_vertices_buf_iter);
            }

            self.vao.bind_for_update().update_instanced_array(
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&buf),
            );

            let num_instances = buf.len() / 4;

            crate::shader::get_shader(&self.gl, shaders, "line_inst_ndc.vert", "line_base.frag")?
                .bind(&self.gl)
                .attach_uniform("u_color", &self.color)
                .attach_uniform("u_width", &(camera.get_width()))
                .attach_uniform("u_height", &(camera.get_height()))
                .attach_uniform("u_thickness", &self.thickness)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_instanced_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    num_instances as i32,
                );
        }

        Ok(())
    }
}

const GRID_STEPS: &[Arc] = &[
    Arc::second(1e-6),
    Arc::second(1e-5),
    Arc::second(1e-4),
    Arc::second(2e-4),
    Arc::second(5e-4),
    Arc::second(0.001),
    Arc::second(0.002),
    Arc::second(0.005),
    Arc::second(0.01),
    Arc::second(0.02),
    Arc::second(0.05),
    Arc::second(0.1),
    Arc::second(1.0),
    Arc::second(5.0),
    Arc::second(15.0),
    Arc::second(30.0),
    Arc::minute(1),
    Arc::minute(5),
    Arc::minute(15),
    Arc::minute(30),
    Arc::deg(1),
    Arc::deg(5),
    Arc::deg(15),
    Arc::deg(30)
];

fn select_fixed_step(fov: f64) -> Arc {
    match GRID_STEPS.binary_search_by(|v| {
        v.to_degrees().partial_cmp(&fov)
            .expect("Couldn't compare values, maybe because the fov given is NaN")
    }) {
        Ok(idx) => GRID_STEPS[idx],
        Err(idx) => {
            if idx == 0 {
                GRID_STEPS[0]
            } else if idx == GRID_STEPS.len() {
                GRID_STEPS[idx - 1]
            } else {
                let a = GRID_STEPS[idx];
                let b = GRID_STEPS[idx - 1];

                if a.to_degrees() - fov > fov - b.to_degrees() {
                    b
                } else {
                    a
                }
            }
        }
    }
}
