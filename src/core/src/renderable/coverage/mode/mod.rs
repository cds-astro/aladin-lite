use crate::healpix::cell::CellVertices;
use crate::renderable::coverage::HEALPixCell;
use crate::HEALPixCoverage;

pub mod edge;
pub mod filled;
pub mod perimeter;

pub(super) trait RenderMode {
    fn build(moc: &HEALPixCoverage) -> Vec<Node>;
}

#[derive(Debug)]
pub struct Node {
    pub cell: HEALPixCell,
    pub vertices: Option<CellVertices>,
}
