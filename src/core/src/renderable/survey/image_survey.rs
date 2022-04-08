use crate::buffer::Texture;

use al_core::{VecData, format::{R32F, RGB8U, RGBA8U}, image::ImageBuffer};
#[cfg(feature = "webgl2")]
use al_core::format::{R16I, R32I, R8UI};
use egui::epaint::ahash::CallHasher;

pub struct TextureToDraw<'a> {
    pub starting_texture: &'a Texture,
    pub ending_texture: &'a Texture,
}

impl<'a> TextureToDraw<'a> {
    fn new(starting_texture: &'a Texture, ending_texture: &'a Texture) -> TextureToDraw<'a> {
        TextureToDraw {
            starting_texture,
            ending_texture,
        }
    }
}

use std::collections::{HashMap, HashSet};
pub struct TexturesToDraw<'a>(Vec<TextureToDraw<'a>>);

impl<'a> TexturesToDraw<'a> {
    fn new(capacity: usize) -> TexturesToDraw<'a> {
        let states = Vec::with_capacity(capacity);

        TexturesToDraw(states)
    }
}

impl<'a> core::ops::Deref for TexturesToDraw<'a> {
    type Target = Vec<TextureToDraw<'a>>;

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}
impl<'a> core::ops::DerefMut for TexturesToDraw<'a> {
    fn deref_mut(&'_ mut self) -> &'_ mut Self::Target {
        &mut self.0
    }
} 

pub trait RecomputeRasterizer {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        view: &HEALPixCellsInView,
        // The survey from which we get the textures to plot
        // Usually it is the most refined survey
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a>;
}

pub struct Move;
pub struct Zoom;
pub struct UnZoom;

impl RecomputeRasterizer for Move {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
        let mut textures = TexturesToDraw::new(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            }
        }

        textures
    }
}

/*fn num_subdivision(depth: u8) -> u8 {
    if depth < 5 {
        std::cmp::max(5 - depth, 2)
    } else {
        0
    }
}*/

// Recursively compute the number of subdivision needed for a cell
// to not be too much skewed
use al_api::coo_system::CooSystem;
use cgmath::InnerSpace;
use crate::healpix_cell::HEALPixCell;

/*fn num_subdivision<P: Projection>(cell: &HEALPixCell, camera: &CameraViewPort, reversed_longitude: bool) -> u8 {
    let skewed_factor = get_skewed_factor::<P>(cell, camera, reversed_longitude);
    al_core::log::log(&format!("skewed factor {:?}", skewed_factor));

    if skewed_factor > 0.8 {
        0
    } else {
        let mut subdivide_further = false;
        for child_cell in cell.get_children_cells(1) {
            let child_cell_skewed_factor = get_skewed_factor::<P>(&child_cell, camera, reversed_longitude);
            if child_cell_skewed_factor > skewed_factor {
                subdivide_further = true;
                break;
            }
        }
    
        if !subdivide_further {
            0
        } else {
            cell.get_children_cells(1)
                .map(|child_cell| {
                    num_subdivision::<P>(&child_cell, camera, reversed_longitude)
                })
                .fold(std::u8::MIN, |a, b| a.max(b)) + 1
        }
    }
}*/

fn num_subdivision(cell: &HEALPixCell) -> u8 {
    let d = cell.depth();
    const MAX_SUBDIVISION: u8 = 4;

    if d == 0 {
        return MAX_SUBDIVISION;
    } else if d == 1 {
        return MAX_SUBDIVISION;
    }

    // Largest deformation cell among the cells of a specific depth
    let largest_center_to_vertex_dist = healpix::largest_center_to_vertex_distance(d, 0.0, healpix::TRANSITION_LATITUDE);
    let smallest_center_to_vertex_dist = healpix::largest_center_to_vertex_distance(d, 0.0, healpix::LAT_OF_SQUARE_CELL);

    let (lon, lat) = cell.center();
    let center_to_vertex_dist = healpix::largest_center_to_vertex_distance(d, lon, lat);

    let skewed_factor = (center_to_vertex_dist - smallest_center_to_vertex_dist) / (largest_center_to_vertex_dist - smallest_center_to_vertex_dist);
    //al_core::log::log(&format!("skewed factor {:?}", skewed_factor));
    debug_assert!(skewed_factor <= 1.0 && skewed_factor >= 0.0);

    (skewed_factor * ((MAX_SUBDIVISION - 1) as f64)) as u8 + 1
}

impl RecomputeRasterizer for Zoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let cells_to_draw = view.get_cells();
        let mut textures = TexturesToDraw::new(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let parent_cell = survey.get_nearest_parent(cell);
                let grand_parent_cell = survey.get_nearest_parent(&parent_cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&grand_parent_cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            }
        }

        textures
    }
}

