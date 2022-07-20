use cgmath::{Vector2, Vector3};
use num::{Float, Zero};

#[derive(Debug)]
pub struct UV<T: Float + Zero>([Vector2<T>; 4]);

use core::ops::Deref;
impl<T> Deref for UV<T>
where
    T: Float + Zero,
{
    type Target = [Vector2<T>; 4];

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

use crate::{
    healpix::cell::HEALPixCell,
    survey::{config::HiPSConfig, texture::Texture},
    utils,
};
pub struct TileUVW([Vector3<f32>; 4]);
impl TileUVW {
    // The texture cell passed must be a child of texture
    pub fn new(
        cell: &HEALPixCell,
        texture: &Texture,
        cfg: &HiPSConfig,
    ) -> TileUVW {
        // Index of the texture in the total set of textures
        let texture_idx = texture.idx();
        // Index of the slice of textures
        let num_textures_by_slice = cfg.num_textures_by_slice();
        let idx_slice = texture_idx / num_textures_by_slice;
        // Index of the texture in its slice
        let idx_in_slice = texture_idx % num_textures_by_slice;

        // Index of the column of the texture in its slice
        let num_textures_by_side_slice = cfg.num_textures_by_side_slice();
        let idx_col_in_slice = idx_in_slice / num_textures_by_side_slice;
        // Index of the row of the texture in its slice
        let idx_row_in_slice = idx_in_slice % num_textures_by_side_slice;

        // Row and column indexes of the tile in its texture
        let (idx_col_in_tex, idx_row_in_tex) = cell.offset_in_parent(texture.cell());

        // Offset in the slice in pixels
        /*let offset = Vector3::new(
            (idx_row_in_slice as i32) * texture_size + (idx_row_in_tex as i32) * tile_size,
            (idx_col_in_slice as i32) * texture_size + (idx_col_in_tex as i32) * tile_size,
            idx_slice,
        );*/

        let num_textures_by_side_slice_f32 = num_textures_by_side_slice as f32;
        let nside = (1 << (cell.depth() - texture.cell().depth())) as f32;
        let u = ((idx_row_in_slice as f32) + ((idx_row_in_tex as f32) / nside)) / num_textures_by_side_slice_f32;
        let v = ((idx_col_in_slice as f32) + ((idx_col_in_tex as f32) / nside)) / num_textures_by_side_slice_f32;

        let ds = 1_f32 / (num_textures_by_side_slice_f32 * nside);

        let w = (texture_idx as f32) / (num_textures_by_slice as f32);
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
impl<T> Index<TileCorner> for UV<T>
where
    T: Float + Zero,
{
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
