use crate::{
    app::App,
    renderable::{
        projection::{Aitoff, Orthographic, Mollweide, AzimuthalEquidistant, Gnomonic, Mercator},
        angle::{Angle, ArcDeg},
    },
    color::Color,
    shaders::Colormap,
    math::LonLatT,
    time::DeltaTime,
    coo_conversion::CooSystem,
};
use cgmath::Vector2;
use wasm_bindgen::prelude::*;

pub enum ProjectionType {
    Aitoff,
    MollWeide,
    Arc,
    Mercator,
    Ortho,
    Gnomonic,
}

impl ProjectionType {
    pub fn set_projection(&mut self, app: &mut App, name: String) -> Result<(), JsValue> {
        match name.as_str() {
            "aitoff" => {
                app.set_projection::<Aitoff>();
                *self = ProjectionType::Aitoff;
                Ok(())
            },
            "sinus" => {
                app.set_projection::<Orthographic>();
                *self = ProjectionType::Ortho;
                Ok(())
            },
            "mollweide" => {
                app.set_projection::<Mollweide>();
                *self = ProjectionType::MollWeide;
                Ok(())
            },
            "arc" => {
                app.set_projection::<AzimuthalEquidistant>();
                *self = ProjectionType::Arc;
                Ok(())
            },
            "tan" => {
                app.set_projection::<Gnomonic>();
                *self = ProjectionType::Gnomonic;
                Ok(())
            },
            "mercator" => {
                app.set_projection::<Mercator>();
                *self = ProjectionType::Mercator;
                Ok(())
            },
            _ => Err(format!("{} is not a valid projection name. aitoff, arc, sinus, tan, mollweide and mercator are accepted", name).into())
        }
    }

    pub fn set_longitude_reversed(&mut self, app: &mut App, reversed: bool) {
        match self {
            ProjectionType::Aitoff => app.set_longitude_reversed::<Aitoff>(reversed),
            ProjectionType::MollWeide => app.set_longitude_reversed::<Mollweide>(reversed),
            ProjectionType::Ortho => app.set_longitude_reversed::<Orthographic>(reversed),
            ProjectionType::Arc => app.set_longitude_reversed::<AzimuthalEquidistant>(reversed),
            ProjectionType::Gnomonic => app.set_longitude_reversed::<Gnomonic>(reversed),
            ProjectionType::Mercator => app.set_longitude_reversed::<Mercator>(reversed),
        };
    }

    pub fn set_catalog_colormap(
        &self,
        app: &mut App,
        name: String,
        colormap: Colormap,
    ) -> Result<(), JsValue> {
        app.set_catalog_colormap(name, colormap)
    }

    pub fn world_to_screen(
        &self,
        app: &App,
        lonlat: &LonLatT<f64>,
    ) -> Result<Option<Vector2<f64>>, String> {
        match self {
            ProjectionType::Aitoff => app.world_to_screen::<Aitoff>(lonlat),
            ProjectionType::MollWeide => app.world_to_screen::<Mollweide>(lonlat),
            ProjectionType::Ortho => app.world_to_screen::<Orthographic>(lonlat),
            ProjectionType::Arc => app.world_to_screen::<AzimuthalEquidistant>(lonlat),
            ProjectionType::Gnomonic => app.world_to_screen::<Gnomonic>(lonlat),
            ProjectionType::Mercator => app.world_to_screen::<Mercator>(lonlat),
        }
    }

    pub fn world_to_screen_vec(&self, app: &App, sources: &Vec<JsValue>) -> Result<Vec<f64>, JsValue> {
        match self {
            ProjectionType::Aitoff => app.world_to_screen_vec::<Aitoff>(sources),
            ProjectionType::MollWeide => app.world_to_screen_vec::<Mollweide>(sources),
            ProjectionType::Ortho => app.world_to_screen_vec::<Orthographic>(sources),
            ProjectionType::Arc => app.world_to_screen_vec::<AzimuthalEquidistant>(sources),
            ProjectionType::Gnomonic => app.world_to_screen_vec::<Gnomonic>(sources),
            ProjectionType::Mercator => app.world_to_screen_vec::<Mercator>(sources),
        }
    }

    pub fn get_max_fov(&self, app: &App) -> f64 {
        match self {
            ProjectionType::Aitoff => app.get_max_fov::<Aitoff>(),
            ProjectionType::MollWeide => app.get_max_fov::<Mollweide>(),
            ProjectionType::Ortho => app.get_max_fov::<Orthographic>(),
            ProjectionType::Arc => app.get_max_fov::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.get_max_fov::<Gnomonic>(),
            ProjectionType::Mercator => app.get_max_fov::<Mercator>(),
        }
    }

