use crate::CameraViewPort;
use cgmath::BaseFloat;

pub struct Triangle<'a, S>
where
    S: BaseFloat,
{
    v1: &'a [S; 2],
    v2: &'a [S; 2],
    v3: &'a [S; 2],
}

impl<'a, S> Triangle<'a, S>
where
    S: BaseFloat,
{
    pub fn new(v1: &'a [S; 2], v2: &'a [S; 2], v3: &'a [S; 2]) -> Self {
        Self { v1, v2, v3 }
    }

    pub fn is_valid(&self, camera: &CameraViewPort) -> bool {
        let tri_ccw = self.is_ccw();
        let reversed_longitude = camera.get_longitude_reversed();

        (!reversed_longitude && tri_ccw) || (reversed_longitude && !tri_ccw)
    }

    pub fn is_ccw(&self) -> bool {
        crate::math::utils::ccw_tri(&self.v1, &self.v2, &self.v3)
    }
}