impl RecomputeRasterizer for UnZoom {
    // Returns:
    // * The UV of the starting tile in the global 4096x4096 texture
    // * The UV of the ending tile in the global 4096x4096 texture
    // * the blending factor between the two tiles in the texture
    fn get_textures_from_survey<'a>(
        view: &HEALPixCellsInView,
        survey: &'a ImageSurveyTextures,
    ) -> TexturesToDraw<'a> {
        let _depth = view.get_depth();
        let _max_depth = survey.config().get_max_depth();

        // We do not draw the parent cells if the depth has not decreased by at least one
        let cells_to_draw = view.get_cells();

        let mut textures = TexturesToDraw::new(view.num_of_cells());

        for cell in cells_to_draw {
            if survey.contains(cell) {
                let ending_cell_in_tex = survey.get(cell).unwrap();
                let starting_cell_in_tex = survey.get(cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            } else {
                let parent_cell = survey.get_nearest_parent(cell);

                let ending_cell_in_tex = survey.get(&parent_cell).unwrap();
                let starting_cell_in_tex = survey.get(&parent_cell).unwrap();

                textures.push(
                    TextureToDraw::new(starting_cell_in_tex, ending_cell_in_tex),
                );
            }
        }

        textures
    }
}

use crate::camera::CameraViewPort;
use al_core::WebGlContext;

use crate::projection::Projection;

use super::RayTracer;
use crate::buffer::ImageSurveyTextures;


trait Draw {
    fn draw<P: Projection>(
        &mut self,
        raytracer: &RayTracer,
        switch_from_raytrace_to_raster: bool,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &HiPSColor,
        opacity: f32,
        colormaps: &Colormaps,
    );
}

use al_core::shader::{Shader};

/*
/// List of the different type of surveys
#[derive(Clone, Debug)]
pub enum Color {
    Colored,
    Grayscale2Colormap {
        colormap: Colormap,
        param: GrayscaleParameter,
        reversed: bool,
    },
    Grayscale2Color {
        // A color associated to the component
        color: [f32; 3],
        k: f32, // factor controlling the amount of this HiPS
        param: GrayscaleParameter,
    },
}
*/
//impl Color {
    pub fn get_raster_shader<'a, P: Projection>(
        color: &HiPSColor,
        gl: &WebGlContext,
        shaders: &'a mut ShaderManager,
        integer_tex: bool,
        unsigned_tex: bool,
    ) -> &'a Shader {
        match color {
            HiPSColor::Color => P::get_raster_shader_color(gl, shaders),
            HiPSColor::Grayscale2Colormap { .. } => {
                if unsigned_tex {
                    return P::get_raster_shader_gray2colormap_unsigned(gl, shaders);
                }

                if integer_tex {
                    return P::get_raster_shader_gray2colormap_integer(gl, shaders);
                }

                P::get_raster_shader_gray2colormap(gl, shaders)
            }
            HiPSColor::Grayscale2Color { .. } => {
                if unsigned_tex {
                    return P::get_raster_shader_gray2color_unsigned(gl, shaders);
                }

                if integer_tex {
                    return P::get_raster_shader_gray2color_integer(gl, shaders);
                }

                P::get_raster_shader_gray2color(gl, shaders)
            }
        }
    }

    pub fn get_raytracer_shader<'a, P: Projection>(
        color: &HiPSColor,
        gl: &WebGlContext,
        shaders: &'a mut ShaderManager,
        integer_tex: bool,
        unsigned_tex: bool,
    ) -> &'a Shader {
        match color {
            HiPSColor::Color => P::get_raytracer_shader_color(gl, shaders),
            HiPSColor::Grayscale2Colormap { .. } => {
                if unsigned_tex {
                    return P::get_raytracer_shader_gray2colormap_unsigned(gl, shaders);
                }

                if integer_tex {
                    return P::get_raytracer_shader_gray2colormap_integer(gl, shaders);
                }

                P::get_raytracer_shader_gray2colormap(gl, shaders)
            }
            HiPSColor::Grayscale2Color { .. } => {
                if unsigned_tex {
                    return P::get_raytracer_shader_gray2color_unsigned(gl, shaders);
                }

                if integer_tex {
                    return P::get_raytracer_shader_gray2color_integer(gl, shaders);
                }

                P::get_raytracer_shader_gray2color(gl, shaders)
            }
        }
    }
//}

// Compute the size of the VBO in bytes
// We do want to draw maximum 768 tiles
const MAX_NUM_CELLS_TO_DRAW: usize = 768;
use cgmath::{Vector3, Vector4};


use crate::renderable::survey::uv::{TileCorner, TileUVW};


