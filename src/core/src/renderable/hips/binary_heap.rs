use crate::renderable::hips::d2::texture::HpxTex;
use crate::renderable::hips::d3::texture::HpxFreqTex;
use crate::time::Time;
use crate::Abort;
use crate::HEALPixCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
pub struct Tile<C> {
    cell: C,
    time_request: Time,
}

impl<C> Tile<C> {
    pub fn reset_time(&mut self) {
        self.time_request = Time::now();
    }

    #[inline(always)]
    pub fn cell(&self) -> &C {
        &self.cell
    }
}

impl Tile<HEALPixCell> {
    pub fn is_root(&self) -> bool {
        self.cell.is_root()
    }
}

impl<C> PartialEq for Tile<C>
where
    C: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}

impl<C> Eq for Tile<C> where C: PartialEq {}

// Ordering based on the time the tile has been requested
impl<C> PartialOrd for Tile<C>
where
    C: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for Tile<C>
where
    C: PartialEq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .time_request
            .partial_cmp(&self.time_request)
            .unwrap_abort()
    }
}

impl From<&HpxTex> for Tile<HEALPixCell> {
    fn from(tex: &HpxTex) -> Self {
        let time_request = tex.time_request;
        let cell = tex.cell;

        Self { cell, time_request }
    }
}
impl From<&HpxFreqTex> for Tile<HEALPixCell> {
    fn from(tex: &HpxFreqTex) -> Self {
        let time_request = tex.time_request;
        let cell = tex.cell;

        Self { cell, time_request }
    }
}

pub struct TileHeap<C>(BinaryHeap<Tile<C>>);

impl<C> TileHeap<C> {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<C> TileHeap<C>
where
    C: PartialEq,
{
    pub fn with_capacity(cap: usize) -> Self {
        Self(BinaryHeap::with_capacity(cap))
    }

    pub fn update_entry<T: Into<Tile<C>>>(&mut self, item: T) {
        let item = item.into();
        self.0 = self
            .0
            .drain()
            // Remove the cell
            .filter(|texture_node| texture_node.cell != item.cell)
            // Collect to a new binary heap that does not have cell anymore
            .collect::<BinaryHeap<_>>();

        self.push(item);
    }

    pub fn push<T: Into<Tile<C>>>(&mut self, item: T) {
        let item = item.into();
        self.0.push(item);
    }

    pub fn pop(&mut self) -> Option<Tile<C>> {
        self.0.pop()
    }
}