    pub fn screen_to_world(&self, app: &App, pos: &Vector2<f64>) -> Option<LonLatT<f64>> {
        match self {
            ProjectionType::Aitoff => app.screen_to_world::<Aitoff>(pos),
            ProjectionType::MollWeide => app.screen_to_world::<Mollweide>(pos),
            ProjectionType::Ortho => app.screen_to_world::<Orthographic>(pos),
            ProjectionType::Arc => app.screen_to_world::<AzimuthalEquidistant>(pos),
            ProjectionType::Gnomonic => app.screen_to_world::<Gnomonic>(pos),
            ProjectionType::Mercator => app.screen_to_world::<Mercator>(pos),
        }
    }

    pub fn go_from_to(&self, app: &mut App, s1x: f64, s1y: f64, s2x: f64, s2y: f64) {
        match self {
            ProjectionType::Aitoff => app.go_from_to::<Aitoff>(s1x, s1y, s2x, s2y),
            ProjectionType::MollWeide => app.go_from_to::<Mollweide>(s1x, s1y, s2x, s2y),
            ProjectionType::Ortho => app.go_from_to::<Orthographic>(s1x, s1y, s2x, s2y),
            ProjectionType::Arc => app.go_from_to::<AzimuthalEquidistant>(s1x, s1y, s2x, s2y),
            ProjectionType::Gnomonic => app.go_from_to::<Gnomonic>(s1x, s1y, s2x, s2y),
            ProjectionType::Mercator => app.go_from_to::<Mercator>(s1x, s1y, s2x, s2y),
        }
    }