//#[cfg(feature = "webgl1")]
fn add_vertices_grid(
    cell: &HEALPixCell,
    position: &mut Vec<f32>,
    uv_start: &mut Vec<f32>,
    uv_end: &mut Vec<f32>,
    time_tile_received: &mut Vec<f32>,
    m0: &mut Vec<f32>,
    m1: &mut Vec<f32>,

    idx_positions: &mut Vec<u16>,

    //cell: &HEALPixCell,
    //sphere_sub: &SphereSubdivided,

    uv_0: &TileUVW,
    uv_1: &TileUVW,
    miss_0: f32,
    miss_1: f32,

    alpha: f32,

    camera: &CameraViewPort,
) {
    let num_subdivision = num_subdivision(cell);

    let n_segments_by_side: usize = 1 << (num_subdivision as usize);
    let n_vertices_per_segment = n_segments_by_side + 1;

    // Indices overwritten
    let off_idx_vertices = (position.len() / 3) as u16;

    let ll = cdshealpix::grid_lonlat::<f64>(cell, n_segments_by_side as u16);
    for i in 0..n_vertices_per_segment {
        for j in 0..n_vertices_per_segment {
            let id_vertex_0 = (j + i * n_vertices_per_segment) as usize;

            let model_pos: Vector4<f64> = ll[id_vertex_0].vector();
            position.extend([model_pos.x as f32, model_pos.y as f32, model_pos.z as f32]);


            let hj0 = (j as f32) / (n_segments_by_side as f32);
            let hi0 = (i as f32) / (n_segments_by_side as f32);

            let d01s = uv_0[TileCorner::BottomRight].x - uv_0[TileCorner::BottomLeft].x;
            let d02s = uv_0[TileCorner::TopLeft].y - uv_0[TileCorner::BottomLeft].y;

            let uv_s_vertex_0 = Vector3::new(
                uv_0[TileCorner::BottomLeft].x + hj0 * d01s,
                uv_0[TileCorner::BottomLeft].y + hi0 * d02s,
                uv_0[TileCorner::BottomLeft].z,
            );

            let d01e = uv_1[TileCorner::BottomRight].x - uv_1[TileCorner::BottomLeft].x;
            let d02e = uv_1[TileCorner::TopLeft].y - uv_1[TileCorner::BottomLeft].y;
            let uv_e_vertex_0 = Vector3::new(
                uv_1[TileCorner::BottomLeft].x + hj0 * d01e,
                uv_1[TileCorner::BottomLeft].y + hi0 * d02e,
                uv_1[TileCorner::BottomLeft].z,
            );

            uv_start.extend([uv_s_vertex_0.x as f32, uv_s_vertex_0.y as f32, uv_s_vertex_0.z as f32]);
            uv_end.extend([uv_e_vertex_0.x as f32, uv_e_vertex_0.y as f32, uv_e_vertex_0.z as f32]);
            time_tile_received.push(alpha);
            m0.push(miss_0);
            m1.push(miss_1);
        }
    }

    for i in 0..n_segments_by_side {
        for j in 0..n_segments_by_side {
            let idx_0 = (j + i * n_vertices_per_segment) as u16;
            let idx_1 = (j + 1 + i * n_vertices_per_segment) as u16;
            let idx_2 = (j + (i + 1) * n_vertices_per_segment) as u16;
            let idx_3 = (j + 1 + (i + 1) * n_vertices_per_segment) as u16;

            idx_positions.push(off_idx_vertices + idx_0);
            idx_positions.push(off_idx_vertices + idx_1);
            idx_positions.push(off_idx_vertices + idx_2);

            idx_positions.push(off_idx_vertices + idx_1);
            idx_positions.push(off_idx_vertices + idx_3);
            idx_positions.push(off_idx_vertices + idx_2);
        }
    }
}

// This method computes positions and UVs of a healpix cells
use crate::cdshealpix;
use al_core::VertexArrayObject;
pub struct ImageSurvey {
    //color: Color,
    // The image survey texture buffer
    textures: ImageSurveyTextures,
    // Keep track of the cells in the FOV
    view: HEALPixCellsInView,

