pub type Url = String;

use super::request::RequestType;
pub trait Query: Sized {
    type Request: From<Self> + Into<RequestType>;

    fn id(&self) -> &QueryId;
}
pub type QueryId = String;

use crate::browser_support::BrowserFeaturesSupport;
use crate::healpix::cell::HEALPixFreqCell;
use al_api::hips::DataproductType;
use al_core::image::format::ImageFormatType;

/// Description of a cell to query
#[derive(Clone, PartialEq, Eq)]
pub enum CellDesc {
    HiPS2D {
        // A description of the tile in space
        cell: HEALPixCell,
        // Size of the tile requested
        tile_size: u32,
    },
    HiPS3D {
        // A description of the tile in space and frequency
        cell: HEALPixFreqCell,
        // Size of the tile requested
        tile_size: u32,
        // Depth of the cubic tile
        tile_depth: u32,
    },
    HiPSCube {
        // A description of the tile in space
        cell: HEALPixCell,
        // size of the tile requested
        tile_size: u32,
        // The channel number to query
        channel: u32,
    },
}

impl CellDesc {
    /*fn get_size(&self) -> (u32, u32, u32) {
        match self {
            Self::HiPS2D { tile_size, .. } => (*tile_size, *tile_size, 1),
            Self::HiPSCube { tile_size, .. } => (*tile_size, *tile_size, 1),
            Self::HiPS3D {
                tile_size,
                tile_depth,
                ..
            } => (*tile_size, *tile_size, *tile_depth),
        }
    }*/

    pub fn get_hpx(&self) -> &HEALPixCell {
        match self {
            Self::HiPS2D { cell, .. } => cell,
            Self::HiPS3D { cell, .. } => &cell.hpx,
            Self::HiPSCube { cell, .. } => cell,
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Tile {
    pub cell: CellDesc,
    pub format: ImageFormatType,
    // The root url of the HiPS
    pub hips_cdid: CreatorDid,
    // The total url of the query
    pub url: Url,
    pub credentials: RequestCredentials,
    pub mode: RequestMode,
    pub id: QueryId,
    pub create_bitmap_support: bool,
}

use crate::healpix::cell::HEALPixCell;
use crate::renderable::hips::config::HiPSConfig;
use crate::renderable::CreatorDid;
use crate::tile_fetcher::HiPSLocalFiles;
use web_sys::{RequestCredentials, RequestMode};
impl Tile {
    pub fn new(
        cell: &HEALPixCell,
        cfg: &HiPSConfig,
        browser_support: &BrowserFeaturesSupport,
    ) -> Self {
        let hips_cdid = cfg.get_creator_did();
        let hips_url = cfg.get_root_url();
        let format = cfg.get_format();
        let credentials = cfg.get_request_credentials();
        let mode = cfg.get_request_mode();

        let ext = format.get_ext_file();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let url = format!("{hips_url}/Norder{depth}/Dir{dir_idx}/Npix{idx}.{ext}");

        let id = format!("{}_{}_{}_{}", hips_cdid, depth, idx, ext);

        let tile_size = cfg.get_tile_size() as u32;
        Tile {
            hips_cdid: hips_cdid.to_string(),
            url,
            cell: CellDesc::HiPS2D {
                cell: *cell,
                tile_size,
            },
            format,
            credentials,
            mode,
            id,
            create_bitmap_support: browser_support.create_image_bitmap,
        }
    }

    pub fn new_with_channel(
        cell: &HEALPixCell,
        channel: u32,
        cfg: &HiPSConfig,
        browser_support: &BrowserFeaturesSupport,
    ) -> Self {
        let hips_cdid = cfg.get_creator_did();
        let hips_url = cfg.get_root_url();
        let format = cfg.get_format();
        let credentials = cfg.get_request_credentials();
        let mode = cfg.get_request_mode();

        let ext = format.get_ext_file();

        let HEALPixCell(depth, idx) = *cell;

        let dir_idx = (idx / 10000) * 10000;

        let url = format!("{hips_url}/Norder{depth}/Dir{dir_idx}/Npix{idx}_{channel:?}.{ext}");

        let id = format!("{}_{}_{}_{}_{}", hips_cdid, depth, idx, channel, ext);

        let tile_size = cfg.get_tile_size() as u32;
        Tile {
            hips_cdid: hips_cdid.to_string(),
            url,
            cell: CellDesc::HiPSCube {
                cell: *cell,
                tile_size,
                channel,
            },
            format,
            credentials,
            mode,
            id,
            create_bitmap_support: browser_support.create_image_bitmap,
        }
    }

    pub fn new_cubic(
        hpx_f_cell: &HEALPixFreqCell,
        cfg: &HiPSConfig,
        browser_support: &BrowserFeaturesSupport,
    ) -> Self {
        let hips_cdid = cfg.get_creator_did();
        let hips_url = cfg.get_root_url();
        let format = cfg.get_format();
        let credentials = cfg.get_request_credentials();
        let mode = cfg.get_request_mode();

        let ext = format.get_ext_file();

        // f hash at order_f

        let HEALPixFreqCell {
            hpx: HEALPixCell(k, n),
            f_hash: m,
            f_depth: l,
        } = *hpx_f_cell;

        let d = (n / 10000) * 10000;
        let e = (m / 10) * 10;

        let url = format!("{hips_url}/Norder{k}_{l}/Dir{d}_{e}/Npix{n}_{m}.{ext}");

        let id = format!("{hips_cdid}_{k}_{l}_{n}_{m}_{ext}");

        let tile_size = cfg.get_tile_size() as u32;
        let tile_depth = cfg.tile_depth.unwrap_or(1) as u32;
        Tile {
            hips_cdid: hips_cdid.to_string(),
            url,
            cell: CellDesc::HiPS3D {
                cell: hpx_f_cell.clone(),
                tile_size,
                tile_depth,
            },
            format,
            credentials,
            mode,
            id,
            create_bitmap_support: browser_support.create_image_bitmap,
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
