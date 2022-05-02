use crate::healpix_cell::HEALPixCell;
use al_core::format::ImageFormatType;
// A tile is described by an image survey
// and an HEALPix cell
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Tile {
    pub cell: HEALPixCell,
    pub root_url: String,
    pub format: ImageFormatType,
}
use crate::buffer::HiPSConfig;
impl Tile {
    pub fn new(cell: &HEALPixCell, config: &HiPSConfig) -> Self {
        Tile {
            cell: *cell,
            root_url: config.root_url.to_string(),
            format: config.format(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}