pub type Url = String;

use super::request::{
    RequestType2,
    Resource
};
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType2>;

    fn url(&self) -> &Url;
}

use al_core::format::ImageFormatType;
pub struct Tile{
    pub cell: HEALPixCell,
    pub format: ImageFormatType,

    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

use crate:: {
    healpix::cell::HEALPixCell,
    survey::config::HiPSConfig,
    time::Time
};

impl Tile {
    pub fn new(cell: &HEALPixCell, cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let format = cfg.get_format();
        let ext = format.get_ext_file();
    
        let HEALPixCell(depth, idx) = *cell;
    
        let dir_idx = (idx / 10000) * 10000;
    
        let url = format!(
            "{}/Norder{}/Dir{}/Npix{}.{}",
            hips_url,
            depth,
            dir_idx,
            idx,
            ext
        );
    
        Tile {
            hips_url,
            url,
            cell: *cell,
            format,
        }
    }

    pub fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}

use super::request::tile::TileRequest;
impl Query for Tile {
    type Request = TileRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}