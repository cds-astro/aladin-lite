use cgmath::{Vector2, Vector3};

#[derive(Debug)]
pub struct UV<T>([Vector2<T>; 4]);

use core::ops::Deref;
impl<T> Deref for UV<T> {
    type Target = [Vector2<T>; 4];

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

use crate::{
    healpix::cell::HEALPixCell,
    survey::{config::HiPSConfig, texture::Texture},
};
pub struct TileUVW([Vector3<f32>; 4]);
impl TileUVW {
    // The texture cell passed must be a child of texture
    pub fn new(cell: &HEALPixCell, texture: &Texture, cfg: &HiPSConfig) -> TileUVW {
        // Index of the texture in the total set of textures
        let texture_idx = texture.idx();

        // Row and column indexes of the tile in its texture
        let (idx_col_in_tex, idx_row_in_tex) = cell.offset_in_parent(texture.cell());

        let nside = (1 << (cell.depth() - texture.cell().depth())) as f32;
        let u = (idx_row_in_tex as f32) / nside;
        let v = (idx_col_in_tex as f32) / nside;

        let ds = 1_f32 / nside;

        let w = texture_idx as f32;
        TileUVW([
            Vector3::new(u, v, w),
            Vector3::new(u + ds, v, w),
            Vector3::new(u, v + ds, w),
            Vector3::new(u + ds, v + ds, w),
        ])
    }
}
#[allow(dead_code)]
pub enum TileCorner {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}
use std::ops::Index;
impl<T> Index<TileCorner> for UV<T> {
    type Output = Vector2<T>;

    fn index(&self, corner: TileCorner) -> &Self::Output {
        match corner {
            TileCorner::BottomLeft => &self.0[0],
            TileCorner::BottomRight => &self.0[1],
            TileCorner::TopLeft => &self.0[2],
            TileCorner::TopRight => &self.0[3],
        }
    }
}
impl Index<TileCorner> for TileUVW {
    type Output = Vector3<f32>;

    fn index(&self, corner: TileCorner) -> &Self::Output {
        match corner {
            TileCorner::BottomLeft => &self.0[0],
            TileCorner::BottomRight => &self.0[1],
            TileCorner::TopLeft => &self.0[2],
            TileCorner::TopRight => &self.0[3],
        }
    }
}
