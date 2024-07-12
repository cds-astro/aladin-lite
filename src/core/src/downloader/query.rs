pub type Url = String;

use super::request::RequestType;
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType>;

    fn id(&self) -> &QueryId;
}

pub type QueryId = String;

use al_core::image::format::ImageFormatType;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Tile {
    pub cell: HEALPixCell,
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub id: QueryId,
}

use crate::renderable::CreatorDid;
use crate::{healpix::cell::HEALPixCell, survey::config::HiPSConfig};
impl Tile {
    pub fn new(
        cell: &HEALPixCell,
        hips_cdid: String,
        hips_url: String,
        format: ImageFormatType,
    ) -> Self {
        let ext = format.get_ext_file();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let url = format!(
            "{}/Norder{}/Dir{}/Npix{}.{}",
            hips_url, depth, dir_idx, idx, ext
        );

        let id = format!("{}{}{}{}", hips_cdid, depth, idx, ext);

        Tile {
            hips_cdid,
            url,
            cell: *cell,
            format,
            id,
        }
    }
}

use super::request::tile::TileRequest;
impl Query for Tile {
    type Request = TileRequest;

    fn id(&self) -> &QueryId {
        &self.id
    }
}

/* ---------------------------------- */
pub struct Allsky {
    pub format: ImageFormatType,
    pub tile_size: i32,
    pub texture_size: i32,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub id: QueryId,
}

impl Allsky {
    pub fn new(cfg: &HiPSConfig) -> Self {
        let hips_cdid = cfg.get_creator_did().to_string();
        let tile_size = cfg.get_tile_size();
        let texture_size = cfg.get_texture_size();
        let format = cfg.get_format();
        let ext = format.get_ext_file();

        let url = format!("{}/Norder3/Allsky.{}", cfg.get_root_url(), ext);

        let id = format!("{}Allsky{}", cfg.get_creator_did(), ext);

        Allsky {
            tile_size,
            texture_size,
            hips_cdid,
            url,
            format,
            id,
        }
    }
}

use super::request::allsky::AllskyRequest;
impl Query for Allsky {
    type Request = AllskyRequest;

    fn id(&self) -> &QueryId {
        &self.id
    }
}

/* ---------------------------------- */
pub struct PixelMetadata {
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub id: QueryId,
}

impl PixelMetadata {
    pub fn new(cfg: &HiPSConfig) -> Self {
        let hips_cdid = cfg.get_creator_did().to_string();
        let format = cfg.get_format();
        let ext = format.get_ext_file();

        let url = format!("{}/Norder3/Allsky.{}", cfg.get_root_url(), ext);

        let id = format!("{}Allsky{}", hips_cdid, ext);
        PixelMetadata {
            hips_cdid,
            url,
            format,
            id,
        }
    }
}

use super::request::blank::PixelMetadataRequest;
impl Query for PixelMetadata {
    type Request = PixelMetadataRequest;

    fn id(&self) -> &QueryId {
        &self.id
    }
}

/* ---------------------------------- */
pub struct Moc {
    // The total url of the query
    pub url: Url,
    pub params: al_api::moc::MOC,
    pub hips_cdid: CreatorDid,
}
impl Moc {
    pub fn new(url: String, hips_cdid: CreatorDid, params: al_api::moc::MOC) -> Self {
        Moc {
            url,
            params,
            hips_cdid,
        }
    }
}

use super::request::moc::MOCRequest;
impl Query for Moc {
    type Request = MOCRequest;

    fn id(&self) -> &QueryId {
        &self.url
    }
}
