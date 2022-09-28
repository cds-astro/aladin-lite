use cgmath::{Vector2, Vector3};
use fitsrs::FitsMemAligned;

#[derive(Debug)]
pub struct Fits<F>
where
    F: FitsImageFormat,
{
    // Fits header properties
    pub blank: f32,
    pub bzero: f32,
    pub bscale: f32,

    // Tile size
    size: Vector2<i32>,

    // Aligned allocation layout
    layout: std::alloc::Layout,
    // Raw pointer to the fits in memory
    aligned_raw_bytes_ptr: *mut u8,
    // Raw pointer to the data part of the fits
    pub aligned_data_raw_bytes_ptr: *const F::Type,
}

use std::alloc::{alloc, Layout};
impl<F> Fits<F>
where
    F: FitsImageFormat,
{
    pub fn new(fits_raw_bytes: &js_sys::Uint8Array) -> Result<Self, JsValue>
    where
        <F as FitsImageFormat>::Type: std::fmt::Debug
    {
        // Create a correctly aligned buffer to the type F
        let align = std::mem::size_of::<F::Type>();
        let layout = Layout::from_size_align(fits_raw_bytes.length() as usize, align)
            .expect("Cannot create sized aligned memory layout");
        // 1. Alloc the aligned memory buffer
        let aligned_raw_bytes_ptr = unsafe { alloc(layout) };

        let FitsMemAligned { data, header } = unsafe {
            // 2. Copy the raw fits bytes into that aligned memory space
            fits_raw_bytes.raw_copy_to_ptr(aligned_raw_bytes_ptr);

            // 3. Convert to a slice of bytes
            let aligned_raw_bytes =
                std::slice::from_raw_parts(aligned_raw_bytes_ptr, fits_raw_bytes.length() as usize);

            // 4. Parse the fits file to extract its data (big endianness is handled inside fitsrs and is O(n))
            FitsMemAligned::<F::Type>::from_byte_slice(aligned_raw_bytes)
                .map_err(|e| JsValue::from_str(&format!("Parsing FITS error: {:?}", e)))?
        };

        let bscale = if let Some(FITSHeaderKeyword::Other { value: FITSKeywordValue::FloatingPoint(bscale), .. }) = header.get("BSCALE") {
            *bscale as f32
        } else {
            1.0
        };

        let bzero = if let Some(FITSHeaderKeyword::Other { value: FITSKeywordValue::FloatingPoint(bzero), .. }) = header.get("BZERO") {
            *bzero as f32
        } else {
            0.0
        };

        let blank = if let Some(FITSHeaderKeyword::Blank(blank)) = header.get("BLANK") {
            *blank as f32
        } else {
            std::f32::NAN
        };

        let width = header
            .get("NAXIS1")
            .and_then(|k| match k {
                FITSHeaderKeyword::NaxisSize { size, .. } => Some(*size as i32),
                _ => None,
            })
            .ok_or_else(|| JsValue::from_str("NAXIS1 not found in the fits"))?;

        let height = header
            .get("NAXIS2")
            .and_then(|k| match k {
                FITSHeaderKeyword::NaxisSize { size, .. } => Some(*size as i32),
                _ => None,
            })
            .ok_or_else(|| JsValue::from_str("NAXIS2 not found in the fits"))?;

        Ok(Self {
            // Metadata fits header properties
            blank,
            bzero,
            bscale,
            // Tile size
            size: Vector2::new(width, height),

            // Allocation info of the layout
            layout,
            aligned_raw_bytes_ptr,

            aligned_data_raw_bytes_ptr: data.as_ptr(),
        })
    }

    pub fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }
}

use crate::image::Image;
use crate::texture::Texture2DArray;
impl<F> Image for Fits<F>
where
    F: FitsImageFormat,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        let num_pixels = self.size.x * self.size.y;
        let slice_raw_bytes = unsafe {
            std::slice::from_raw_parts(
                self.aligned_data_raw_bytes_ptr as *const _,
                num_pixels as usize,
            )
        };

        let array = unsafe { F::view(slice_raw_bytes) };
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                offset.x,
                offset.y,
                self.size.x,
                self.size.y,
                Some(array.as_ref()),
            );
    }

    // The size of the image
    /*fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }*/
}

impl<F> Drop for Fits<F>
where
    F: FitsImageFormat,
{
    fn drop(&mut self) {
        //al_core::log("dealloc fits tile");
        unsafe {
            std::alloc::dealloc(self.aligned_raw_bytes_ptr, self.layout);
        }
    }
}

use fitsrs::{FITSHeaderKeyword, FITSKeywordValue};
use wasm_bindgen::JsValue;

use crate::image::format::ImageFormat;
use fitsrs::ToBigEndian;
pub trait FitsImageFormat: ImageFormat {
    type Type: ToBigEndian + Clone;
    type ArrayBufferView: AsRef<js_sys::Object>;

    /// Creates a JS typed array which is a view into wasm's linear memory at the slice specified.
    /// This function returns a new typed array which is a view into wasm's memory. This view does not copy the underlying data.
    ///
    /// # Safety
    ///
    /// Views into WebAssembly memory are only valid so long as the backing buffer isn't resized in JS. Once this function is called any future calls to Box::new (or malloc of any form) may cause the returned value here to be invalidated. Use with caution!
    ///
    /// Additionally the returned object can be safely mutated but the input slice isn't guaranteed to be mutable.
    ///
    /// Finally, the returned object is disconnected from the input slice's lifetime, so there's no guarantee that the data is read at the right time.
    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView;
}

use crate::image::R32F;
impl FitsImageFormat for R32F {
    type Type = f32;
    type ArrayBufferView = js_sys::Float32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
use crate::image::{R16I, R32I, R8UI, R64F};
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R64F {
    type Type = f64;

    type ArrayBufferView = js_sys::Float64Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}

#[cfg(feature = "webgl2")]
impl FitsImageFormat for R32I {
    type Type = i32;

    type ArrayBufferView = js_sys::Int32Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R16I {
    type Type = i16;

    type ArrayBufferView = js_sys::Int16Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
#[cfg(feature = "webgl2")]
impl FitsImageFormat for R8UI {
    type Type = u8;

    type ArrayBufferView = js_sys::Uint8Array;

    unsafe fn view(s: &[Self::Type]) -> Self::ArrayBufferView {
        Self::ArrayBufferView::view(s)
    }
}
