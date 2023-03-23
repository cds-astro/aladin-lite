pub type Url = String;

use super::request::{RequestType};
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType>;

    fn url(&self) -> &Url;
    fn id(&self) -> QueryId;
}

pub type QueryId = (&'static str, Url);

use al_core::image::format::ImageFormatType;
pub struct Tile {
    pub cell: HEALPixCell,
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_url: Url,
    // The total url of the query
    pub url: Url,
}

use crate::{healpix::cell::HEALPixCell, survey::config::HiPSConfig};
impl Tile {
    pub fn new(cell: &HEALPixCell, cfg: &HiPSConfig) -> Self {
        let hips_url = cfg.get_root_url().clone();
        let format = cfg.get_format();
        let ext = format.get_ext_file();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let url = format!(
            "{}/Norder{}/Dir{}/Npix{}.{}",
            hips_url, depth, dir_idx, idx, ext
        );

        // Check if this is a root tile
        /*let delta_depth = cfg.delta_depth();
        let texture_depth = if depth > delta_depth {
            depth - delta_depth
        } else {
            0
        };
        let is_root = texture_depth == 0;*/

        Tile {
            hips_url,
            url,
            cell: *cell,
            format,
        }
    }
}

use super::request::tile::TileRequest;
impl Query for Tile {
    type Request = TileRequest;

    fn url(&self) -> &Url {
        &self.url
    }

    fn id(&self) -> QueryId {
        ("Tile", self.url().to_string())
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

    fn id(&self) -> QueryId {
        ("Allsky", self.url().to_string())
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

    fn id(&self) -> QueryId {
        ("PixelMetadata", self.url().to_string())
    }
}

/* ---------------------------------- */
pub struct Moc {
    // The total url of the query
    pub url: Url,
    pub params: al_api::moc::MOC,
}
impl Moc {
    pub fn new(url: String, params: al_api::moc::MOC) -> Self {
        Moc {
            url,
            params,
        }
    }
}

use super::request::moc::MOCRequest;
impl Query for Moc {
    type Request = MOCRequest;

    fn url(&self) -> &Url {
        &self.url
    }

    fn id(&self) -> QueryId {
        ("MOC", self.url().to_string())
    }
}