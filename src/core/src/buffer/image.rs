use crate::core::Texture2DArray;
pub trait Image {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    );

    // The size of the image
    fn get_size(&self) -> &Vector2<i32>;

    //fn get_cutoff_values(&self) -> Option<(f32, f32)>;
}

impl<T> Image for Rc<T>
where
    T: Image,
{
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        let image = &**self;
        image.tex_sub_image_3d(textures, offset);
    }

    fn get_size(&self) -> &Vector2<i32> {
        let image = &**self;
        image.get_size()
    }
}

#[derive(Debug)]
pub struct TileArrayBuffer<T: ArrayBuffer> {
    buf: T,
    size: Vector2<i32>,
}

impl<T> TileArrayBuffer<T>
where
    T: ArrayBuffer,
{
    pub fn new(buf: &[T::Item], width: i32, num_channels: i32) -> Self {
        let size_buf = width * width * num_channels;
        assert_eq!(size_buf, buf.len() as i32);
        let buf = T::new(buf);
        let size = Vector2::new(width, width);
        Self { buf, size }
    }
}

pub trait ArrayBuffer: AsRef<js_sys::Object> {
    type Item: std::cmp::PartialOrd + Clone + Copy + std::fmt::Debug;

    fn new(buf: &[Self::Item]) -> Self;
    fn empty(size: u32, blank_value: Self::Item) -> Self;

    fn to_vec(&self) -> Vec<Self::Item>;
}
#[derive(Debug)]
pub struct ArrayU8(js_sys::Uint8Array);
impl AsRef<js_sys::Object> for ArrayU8 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayU8 {
    type Item = u8;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayU8(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let uint8_arr = js_sys::Uint8Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayU8(uint8_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }
}
#[derive(Debug)]
pub struct ArrayI16(js_sys::Int16Array);
impl AsRef<js_sys::Object> for ArrayI16 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayI16 {
    type Item = i16;
    fn new(buf: &[Self::Item]) -> Self {
        ArrayI16(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let int16_arr = js_sys::Int16Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayI16(int16_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }
}
#[derive(Debug)]
pub struct ArrayI32(js_sys::Int32Array);
impl AsRef<js_sys::Object> for ArrayI32 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}
impl ArrayBuffer for ArrayI32 {
    type Item = i32;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayI32(buf.into())
    }

    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let int32_arr = js_sys::Int32Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayI32(int32_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }
}
#[derive(Debug)]
pub struct ArrayF32(js_sys::Float32Array);
impl AsRef<js_sys::Object> for ArrayF32 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayF32 {
    type Item = f32;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayF32(buf.into())
    }
    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let f32_arr = js_sys::Float32Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayF32(f32_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }
}

#[derive(Debug)]
pub struct ArrayF64(js_sys::Float64Array);
impl AsRef<js_sys::Object> for ArrayF64 {
    fn as_ref(&self) -> &js_sys::Object {
        self.0.as_ref()
    }
}

impl ArrayBuffer for ArrayF64 {
    type Item = f64;

    fn new(buf: &[Self::Item]) -> Self {
        ArrayF64(buf.into())
    }
    fn empty(size: u32, blank_value: Self::Item) -> Self {
        let f64_arr = js_sys::Float64Array::new_with_length(size).fill(blank_value, 0, size);
        let array = ArrayF64(f64_arr);
        array
    }

