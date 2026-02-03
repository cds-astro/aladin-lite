use crate::time::Time;

use crate::renderable::hips::d3::Freq;
use crate::Abort;
use crate::WebGlContext;
use al_core::image::fits::FitsImage;
use al_core::image::raw::ImageBuffer;
use al_core::image::Image;
use al_core::texture::format::{PixelType, R16I, R32F, R32I, R8U};
use al_core::texture::Texture3D;
use al_core::webgl_ctx::WebGlRenderingCtx;
use cgmath::Vector3;
use fitsrs::hdu::header::Bitpix;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::ops::Range;
use wasm_bindgen::JsValue;

pub enum HpxFreqData {
    Fits {
        // The raw bytes of the whole cubic FITS file, data big endian
        raw_bytes: Box<[u8]>,
        // Offset to the data bytes of the cubic tile
        data_byte_offset: Range<usize>,
        // Number of bytes per pixel (deduced from the bitpix)
        bitpix: Bitpix,
        // Triming offset indices when reading the data
        trim: (u32, u32, u32),
        // Naxis
        naxis: (u32, u32, u32),
        // Scaling value
        bscale: f32,
        // Offset value
        bzero: f32,
        // The real size of the cube
        size: (u32, u32, u32),
    },
    Jpeg {
        data: Box<[u8]>,
        size: (u32, u32, u32),
    },
    Png {
        data: Box<[u8]>,
        size: (u32, u32, u32),
    },
}

pub enum Pixel {
    F32(f32),
    I32(i32),
    I16(i16),
    U8(u8),
}

impl Pixel {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Pixel::F32(v) => v,
            Pixel::I16(v) => v as f32,
            Pixel::I32(v) => v as f32,
            Pixel::U8(v) => v as f32,
        }
    }
}

impl HpxFreqData {
    pub fn read_pixel(&self, x: u32, y: u32, z: u32) -> Option<f32> {
        match self {
            HpxFreqData::Fits {
                raw_bytes,
                data_byte_offset,
                bitpix,
                trim,
                naxis,
                bscale,
                bzero,
                size,
            } => {
                // Do not remember the origin in fits image data is left-down corner
                let y = size.1 - y;

                let x_in_data = (trim.0..(trim.0 + naxis.0)).contains(&x);
                let y_in_data = (trim.1..(trim.1 + naxis.1)).contains(&y);
                let z_in_data = (trim.2..(trim.2 + naxis.2)).contains(&z);

                if !x_in_data || !y_in_data || !z_in_data {
                    None
                } else {
                    let x = x - trim.0;
                    let y = y - trim.1;
                    let z = z - trim.2;

                    let data_raw_bytes = &raw_bytes[data_byte_offset.clone()];
                    let bytes_per_pixel = bitpix.byte_size();
                    let pixel_bytes_off =
                        bytes_per_pixel * (x + y * naxis.0 + z * (naxis.0 * naxis.1)) as usize;

                    let p = &data_raw_bytes[pixel_bytes_off..(pixel_bytes_off + bytes_per_pixel)];

                    let pixel = match bitpix {
                        Bitpix::U8 => Pixel::U8(p[0]),
                        Bitpix::I16 => Pixel::I16(i16::from_be_bytes([p[0], p[1]])),
                        Bitpix::I32 => Pixel::I32(i32::from_be_bytes([p[0], p[1], p[2], p[3]])),
                        Bitpix::F32 => Pixel::F32(f32::from_be_bytes([p[0], p[1], p[2], p[3]])),
                        Bitpix::F64 => Pixel::F32(f64::from_be_bytes([
                            p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7],
                        ]) as f32),
                        Bitpix::I64 => Pixel::I32(i64::from_be_bytes([
                            p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7],
                        ]) as i32),
                    };

                    Some(pixel.to_f32() * (*bscale) + (*bzero))
                }
            }
            HpxFreqData::Jpeg { data, size } => {
                let pixel_bytes_off = (x + y * size.0 + z * (size.0 * size.1)) as usize;

                let p = data[pixel_bytes_off];
                Some(p as f32)
            }
            HpxFreqData::Png { data, size } => {
                let pixel_bytes_off = (x + y * size.0 + z * (size.0 * size.1)) as usize;

                let p = data[2 * pixel_bytes_off];
                Some(p as f32)
            }
        }
    }
}