    // The projected vertices data
    // For WebGL2 wasm, the data are interleaved
    //#[cfg(feature = "webgl2")]
    //vertices: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 0) in vec3 position;
    position: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 1) in vec3 uv_start;
    uv_start: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 2) in vec3 uv_end;
    uv_end: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 3) in float time_tile_received;
    time_tile_received: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 4) in float m0;
    m0: Vec<f32>,
    //#[cfg(feature = "webgl1")]
    // layout (location = 5) in float m1;
    m1: Vec<f32>,

    idx_vertices: Vec<u16>,

    num_idx: usize,
    
    vao: VertexArrayObject,
    gl: WebGlContext,
}
use crate::camera::UserAction;
use crate::math::LonLatT;
use crate::utils;
use al_core::pixel::PixelType;
use web_sys::{WebGl2RenderingContext, WheelEvent};
impl ImageSurvey {
    fn new(
        config: HiPSConfig,
        gl: &WebGlContext,
        camera: &CameraViewPort,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<Self, JsValue> {
        let mut vao = VertexArrayObject::new(gl);

        // layout (location = 0) in vec2 lonlat;
        // layout (location = 1) in vec3 position;
        // layout (location = 2) in vec3 uv_start;
        // layout (location = 3) in vec3 uv_end;
        // layout (location = 4) in float time_tile_received;
        // layout (location = 5) in float m0;
        // layout (location = 6) in float m1;
        //let vertices = vec![0.0; MAX_NUM_FLOATS_TO_DRAW];
        //let indices = vec![0_u16; MAX_NUM_INDICES_TO_DRAW];
        
        //let vertices = vec![];
        let position = vec![];
        let uv_start = vec![];
        let uv_end = vec![];
        let time_tile_received = vec![];
        let m0 = vec![];
        let m1 = vec![];
        let idx_vertices = vec![];

        #[cfg(feature = "webgl2")]
        vao.bind_for_update()
            .add_array_buffer_single(
                3,
                "position",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            .add_array_buffer_single(
                3,
                "uv_start",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_start),
            )
            .add_array_buffer_single(
                3,
                "uv_end",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_end),
            )
            .add_array_buffer_single(
                1,
                "time_tile_received",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&time_tile_received),
            )
            .add_array_buffer_single(
                1,
                "m0",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m0),
            )
            .add_array_buffer_single(
                1,
                "m1",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m1),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            ).unbind();
        #[cfg(feature = "webgl1")]
        vao.bind_for_update()
            .add_array_buffer(
                3,
                "position",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&position),
            )
            .add_array_buffer(
                3,
                "uv_start",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_start),
            )
            .add_array_buffer(
                3,
                "uv_end",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&uv_end),
            )
            .add_array_buffer(
                1,
                "time_tile_received",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&time_tile_received),
            )
            .add_array_buffer(
                1,
                "m0",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m0),
            )
            .add_array_buffer(
                1,
                "m1",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<f32>(&m1),
            )
            // Set the element buffer
            .add_element_buffer(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u16>(&idx_vertices),
            )
            .unbind();

        let num_idx = 0;

        let textures = ImageSurveyTextures::new(gl, config, exec)?;
        let conf = textures.config();
        let view = HEALPixCellsInView::new(conf.get_tile_size(), conf.get_max_depth(), camera);

        let gl = gl.clone();

        Ok(ImageSurvey {
            //color,
            // The image survey texture buffer
            textures,
            // Keep track of the cells in the FOV
            view,

            num_idx,

            vao,

            gl,
            //vertices,

            position,
            uv_start,
            uv_end,
            time_tile_received,
            m0,
            m1,

            idx_vertices,
        })
    }

    pub fn longitude_reversed(&self) -> bool {
        self.textures.config().longitude_reversed
    }

    fn reset_frame(&mut self) {
        self.view.reset_frame();
    }

    pub fn read_pixel(&self, pos: &LonLatT<f64>) -> Result<PixelType, JsValue> {
        // Get the array of textures from that survey
        let pos_tex = self
            .textures
            .get_pixel_position_in_texture(pos, self.view.get_depth())?;

        let slice_idx = pos_tex.z as usize;
        let texture_array = self.textures.get_texture_array();
        texture_array[slice_idx].read_pixel(pos_tex.x, pos_tex.y)
    }

    pub fn recompute_vertices(&mut self, camera: &CameraViewPort) {
        let last_user_action = camera.get_last_user_action();
        match last_user_action {
            UserAction::Unzooming => {
                self.update_vertices::<UnZoom>(camera);
            }
            UserAction::Zooming => {
                self.update_vertices::<Zoom>(camera);
            }
            _ => {
                self.update_vertices::<Move>(camera);
            }
        }
    }

    /*#[cfg(feature = "webgl2")]
    fn update_vertices<P: Projection, T: RecomputeRasterizer>(&mut self, camera: &CameraViewPort) {
        let textures = T::get_textures_from_survey(camera, &mut self.view, &self.textures);

        self.vertices.clear();
        self.idx_vertices.clear();

        let survey_config = self.textures.config();

        for TextureToDraw { cell, starting_texture, ending_texture } in textures.iter() {
            let uv_0 = TileUVW::new(cell, starting_texture, survey_config);
            let uv_1 = TileUVW::new(cell, ending_texture, survey_config);
            let start_time = ending_texture.start_time();
            let miss_0 = starting_texture.is_missing() as f32;
            let miss_1 = ending_texture.is_missing() as f32;

            add_vertices_grid::<P, T>(
                &mut self.vertices,
                &mut self.idx_vertices,
                cell,
                &self.sphere_sub,
                &uv_0,
                &uv_1,
                miss_0,
                miss_1,
                start_time.as_millis(),
                camera,
            );
        }
        self.num_idx = self.idx_vertices.len();

        let mut vao = self.vao.bind_for_update();
        vao.update_array(0, WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.vertices))
            .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.idx_vertices));
    }*/

    //#[cfg(feature = "webgl1")]
    fn update_vertices<T: RecomputeRasterizer>(&mut self, camera: &CameraViewPort) {
        self.position.clear();
        self.uv_start.clear();
        self.uv_end.clear();
        self.time_tile_received.clear();
        self.m0.clear();
        self.m1.clear();
        self.idx_vertices.clear();

        let textures = T::get_textures_from_survey(&self.view, &self.textures);

        let survey_config = self.textures.config();
        let depth = self.view.get_depth();
        for (TextureToDraw { starting_texture, ending_texture }, cell) in textures.iter().zip(self.view.get_cells()) {
            let uv_0 = TileUVW::new(cell, starting_texture, survey_config);
            let uv_1 = TileUVW::new(cell, ending_texture, survey_config);
            let start_time = ending_texture.start_time();
            let miss_0 = starting_texture.is_missing() as f32;
            let miss_1 = ending_texture.is_missing() as f32;

            add_vertices_grid(
                cell,
                &mut self.position,
                &mut self.uv_start,
                &mut self.uv_end,
                &mut self.time_tile_received,
                &mut self.m0,
                &mut self.m1,
                &mut self.idx_vertices,
                //&cell,
                //&self.sphere_sub,
                &uv_0,
                &uv_1,
                miss_0,
                miss_1,
                start_time.as_millis(),
                camera,
            );
        }
        self.num_idx = self.idx_vertices.len();

        let mut vao = self.vao.bind_for_update();
        vao.update_array("position", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.position))
            .update_array("uv_start", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.uv_start))
            .update_array("uv_end", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.uv_end))
            .update_array("time_tile_received", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.time_tile_received))
            .update_array("m0", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.m0))
            .update_array("m1", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.m1))
            .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.idx_vertices));
    }

    /*fn set_positions<P>(&mut self, camera: &CameraViewPort, reversed_longitude: bool)
    where
        P: Projection
    {
        self.position.clear();
        self.idx_vertices.clear();
        let depth = self.view.get_depth();

        for cell in self.view.get_cells() {
            let num_subdivision = num_subdivision::<P>(cell);
            let n_segments_by_side: usize = 1 << num_subdivision;
            let n_vertices_per_segment = n_segments_by_side + 1;


        }

        let mut vao = self.vao.bind_for_update();
        vao.update_array("position", WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.position))
            .update_element_array(WebGl2RenderingContext::DYNAMIC_DRAW, VecData(&self.idx_vertices));
    }
*/
    fn refresh_view(&mut self, camera: &CameraViewPort) {
        let cfg = self.textures.config();

        let tile_size = cfg.get_tile_size();
        let max_depth = cfg.get_max_depth();
        let hips_frame = cfg.frame;

        self.view.refresh_cells(tile_size, max_depth, camera, hips_frame);
    }

    #[inline]
    pub fn get_textures(&self) -> &ImageSurveyTextures {
        &self.textures
    }

    pub fn get_textures_mut(&mut self) -> &mut ImageSurveyTextures {
        &mut self.textures
    }

    #[inline]
    pub fn get_view(&self) -> &HEALPixCellsInView {
        &self.view
    }
}
use cgmath::Matrix4;
// Identity matrix
const Id: &'static Matrix4<f64> = &Matrix4::new(
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
);
// Longitude reversed identity matrix
const IdR: &'static Matrix4<f64> = &Matrix4::new(
    -1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
);

