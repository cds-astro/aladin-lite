

pub mod projection;
pub mod grid;

pub mod catalog;

pub mod uv;

pub mod text;
pub use text::TextManager;

pub mod angle;
pub use angle::{
    ArcDeg,
    ArcMin,
    ArcSec,
    Angle,
    SerializeToString,
    FormatType,
};

mod ray_tracer;

use ray_tracer::RayTracer;

mod rasterizer;
use rasterizer::Rasterizer;

mod triangulation;
use triangulation::Triangulation;

pub mod view_on_surveys;
use view_on_surveys::{HEALPixCellsInView, NewHEALPixCells, get_cells_in_camera};
pub use view_on_surveys::HEALPixCells;

pub mod image_survey;
pub use image_survey::{
    RecomputeRasterizer,
    Zoom,
    UnZoom,
    Move,
    TexturesToDraw,
    MAX_NUM_VERTICES_TO_DRAW,
    ImageSurvey,
};

pub use catalog::Manager;
pub use grid::ProjetedGrid;
/*
pub struct Renderable<T>
where T: Mesh + DisableDrawing {
    //scale_mat: cgmath::Matrix4::<f32>,
    //rotation_mat: cgmath::Matrix4::<f32>,
    //translation_mat: cgmath::Matrix4::<f32>,

    mesh: T,

    gl: WebGl2Context,
}

use cgmath;
impl<T> Renderable<T>
where T: Mesh + DisableDrawing {
    pub fn new(gl: &WebGl2Context, shaders: &HashMap<&'static str, Shader>, mut mesh: T) -> Renderable<T> {
        let shader = mesh.get_shader(shaders);
        shader.bind(gl);
        mesh.create_buffers(gl);

        //let scale_mat = cgmath::Matrix4::identity();
        //let rotation_mat = cgmath::Matrix4::identity();
        //let translation_mat = cgmath::Matrix4::identity();

        let gl = gl.clone();
        Renderable {
            // And its submatrices
            //scale_mat,
            //rotation_mat,
            //translation_mat,

            mesh,
            gl,
        }
    }
}
*/