pub struct HpxFreqTex {
    pub cell: HEALPixFreqCell,
    // The time the texture has been received
    // If the texture contains multiple tiles, then the receiving time
    // is set when all the tiles have been copied to the buffer
    pub start_time: Option<Time>,
    // The time request of the texture is the time request
    // of the first tile being inserted in it
    // It is then only given in the constructor of Texture
    // This is approximate, it should correspond to the minimum
    // of the time requests of the cells currenlty contained in the
    // texture. But this is too expensive because at each tile inserted
    // in the buffer, one should reevalute the priority of the texture
    // in the buffer's binary heap.
    pub time_request: Time,

    // OLD CODE
    // We autorize 512 cubic tiles of size 32 each which allows to store max 16384 slices
    //textures: Vec<Option<Texture3D>>,
    // A set of already inserted slices. Each cubic tiles can have 32 slices. The occupancy of the
    // slices inside a cubic tile is done with a u32 mask. Limited to 16384 slices
    //blocks: [u32; 512],
    // sorted index list of 32-length blocks that are not empty
    //block_indices: Vec<usize>,
    /// The webgl2 3D texture of the cubic tile
    pub texture: Texture3D,

    data: Option<HpxFreqData>,

    // The real image data for accessing the pixel values
    //data: ImageType,
    /// A bitvector keeping track of the slices that have been inserted into the 3D texture
    /// It is limited to a cube depth of 256 (~ to the max texture size).
    slice_idx: [u32; 8],

    /// Depth of the tile
    num_slices: u16,
    /// Number of slices copied (concerns only HiPSCube)
    num_stored_slices: u16,
}

const TEX_PARAMS: &[(u32, u32)] = &[
    (
        WebGlRenderingCtx::TEXTURE_MIN_FILTER,
        WebGlRenderingCtx::NEAREST,
    ),
    (
        WebGlRenderingCtx::TEXTURE_MAG_FILTER,
        WebGlRenderingCtx::NEAREST,
    ),
    // Prevents s-coordinate wrapping (repeating)
    (
        WebGlRenderingCtx::TEXTURE_WRAP_S,
        WebGlRenderingCtx::CLAMP_TO_EDGE,
    ),
    // Prevents t-coordinate wrapping (repeating)
    (
        WebGlRenderingCtx::TEXTURE_WRAP_T,
        WebGlRenderingCtx::CLAMP_TO_EDGE,
    ),
    // Prevents r-coordinate wrapping (repeating)
    (
        WebGlRenderingCtx::TEXTURE_WRAP_R,
        WebGlRenderingCtx::CLAMP_TO_EDGE,
    ),
];

use crate::healpix::cell::HEALPixFreqCell;
impl HpxFreqTex {
    pub fn new(
        // The cubic tile definition to locate the cube in the sky + spectral axis
        cell: HEALPixFreqCell,
        // The time the request has been made, i.e. when the tile was needed
        time_request: Time,
        // The size of the cubis tile
        tile_size: u16,
        // The depth of the cubic tile. Must be a power of two
        num_slices: u16,
        // pixel format
        pixel_format: PixelType,
        // The Gl context
        gl: &WebGlContext,
    ) -> Result<Self, JsValue> {
        let start_time = None;

        let texture = match pixel_format {
            // alpha transparency
            PixelType::RGBA8U => Texture3D::create_empty::<R16I>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::RGB8U => Texture3D::create_empty::<R8U>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R8U => Texture3D::create_empty::<R8U>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R32F => Texture3D::create_empty::<R32F>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R16I => Texture3D::create_empty::<R16I>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
            PixelType::R32I => Texture3D::create_empty::<R32I>(
                gl,
                tile_size as i32,
                tile_size as i32,
                num_slices as i32,
                TEX_PARAMS,
            ),
        }?;

        let data = None;
        let num_stored_slices = 0;
        let slice_idx = [0x0; 8];
        Ok(Self {
            cell,
            slice_idx,
            time_request,
            start_time,
            data,
            texture,
            num_slices,
            num_stored_slices,
        })
    }