use cgmath::Matrix;
use al_api::coo_system::CooBaseFloat;
impl Draw for ImageSurvey {
    fn draw<P: Projection>(
        &mut self,
        raytracer: &RayTracer,
        switch_from_raytrace_to_raster: bool,
        shaders: &mut ShaderManager,
        camera: &CameraViewPort,
        color: &HiPSColor,
        opacity: f32,
        colormaps: &Colormaps,
    ) {
        if !self.textures.is_ready() {
            // Do not render while the 12 base cell textures
            // are not loaded
            return;
        }

        // Get the coo system transformation matrix
        let selected_frame = camera.get_system();
        let hips_frame = self.textures
            .config()
            .get_frame();
        let C = selected_frame.to(&hips_frame);

        // Get whether the camera mode is longitude reversed
        let longitude_reversed = self.textures
            .config()
            .longitude_reversed;
        let RL = if longitude_reversed {
            IdR
        } else {
            Id
        };

        // Retrieve the model and inverse model matrix
        let w2v = C * (*camera.get_w2m()) * RL;
        let v2w = w2v.transpose();

        let raytracing = raytracer.is_rendering::<P>(camera);
        if raytracing {
            let shader = get_raytracer_shader::<P>(
                color,
                &self.gl,
                shaders,
                self.textures.config.tex_storing_integers,
                self.textures.config.tex_storing_unsigned_int,
            );

            let shader = shader.bind(&self.gl);
            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("model", &w2v)
                .attach_uniform("inv_model", &v2w)
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniforms_from(colormaps);

            raytracer.draw(&shader);
        } else {
            // The rasterizer has a buffer containing:
            // - The vertices of the HEALPix cells for the most refined survey
            // - The starting and ending uv for the blending animation
            // - The time for each HEALPix cell at which the animation begins
            //
            // Each of these data can be changed at different circumstances:
            // - The vertices are changed if:
            //     * new cells are added/removed (because new cells are added)
            //       to the previous frame.
            // - The UVs are changed if:
            //     * new cells are added/removed (because new cells are added)
            //     * there are new available tiles for the GPU
            // - The             
            let shader = get_raster_shader::<P>(
                color,
                &self.gl,
                shaders,
                self.textures.config.tex_storing_integers,
                self.textures.config.tex_storing_unsigned_int,
            )
            .bind(&self.gl);

            let vertices_recomputation_needed = self.view.is_there_new_cells_added() | self.textures.is_there_available_tiles() | switch_from_raytrace_to_raster;
            if vertices_recomputation_needed {
                self.recompute_vertices(camera);
            }

            shader
                .attach_uniforms_from(camera)
                .attach_uniforms_from(&self.textures)
                .attach_uniforms_from(color)
                .attach_uniform("model", &w2v)
                .attach_uniform("inv_model", &v2w)
                .attach_uniform("current_time", &utils::get_current_time())
                .attach_uniform("opacity", &opacity)
                .attach_uniforms_from(colormaps)
                .bind_vertex_array_object_ref(&self.vao)
                .draw_elements_with_i32(
                    WebGl2RenderingContext::TRIANGLES,
                    Some(self.num_idx as i32), 
                    WebGl2RenderingContext::UNSIGNED_SHORT, 
                    0
                );
        }
    }
}

