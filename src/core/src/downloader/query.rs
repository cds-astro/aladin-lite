pub type Url = String;

use super::request::RequestType;
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType>;

    fn id(&self) -> &QueryId;
}

pub type QueryId = String;

use al_api::hips::DataproductType;
use al_core::image::format::ImageFormatType;

#[derive(Eq, PartialEq, Clone)]
pub struct Tile {
    pub cell: HEALPixCell,
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub size: u32, // size of the tile requested
    pub credentials: RequestCredentials,
    pub mode: RequestMode,
    pub id: QueryId,
    pub channel: Option<u32>,
}

/*pub enum  {

}*/

use crate::healpix::cell::HEALPixCell;
use crate::renderable::hips::config::HiPSConfig;
use crate::renderable::CreatorDid;
use crate::tile_fetcher::HiPSLocalFiles;
use web_sys::{RequestCredentials, RequestMode};
impl Tile {
    pub fn new(cell: &HEALPixCell, channel: Option<u32>, cfg: &HiPSConfig) -> Self {
        let hips_cdid = cfg.get_creator_did();
        let hips_url = cfg.get_root_url();
        let format = cfg.get_format();
        let credentials = cfg.get_request_credentials();
        let mode = cfg.get_request_mode();

        let ext = format.get_ext_file();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let mut url = format!("{hips_url}/Norder{depth}/Dir{dir_idx}/Npix{idx}");

        // handle cube case
        if let Some(channel) = channel {
            if channel > 0 {
                url.push_str(&format!("_{channel:?}"));
            }
        }

        // add the tile format
        url.push_str(&format!(".{ext}"));

        let id = format!(
            "{}{}{}{}{}",
            hips_cdid,
            depth,
            idx,
            channel.unwrap_or(0),
            ext
        );

        let size = cfg.get_tile_size() as u32;
        Tile {
            hips_cdid: hips_cdid.to_string(),
            url,
            cell: *cell,
            format,
            credentials,
            mode,
            id,
            channel,
            size,
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
    pub allsky_tile_size: i32,
    pub channel: Option<u32>,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub credentials: RequestCredentials,
    pub mode: RequestMode,
    pub id: QueryId,
}

impl Allsky {
    pub fn new(cfg: &HiPSConfig, channel: Option<u32>) -> Self {
        let hips_cdid = cfg.get_creator_did().to_string();
        let allsky_tile_size = cfg.allsky_tile_size();

        let tile_size = cfg.get_tile_size();

        let format = cfg.get_format();
        let ext = format.get_ext_file();
        let credentials = cfg.get_request_credentials();
        let mode = cfg.get_request_mode();

        let mut url = format!("{}/Norder3/Allsky", cfg.get_root_url());

        // handle cube case
        if let Some(channel) = channel {
            if channel > 0 {
                url.push_str(&format!("_{channel:?}"));
            }
        }

        // add the tile format
        url.push_str(&format!(".{ext}"));

        let id = format!(
            "{}Allsky{}{}",
            cfg.get_creator_did(),
            ext,
            channel.unwrap_or(0)
        );

        Allsky {
            tile_size,
            allsky_tile_size,
            hips_cdid,
            url,
            format,
            id,
            credentials,
            mode,
            channel,
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
use al_api::moc::MOCOptions;

pub struct Moc {
    // The total url of the query
    pub url: Url,
    pub mode: RequestMode,
    pub credentials: RequestCredentials,
    pub params: MOCOptions,
    pub hips_cdid: CreatorDid,
    pub dataproduct_type: DataproductType,
}
use std::collections::HashMap;
impl Moc {
    pub fn new(
        cfg: &HiPSConfig,
        hips_local_files: &HashMap<String, HiPSLocalFiles>,
        params: MOCOptions,
    ) -> Self {
        // Try to fetch the MOC
        let hips_cdid = cfg.get_creator_did();
        let url = if let Some(local_hips) = hips_local_files.get(hips_cdid) {
            if let Ok(url) =
                web_sys::Url::create_object_url_with_blob(local_hips.get_moc().as_ref())
            {
                url
            } else {
                format!("{}/Moc.fits", cfg.get_root_url())
            }
        } else {
            format!("{}/Moc.fits", cfg.get_root_url())
        };

        let mode = cfg.get_request_mode();
        let credentials = cfg.get_request_credentials();
        let hips_cdid = cfg.get_creator_did().to_string();
        let dataproduct_type = cfg.dataproduct_type;

        Moc {
            url,
            params,
            hips_cdid,
            mode,
            credentials,
            dataproduct_type,
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