    pub fn update(&mut self, app: &mut App, dt: DeltaTime, force: bool) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.update::<Aitoff>(dt, force),
            ProjectionType::MollWeide => app.update::<Mollweide>(dt, force),
            ProjectionType::Ortho => app.update::<Orthographic>(dt, force),
            ProjectionType::Arc => app.update::<AzimuthalEquidistant>(dt, force),
            ProjectionType::Gnomonic => app.update::<Gnomonic>(dt, force),
            ProjectionType::Mercator => app.update::<Mercator>(dt, force),
        }
    }

    pub fn render(&mut self, app: &mut App, force: bool) -> Result<(), JsValue> {
        match self {
            ProjectionType::Aitoff => app.render::<Aitoff>(force)?,
            ProjectionType::MollWeide => app.render::<Mollweide>(force)?,
            ProjectionType::Ortho => app.render::<Orthographic>(force)?,
            ProjectionType::Arc => app.render::<AzimuthalEquidistant>(force)?,
            ProjectionType::Gnomonic => app.render::<Gnomonic>(force)?,
            ProjectionType::Mercator => app.render::<Mercator>(force)?,
        };

        Ok(())
    }

    pub fn add_catalog(&mut self, app: &mut App, name: String, table: JsValue, colormap: String) {
        app.add_catalog(name, table, colormap);
    }

    pub fn resize(&mut self, app: &mut App, width: f32, height: f32) {
        match self {
            ProjectionType::Aitoff => app.resize_window::<Aitoff>(width, height),
            ProjectionType::MollWeide => app.resize_window::<Mollweide>(width, height),
            ProjectionType::Ortho => app.resize_window::<Orthographic>(width, height),
            ProjectionType::Arc => app.resize_window::<AzimuthalEquidistant>(width, height),
            ProjectionType::Gnomonic => app.resize_window::<Gnomonic>(width, height),
            ProjectionType::Mercator => app.resize_window::<Mercator>(width, height),
        };
    }

    pub fn set_kernel_strength(
        &mut self,
        app: &mut App,
        name: String,
        strength: f32,
    ) -> Result<(), JsValue> {
        app.set_kernel_strength(name, strength)
    }

    pub fn set_heatmap_opacity(
        &mut self,
        app: &mut App,
        name: String,
        opacity: f32,
    ) -> Result<(), JsValue> {
        app.set_heatmap_opacity(name, opacity)
    }

    pub fn set_center(&mut self, app: &mut App, lonlat: LonLatT<f64>) {
        match self {
            ProjectionType::Aitoff => app.set_center::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.set_center::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.set_center::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.set_center::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.set_center::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.set_center::<Mercator>(&lonlat),
        };
    }

    pub fn start_moving_to(&mut self, app: &mut App, lonlat: LonLatT<f64>) {
        match self {
            ProjectionType::Aitoff => app.start_moving_to::<Aitoff>(&lonlat),
            ProjectionType::MollWeide => app.start_moving_to::<Mollweide>(&lonlat),
            ProjectionType::Ortho => app.start_moving_to::<Orthographic>(&lonlat),
            ProjectionType::Arc => app.start_moving_to::<AzimuthalEquidistant>(&lonlat),
            ProjectionType::Gnomonic => app.start_moving_to::<Gnomonic>(&lonlat),
            ProjectionType::Mercator => app.start_moving_to::<Mercator>(&lonlat),
        };
    }

    pub fn start_zooming_to(&mut self, app: &mut App, fov: Angle<f64>) {
        match self {
            ProjectionType::Aitoff => app.start_zooming_to::<Aitoff>(fov),
            ProjectionType::MollWeide => app.start_zooming_to::<Mollweide>(fov),
            ProjectionType::Ortho => app.start_zooming_to::<Orthographic>(fov),
            ProjectionType::Arc => app.start_zooming_to::<AzimuthalEquidistant>(fov),
            ProjectionType::Gnomonic => app.start_zooming_to::<Gnomonic>(fov),
            ProjectionType::Mercator => app.start_zooming_to::<Mercator>(fov),
        };
    }

    pub fn project_line(
        &self,
        app: &App,
        lon1: f64,
        lat1: f64,
        lon2: f64,
        lat2: f64,
    ) -> Vec<Vector2<f64>> {
        match self {
            ProjectionType::Aitoff => app.project_line::<Aitoff>(lon1, lat1, lon2, lat2),
            ProjectionType::MollWeide => app.project_line::<Mollweide>(lon1, lat1, lon2, lat2),
            ProjectionType::Ortho => app.project_line::<Orthographic>(lon1, lat1, lon2, lat2),
            ProjectionType::Arc => app.project_line::<AzimuthalEquidistant>(lon1, lat1, lon2, lat2),
            ProjectionType::Gnomonic => app.project_line::<Gnomonic>(lon1, lat1, lon2, lat2),
            ProjectionType::Mercator => app.project_line::<Mercator>(lon1, lat1, lon2, lat2),
        }
    }

    pub fn get_center(&self, app: &App) -> LonLatT<f64> {
        match self {
            ProjectionType::Aitoff => app.get_center::<Aitoff>(),
            ProjectionType::MollWeide => app.get_center::<Mollweide>(),
            ProjectionType::Ortho => app.get_center::<Orthographic>(),
            ProjectionType::Arc => app.get_center::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.get_center::<Gnomonic>(),
            ProjectionType::Mercator => app.get_center::<Mercator>(),
        }
    }

    pub fn enable_grid(&mut self, app: &mut App) {
        match self {
            ProjectionType::Aitoff => app.enable_grid::<Aitoff>(),
            ProjectionType::MollWeide => app.enable_grid::<Mollweide>(),
            ProjectionType::Ortho => app.enable_grid::<Orthographic>(),
            ProjectionType::Arc => app.enable_grid::<AzimuthalEquidistant>(),
            ProjectionType::Gnomonic => app.enable_grid::<Gnomonic>(),
            ProjectionType::Mercator => app.enable_grid::<Mercator>(),
        };
    }

    pub fn set_coo_system(&mut self, app: &mut App, system: CooSystem) {
        match self {
            ProjectionType::Aitoff => app.set_coo_system::<Aitoff>(system),
            ProjectionType::MollWeide => app.set_coo_system::<Mollweide>(system),
            ProjectionType::Ortho => app.set_coo_system::<Orthographic>(system),
            ProjectionType::Arc => app.set_coo_system::<AzimuthalEquidistant>(system),
            ProjectionType::Gnomonic => app.set_coo_system::<Gnomonic>(system),
            ProjectionType::Mercator => app.set_coo_system::<Mercator>(system),
        };
    }

    pub fn rotate_around_center(&mut self, app: &mut App, theta: ArcDeg<f64>) {
        match self {
            ProjectionType::Aitoff => app.rotate_around_center::<Aitoff>(theta),
            ProjectionType::MollWeide => app.rotate_around_center::<Mollweide>(theta),
            ProjectionType::Ortho => app.rotate_around_center::<Orthographic>(theta),
            ProjectionType::Arc => app.rotate_around_center::<AzimuthalEquidistant>(theta),
            ProjectionType::Gnomonic => app.rotate_around_center::<Gnomonic>(theta),
            ProjectionType::Mercator => app.rotate_around_center::<Mercator>(theta),
        };
    }

    pub fn hide_grid_labels(&mut self, app: &mut App) {
        app.hide_grid_labels();
    }

    pub fn show_grid_labels(&mut self, app: &mut App) {
        app.show_grid_labels();
    }

    pub fn disable_grid(&mut self, app: &mut App) {
        app.disable_grid();
    }

    pub fn set_grid_color(&mut self, app: &mut App, color: Color) {
        app.set_grid_color(color);
    }
}