use wasm_bindgen::JsValue;
//pub trait HiPS {
    /*fn create(
        self,
        gl: &WebGlContext,
        camera: &CameraViewPort,
        surveys: &ImageSurveys,
        exec: Rc<RefCell<TaskExecutor>>,
    ) -> Result<ImageSurvey, JsValue>;*/
    //fn color(&self, colormaps: &Colormaps) -> HiPSColor;
//}

use crate::{HiPSColor, SimpleHiPS};
use std::cell::RefCell;
use std::rc::Rc;
/*impl HiPS for SimpleHiPS {
    fn color(&self, colormaps: &Colormaps) -> Color {
        let color = match self.color.clone() {
            HiPSColor::Color => Color::Colored,
            HiPSColor::Grayscale2Color { color, transfer, k } => Color::Grayscale2Color {
                color,
                k,
                param: GrayscaleParameter {
                    h: transfer.into(),
                    min_value: self.properties.min_cutout.unwrap_or(0.0),
                    max_value: self.properties.max_cutout.unwrap_or(1.0),
                },
            },
            HiPSColor::Grayscale2Colormap {
                colormap,
                transfer,
                reversed,
            } => Color::Grayscale2Colormap {
                colormap: colormaps.get(&colormap),
                reversed,
                param: GrayscaleParameter {
                    h: transfer.into(),
                    min_value: self.properties.min_cutout.unwrap_or(0.0),
                    max_value: self.properties.max_cutout.unwrap_or(1.0),
                },
            },
        };

        color
    }
}*/


use al_api::hips::ImageSurveyMeta;

use crate::renderable::survey::view_on_surveys::HEALPixCellsInView;
pub(crate) type Url = String;
type LayerId = String;
pub struct ImageSurveys {
    // Surveys to query
    surveys: HashMap<Url, ImageSurvey>,
    // The meta data associated with a layer
    meta: HashMap<LayerId, ImageSurveyMeta>,
    // Hashmap between urls and layers
    pub urls: HashMap<LayerId, Url>,
    // Layers given in a specific order to draw
    layers: Vec<LayerId>,

    most_precise_survey: Url,

    raytracer: RayTracer,

    past_rendering_mode: RenderingMode,
    current_rendering_mode: RenderingMode,

    gl: WebGlContext,
}

#[derive(PartialEq, Eq)]
#[derive(Clone, Copy)]
enum RenderingMode {
    Raytrace,
    Rasterize
}

use crate::buffer::{FitsImage, HTMLImage, TileConfigType};
use crate::buffer::{ResolvedTiles, RetrievedImageType, TileResolved};


use crate::shaders::Colormaps;
use crate::Resources;

use crate::buffer::Tile;
use al_core::webgl_ctx::GlWrapper;
impl ImageSurveys {
    pub fn new<P: Projection>(
        gl: &WebGlContext,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) -> Self {
        let surveys = HashMap::new();
        let meta = HashMap::new();
        let urls = HashMap::new();
        let layers = Vec::new();

        // - The raytracer is a mesh covering the view. Each pixel of this mesh
        //   is unprojected to get its (ra, dec). Then we query ang2pix to get
        //   the HEALPix cell in which it is located.
        //   We get the texture from this cell and draw the pixel
        //   This mode of rendering is used for big FoVs
        let longitude_reversed = false;
        let raytracer = RayTracer::new::<P>(gl);
        let gl = gl.clone();
        let most_precise_survey = String::new();

        let past_rendering_mode = RenderingMode::Raytrace;
        let current_rendering_mode = RenderingMode::Raytrace;

        ImageSurveys {
            surveys,
            meta,
            urls,
            layers,

            most_precise_survey,

            raytracer,

            past_rendering_mode,
            current_rendering_mode,

            gl,
        }
    }

    pub fn last(&self) -> Option<&ImageSurvey> {
        if let Some(last_rendered_layer) = self.layers.last() {
            let url = self.urls.get(last_rendered_layer).expect("Url from layer name not found.");

            self.surveys.get(url)
        } else {
            None
        }
    }

