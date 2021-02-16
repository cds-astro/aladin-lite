pub mod viewport;
pub use viewport::{CameraViewPort, UserAction};

pub mod fov_vertices;
pub use fov_vertices::FieldOfViewVertices;
pub use fov_vertices::{ModelCoord, NormalizedDeviceCoord, WorldCoord};

use cgmath::Matrix4;
/*
pub const J2000_TO_GALACTIC: Matrix4<f64> = Matrix4::new(
    -0.867_666_1,
    -0.054_875_56,
    0.494_109_42,
    0.0,

    -0.198_076_37,
    -0.873_437_1,
    -0.444_829_64,
    0.0,

    0.455_983_8,
    -0.483_835,
    0.746_982_2,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);
use cgmath::SquareMatrix;
pub const GALACTIC_TO_J2000: Matrix4<f64> = J2000_TO_GALACTIC.invert().unwrap();
*/
pub const GALACTIC_TO_J2000: Matrix4<f64> = Matrix4::new(
    -0.8676661489811610,
    -0.1980763734646737,
    0.4559837762325372,
    0.0,
    
    -0.0548755604024359, 
    -0.8734370902479237, 
    -0.4838350155267381,
    0.0,

    0.4941094279435681,
    -0.4448296299195045,
    0.7469822444763707,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);
 
pub const J2000_TO_GALACTIC: Matrix4<f64> = Matrix4::new(
    -0.4838350155267381,
    0.7469822444763707,
    0.4559837762325372,
    0.0,

    -0.0548755604024359,
    0.4941094279435681,
    -0.8676661489811610,
    0.0,

    -0.873437090247923,
    -0.4448296299195045,
    -0.1980763734646737,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);