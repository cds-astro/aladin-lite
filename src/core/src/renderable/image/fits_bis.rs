use core::num;
use std::vec;
use std::marker::Unpin;
use std::fmt::Debug;
//use std::io::Cursor;

use al_core::texture::MAX_TEX_SIZE;
use futures::stream::{TryStreamExt};
use futures::AsyncRead;

use wasm_bindgen::{JsValue, UnwrapThrowExt};

use web_sys::WebGl2RenderingContext;

use moclib::moc::range::RangeMOC;
use moclib::qty::Hpx;
use moclib::elem::cell::Cell;
use moclib::moc::{RangeMOCIterator, RangeMOCIntoIterator};

use fitsrs::{
    fits::AsyncFits,
    hdu::{
        data::stream,
        header::extension::Xtension
    }
};
use wcs::{ImgXY, WCS, LonLat};

use al_api::cell::HEALPixCellProjeted;
use al_api::coo_system::CooSystem;
use al_api::hips::ImageMetadata;

use al_core::{VertexArrayObject, Texture2D};
use al_core::WebGlContext;
use al_core::VecData;
use al_core::webgl_ctx::GlWrapper;
use al_core::image::format::*;
use al_core::image::format::ImageFormatType;

use crate::math::projection::coo_space::XYNDC;
use crate::camera::CameraViewPort;
use crate::ProjectionType;
use crate::healpix::cell::HEALPixCell;
use crate::ShaderManager;
use crate::Colormaps;
use super::subdivide_texture::build;

pub struct FitsImage {
    // A reference to the GL context
    gl: WebGlContext,

    // The vertex array object of the screen in NDC
    vao: VertexArrayObject,
    num_indices: Vec<u32>,
    indices: Vec<u32>,
    pos: Vec<f32>,
    uv: Vec<f32>,

    // Parameters extracted from the fits
    wcs: WCS,
    blank: f32,
    scale: f32,
    offset: f32,

    // The center of the fits
    center: LonLat,

    // Texture format
    format: ImageFormatType,
    // Texture chunks objects
    textures: Vec<Texture2D>,
}

