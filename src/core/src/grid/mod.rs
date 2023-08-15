pub mod label;
pub mod meridian;
pub mod parallel;

use crate::math::projection::coo_space::XYScreen;
use crate::Abort;

use crate::camera::CameraViewPort;
use crate::math::angle;
use crate::math::HALF_PI;
use crate::renderable::line;
use crate::renderable::line::PathVertices;
use crate::renderable::Renderer;
use crate::ProjectionType;
use al_api::color::ColorRGBA;

use al_api::grid::GridCfg;

use crate::grid::label::Label;
pub struct ProjetedGrid {
    // Properties
    pub color: ColorRGBA,
    pub show_labels: bool,
    pub enabled: bool,
    pub label_scale: f32,
    thickness: f32,

    // Render Text Manager
    text_renderer: TextRenderManager,
    fmt: angle::SerializeFmt,

    line_style: line::Style,
}

use crate::shader::ShaderManager;
use wasm_bindgen::JsValue;

use crate::renderable::line::RasterizedLineRenderer;
use crate::renderable::text::TextRenderManager;

impl ProjetedGrid {
    pub fn new() -> Result<ProjetedGrid, JsValue> {
        let text_renderer = TextRenderManager::new()?;

        let color = ColorRGBA {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 0.5,
        };
        let show_labels = true;
        let enabled = false;
        let label_scale = 1.0;
        let line_style = line::Style::None;
        let fmt = angle::SerializeFmt::DMS;
        let thickness = 3.0;

        let grid = ProjetedGrid {
            color,
            line_style,
            show_labels,
            enabled,
            label_scale,
            thickness,

            text_renderer,
            fmt,
        };
        // Initialize the vertices & labels
        //grid.force_update(camera, projection, line_renderer);

        Ok(grid)
    }

    pub fn set_cfg(
        &mut self,
        new_cfg: GridCfg,
        _camera: &CameraViewPort,
        _projection: &ProjectionType,
    ) -> Result<(), JsValue> {
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
            self.text_renderer.set_color(&color);
        }

        if let Some(opacity) = opacity {
            self.color.a = opacity;
        }

        if let Some(thickness) = thickness {
            // convert thickness in pixels to ndc
            self.thickness = thickness;
        }

        if let Some(show_labels) = show_labels {
            self.show_labels = show_labels;
        }

        if let Some(fmt) = fmt {
            self.fmt = fmt.into();
        }

        if let Some(label_size) = label_size {
            self.label_scale = label_size;
            self.text_renderer.set_font_size(label_size as u32);
        }

        if let Some(enabled) = enabled {
            self.enabled = enabled;

            if !self.enabled {
                self.text_renderer.clear_text_canvas();
            }
        }

        Ok(())
    }

    // Update the grid whenever the camera moved
    fn update(
        &mut self,
        camera: &CameraViewPort,
        projection: &ProjectionType,
        rasterizer: &mut RasterizedLineRenderer,
    ) -> Result<(), JsValue> {
        let fov = camera.get_field_of_view();
        let bbox = fov.get_bounding_box();
        let max_dim_px = camera.get_width().max(camera.get_height()) as f64;
        let step_line_px = max_dim_px * 0.2;

        // update meridians
        let meridians = {
            // Select the good step with a binary search
            let step_lon_precised =
                (bbox.get_lon_size() as f64) * step_line_px / (camera.get_width() as f64);
            let step_lon = select_fixed_step(step_lon_precised);

            // Add meridians
            let start_lon = bbox.lon_min() - (bbox.lon_min() % step_lon);
            let mut stop_lon = bbox.lon_max();
            if bbox.all_lon() {
                stop_lon -= 1e-3;
            }

            let mut meridians = vec![];
            let mut lon = start_lon;
            while lon < stop_lon {
                if let Some(p) =
                    meridian::get_intersecting_meridian(lon, camera, projection, &self.fmt)
                {
                    meridians.push(p);
                }
                lon += step_lon;
            }
            meridians
        };

        let parallels = {
            let step_lat_precised =
                (bbox.get_lat_size() as f64) * step_line_px / (camera.get_height() as f64);
            let step_lat = select_fixed_step(step_lat_precised);

            let mut start_lat = bbox.lat_min() - (bbox.lat_min() % step_lat);
            if start_lat == -HALF_PI {
                start_lat += step_lat;
            }
            let stop_lat = bbox.lat_max();
            let mut lat = start_lat;

            let mut parallels = vec![];
            while lat < stop_lat {
                if let Some(p) = parallel::get_intersecting_parallel(lat, camera, projection) {
                    parallels.push(p);
                }
                lat += step_lat;
            }
            parallels
        };

        // update the line buffers
        let paths = meridians
            .iter()
            .map(|meridian| meridian.get_lines_vertices())
            .chain(
                parallels
                    .iter()
                    .map(|parallel| parallel.get_lines_vertices()),
            )
            .flatten()
            .map(|vertices| PathVertices {
                closed: false,
                vertices,
            });

        rasterizer.add_stroke_paths(
            paths,
            self.thickness * 2.0 / camera.get_width(),
            &self.color,
            &self.line_style,
        );

        // update labels
        {
            let labels = meridians
                .iter()
                .filter_map(|m| m.get_label())
                .chain(parallels.iter().filter_map(|p| p.get_label()));

            let dpi = camera.get_dpi();
            self.text_renderer.begin();
            for Label {
                content,
                position,
                rot,
            } in labels
            {
                let position = position.cast::<f32>().unwrap_abort();
                self.text_renderer
                    .add_label(&content, &position, cgmath::Rad(*rot as f32))?;
            }
            self.text_renderer.end();
        }

        Ok(())
    }

    pub fn draw(
        &mut self,
        camera: &CameraViewPort,
        _shaders: &mut ShaderManager,
        projection: &ProjectionType,
        rasterizer: &mut RasterizedLineRenderer,
    ) -> Result<(), JsValue> {
        if self.enabled {
            self.update(camera, projection, rasterizer)?;
        }

        Ok(())
    }
}

const GRID_STEPS: &[f64] = &[
    0.0000000000048481367,
    0.000000000009696274,
    0.000000000024240685,
    0.000000000048481369,
    0.000000000096962737,
    0.00000000024240683,
    0.00000000048481364,
    0.0000000009696274,
    0.0000000024240686,
    0.000000004848138,
    0.000000009696275,
    0.000000024240685,
    0.00000004848138,
    0.00000009696275,
    0.00000024240687,
    0.0000004848138,
    0.0000009696275,
    0.0000024240686,
    0.000004848138,
    0.000009696275,
    0.000024240685,
    0.000048481369,
    0.000072722055,
    0.00014544412,
    0.00029088823,
    0.00058177644,
    0.0014544412,
    0.0029088823,
    0.004363324,
    0.008726647,
    0.017453293,
    0.034906586,
    0.08726647,
    0.17453293,
    0.34906585,
    std::f64::consts::FRAC_PI_4,
];

fn select_fixed_step(fov: f64) -> f64 {
    match GRID_STEPS.binary_search_by(|v| {
        v.partial_cmp(&fov)
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

                if a - fov > fov - b {
                    b
                } else {
                    a
                }
            }
        }
    }
}