    pub fn reset_frame(&mut self) {
        for survey in self.surveys.values_mut() {
            survey.reset_frame();
        }
    }

    pub fn read_pixel(&self, pos: &LonLatT<f64>, url: &Url) -> Result<PixelType, JsValue> {
        if let Some(survey) = self.surveys.get(url) {
            // Read the pixel from the first survey of layer
            survey.read_pixel(pos)
        } else {
            Err(JsValue::from_str("No survey found"))
        }
    }

    pub fn set_projection<P: Projection>(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
    ) {
        // Recompute the raytracer
        self.raytracer = RayTracer::new::<P>(&self.gl);
    }

    pub fn draw<P: Projection>(
        &mut self,
        camera: &CameraViewPort,
        shaders: &mut ShaderManager,
        colormaps: &Colormaps,
    ) {
        let raytracing = self.raytracer.is_rendering::<P>(camera);

        let mut switch_from_raytrace_to_raster = false;
        if raytracing {
            self.current_rendering_mode = RenderingMode::Raytrace;
        } else {
            self.current_rendering_mode = RenderingMode::Rasterize;
            if self.past_rendering_mode == RenderingMode::Raytrace {
                switch_from_raytrace_to_raster = true;
            }
        }

        // The first layer must be paint independently of its alpha channel
        self.gl.enable(WebGl2RenderingContext::BLEND);
        let raytracer = &self.raytracer;

        for layer in self.layers.iter() {
            let meta = self.meta.get(layer).expect("Meta should be found");
            if meta.visible() {
                let ImageSurveyMeta {
                    color,
                    opacity,
                    blend_cfg,
                } = meta;

                let url = self.urls.get(layer).expect("Url should be found");
                let survey = self.surveys.get_mut(url).unwrap();

                // Get the reverse longitude flag
                let longitude_reversed = survey.get_textures().config()
                    .longitude_reversed;
                if raytracing || !longitude_reversed {
                    self.gl.cull_face(WebGl2RenderingContext::BACK);
                } else {
                    self.gl.cull_face(WebGl2RenderingContext::FRONT);
                }

                blend_cfg.enable(&self.gl, || {
                    survey.draw::<P>(
                        raytracer,
                        switch_from_raytrace_to_raster,
                        shaders,
                        camera,
                        color,
                        *opacity,
                        colormaps,
                    );
                });
            }
        }

        self.gl.blend_func_separate(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE,
        );
        self.gl.disable(WebGl2RenderingContext::BLEND);

        self.past_rendering_mode = self.current_rendering_mode;
    }

    pub fn set_image_surveys(
        &mut self,
        hipses: Vec<SimpleHiPS>,
        gl: &WebGlContext,
        camera: &mut CameraViewPort,
        exec: Rc<RefCell<TaskExecutor>>,
        _colormaps: &Colormaps,
    ) -> Result<Vec<String>, JsValue> {
        // 1. Check if layer duplicated have been given
        for i in 0..hipses.len() {
            for j in 0..i {
                if hipses[i].get_layer() == hipses[j].get_layer() {
                    let layer = &hipses[i].get_layer();
                    return Err(JsValue::from_str(&format!("{:?} layer name are duplicates", layer)));
                }
            }
        }

        let mut new_survey_urls = Vec::new();

        let mut current_needed_surveys = HashSet::new();
        for hips in hipses.iter() {
            let url = hips.get_properties().get_url();
            current_needed_surveys.insert(url);
        }

        // Remove surveys that are not needed anymore
        self.surveys = self
            .surveys
            .drain()
            .filter(|(_, m)| current_needed_surveys.contains(&m.textures.config().root_url))
            .collect();
        
        // Create the new surveys
        let mut max_depth_among_surveys = 0;

        self.meta.clear();
        self.layers.clear();
        self.urls.clear();
        for SimpleHiPS { layer, properties, meta } in hipses.into_iter() {
            let config = HiPSConfig::new(gl, &properties)?;

            // Get the most precise survey from all the ones given
            let url = properties.get_url();
            let max_order = properties.get_max_order();
            if max_order > max_depth_among_surveys {
                max_depth_among_surveys = max_order;
                self.most_precise_survey = url.clone();
            }

            // Add the new surveys
            if !self.surveys.contains_key(&url) {
                let survey = ImageSurvey::new(config, gl, camera, exec.clone())?;
                self.surveys.insert(url.clone(), survey);
                new_survey_urls.push(url.clone());
            }

            self.meta.insert(layer.clone(), meta);
            self.urls.insert(layer.clone(), url);

            self.layers.push(layer);
        }
        al_core::log::log(&format!("List of surveys: {:?}\nmeta: {:?}\nlayers: {:?}\n", self.surveys.keys(), self.meta, self.layers));

        // Set the reversed longitude
        if let Some(survey) = self.last() {
            camera.set_longitude_reversed(survey.longitude_reversed());
        }

        Ok(new_survey_urls)
    }