    fn to_vec(&self) -> Vec<Self::Item> {
        self.0.to_vec()
    }
}
use super::TileArrayBufferImage;
impl Image for TileArrayBufferImage {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        match &self {
            TileArrayBufferImage::U8(b) => textures[offset.z as usize]
                .bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    offset.x,
                    offset.y,
                    b.size.x,
                    b.size.y,
                    Some(b.buf.as_ref()),
                ),
            TileArrayBufferImage::I16(b) => textures[offset.z as usize]
                .bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    offset.x,
                    offset.y,
                    b.size.x,
                    b.size.y,
                    Some(b.buf.as_ref()),
                ),
            TileArrayBufferImage::I32(b) => textures[offset.z as usize]
                .bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    offset.x,
                    offset.y,
                    b.size.x,
                    b.size.y,
                    Some(b.buf.as_ref()),
                ),
            TileArrayBufferImage::F32(b) => textures[offset.z as usize]
                .bind()
                .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    offset.x,
                    offset.y,
                    b.size.x,
                    b.size.y,
                    Some(b.buf.as_ref()),
                ),
            _ => unimplemented!(),
        }
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        match &self {
            TileArrayBufferImage::U8(b) => &b.size,
            TileArrayBufferImage::I16(b) => &b.size,
            TileArrayBufferImage::I32(b) => &b.size,
            TileArrayBufferImage::F32(b) => &b.size,
            _ => unimplemented!(),
        }
    }
}

use crate::{healpix_cell::HEALPixCell, time::Time};
use js_sys::Function;
use std::cell::Cell;
use std::rc::Rc;

pub struct FITSMetaData {
    pub blank: Option<f32>,
    pub bzero: f32,
    pub bscale: f32,
}

pub enum RetrievedImageType {
    FITSImage {
        image: TileArrayBufferImage,
        metadata: FITSMetaData,
    },
    CompressedImage {
        image: TileHTMLImage,
    },
}

pub enum RequestType {
    File,
    HtmlImage,
}

use crate::image_fmt::FormatImageType;
pub trait ImageRequest {
    fn new() -> Self;
    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue>;
    fn image(
        &self,
        tile_width: i32,
        format: &FormatImageType,
    ) -> Result<RetrievedImageType, JsValue>;

    const REQUEST_TYPE: RequestType;
}

enum ImageRequestType {
    FITSImageRequest(FITSImageRequest),
    CompressedImageRequest(CompressedImageRequest),
}
impl ImageRequestType {
    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue> {
        match self {
            ImageRequestType::FITSImageRequest(r) => r.send(success, fail, url),
            ImageRequestType::CompressedImageRequest(r) => r.send(success, fail, url),
        }
    }
    fn image(
        &self,
        tile_width: i32,
        format: &FormatImageType,
    ) -> Result<RetrievedImageType, JsValue> {
        match self {
            ImageRequestType::FITSImageRequest(r) => r.image(tile_width, format),
            ImageRequestType::CompressedImageRequest(r) => r.image(tile_width, format),
        }
    }
}

pub struct TileRequest {
    // Is none when it is available for downloading a new fits
    // or image tile
    req: ImageRequestType,
    time_request: Time,
    // Flag telling if the tile has been copied so that
    // the HtmlImageElement can be reused to download another tile
    ready: bool,
    resolved: Rc<Cell<ResolvedStatus>>,
    pub tile: Option<Tile>,
    closures: [Closure<dyn FnMut(&web_sys::Event)>; 2],
}
#[derive(Clone, Copy, PartialEq)]
pub enum ResolvedStatus {
    NotResolved,
    Missing,
    Found,
}
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use super::Tile;
impl TileRequest {
    pub fn new<R: ImageRequest>() -> Self {
        // By default, all the requests are parametrized to load
        // compressed image requests
        let req = match R::REQUEST_TYPE {
            RequestType::File => ImageRequestType::FITSImageRequest(FITSImageRequest::new()),
            RequestType::HtmlImage => {
                ImageRequestType::CompressedImageRequest(CompressedImageRequest::new())
            }
        };

        // By default, we say the tile is available to be reused
        let resolved = Rc::new(Cell::new(ResolvedStatus::NotResolved));
        let tile = None;
        let closures = [
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
        ];
        let ready = true;
        let time_request = Time::now();
        Self {
            req,
            resolved,
            ready,
            tile,
            closures,
            time_request,
        }
    }

    /*pub fn is<R: ImageRequest>(&self) -> bool {
        match (R::REQUEST_TYPE, self.req) {
            (RequestType::File, ImageRequestType::FITSImageRequest(_)) => true,
            (RequestType::HtmlImage, ImageRequestType::CompressedImageRequest(_)) => true,
            _ => false
        }
    }*/

