use cgmath::{Vector2, Vector3};

pub type XYScreen<S> = Vector2<S>;
pub type XYNDC<S> = Vector2<S>;
pub type XYClip<S> = Vector2<S>;
pub type XYZWorld<S> = Vector3<S>;
pub type XYZModel<S> = Vector3<S>;

pub enum CooSpace {
    Screen,
    NDC,
    Clip,
    World,
    Model,
    LonLat,
}