    pub fn get_image_survey_color_cfg(&self, layer: &str) -> Result<ImageSurveyMeta, JsValue> {
        self.meta
            .get(layer).copied()
            .ok_or_else(|| JsValue::from(js_sys::Error::new("Survey not found")))
    }

    pub fn set_image_survey_color_cfg(&mut self, layer: String, meta: ImageSurveyMeta) -> Result<(), JsValue> {
        // Expect the image survey to be found in the hash map
        self.meta.insert(layer.clone(), meta)
            .ok_or_else(|| JsValue::from(js_sys::Error::new(&format!("{:?} layer not found", layer))))?;
        
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        let ready = self
            .surveys
            .iter()
            .map(|(_, survey)| survey.textures.is_ready())
            .fold(true, |acc, x| acc & x);

        ready
    }

    pub fn get_view(&self) -> Option<&HEALPixCellsInView> {
        if self.surveys.is_empty() {
            None
        } else {
            Some(
                self.surveys
                    .get(&self.most_precise_survey)
                    .unwrap()
                    .get_view(),
            )
        }
    }

    pub fn refresh_views(&mut self, camera: &CameraViewPort) {
        for survey in self.surveys.values_mut() {
            survey.refresh_view(camera);
        }
    }

    // Update the surveys by telling which tiles are available
    pub fn set_available_tiles(&mut self, available_tiles: &HashSet<Tile>) {
        for tile in available_tiles {
            let textures = &mut self
                .surveys
                .get_mut(&tile.root_url)
                .unwrap()
                .get_textures_mut();
            textures.register_available_tile(tile);
        }
    }

    // Update the surveys by adding to the surveys the tiles
    // that have been resolved
    pub fn add_resolved_tiles(&mut self, resolved_tiles: ResolvedTiles) {
        for (tile, result) in resolved_tiles.into_iter() {
            if let Some(survey) = self.surveys.get_mut(&tile.root_url) {
                let textures = survey.get_textures_mut();
                match result {
                    TileResolved::Missing { time_req } => {
                        let missing = true;
                        let tile_conf = &textures.config.tile_config;
                        match tile_conf {
                            TileConfigType::RGBA8U { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<RGBA8U>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                            TileConfigType::RGB8U { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<RGB8U>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                            TileConfigType::R32F { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<R32F>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                            #[cfg(feature = "webgl2")]
                            TileConfigType::R8UI { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<R8UI>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                            #[cfg(feature = "webgl2")]
                            TileConfigType::R16I { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<R16I>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                            #[cfg(feature = "webgl2")]
                            TileConfigType::R32I { config } => {
                                let missing_tile_image = config.get_default_tile();
                                textures.push::<Rc<ImageBuffer<R32I>>>(
                                    tile,
                                    missing_tile_image,
                                    time_req,
                                    missing,
                                );
                            }
                        }
                    }
                    TileResolved::Found { image, time_req } => {
                        let missing = false;
                        match image {
                            RetrievedImageType::FitsImageR32f { image } => {
                                // update the metadata
                                textures.config.set_fits_metadata(
                                    image.bscale,
                                    image.bzero,
                                    image.blank,
                                );
                                textures.push::<FitsImage<R32F>>(tile, image, time_req, missing);
                            }
                            #[cfg(feature = "webgl2")]
                            RetrievedImageType::FitsImageR32i { image } => {
                                textures.config.set_fits_metadata(
                                    image.bscale,
                                    image.bzero,
                                    image.blank,
                                );
                                textures.push::<FitsImage<R32I>>(tile, image, time_req, missing);
                            }
                            #[cfg(feature = "webgl2")]
                            RetrievedImageType::FitsImageR16i { image } => {
                                textures.config.set_fits_metadata(
                                    image.bscale,
                                    image.bzero,
                                    image.blank,
                                );
                                textures.push::<FitsImage<R16I>>(tile, image, time_req, missing);
                            }
                            #[cfg(feature = "webgl2")]
                            RetrievedImageType::FitsImageR8ui { image } => {
                                textures.config.set_fits_metadata(
                                    image.bscale,
                                    image.bzero,
                                    image.blank,
                                );
                                textures.push::<FitsImage<R8UI>>(tile, image, time_req, missing);
                            }
                            RetrievedImageType::PngImageRgba8u { image } => {
                                textures.push::<HTMLImage<RGBA8U>>(tile, image, time_req, missing);
                            }
                            RetrievedImageType::JpgImageRgb8u { image } => {
                                textures.push::<HTMLImage<RGB8U>>(tile, image, time_req, missing);
                            }
                        }
                    }
                }
            }
        }
    }

    // Accessors
    pub fn get(&self, root_url: &str) -> Option<&ImageSurvey> {
        self.surveys.get(root_url)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, String, ImageSurvey> {
        self.surveys.iter_mut()
    }
}

use crate::{async_task::TaskExecutor, buffer::HiPSConfig, shader::ShaderManager};
use std::collections::hash_map::IterMut;