    pub fn send(&mut self, tile: Tile) -> Result<(), JsValue> {
        assert!(self.is_ready());

        self.tile = Some(tile.clone());
        let Tile {
            cell,
            root_url,
            format,
        } = tile;

        self.ready = false;

        let url = {
            let HEALPixCell(depth, idx) = cell;

            let dir_idx = (idx / 10000) * 10000;

            let url = format!(
                "{}/Norder{}/Dir{}/Npix{}.{}",
                root_url,
                depth.to_string(),
                dir_idx.to_string(),
                idx.to_string(),
                format.get_ext_file()
            );

            url
        };

        let success = {
            let resolved = self.resolved.clone();

            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                resolved.set(ResolvedStatus::Found);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        let fail = {
            let resolved = self.resolved.clone();
            Closure::wrap(Box::new(move |_: &web_sys::Event| {
                resolved.set(ResolvedStatus::Missing);
            }) as Box<dyn FnMut(&web_sys::Event)>)
        };

        self.resolved.set(ResolvedStatus::NotResolved);

        self.req.send(
            Some(success.as_ref().unchecked_ref()),
            Some(fail.as_ref().unchecked_ref()),
            &url,
        )?;

        self.closures = [success, fail];
        self.time_request = Time::now();

        Ok(())
    }

    pub fn get_tile(&self) -> &Tile {
        self.tile.as_ref().unwrap()
    }

    pub fn get_time_request(&self) -> Time {
        self.time_request
    }

    pub fn is_resolved(&self) -> bool {
        let resolved = self.resolve_status();
        resolved == ResolvedStatus::Found || resolved == ResolvedStatus::Missing
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn set_ready(&mut self) {
        self.ready = true;
    }

    pub fn clear(&mut self) {
        self.req.send(None, None, "").unwrap();
        self.ready = true;
        self.resolved.set(ResolvedStatus::NotResolved);
        self.closures = [
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
            Closure::wrap(
                Box::new(|_events: &web_sys::Event| {}) as Box<dyn FnMut(&web_sys::Event)>
            ),
        ];
        //self.tile = HEALPixCell(0, 13);
        self.time_request = Time::now();
    }

    pub fn resolve_status(&self) -> ResolvedStatus {
        self.resolved.get()
    }

    pub fn get_image(
        &self,
        tile_width: i32,
        format: &FormatImageType,
    ) -> Result<RetrievedImageType, JsValue> {
        assert!(self.is_resolved());
        self.req.image(tile_width, format)
    }
}

pub struct CompressedImageRequest {
    image: web_sys::HtmlImageElement,
}

impl ImageRequest for CompressedImageRequest {
    const REQUEST_TYPE: RequestType = RequestType::HtmlImage;

    fn new() -> Self {
        let image = web_sys::HtmlImageElement::new().unwrap();
        image.set_cross_origin(Some(""));

        Self { image }
    }

    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue> {
        self.image.set_src(&url);
        self.image.set_onload(success);
        self.image.set_onerror(fail);

        Ok(())
    }

    fn image(
        &self,
        _tile_width: i32,
        _format: &FormatImageType,
    ) -> Result<RetrievedImageType, JsValue> {
        let width = self.image.width() as i32;
        let height = self.image.height() as i32;

        let size = Vector2::new(width, height);
        Ok(RetrievedImageType::CompressedImage {
            image: TileHTMLImage {
                size,
                image: self.image.clone(),
            },
        })
    }
}

use web_sys::XmlHttpRequest;
pub struct FITSImageRequest {
    image: XmlHttpRequest,
}
use fitsrs::{DataType, Fits};
use fitsrs::{FITSHeaderKeyword, FITSKeywordValue};
use wasm_bindgen::JsValue;
use web_sys::XmlHttpRequestResponseType;
impl ImageRequest for FITSImageRequest {
    const REQUEST_TYPE: RequestType = RequestType::File;

    fn new() -> Self {
        let image = XmlHttpRequest::new().unwrap();
        image.set_response_type(XmlHttpRequestResponseType::Arraybuffer);

        Self { image }
    }

    fn send(
        &self,
        success: Option<&Function>,
        fail: Option<&Function>,
        url: &str,
    ) -> Result<(), JsValue> {
        self.image.open_with_async("GET", url, true)?;
        self.image.set_onload(success);
        self.image.set_onerror(fail);

        self.image.send().unwrap();

        Ok(())
    }

    fn image(
        &self,
        tile_width: i32,
        format: &FormatImageType,
    ) -> Result<RetrievedImageType, JsValue> {
        // We know at this point the request is resolved
        let array_buf = js_sys::Uint8Array::new(self.image.response().unwrap().as_ref());
        let bytes = &array_buf.to_vec();
        let Fits { data, header } = Fits::from_byte_slice(bytes).map_err(|e| {
            JsValue::from_str(&format!(
                "Parsing FITS error of {:?}: {:?}",
                self.image.response_url(),
                e
            ))
        })?;

        let num_channels = format.get_num_channels() as i32;

        let image = match data {
            DataType::U8(data) => TileArrayBufferImage::U8(TileArrayBuffer::<ArrayU8>::new(
                &data.0,
                tile_width,
                num_channels,
            )),
            DataType::I16(data) => TileArrayBufferImage::I16(TileArrayBuffer::<ArrayI16>::new(
                &data.0,
                tile_width,
                num_channels,
            )),
            DataType::I32(data) => TileArrayBufferImage::I32(TileArrayBuffer::<ArrayI32>::new(
                &data.0,
                tile_width,
                num_channels,
            )),
            DataType::F32(data) => TileArrayBufferImage::F32(TileArrayBuffer::<ArrayF32>::new(
                &data.0,
                tile_width,
                num_channels,
            )),
            DataType::F64(data) => {
                let data = data.0.into_iter().map(|v| v as f32).collect::<Vec<_>>();
                TileArrayBufferImage::F32(TileArrayBuffer::<ArrayF32>::new(
                    &data,
                    tile_width,
                    num_channels,
                ))
            }
            _ => unimplemented!(),
        };

        let bscale = if let Some(FITSHeaderKeyword::Other { value, .. }) = header.get("BSCALE") {
            if let FITSKeywordValue::FloatingPoint(bscale) = value {
                *bscale as f32
            } else {
                1.0
            }
        } else {
            1.0
        };
        let bzero = if let Some(FITSHeaderKeyword::Other { value, .. }) = header.get("BZERO") {
            if let FITSKeywordValue::FloatingPoint(bzero) = value {
                *bzero as f32
            } else {
                0.0
            }
        } else {
            0.0
        };
        let blank = if let Some(FITSHeaderKeyword::Blank(blank)) = header.get("BLANK") {
            Some(*blank as f32)
        } else {
            Some(-100.0)
        };

        let metadata = FITSMetaData {
            blank,
            bscale,
            bzero,
        };
        Ok(RetrievedImageType::FITSImage { image, metadata })
    }
}

impl Default for TileRequest {
    fn default() -> Self {
        TileRequest::new::<CompressedImageRequest>()
    }
}

pub struct TileHTMLImage {
    image: web_sys::HtmlImageElement,
    size: Vector2<i32>,
}
use cgmath::{Vector2, Vector3};
impl Image for TileHTMLImage {
    fn tex_sub_image_3d(
        &self,
        // The texture array
        textures: &Texture2DArray,
        // An offset to write the image in the texture array
        offset: &Vector3<i32>,
    ) {
        let _size = self.get_size();
        textures[offset.z as usize]
            .bind()
            .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                offset.x,
                offset.y,
                &self.image,
            );
    }

    // The size of the image
    fn get_size(&self) -> &Vector2<i32> {
        &self.size
    }

    /*fn get_cutoff_values(&self) -> std::option::Option<(f32, f32)> {
        None
    }*/
}

impl Drop for TileRequest {
    fn drop(&mut self) {}
}
