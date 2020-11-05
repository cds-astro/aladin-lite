pub mod grid;
pub mod projection;

pub mod catalog;

pub mod angle;
pub mod uv;
pub use angle::{Angle, ArcDeg, ArcMin, ArcSec, FormatType, SerializeToString};

mod ray_tracer;

use ray_tracer::RayTracer;

mod triangulation;
use triangulation::Triangulation;

pub mod view_on_surveys;
pub use view_on_surveys::HEALPixCells;
use view_on_surveys::HEALPixCellsInView;

pub mod image_survey;
pub use image_survey::{
    ImageSurvey, Move, RecomputeRasterizer, TexturesToDraw, UnZoom, Zoom, MAX_NUM_VERTICES_TO_DRAW,
};

pub use catalog::Manager;
pub use grid::ProjetedGrid;