    pub fn set_data_from_fits(
        &mut self,
        // the tile image of the whole cubic tile
        raw_bytes: js_sys::Uint8Array,
        // size of the cube
        size: (u32, u32, u32),
    ) -> Result<(), JsValue> {
        let raw_bytes = raw_bytes.to_vec().into_boxed_slice();

        self.data = {
            let image = FitsImage::from_raw_bytes(&raw_bytes[..])?.pop().unwrap();
            image.insert_into_3d_texture(&self.texture, &Vector3::<i32>::new(0, 0, 0))?;

            let bitpix = image.bitpix;
            let trim = (image.trim1, image.trim2, image.trim3);
            let naxis = (image.width, image.height, image.depth);
            let bscale = image.bscale;
            let bzero = image.bzero;

            if let Cow::Owned(uncompressed_bytes) = image.raw_bytes {
                Some(HpxFreqData::Fits {
                    data_byte_offset: 0..uncompressed_bytes.len(),
                    raw_bytes: uncompressed_bytes.into_boxed_slice(),
                    bitpix,
                    trim,
                    naxis,
                    bscale,
                    bzero,
                    size,
                })
            } else {
                let data_byte_offset = image.data_byte_offset.clone();

                std::mem::drop(image);

                Some(HpxFreqData::Fits {
                    raw_bytes,
                    data_byte_offset,
                    bitpix,
                    trim,
                    naxis,
                    bscale,
                    bzero,
                    size,
                })
            }
        };

        self.num_stored_slices = self.num_slices;
        self.start_time = Some(Time::now());

        Ok(())
    }

    pub fn read_pixel(&self, x: u32, y: u32, z: u32) -> Option<f32> {
        if let Some(data) = &self.data {
            data.read_pixel(x, y, z)
        } else {
            None
        }
    }

    pub fn frequencies(&self) -> Vec<f32> {
        let delta_depth = self.num_slices.trailing_zeros();
        let pixel_depth = self.cell.f_depth + delta_depth as u8;

        let h0 = self.cell.f_hash << delta_depth;
        let h1 = (self.cell.f_hash + 1) << delta_depth;

        (h0..h1)
            .map(|hash| Freq::from_hash_with_order(hash, pixel_depth).0 as f32)
            .collect()
    }

    pub fn set_data_from_jpeg(
        &mut self,
        // the tile image of the whole cubic tile
        decoded_bytes: Box<[u8]>,
        // size of the cube
        size: (u32, u32, u32),
    ) -> Result<(), JsValue> {
        let cubic_tile = ImageBuffer::<R8U>::new(decoded_bytes, size.0, size.1, size.2);

        cubic_tile.insert_into_3d_texture(&self.texture, &Vector3::<i32>::new(0, 0, 0))?;

        self.data = Some(HpxFreqData::Jpeg {
            data: cubic_tile.data,
            size,
        });
        self.num_stored_slices = self.num_slices;
        self.start_time = Some(Time::now());

        Ok(())
    }

    pub fn set_data_from_png(
        &mut self,
        // the tile image of the whole cubic tile
        decoded_bytes: Box<[u8]>,
        // size of the cube
        size: (u32, u32, u32),
    ) -> Result<(), JsValue> {
        let cubic_tile = ImageBuffer::<R8U>::new(decoded_bytes, size.0, size.1, size.2);

        cubic_tile.insert_into_3d_texture(&self.texture, &Vector3::<i32>::new(0, 0, 0))?;

        self.data = Some(HpxFreqData::Png {
            data: cubic_tile.data,
            size,
        });
        self.num_stored_slices = self.num_slices;
        self.start_time = Some(Time::now());

        Ok(())
    }

    // Panic if cell is not contained in the texture
    // Do nothing if the texture is full
    // Return true if the tile is newly added
    // Used by HiPS Cubes
    pub fn append_tile_slice<I: Image>(
        &mut self,
        // the tile image of 1 slice
        image: I,
        // the slice offset in the cubic tile
        offset: u16,
    ) -> Result<(), JsValue> {
        // If there is already something, do not tex sub
        let block_idx = (offset >> 5) as usize;
        let slice_idx = (offset & 0x1f) as u8;

        if self.slice_idx[block_idx] & (1 << (31 - slice_idx)) == 0 {
            image.insert_into_3d_texture(
                &self.texture,
                &Vector3::<i32>::new(0, 0, slice_idx as i32),
            )?;

            self.slice_idx[block_idx] |= 1 << (31 - slice_idx);
            self.num_stored_slices += 1;
        }

        self.start_time = Some(Time::now());

        Ok(())
    }

    // Cell must be contained in the texture
    pub fn contains_slice(&self, offset: u16) -> bool {
        let block_idx = (offset >> 5) as usize;
        let slice_idx = offset & 0x1f;

        (self.slice_idx[block_idx] >> (31 - slice_idx)) & 0x1 == 1
    }
}

impl PartialOrd for HpxFreqTex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HpxFreqTex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_abort()
    }
}

impl PartialEq for HpxFreqTex {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}
impl Eq for HpxFreqTex {}