use futures::io::BufReader;
impl FitsImage {
    pub async fn new_async<'a, R>(
        gl: &WebGlContext,
        reader: &'a mut BufReader<R>,
    ) -> Result<Self, JsValue>
    where
        R: AsyncRead + Unpin + Debug
    {
        // Load the fits file
        let AsyncFits { mut hdu } = AsyncFits::from_reader(reader)
            .await
            .map_err(|_| JsValue::from_str("Error parsing the fits"))?;

        let header = hdu.get_header();

        let num_bytes_to_read = header.get_xtension().get_num_bytes_data_block();
        let naxis1 = *header.get_xtension().get_naxisn(1).unwrap_throw();
        let naxis2 = *header.get_xtension().get_naxisn(2).unwrap_throw();
        let bitpix = header.get_xtension().get_bitpix() as i32;

        al_core::log(&format!("size image {naxis1} {naxis2} {num_bytes_to_read} {bitpix}"));

        let scale = header
            .get_parsed::<f64>(b"BSCALE  ")
            .unwrap_or(Ok(1.0))
            .unwrap() as f32;
        let offset = header
            .get_parsed::<f64>(b"BZERO   ")
            .unwrap_or(Ok(0.0))
            .unwrap() as f32;
        let blank = header
            .get_parsed::<f64>(b"BLANK   ")
            .unwrap_or(Ok(std::f64::NAN))
            .unwrap() as f32;

        // Create a WCS from a specific header unit
        let wcs = WCS::new(&header).map_err(|_| JsValue::from_str("Failed to parse the WCS"))?;

        let (w, h) = wcs.img_dimensions();
        let width = w as f64;
        let height = h as f64;

        let data = hdu.get_data_mut();
        
        let (textures, format) = match data {
            stream::Data::U8(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R8UI, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R8UI)
            },
            stream::Data::I16(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R16I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R16I)
            },
            stream::Data::I32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::I64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as i32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32I, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32I)
            },
            stream::Data::F32(data) => {
                let reader = data
                    .map_ok(|v| {
                        v[0].to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
            stream::Data::F64(data) => {
                let reader = data
                    .map_ok(|v| {
                        let v = v[0] as f32;
                        v.to_le_bytes()
                    })
                    .into_async_read();

                let textures = build::<R32F, _>(gl, w, h, reader).await?;
                (textures, ImageFormatType::R32F)
            },
        };

        let center = wcs.unproj_lonlat(&ImgXY::new(width / 2.0, height / 2.0)).ok_or(JsValue::from_str("(w / 2, h / 2) px cannot be unprojected"))?;

        let num_indices = vec![];
        let indices = vec![];
        let pos = vec![];
        let uv = vec![];
        // Define the buffers
        let vao = {
            let mut vao = VertexArrayObject::new(gl);
            
            #[cfg(feature = "webgl2")]
            vao.bind_for_update()
                // layout (location = 0) in vec2 ndc_pos;
                .add_array_buffer_single(
                    2,
                    "ndc_pos",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&pos),
                )
                .add_array_buffer_single(
                    2,
                    "uv",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&uv),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<u32>(&indices),
                )
                .unbind();
            #[cfg(feature = "webgl1")]
            vao.bind_for_update()
                .add_array_buffer_single(
                    2,
                    "ndc_pos",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&pos),
                )
                .add_array_buffer_single(
                    2,
                    "uv",
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<f32>(&uv),
                )
                // Set the element buffer
                .add_element_buffer(
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                    VecData::<u32>(&indices),
                )
                .unbind();

            vao
        };

        let gl = gl.clone();
        let image = FitsImage {
            gl,

            // The positions
            vao,
            num_indices,
            pos,
            uv,
            indices,

            // Metadata extracted from the fits
            wcs,
            scale,
            offset,
            blank,
            center,

            // Texture parameters
            format,
            textures,
        };

        Ok(image)
    }

    pub fn update(&mut self, camera: &CameraViewPort, projection: &ProjectionType) -> Result<(), JsValue> {
        if !camera.has_moved() {
            return Ok(());
        }

        use crate::math::lonlat::LonLat;
        self.uv.clear();
        self.pos.clear();

        let dim = self.wcs.img_dimensions();
        let width = dim.0 as f64;
        let height = dim.1 as f64;

        // 1. TODO: Project the camera field of view inside the wcs
        let (xy_min, xy_max, num_vertices) = if let Some(vertices) = camera.get_vertices() {
            /*let (mut xy_min, mut xy_max) = vertices.iter()
                .map(|vertex| {
                    let lonlat = vertex.lonlat();

                    let lon = lonlat.lon();
                    let lat = lonlat.lat();

                    self.wcs.proj(&wcs::LonLat::new(lon.to_radians(), lat.to_radians()))
                })
                .fold(((width, height), (0.0_f64, 0.0_f64)), |(mut xy_min, mut xy_max), vertex| {
                    if let Some(vertex) = vertex {
                        xy_min.0 = xy_min.0.min(vertex.x());
                        xy_min.1 = xy_min.1.min(vertex.y());
    
                        xy_max.0 = xy_max.0.max(vertex.x());
                        xy_max.1 = xy_max.1.max(vertex.y());
    
                        (xy_min, xy_max)
                    } else {
                        ((0.0, 0.0), (width, height))
                    }
                });

            let fov_width = (xy_max.0 - xy_min.0).max(xy_max.1 - xy_min.1);
            let num_vertices = ((width / fov_width).clamp(0.0, 1.0) * 10.0).ceil() as u64;

            // clamp the texture
            xy_min.0 = xy_min.0.max(0.0);
            xy_min.1 = xy_min.1.max(0.0);

            xy_max.0 = xy_max.0.min(width);
            xy_max.1 = xy_max.1.min(height);

            al_core::log(&format!("{num_vertices}"));
            (xy_min, xy_max, num_vertices)
            */
            ((0.0, 0.0), (width, height), 10)
        } else {
            // allsky
            ((0.0, 0.0), (width, height), 10)
        };

        let xy_min = ImgXY::new(xy_min.0, xy_min.1);
        let xy_max = ImgXY::new(xy_max.0, xy_max.1);

        let (mut pos, mut uv, indices, num_indices) = super::grid::get_grid_vertices(&xy_min, &xy_max, MAX_TEX_SIZE as u64, num_vertices, camera, &self.wcs, projection);
        self.pos = unsafe {
            pos.set_len(pos.len() * 2);
            std::mem::transmute(pos)
        };
        self.uv = unsafe {
            uv.set_len(uv.len() * 2);
            std::mem::transmute(uv)
        };
        // Update num_indices
        self.indices = indices;
        self.num_indices = num_indices;

        // vertices contains ndc positions and texture UVs
        self.vao.bind_for_update()
            .update_array(
                "ndc_pos",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.pos),
            )
            .update_array(
                "uv",
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData(&self.uv),
            )
            .update_element_array(
                WebGl2RenderingContext::DYNAMIC_DRAW,
                VecData::<u32>(&self.indices),
            );

        Ok(())
    }

    // Draw the image
    pub fn draw(&self, shaders: &mut ShaderManager, colormaps: &Colormaps, cfg: &ImageMetadata) -> Result<(), JsValue> {
        self.gl.enable(WebGl2RenderingContext::BLEND);

        let ImageMetadata {
            color,
            opacity,
            blend_cfg,
            ..
        } = cfg;

        let mut shader = match self.format {
            ImageFormatType::R32F => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFS")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R32I => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSInteger")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R16I => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSInteger")?,
            #[cfg(feature = "webgl2")]
            ImageFormatType::R8UI => crate::shader::get_shader(&self.gl, shaders, "FitsVS", "FitsFSUnsigned")?,
            _ => return Err(JsValue::from_str("Image format type not supported"))
        };

        // 2. Draw it if its opacity is not null
        blend_cfg.enable(&self.gl, || {
            let mut off_indices = 0;
            for (idx, texture) in self.textures.iter().enumerate() {
                let num_indices = self.num_indices[idx] as i32;

                shader.bind(&self.gl)               
                    .attach_uniforms_from(colormaps)
                    .attach_uniforms_with_params_from(color, colormaps)
                    .attach_uniform("opacity", opacity)
                    .attach_uniform("tex", texture)
                    .attach_uniform("scale", &self.scale)
                    .attach_uniform("offset", &self.offset)
                    .attach_uniform("blank", &self.blank)
                    .bind_vertex_array_object_ref(&self.vao)
                    .draw_elements_with_i32(
                        WebGl2RenderingContext::TRIANGLES,
                        Some(num_indices),
                        WebGl2RenderingContext::UNSIGNED_INT,
                        ((off_indices as usize) * std::mem::size_of::<u32>()) as i32,
                    );

                off_indices += num_indices;
            }

            Ok(())
        })?;

        self.gl.disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    pub fn get_center(&self) -> &LonLat {
        &self.center
    }
}