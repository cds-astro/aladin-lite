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

use crate::{buffer::HiPSConfig, buffer::Texture, healpix_cell::HEALPixCell, utils};
pub struct TileUVW([Vector3<f32>; 4]);
impl TileUVW {
    // The texture cell passed must be a child of texture
    pub fn new(
        child_texture_cell: &HEALPixCell,
        texture: &Texture,
        config: &HiPSConfig,
    ) -> TileUVW {
        let HEALPixCell(depth, idx) = *child_texture_cell;
        let HEALPixCell(parent_depth, parent_idx) = *texture.cell();

        let idx_off = parent_idx << (2 * (depth - parent_depth));

        assert!(idx >= idx_off);
        assert!(depth >= parent_depth);
        let nside = (1 << (depth - parent_depth)) as f32;

        let (x, y) = utils::unmortonize(idx - idx_off);
        let x = x as f32;
        let y = y as f32;
        assert!(x < nside);
        assert!(y < nside);

        let parent_idx_texture = texture.idx();
        let idx_texture = (parent_idx_texture / config.num_textures_by_slice()) as f32;
        let parent_idx_in_texture = parent_idx_texture % config.num_textures_by_slice();

        let parent_idx_row = (parent_idx_in_texture / config.num_textures_by_side_slice()) as f32; // in [0; 7]
        let parent_idx_col = (parent_idx_in_texture % config.num_textures_by_side_slice()) as f32; // in [0; 7]

        let num_textures_by_side_slice_f32 = config.num_textures_by_side_slice() as f32;
        let u = (parent_idx_col + (y / nside)) / num_textures_by_side_slice_f32;
        let v = (parent_idx_row + (x / nside)) / num_textures_by_side_slice_f32;

        let ds = 1_f32 / (num_textures_by_side_slice_f32 * nside);

        TileUVW([
            Vector3::new(u, v, idx_texture),
            Vector3::new(u + ds, v, idx_texture),
            Vector3::new(u, v + ds, idx_texture),
            Vector3::new(u + ds, v + ds, idx_texture),
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
