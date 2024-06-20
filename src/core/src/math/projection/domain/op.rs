use super::sdf::ProjDef;
use crate::math::projection::XYClip;
use cgmath::Vector2;

pub struct Scale<T>
where
    T: ProjDef,
{
    pub scale: Vector2<f64>,
    pub def: T,
}

impl<T> ProjDef for Scale<T>
where
    T: ProjDef,
{
    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        self.def
            .sdf(&Vector2::new(xy.x / self.scale.x, xy.y / self.scale.y))
    }
}

pub struct Translate<T>
where
    T: ProjDef,
{
    pub off: Vector2<f64>,
    pub def: T,
}

impl<T> ProjDef for Translate<T>
where
    T: ProjDef,
{
    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        self.def.sdf(&(*xy - self.off))
    }
}

// Union of two projection domain sdf
pub struct Union<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    sdf1: T,
    sdf2: U,
}

impl<T, U> Union<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    pub fn new(sdf1: T, sdf2: U) -> Self {
        Self { sdf1, sdf2 }
    }
}

impl<T, U> ProjDef for Union<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let s1 = self.sdf1.sdf(xy);
        let s2 = self.sdf2.sdf(xy);

        // intersection
        s1.min(s2)
    }
}

pub struct Inter<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    sdf1: T,
    sdf2: U,
}

impl<T, U> Inter<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    pub fn new(sdf1: T, sdf2: U) -> Self {
        Self { sdf1, sdf2 }
    }
}

impl<T, U> ProjDef for Inter<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let s1 = self.sdf1.sdf(xy);
        let s2 = self.sdf2.sdf(xy);

        // intersection
        s1.max(s2)
    }
}

pub struct Diff<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    sdf1: T,
    sdf2: U,
}

impl<T, U> Diff<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    pub fn new(sdf1: T, sdf2: U) -> Self {
        Self { sdf1, sdf2 }
    }
}

impl<T, U> ProjDef for Diff<T, U>
where
    T: ProjDef,
    U: ProjDef,
{
    /// Signed distance function to the definition domain region
    fn sdf(&self, xy: &XYClip<f64>) -> f64 {
        let s1 = self.sdf1.sdf(xy);
        let s2 = self.sdf2.sdf(xy);

        // intersection
        (-s2).max(s1)
    }
}
