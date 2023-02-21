/* This crate defines the API types that the core can manage
   It is used by al-ui and any javascript application calling
   the WASM core of aladin lite v3
*/
pub mod blend;
pub mod color;
pub mod colormap;
pub mod coo_system;
pub mod grid;
pub mod hips;
pub mod moc;
pub mod resources;
pub mod cell;
pub mod fov;

pub trait Abort {
    type Item;
    fn unwrap_abort(self) -> Self::Item where Self: Sized;
}

impl<T> Abort for Option<T> {
    type Item = T;
    
    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Some(t) => t,
            None => process::abort(),
        }
    }
}
impl<T, E> Abort for Result<T, E> {
    type Item = T;

    #[inline]
    fn unwrap_abort(self) -> Self::Item {
        use std::process;
        match self {
            Ok(t) => t,
            Err(_) => process::abort(),
        }
    }
}
