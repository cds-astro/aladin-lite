pub type Url = String;

use super::request::{RequestType};
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType>;

    fn url(&self) -> &Url;
}

use al_core::image::format::ImageFormatType;
pub struct Tile {
    pub cell: HEALPixCell,
    pub format: ImageFormatType,
    pub system: CooSystem,
    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

use crate::{healpix::cell::HEALPixCell, survey::config::HiPSConfig};
use al_api::coo_system::CooSystem;
impl Tile {
    pub fn new(cell: &HEALPixCell, cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let format = cfg.get_format();
        let ext = format.get_ext_file();
        let system = cfg.get_frame();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let url = format!(
            "{}/Norder{}/Dir{}/Npix{}.{}",
            hips_url, depth, dir_idx, idx, ext
        );

        Tile {
            hips_url,
            url,
            cell: *cell,
            format,
            system
        }
    }
}

use super::request::tile::TileRequest;
impl Query for Tile {
    type Request = TileRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}

/* ---------------------------------- */
pub struct Allsky {
    pub format: ImageFormatType,
    pub tile_size: i32,
    pub texture_size: i32,
    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

impl Allsky {
    pub fn new(cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let tile_size = cfg.get_tile_size();
        let texture_size = cfg.get_texture_size();
        let format = cfg.get_format();
        let ext = format.get_ext_file();

        let url = format!("{}/Norder3/Allsky.{}", hips_url, ext);

        Allsky {
            tile_size,
            texture_size,
            hips_url,
            url,
            format,
        }
    }
}

use super::request::allsky::AllskyRequest;
impl Query for Allsky {
    type Request = AllskyRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}


/* ---------------------------------- */
pub struct PixelMetadata {
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

impl PixelMetadata {
    pub fn new(cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let format = cfg.get_format();
        let ext = format.get_ext_file();

        let url = format!("{}/Norder3/Allsky.{}", hips_url, ext);

        PixelMetadata {
            hips_url,
            url,
            format,
        }
    }
}

use super::request::blank::PixelMetadataRequest;
impl Query for PixelMetadata {
    type Request = PixelMetadataRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}

/* ---------------------------------- */
pub struct MOC {
    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

impl MOC {
    pub fn new(cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().to_string();
        let url = format!("{}/Moc.fits", hips_url);

        MOC {
            hips_url,
            url,
        }
    }
}

use super::request::moc::MOCRequest;
impl Query for MOC {
    type Request = MOCRequest;

    fn url(&self) -> &Url {
        &self.url
    }
}