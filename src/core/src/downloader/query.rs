pub type Url = String;

use super::request::{
    RequestType2,
    Resource
};
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType2>;

    fn url(&self) -> &Url;
}

pub struct Tile<'a, 'b> {
    pub cell: &'a HEALPixCell,
    pub cfg: &'b HiPSConfig,

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

impl<'a, 'b> Tile<'a, 'b> {
    pub fn new(cell: &'a HEALPixCell, cfg: &'b HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let ext = cfg.get_format().get_ext_file();
    
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
            cell,
            cfg,
        }
    }
}

use super::request::tile::TileRequest;
impl<'a, 'b> Query for Tile<'a, 'b> {
    type Request = TileRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}